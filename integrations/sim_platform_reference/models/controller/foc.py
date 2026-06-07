"""FOC (Field-Oriented Control) and PID controllers.

Implements: Clarke/Park transforms, PI current loop, PI speed loop,
SVPWM modulation, anti-windup.

Security:
  - CWE-754: NaN/Inf guards on all numeric entry points
  - CWE-369: Zero-divide guard in svpwm (v_bus near-zero)
  - CWE-20: Input validation on PI update

Numerical Limitations (IEEE 754 float64):
  - Clarke/Park transforms: pure trigonometry, machine precision (~1e-15).
    Roundtrip error is typically 0 (exact for balanced 3-phase systems).
  - SVPWM: duty cycles clamped to [0, 1]. Overmodulation uses linear
    scaling which preserves relative phase relationships.
  - PI controller: anti-windup via back-calculation. Integral term is
    bounded by [out_min, out_max]. For very small kp (<1e-12), anti-windup
    gain defaults to 1.0 to avoid division by zero.
  - v_bus: clamped to minimum PWM_EPS_V (0.1V) to prevent division by zero.
    If v_bus is near zero, SVPWM returns 50% duty (zero voltage output).
  - Hot-path optimization: guards are consolidated at entry points (FOC.update),
    not in individual transform functions. This is intentional for performance.
"""

import math

from sim_platform.core.constants import DEFAULT_DT_S as _DEFAULT_DT_S
from sim_platform.core.utils import guard_numeric as _guard_numeric

# ── Coordinate Transforms ────────────────────────────────────
# Re-exported from common module (single source of truth — eliminates duplication).
# Import paths kept for backward compatibility; new code should import directly from
# sim_platform.models.common.transforms.
from sim_platform.models.common.transforms import (
    clarke_transform,
    inverse_park,
    park_transform,
    svpwm,
)

# Precomputed constants (used by PIController reference below)
_SQRT3_INV = 1.0 / math.sqrt(3)
_SQRT3_HALF = math.sqrt(3) / 2


# ── PI Controller with Anti-Windup ───────────────────────────

class PIController:
    """Discrete-time PI controller with anti-windup.

    SECURITY (CWE-20): NaN/Inf guard on inputs (CWE-754).
    """

    def __init__(self, *, kp: float, ki: float, ts: float,
                 out_min: float = -float("inf"), out_max: float = float("inf"),
                 k_aw: float = None):
        self.kp = _guard_numeric(kp, 1.0)
        self.ki = _guard_numeric(ki, 10.0)
        self.ts = max(_guard_numeric(ts, 1e-6), 1e-12)

        # Ensure out_min < out_max
        if out_min > out_max:
            out_min, out_max = out_max, out_min
        self.out_min = out_min if math.isfinite(out_min) else -1e6
        self.out_max = out_max if math.isfinite(out_max) else 1e6

        # Anti-windup gain: guard against kp=0 (CWE-369)
        if k_aw is not None:
            self.k_aw = _guard_numeric(k_aw, 1.0)
        else:
            # Default: k_aw = ki / kp, but guard against kp=0
            if abs(self.kp) > 1e-12:
                self.k_aw = self.ki / self.kp
            else:
                self.k_aw = 1.0  # Safe fallback when kp is near zero

        self.reset()

    def reset(self) -> None:
        self.integral = 0.0
        self.prev_error = 0.0
        self.prev_output = 0.0
        self.saturated = False

    def update(self, setpoint: float, measurement: float) -> float:
        """Compute control output with NaN/Inf guards.

        Args:
            setpoint: Reference value.
            measurement: Measured value.

        Returns:
            Control output, clamped to [out_min, out_max].
        """
        # SECURITY: Guard inputs at entry (CWE-754)
        if not math.isfinite(setpoint) or not math.isfinite(measurement):
            return self.prev_output

        error = setpoint - measurement

        p_term = self.kp * error
        i_term = self.integral + self.ki * self.ts * error

        u_pre = p_term + i_term
        u = max(self.out_min, min(self.out_max, u_pre))
        # Use tolerance comparison for floating point (avoid direct ==)
        self.saturated = abs(u - u_pre) > 1e-12

        # Back-calculation anti-windup
        if self.saturated:
            self.integral = u - p_term
            self.integral = max(self.out_min, min(self.out_max, self.integral))
        else:
            self.integral = i_term

        self.prev_error = error
        self.prev_output = u
        return u


