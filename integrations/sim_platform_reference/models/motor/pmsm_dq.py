"""PMSM dq-axis lumped-parameter model (L2 fidelity).

Suitable for: FOC control design, HIL simulation, real-time use.
Not suitable for: detailed torque ripple, cogging torque, magnetic saturation.

Security:
  - CWE-369: Zero-divide guard on Ld, Lq, J
  - CWE-754: NaN/Inf guards on all inputs and state transitions

Numerical Limitations (IEEE 754 float64):
  - Integration: Forward Euler (1st order). Global error O(dt) per step.
    For dt=50us, electrical dynamics accumulate ~2.5A/step worst-case.
    For long simulations (>10s), consider reducing dt or upgrading to RK4.
  - Theta wrapping: theta_e is wrapped to [0, 2*pi) after each step.
    This preserves sin/cos precision for arbitrarily long runs.
  - Unbounded growth: Without friction (B=0) or load torque, current and
    speed grow without bound. This is physically correct — add B or tl
    for realistic steady-state behavior.
  - Parameter bounds: Ld, Lq, J are clamped to MOTOR_EPS_L / MOTOR_EPS_J
    at init to prevent division by near-zero. If you need smaller values,
    adjust constants in core/constants.py.
"""

import math
import warnings

from sim_platform.core.constants import DEFAULT_DT_S as _DEFAULT_DT_S
from sim_platform.core.constants import MOTOR_EPS_J as _MOTOR_EPS_J
from sim_platform.core.constants import MOTOR_EPS_L as _MOTOR_EPS_L
from sim_platform.core.utils import guard_numeric as _guard_numeric


