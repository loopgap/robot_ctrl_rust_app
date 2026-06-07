"""Global simulation clock module.

Provides a unified time base for all solvers, supporting both
offline (free-running) and realtime (wall-clock constrained) modes.

Security:
  - CWE-754: Busy-wait timeout guard (DoS prevention)
  - CWE-682: s_to_ns uses round() not int() to prevent truncation drift
  - CWE-20: Input validation on advance(), restore(), and mode changes
  - CWE-400: Sleep-based wait with sub-ms spin instead of full busy-wait
"""

import logging
import math
import time
from dataclasses import dataclass, field
from enum import Enum

logger = logging.getLogger(__name__)

NS_PER_SEC = 1_000_000_000


class ClockMode(Enum):
    OFFLINE = "OFFLINE"
    REALTIME = "REALTIME"
    HIL = "HIL"


@dataclass
class ClockState:
    """Snapshot of clock state for checkpoint/restore."""
    sim_time_ns: int = 0
    wall_time_base_ns: int = 0
    step_count: int = 0
    mode: ClockMode = ClockMode.OFFLINE


@dataclass
class GlobalClock:
    """Unified global simulation clock with nanosecond resolution.

    Attributes:
        resolution_ns: Clock resolution in nanoseconds (default 1).
        mode: OFFLINE (free-running) or REALTIME (wall-clock constrained) or HIL.
        sim_time_ns: Current simulation time in nanoseconds.
        step_count: Total simulation steps executed.
        realtime_factor: sim_time / wall_time ratio (>1 = faster than real).
    """

    resolution_ns: int = 1
    mode: ClockMode = ClockMode.OFFLINE
    sim_time_ns: int = 0
    step_count: int = 0
    realtime_factor: float = 0.0

    _wall_start_ns: int = field(default=0, repr=False)
    _last_wall_ns: int = field(default=0, repr=False)
    _paused: bool = field(default=False, repr=False)
    _diverged: bool = field(default=False, repr=False)

    def __post_init__(self):
        self._wall_start_ns = time.time_ns()

    # ── time queries ──────────────────────────────────────────

    @property
    def sim_time_s(self) -> float:
        """Simulation time in seconds."""
        return self.sim_time_ns / NS_PER_SEC

    @property
    def paused(self) -> bool:
        return self._paused

    @property
    def diverged(self) -> bool:
        return self._diverged

    # ── core operations ───────────────────────────────────────

    def advance(self, dt_ns: int) -> None:
        """Advance simulation by `dt_ns` nanoseconds.

        In REALTIME/HIL mode, blocks until wall clock catches up.

        SECURITY (M-05, CWE-20): Type and range validation.
        """
        if not isinstance(dt_ns, int):
            raise TypeError(f"dt_ns must be int, got {type(dt_ns).__name__}")
        if dt_ns < 0:
            raise ValueError(f"dt_ns must be non-negative, got {dt_ns}")
        if self._paused:
            return
        self.sim_time_ns += dt_ns
        self.step_count += 1
        if self.mode in (ClockMode.REALTIME, ClockMode.HIL):
            self._sync_wallclock(dt_ns)

    def _sync_wallclock(self, dt_ns: int) -> None:
        """Wait for wall clock to keep pace with simulation.

        SECURITY:
          - CWE-754: Maximum wait = 10 × dt_ns to prevent DoS.
          - CWE-400 (M-07): Sleep-based wait with sub-ms spin for precision.
        """
        target_wall = self._wall_start_ns + self.sim_time_ns
        now = time.time_ns()
        if now >= target_wall:
            self._update_rt_factor()
            return

        sleep_ns = target_wall - now
        max_wait_ns = dt_ns * 10  # never wait > 10× step time
        if sleep_ns > max_wait_ns:
            logger.warning("Clock sync: required sleep=%dns > max_wait=%dns, "
                           "falling back to OFFLINE", sleep_ns, max_wait_ns)
            self.mode = ClockMode.OFFLINE
            self._update_rt_factor()
            return

        # M-07: Sleep for bulk, spin for sub-ms precision
        sleep_s = max(0, (sleep_ns - 1_000_000)) / 1e9  # wake 1ms early
        if sleep_s > 0:
            time.sleep(sleep_s)

        # Final spin for sub-millisecond precision (bounded by max_wait)
        timeout = now + max_wait_ns
        while time.time_ns() < target_wall:
            if time.time_ns() > timeout:
                logger.warning("Clock sync timeout exceeded, falling back")
                self.mode = ClockMode.OFFLINE
                break
        self._update_rt_factor()

    def _update_rt_factor(self) -> None:
        now = time.time_ns()
        elapsed_wall = now - self._wall_start_ns
        if elapsed_wall > 0:
            raw_factor = self.sim_time_ns / elapsed_wall
            # L-09: Cap realtime_factor to prevent extreme values
            self.realtime_factor = min(raw_factor, 1000.0)
        self._last_wall_ns = now

    def pause(self) -> None:
        self._paused = True

    def resume(self) -> None:
        """Resume clock. L-08: No-op if not paused to prevent time discontinuity."""
        if not self._paused:
            return
        self._paused = False
        if self.mode in (ClockMode.REALTIME, ClockMode.HIL):
            self._wall_start_ns = time.time_ns() - self.sim_time_ns

    def mark_diverged(self) -> None:
        self._diverged = True

    def reset(self) -> None:
        """Reset clock to initial state."""
        self.sim_time_ns = 0
        self.step_count = 0
        self._paused = False
        self._diverged = False
        self._wall_start_ns = time.time_ns()

    # ── checkpoint ────────────────────────────────────────────

    def snapshot(self) -> ClockState:
        return ClockState(
            sim_time_ns=self.sim_time_ns,
            wall_time_base_ns=self._wall_start_ns,
            step_count=self.step_count,
            mode=self.mode,
        )

    def restore(self, state: ClockState) -> None:
        """Restore clock state from checkpoint.

        SECURITY (M-06, CWE-20): Validate checkpoint integrity.
        """
        if not isinstance(state, ClockState):
            raise TypeError(f"Expected ClockState, got {type(state).__name__}")
        if state.sim_time_ns < 0:
            raise ValueError(f"Invalid checkpoint: sim_time_ns={state.sim_time_ns} < 0")
        if state.step_count < 0:
            raise ValueError(f"Invalid checkpoint: step_count={state.step_count} < 0")
        if state.mode not in ClockMode:
            raise ValueError(f"Invalid checkpoint: unknown mode={state.mode}")
        self.sim_time_ns = state.sim_time_ns
        self._wall_start_ns = state.wall_time_base_ns
        self.step_count = state.step_count
        self.mode = state.mode


def ns_to_s(ns: int) -> float:
    return ns / NS_PER_SEC


def s_to_ns(s: float) -> int:
    """Convert seconds to nanoseconds with proper rounding (CWE-682)."""
    # Guard NaN/Inf (CWE-754)
    if math.isnan(s) or math.isinf(s):
        # L-07: Log warning instead of silent return
        logger.warning("s_to_ns: NaN/Inf input, returning 0")
        return 0
    return int(round(s * NS_PER_SEC))
