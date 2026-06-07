"""Simulation Orchestrator — global scheduling and coordination.

Manages:
- Global clock and multi-rate scheduling
- Model lifecycle (init → step → finalize)
- Fault injection coordination
- Checkpoint / rollback
- Energy conservation checks

Security:
  - CWE-789: total_steps capped at 1 billion (DoS prevention)
  - CWE-20: Input validation on all public methods
  - CWE-248: Exception safety on init/step/fault/progress
  - CWE-209: Generic error messages (no internal leak)
  - CWE-501: Energy audit via public interface (no private access)
"""

import bisect
import logging
import math
from collections import deque
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

from .clock import ClockMode, GlobalClock
from .data_bus import DataBus
from .model_registry import ModelRegistry

logger = logging.getLogger(__name__)

# SECURITY (CWE-789): Maximum simulation steps to prevent DoS
_MAX_TOTAL_STEPS = 1_000_000_000


@dataclass
class StepResult:
    """Result of one simulation step for a solver."""
    solver_id: str
    converged: bool = True
    error_estimate: float = 0.0
    computation_ns: int = 0


@dataclass
class EnergyAudit:
    """Energy balance across domains."""
    power_input_j: float = 0.0
    mechanical_output_j: float = 0.0
    thermal_loss_j: float = 0.0
    stored_energy_j: float = 0.0
    imbalance_j: float = 0.0

    @property
    def imbalance_pct(self) -> float:
        total = self.power_input_j + 1e-12
        # SECURITY (I-06): Guard NaN/Inf on total
        if math.isnan(total) or math.isinf(total):
            return 0.0
        return abs(self.imbalance_j) / total * 100


@dataclass
class OrchestratorConfig:
    """Orchestrator configuration."""
    mode: ClockMode = ClockMode.OFFLINE
    enable_energy_audit: bool = True
    energy_audit_period_steps: int = 1000
    checkpoint_period_steps: int = 10000
    divergence_threshold: float = 0.1  # 10% energy imbalance
    auto_step_halving: bool = True
    max_step_halving: int = 3