# ── FOC Current Controller ──────────────────────────────────

class FOCController:
    """Field-Oriented Control with dual PI current loops + SVPWM.

    SECURITY: NaN/Inf guards on all inputs.
    """

    def __init__(self, *, kp_id: float, ki_id: float,
                 kp_iq: float, ki_iq: float,
                 ts: float, v_bus: float = 48.0,
                 id_max: float = 100.0, iq_max: float = 200.0):
        # Guard initialization parameters
        kp_id = _guard_numeric(kp_id, 5.0)
        ki_id = _guard_numeric(ki_id, 500.0)
        kp_iq = _guard_numeric(kp_iq, 5.0)
        ki_iq = _guard_numeric(ki_iq, 500.0)
        ts = max(_guard_numeric(ts, _DEFAULT_DT_S), 1e-12)
        v_bus = max(_guard_numeric(v_bus, 48.0), 1.0)

        self.pi_id = PIController(kp=kp_id, ki=ki_id, ts=ts,
                                  out_min=-v_bus, out_max=v_bus)
        self.pi_iq = PIController(kp=kp_iq, ki=ki_iq, ts=ts,
                                  out_min=-v_bus, out_max=v_bus)
        self.v_bus = v_bus
        self.id_ref = 0.0
        self.iq_ref = 0.0
        self.vd_ref = 0.0
        self.vq_ref = 0.0
        self.duty_a = self.duty_b = self.duty_c = 0.5

    def update(self, ia: float, ib: float, ic: float, theta_e: float,
               id_ref: float, iq_ref: float) -> tuple:
        """Compute SVPWM duty cycles with input guarding."""
        # SECURITY: Guard inputs at entry (CWE-754) — single check per value
        if not (math.isfinite(ia) and math.isfinite(ib) and math.isfinite(ic)
                and math.isfinite(theta_e) and math.isfinite(id_ref) and math.isfinite(iq_ref)):
            return self.duty_a, self.duty_b, self.duty_c

        self.id_ref = id_ref
        self.iq_ref = iq_ref

        i_alpha, i_beta = clarke_transform(ia, ib, ic)
        id_meas, iq_meas = park_transform(i_alpha, i_beta, theta_e)

        self.vd_ref = self.pi_id.update(id_ref, id_meas)
        self.vq_ref = self.pi_iq.update(iq_ref, iq_meas)

        v_alpha, v_beta = inverse_park(self.vd_ref, self.vq_ref, theta_e)
        self.duty_a, self.duty_b, self.duty_c = svpwm(v_alpha, v_beta, self.v_bus)

        return self.duty_a, self.duty_b, self.duty_c

    def reset(self) -> None:
        self.pi_id.reset()
        self.pi_iq.reset()
        self.vd_ref = 0.0
        self.vq_ref = 0.0
        self.duty_a = self.duty_b = self.duty_c = 0.5


# ── Speed PI Controller ─────────────────────────────────────

class SpeedController:
    """Outer-loop PI speed controller producing iq_ref."""

    def __init__(self, *, kp: float, ki: float, ts: float,
                 iq_min: float = -200.0, iq_max: float = 200.0):
        self.pi = PIController(kp=kp, ki=ki, ts=ts,
                               out_min=iq_min, out_max=iq_max)

    def update(self, speed_ref: float, speed_meas: float) -> float:
        return self.pi.update(speed_ref, speed_meas)

    def reset(self) -> None:
        self.pi.reset()
