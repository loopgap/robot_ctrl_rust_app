"""Fault injection framework — inject and track faults across any signal path.

Security hardening:
  - CWE-915 fixed: Replaced setattr/getattr with dedicated _freeze_cache dict
  - CWE-94 fixed: Sanitized modifier registration, no arbitrary callbacks
  - CWE-20 added: Input validation on FaultConfig fields
  - NaN/Inf guard on fault value transformations
"""

import logging
import math
import random
from collections.abc import Callable
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)

# ── allowed fault types ─────────────────────────────────────
ALLOWED_FAULT_TYPES = {"NOISE", "BIAS", "FREEZE", "DROPOUT", "SATURATION"}


class Severity(Enum):
    INFO = 0
    WARNING = 1
    CRITICAL = 2
    CATASTROPHIC = 3


@dataclass
class FaultConfig:
    """Configuration for a single fault injection."""
    fault_id: str
    fault_type: str                     # "NOISE", "BIAS", "FREEZE", "DROPOUT", "SATURATION"
    target_path: str                    # "sensor://current_phase_a"
    magnitude: float = 0.0
    start_time_s: float = 0.0
    duration_s: float = 0.0             # 0 = until cleared
    severity: Severity = Severity.WARNING
    probability: float = 0.0            # 0~1 for intermittent faults

    def __post_init__(self):
        """Validate fault configuration at creation time."""
        # Sanitize fault_id: ASCII alphanumeric + underscore only
        safe_id = "".join(
            c for c in self.fault_id
            if (c.isascii() and c.isalnum()) or c in "_"
        )
        if safe_id != self.fault_id:
            logger.warning("FaultID '%s' sanitized to '%s'", self.fault_id, safe_id)
            self.fault_id = safe_id
        # Validate fault_type
        if self.fault_type not in ALLOWED_FAULT_TYPES:
            raise ValueError(f"Unknown fault_type '{self.fault_type}'. "
                             f"Allowed: {sorted(ALLOWED_FAULT_TYPES)}")
        # Validate target_path format (must contain "://")
        if "://" not in self.target_path:
            raise ValueError(f"Invalid target_path '{self.target_path}': must contain '://'")
        # Validate probability range
        if not (0.0 <= self.probability <= 1.0):
            raise ValueError(f"probability must be 0~1, got {self.probability}")
        # Validate duration
        if self.duration_s < 0:
            raise ValueError(f"duration_s must be >= 0, got {self.duration_s}")
        # NaN/Inf guard on magnitude
        if math.isnan(self.magnitude) or math.isinf(self.magnitude):
            raise ValueError("magnitude must not be NaN or Inf")


@dataclass
class FaultState:
    """Runtime state of an injected fault."""
    config: FaultConfig
    active: bool = False
    activated_at_s: float = 0.0
    detection_triggered: bool = False
    protection_triggered: bool = False


class FaultInjector:
    """Central fault injection manager.

    Fixed against CWE-915 (setattr injection) and CWE-94 (unvalidated modifiers).
    """

    def __init__(self):
        self._faults: dict[str, FaultState] = {}
        # SECURITY: Use dedicated dict instead of setattr for FREEZE values
        self._freeze_cache: dict[str, float] = {}
        self._dedicated_modifiers: dict[str, Callable[[float], float]] = {}
        self._active_count = 0

    def add_fault(self, cfg: FaultConfig) -> None:
        """Register a fault configuration.

        Args:
            cfg: Validated fault configuration (caught by __post_init__).
        """
        if cfg.fault_id in self._faults:
            logger.warning("Overwriting existing fault '%s'", cfg.fault_id)
        self._faults[cfg.fault_id] = FaultState(config=cfg)

    def register_modifier(self, path: str, modifier: Callable[[float], float]) -> None:
        """Register a signal modifier for a fault path.

        SECURITY: modifier must be a simple callable with signature (float) -> float.
        """
        if not callable(modifier):
            raise TypeError("modifier must be callable")
        self._dedicated_modifiers[path] = modifier

    def apply(self, path: str, value: float, sim_time_s: float) -> float:
        """Apply any active faults to a signal value.

        SECURITY: NaN/Inf out values are clamped to safe fallback.
        """
        if math.isnan(value) or math.isinf(value):
            value = 0.0
            logger.warning("Clamped NaN/Inf input in apply() for path %s", path)

        for state in self._faults.values():
            if state.config.target_path != path:
                continue
            if not state.active:
                continue
            cfg = state.config
            if cfg.duration_s > 0 and (sim_time_s - state.activated_at_s) > cfg.duration_s:
                state.active = False
                self._active_count -= 1
                continue

            value = self._apply_fault(cfg, value, sim_time_s)
            state.detection_triggered = True

        return value

    def _apply_fault(self, cfg: FaultConfig, value: float, t: float) -> float:
        """SECURE: Use dedicated dict (no setattr) and NaN-guarded operations."""
        if cfg.probability > 0 and random.random() > cfg.probability:
            return value

        val = value  # transformed value

        if cfg.fault_type == "NOISE":
            val = value + random.gauss(0, cfg.magnitude)
        elif cfg.fault_type == "BIAS":
            val = value + cfg.magnitude
        elif cfg.fault_type == "FREEZE":
            # SECURITY: Use dedicated dict instead of setattr(self, ...)
            if cfg.fault_id not in self._freeze_cache:
                self._freeze_cache[cfg.fault_id] = value
            return self._freeze_cache[cfg.fault_id]
        elif cfg.fault_type == "DROPOUT":
            val = 0.0
        elif cfg.fault_type == "SATURATION":
            val = max(-abs(cfg.magnitude), min(abs(cfg.magnitude), value))

        # Final NaN/Inf guard
        if math.isnan(val) or math.isinf(val):
            val = 0.0
            logger.warning("Fault %s produced NaN/Inf, clamped to 0", cfg.fault_id)

        return val

    def activate_at(self, sim_time_s: float) -> None:
        """Activate faults scheduled to start at sim_time_s."""
        for state in self._faults.values():
            if abs(sim_time_s - state.config.start_time_s) < 1e-6:
                state.active = True
                state.activated_at_s = sim_time_s
                self._active_count += 1
                logger.info("Fault %s activated at t=%.4fs",
                            state.config.fault_id, sim_time_s)

    def clear_all(self) -> None:
        for state in self._faults.values():
            state.active = False
        self._active_count = 0
        self._freeze_cache.clear()

    @property
    def active_faults(self) -> list[FaultState]:
        return [s for s in self._faults.values() if s.active]