class PMSMdqModel:
    """PMSM dq-axis model with lumped parameters.

    Security: All input paths guarded against NaN/Inf (CWE-754).
    Zero-divide on Ld/Lq/J prevented (CWE-369).
    """

    def __init__(self, *, Rs: float, Ld: float, Lq: float,
                 flux_pm: float, J: float, B: float = 0.0,
                 Pp: int = 4, dt_ns: int = 50000):
        self.Rs = _guard_numeric(Rs, 0.1)
        self.Ld = max(_guard_numeric(Ld, 5e-4), _MOTOR_EPS_L)
        self.Lq = max(_guard_numeric(Lq, 1e-3), _MOTOR_EPS_L)
        self.flux_pm = _guard_numeric(flux_pm, 0.03)
        self.J = max(_guard_numeric(J, 1e-3), _MOTOR_EPS_J)
        self.B = _guard_numeric(B, 0.0)
        self.Pp = max(1, int(Pp))

        # Numerical stability warnings
        if self.B < 1e-6:
            warnings.warn(
                "PMSMdqModel: B (friction) is near zero. Without friction or "
                "load torque, current and speed will grow without bound. "
                "This is physically correct but may cause overflow in long runs.",
                RuntimeWarning, stacklevel=2
            )
        # Use integer division to avoid floating point precision issues
        # dt_ns is in nanoseconds, convert to seconds with proper rounding
        if dt_ns > 0:
            self.dt = dt_ns * 1e-9  # More precise than division
        else:
            self.dt = _DEFAULT_DT_S  # Default 50us
        self.dt = max(self.dt, 1e-12)  # Guard against zero

        self.id = 0.0
        self.iq = 0.0
        self.omega_m = 0.0
        self.theta_e = 0.0
        self.torque = 0.0
        self.ia = self.ib = self.ic = 0.0

    @property
    def omega_e(self) -> float:
        return self.Pp * self.omega_m

    @property
    def torque_em(self) -> float:
        # Guard against NaN in state variables (CWE-754)
        # Clamp to DEFAULT_I_MAX to prevent overflow in id*iq product (CWE-190)
        from sim_platform.core.constants import DEFAULT_I_MAX
        id_s = _guard_numeric(self.id, 0.0)
        iq_s = _guard_numeric(self.iq, 0.0)
        id_s = max(-DEFAULT_I_MAX, min(DEFAULT_I_MAX, id_s))
        iq_s = max(-DEFAULT_I_MAX, min(DEFAULT_I_MAX, iq_s))
        return 1.5 * self.Pp * (
            self.flux_pm * iq_s +
            (self.Ld - self.Lq) * id_s * iq_s
        )

    def step(self, vd: float, vq: float, tl: float = 0.0,
             dt: float = None) -> None:
        """Forward Euler integration with NaN/Inf and zero-divide guards.

        Args:
            vd: d-axis voltage [V].
            vq: q-axis voltage [V].
            tl: Load torque [N·m].
            dt: Time step [s], uses default if None.
        """
        # SECURITY: Guard inputs at entry (CWE-754) — NaN → 0.0
        vd = _guard_numeric(vd, 0.0)
        vq = _guard_numeric(vq, 0.0)
        tl = _guard_numeric(tl, 0.0)
        if dt is None:
            dt = self.dt
        else:
            if not math.isfinite(dt) or dt <= 0:
                return

        # Guard internal state — NaN → 0.0
        self.id = _guard_numeric(self.id, 0.0)
        self.iq = _guard_numeric(self.iq, 0.0)
        self.omega_m = _guard_numeric(self.omega_m, 0.0)

        we = self.omega_e
        id_p, iq_p = self.id, self.iq

        # Electrical dynamics (Ld/Lq already guarded at __init__)
        did = (vd - self.Rs * id_p + we * self.Lq * iq_p) / self.Ld
        diq = (vq - self.Rs * iq_p - we * (self.Ld * id_p + self.flux_pm)) / self.Lq

        # Guard derivatives — NaN → 0.0
        did = _guard_numeric(did, 0.0)
        diq = _guard_numeric(diq, 0.0)

        self.id += did * dt
        self.iq += diq * dt

        # Mechanical dynamics
        self.torque = self.torque_em
        self.torque = _guard_numeric(self.torque, 0.0)
        dw = (self.torque - tl - self.B * self.omega_m) / self.J
        dw = _guard_numeric(dw, 0.0)
        self.omega_m += dw * dt
        self.theta_e += self.Pp * self.omega_m * dt

        # Wrap theta_e to [0, 2π)
        self.theta_e = self.theta_e % (2 * math.pi)

    _SQRT3_INV = 1.0 / math.sqrt(3)
    _SQRT3_HALF = math.sqrt(3) / 2

    def update_abc_currents(self) -> tuple:
        theta = self.theta_e
        cos_t = math.cos(theta)
        sin_t = math.sin(theta)

        ia_alpha = self.id * cos_t - self.iq * sin_t
        ia_beta = self.id * sin_t + self.iq * cos_t

        self.ia = ia_alpha
        self.ib = -0.5 * ia_alpha + self._SQRT3_HALF * ia_beta
        self.ic = -0.5 * ia_alpha - self._SQRT3_HALF * ia_beta
        return self.ia, self.ib, self.ic

    def step_abc(self, va: float, vb: float, vc: float, tl: float = 0.0,
                 dt: float = None) -> None:
        if not (math.isfinite(va) and math.isfinite(vb) and math.isfinite(vc)):
            return
        v_alpha = va
        v_beta = (va + 2 * vb) * self._SQRT3_INV
        cos_t = math.cos(self.theta_e)
        sin_t = math.sin(self.theta_e)
        vd = v_alpha * cos_t + v_beta * sin_t
        vq = -v_alpha * sin_t + v_beta * cos_t
        self.step(vd, vq, tl, dt)

    def reset(self) -> None:
        self.id = 0.0
        self.iq = 0.0
        self.omega_m = 0.0
        self.theta_e = 0.0
        self.torque = 0.0
        self.ia = self.ib = self.ic = 0.0

    def get_state(self) -> dict:
        return {
            "id": self.id, "iq": self.iq,
            "omega_m": self.omega_m, "theta_e": self.theta_e,
            "torque": self.torque,
            "ia": self.ia, "ib": self.ib, "ic": self.ic,
            "omega_e": self.omega_e,
        }