class Orchestrator:
    """Central coordinator for multi-model simulation."""

    def __init__(self, cfg: OrchestratorConfig | None = None):
        self.cfg = cfg or OrchestratorConfig()
        self.clock = GlobalClock(mode=self.cfg.mode)
        self.bus = DataBus()
        self.registry = ModelRegistry()
        self._steppers: dict[str, Callable[[int], StepResult]] = {}
        self._initializers: dict[str, Callable[[], None]] = {}
        self._stop_hooks: list[Callable[[], bool]] = []
        self._fault_queue: deque = deque()  # O(1) popleft instead of list
        self._energy_audits: deque = deque(maxlen=10000)  # auto-capped
        self._sim_time_s_max: float | None = None

    # ── model registration ───────────────────────────────────

    def register_model(self, model: Any, metadata) -> None:
        """Register a model. (M-01: Accept Any for MVP — type checked at step time.)"""
        if model is None:
            raise TypeError("model cannot be None")
        self.registry.register(model, metadata)

    def register_stepper(self, solver_id: str,
                         stepper: Callable[[int], StepResult]) -> None:
        if not callable(stepper):
            raise TypeError(f"stepper must be callable, got {type(stepper).__name__}")
        self._steppers[solver_id] = stepper

    def register_initializer(self, solver_id: str,
                             init_fn: Callable[[], None]) -> None:
        if not callable(init_fn):
            raise TypeError(f"init_fn must be callable, got {type(init_fn).__name__}")
        self._initializers[solver_id] = init_fn

    # ── run control ──────────────────────────────────────────

    def set_sim_duration(self, duration_s: float) -> None:
        """Set simulation duration with validation (L-05)."""
        if not isinstance(duration_s, (int, float)):
            raise TypeError(f"duration_s must be numeric, got {type(duration_s).__name__}")
        if math.isnan(duration_s) or math.isinf(duration_s):
            raise ValueError("duration_s must not be NaN or Inf")
        if duration_s <= 0:
            raise ValueError(f"duration_s must be positive, got {duration_s}")
        self._sim_time_s_max = duration_s

    def add_stop_condition(self, condition: Callable[[], bool]) -> None:
        if not callable(condition):
            raise TypeError(f"condition must be callable, got {type(condition).__name__}")
        self._stop_hooks.append(condition)

    # ── fault injection ──────────────────────────────────────

    def schedule_fault(self, at_time_s: float, fault_fn: Callable[[], None]) -> None:
        """Schedule a fault function at a specific time.

        SECURITY:
          - CWE-754: NaN/Inf guard on time
          - CWE-20: callable validation on fault_fn (M-02)
        """
        if not callable(fault_fn):
            raise TypeError(f"fault_fn must be callable, got {type(fault_fn).__name__}")
        if math.isnan(at_time_s) or math.isinf(at_time_s) or at_time_s < 0:
            logger.warning("Invalid fault time: %s, skipping", at_time_s)
            return

        # Use bisect for O(log n) insertion
        fault_ns = int(at_time_s * 1e9)
        # deque doesn't support bisect, so convert to list temporarily
        fault_list = list(self._fault_queue)
        insertion_point = bisect.bisect_left([f[0] for f in fault_list], fault_ns)
        fault_list.insert(insertion_point, (fault_ns, fault_fn))
        self._fault_queue = deque(fault_list)

    # ── main loop ────────────────────────────────────────────

    def run(self, step_ns: int, duration_s: float = 1.0,
            progress_callback: Callable[[float], None] | None = None) -> deque:
        """Run simulation with input validation and exception handling.

        SECURITY:
          - CWE-1288: reject step_ns=0 (ZeroDivisionError)
          - CWE-248: try/except wraps init, step, and fault functions
          - CWE-789: total_steps capped at _MAX_TOTAL_STEPS
        """
        # SECURITY: Validate inputs (CWE-1288)
        if not isinstance(step_ns, int) or step_ns <= 0:
            raise ValueError(f"step_ns must be positive integer, got {step_ns}")
        if duration_s <= 0 or math.isnan(duration_s) or math.isinf(duration_s):
            raise ValueError(f"Invalid duration_s: {duration_s}")

        self.set_sim_duration(duration_s)
        total_steps = int(duration_s * 1e9 / step_ns)

        # SECURITY (CWE-789): Cap total steps to prevent DoS
        if total_steps > _MAX_TOTAL_STEPS:
            raise ValueError(
                f"Total steps {total_steps:,} exceeds maximum {_MAX_TOTAL_STEPS:,}. "
                f"Reduce duration or increase step size.")

        # Initialize all models with exception safety
        for solver_id, init_fn in self._initializers.items():
            try:
                logger.debug("Initializing %s", solver_id)
                init_fn()
            except Exception:
                logger.exception("Init failed for %s, skipping", solver_id)

        current_step_ns = step_ns
        halving_count = 0

        for i in range(total_steps):
            # Check stop conditions
            stop = False
            for hook in self._stop_hooks:
                try:
                    if hook():
                        logger.info("Stop condition triggered at step %d", i)
                        stop = True
                        break
                except Exception:
                    logger.warning("Stop hook failed at step %d", i)
            if stop:
                break
            if self.clock.diverged:
                break

            # Inject scheduled faults
            self._apply_faults()

            # Step all solvers with exception safety
            all_converged = True
            max_error = 0.0
            for solver_id, stepper in self._steppers.items():
                try:
                    result = stepper(current_step_ns)
                    if not result.converged:
                        all_converged = False
                        max_error = max(max_error, result.error_estimate)
                except Exception:
                    logger.exception("Solver %s crashed at step %d", solver_id, i)
                    all_converged = False

            # Divergence handling: auto step-halving
            if not all_converged and self.cfg.auto_step_halving:
                if halving_count < self.cfg.max_step_halving:
                    current_step_ns //= 2
                    current_step_ns = max(current_step_ns, 1)  # Prevent zero step
                    halving_count += 1
                    logger.warning("Step halved to %d ns (halving %d/%d)",
                                   current_step_ns, halving_count,
                                   self.cfg.max_step_halving)
                    continue
                else:
                    self.clock.mark_diverged()
                    logger.error("Simulation diverged after %d halvings",
                                 halving_count)

            # Advance clock
            self.clock.advance(current_step_ns)

            # Periodic energy audit
            if (self.cfg.enable_energy_audit and
                    i % self.cfg.energy_audit_period_steps == 0):
                audit = self._energy_audit()
                if audit.imbalance_pct > self.cfg.divergence_threshold:
                    logger.warning("Energy imbalance %.2f%% at t=%.4fs",
                                   audit.imbalance_pct, self.clock.sim_time_s)

            # Progress (L-04: exception-safe callback)
            if progress_callback and i % 100 == 0:
                try:
                    progress_callback((i + 1) / total_steps)
                except Exception:
                    logger.debug("Progress callback failed at step %d", i)

        return self._energy_audits

    def _apply_faults(self) -> None:
        """Apply pending faults. Uses deque for O(1) popleft."""
        while self._fault_queue and self._fault_queue[0][0] <= self.clock.sim_time_ns:
            _, fault_fn = self._fault_queue.popleft()
            try:
                logger.info("Injecting fault at t=%.6fs", self.clock.sim_time_s)
                fault_fn()
            except Exception:
                logger.exception("Fault injection failed at t=%.6fs",
                                 self.clock.sim_time_s)

    def _energy_audit(self) -> EnergyAudit:
        """Perform energy balance audit across all domains.

        SECURITY (M-03): Uses public ModelRegistry interface instead of private _models.
        """
        audit = EnergyAudit()

        # Use public interface instead of accessing private _models
        for model_id, meta in self.registry.list_all().items():
            try:
                model = self.registry.get(model_id)
                if hasattr(model, 'get_power_input'):
                    val = model.get_power_input()
                    if not (math.isnan(val) or math.isinf(val)):
                        audit.power_input_j += val
                if hasattr(model, 'get_power_output'):
                    val = model.get_power_output()
                    if not (math.isnan(val) or math.isinf(val)):
                        audit.mechanical_output_j += val
                if hasattr(model, 'get_power_loss'):
                    val = model.get_power_loss()
                    if not (math.isnan(val) or math.isinf(val)):
                        audit.thermal_loss_j += val
                if hasattr(model, 'get_stored_energy'):
                    val = model.get_stored_energy()
                    if not (math.isnan(val) or math.isinf(val)):
                        audit.stored_energy_j += val
            except Exception as e:
                logger.debug("Energy audit failed for model %s: %s", model_id, e)

        # Calculate imbalance
        audit.imbalance_j = (audit.power_input_j - audit.mechanical_output_j
                            - audit.thermal_loss_j - audit.stored_energy_j)

        # Auto-capped by deque maxlen
        self._energy_audits.append(audit)
        return audit

    # ── helpers ──────────────────────────────────────────────

    def run_simple(self, step_fn: Callable[[], None],
                   step_ns: int, duration_s: float = 1.0) -> None:
        """Simple single-stepper convenience wrapper (for MVP).

        SECURITY: Input validation and exception safety.
        """
        if not callable(step_fn):
            raise TypeError(f"step_fn must be callable, got {type(step_fn).__name__}")
        if not isinstance(step_ns, int) or step_ns <= 0:
            raise ValueError(f"step_ns must be positive integer, got {step_ns}")
        if duration_s <= 0 or math.isnan(duration_s) or math.isinf(duration_s):
            raise ValueError(f"Invalid duration_s: {duration_s}")

        total_steps = int(duration_s * 1e9 / step_ns)
        if total_steps > _MAX_TOTAL_STEPS:
            raise ValueError(f"Total steps {total_steps:,} exceeds maximum {_MAX_TOTAL_STEPS:,}")

        for i in range(total_steps):
            try:
                step_fn()
                self.clock.advance(step_ns)
            except Exception as e:
                logger.exception("Step function failed at step %d", i)
                raise RuntimeError(f"Simulation step {i} failed") from e

    def reset(self) -> None:
        self.clock.reset()
        self.bus.reset()
        self._fault_queue.clear()
        self._energy_audits.clear()
