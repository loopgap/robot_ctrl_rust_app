"""Induction Motor (IM) dq-axis lumped-parameter model (L2 fidelity).

Implements squirrel-cage induction motor model in synchronous rotating dq-frame.
Suitable for: Vector control (RFOC), HIL simulation, real-time use.
Not suitable for: Detailed slot harmonics, saturation effects, deep-bar effects.

Security:
  - CWE-369: Zero-divide guard on Ls, Lr, Lm, J
  - CWE-754: NaN/Inf guards on all inputs and state transitions
  - CWE-190: Overflow prevention on flux magnitude (psi_rd^2 + psi_rq^2)

Numerical Limitations (IEEE 754 float64):
  - Integration: Forward Euler (1st order). Same accuracy bounds as PMSM.
  - Theta wrapping: theta_e wrapped to [0, 2*pi) each step.
  - Rotor time constant: Tr = Lr/Rr. When Rr is very small (<1e-10),
    Tr approaches infinity and rotor dynamics freeze. This is physically
    correct (superconducting rotor) but may cause slow convergence.
  - Flux magnitude: psi_rd/psi_rq clamped to +/-1e100 before squaring
    to prevent OverflowError.
  - step() requires omega_e as explicit parameter (not auto-computed).
    Caller must pass omega_e = Pp * omega_m for correct coupling.
"""

import math

from sim_platform.core.constants import DEFAULT_DT_S as _DEFAULT_DT_S
from sim_platform.core.constants import MOTOR_EPS_J as _MOTOR_EPS_J
from sim_platform.core.constants import MOTOR_EPS_L as _MOTOR_EPS_L
from sim_platform.core.utils import guard_numeric as _guard_numeric


class IMdqModel:
    """Induction motor dq-axis model with lumped parameters.

    Implements:
    - Stator current dynamics (ids, iqs)
    - Rotor flux dynamics (ψrd, ψrq)
    - Electromagnetic torque calculation
    - Mechanical dynamics (speed, angle)

    Security: All input paths guarded against NaN/Inf (CWE-754).
    Zero-divide on inductances/inertia prevented (CWE-369).
    """

    def __init__(self, *, Rs: float, Rr: float, Ls: float, Lr: float,
                 Lm: float, J: float, B: float = 0.0,
                 Pp: int = 2, dt_ns: int = 50000):
        """
        Args:
            Rs: Stator resistance [Ω]
            Rr: Rotor resistance [Ω]
            Ls: Stator inductance [H]
            Lr: Rotor inductance [H]
            Lm: Mutual inductance [H]
            J: Rotor inertia [kg·m²]
            B: Viscous friction [N·m·s/rad]
            Pp: Pole pairs
            dt_ns: Default time step [ns]
        """
        # Guard parameters
        self.Rs = _guard_numeric(Rs, 0.5)
        self.Rr = _guard_numeric(Rr, 0.5)
        self.Ls = max(_guard_numeric(Ls, 0.01), _MOTOR_EPS_L)
        self.Lr = max(_guard_numeric(Lr, 0.01), _MOTOR_EPS_L)
        self.Lm = max(_guard_numeric(Lm, 0.009), _MOTOR_EPS_L)
        self.J = max(_guard_numeric(J, 0.01), _MOTOR_EPS_J)
        self.B = _guard_numeric(B, 0.0)
        self.Pp = max(1, int(Pp))

        # Time step
        if dt_ns > 0:
            self.dt = dt_ns * 1e-9
        else:
            self.dt = _DEFAULT_DT_S
        self.dt = max(self.dt, 1e-12)

        # Derived parameters (pre-computed for efficiency)
        self._update_derived_params()

        # State variables
        self.ids = 0.0      # d-axis stator current [A]
        self.iqs = 0.0      # q-axis stator current [A]
        self.psi_rd = 0.0   # d-axis rotor flux [Wb]
        self.psi_rq = 0.0   # q-axis rotor flux [Wb]
        self.omega_m = 0.0  # Mechanical speed [rad/s]
        self._omega_e = 0.0 # Synchronous electrical speed [rad/s]
        self.theta_e = 0.0  # Electrical angle [rad]
        self.torque = 0.0   # Electromagnetic torque [N·m]

        # Phase currents (for output)
        self.ia = 0.0
        self.ib = 0.0
        self.ic = 0.0

    def _update_derived_params(self) -> None:
        """Update derived parameters from motor constants."""
        # Leakage coefficient: σ = 1 - Lm²/(Ls*Lr)
        LsLr = self.Ls * self.Lr
        if LsLr > _MOTOR_EPS_L:
            self.sigma = max(0.0, min(1.0, 1.0 - (self.Lm ** 2) / LsLr))
        else:
            self.sigma = 0.5  # Safe fallback

        # Rotor time constant: Tr = Lr/Rr
        if self.Rr > 1e-12:
            self.Tr = self.Lr / self.Rr
        else:
            self.Tr = 1.0  # Safe fallback

        # Mutual inductance factor: Lm/Lr
        if self.Lr > _MOTOR_EPS_L:
            self.Lm_over_Lr = self.Lm / self.Lr
        else:
            self.Lm_over_Lr = 0.0

        # Stator transient inductance: σ*Ls
        self.sigma_Ls = self.sigma * self.Ls

    @property
    def omega_e(self) -> float:
        """Electrical angular velocity [rad/s]."""
        return self._omega_e if hasattr(self, '_omega_e') else self.Pp * self.omega_m

    @property
    def rpm(self) -> float:
        """Mechanical speed in RPM."""
        return self.omega_m * 60.0 / (2 * math.pi)

    @property
    def flux_rd_mag(self) -> float:
        """Rotor flux magnitude [Wb]."""
        # Clamp to prevent OverflowError on squaring (CWE-190)
        rd = max(min(self.psi_rd, 1e100), -1e100)
        rq = max(min(self.psi_rq, 1e100), -1e100)
        return math.sqrt(rd ** 2 + rq ** 2)

    @property
    def slip_freq(self) -> float:
        """Slip frequency [rad/s]."""
        # Slip = ωe - ωr
        return self.omega_e - self.Pp * self.omega_m

    @property
    def torque_em(self) -> float:
        """Electromagnetic torque [N·m].
        
        Te = 1.5 * Pp * (Lm/Lr) * (ψrd * iqs - ψrq * ids)
        """
        ids_s = _guard_numeric(self.ids, 0.0)
        iqs_s = _guard_numeric(self.iqs, 0.0)
        psi_rd_s = _guard_numeric(self.psi_rd, 0.0)
        psi_rq_s = _guard_numeric(self.psi_rq, 0.0)

        torque = 1.5 * self.Pp * self.Lm_over_Lr * (
            psi_rd_s * iqs_s - psi_rq_s * ids_s
        )
        return _guard_numeric(torque, 0.0)

    def step(self, vsd: float, vsq: float, omega_e: float,
             tl: float = 0.0, dt: float = None) -> None:
        """Step the induction motor model.

        Args:
            vsd: d-axis stator voltage [V]
            vsq: q-axis stator voltage [V]
            omega_e: Synchronous electrical angular velocity [rad/s]
            tl: Load torque [N·m]
            dt: Time step [s]
        """
        # Guard inputs
        vsd = _guard_numeric(vsd, 0.0)
        vsq = _guard_numeric(vsq, 0.0)
        omega_e = _guard_numeric(omega_e, 0.0)
        tl = _guard_numeric(tl, 0.0)
        if dt is None:
            dt = self.dt
        else:
            dt = max(_guard_numeric(dt, self.dt), 1e-12)

        # Store synchronous speed for slip calculation
        self._omega_e = omega_e

        # Pre-compute common terms
        omega_slip = omega_e - self.Pp * self.omega_m  # Slip frequency
        sigma_Ls = self.sigma_Ls
        Lm_Tr = self.Lm / self.Tr if self.Tr > 1e-12 else 0.0

        # Stator current dynamics (dq-frame)
        # σ*Ls * d(ids)/dt = vsd - Rs*ids + ωe*σ*Ls*iqs - (Lm/Tr)*dψrd/dt
        # σ*Ls * d(iqs)/dt = vsq - Rs*iqs - ωe*σ*Ls*ids - (Lm/Tr)*dψrq/dt

        # First compute rotor flux derivatives
        dpsi_rd = (self.Lm * self.ids - self.psi_rd) / self.Tr + omega_slip * self.psi_rq
        dpsi_rq = (self.Lm * self.iqs - self.psi_rq) / self.Tr - omega_slip * self.psi_rd

        # Guard derivatives
        dpsi_rd = _guard_numeric(dpsi_rd, 0.0)
        dpsi_rq = _guard_numeric(dpsi_rq, 0.0)

        # Stator current derivatives
        if sigma_Ls > _MOTOR_EPS_L:
            dids = (vsd - self.Rs * self.ids + omega_e * sigma_Ls * self.iqs
                    - Lm_Tr * dpsi_rd) / sigma_Ls
            diqs = (vsq - self.Rs * self.iqs - omega_e * sigma_Ls * self.ids
                    - Lm_Tr * dpsi_rq) / sigma_Ls
        else:
            dids = 0.0
            diqs = 0.0

        # Guard derivatives
        dids = _guard_numeric(dids, 0.0)
        diqs = _guard_numeric(diqs, 0.0)

        # Update stator currents (Forward Euler)
        self.ids += dids * dt
        self.iqs += diqs * dt

        # Guard currents
        self.ids = _guard_numeric(self.ids, 0.0)
        self.iqs = _guard_numeric(self.iqs, 0.0)

        # Update rotor fluxes
        self.psi_rd += dpsi_rd * dt
        self.psi_rq += dpsi_rq * dt

        # Guard fluxes
        self.psi_rd = _guard_numeric(self.psi_rd, 0.0)
        self.psi_rq = _guard_numeric(self.psi_rq, 0.0)

        # Electromagnetic torque
        self.torque = self.torque_em
        self.torque = _guard_numeric(self.torque, 0.0)

        # Mechanical dynamics
        dw = (self.torque - tl - self.B * self.omega_m) / self.J
        dw = _guard_numeric(dw, 0.0)
        self.omega_m += dw * dt
        self.omega_m = _guard_numeric(self.omega_m, 0.0)

        # Update electrical angle
        self.theta_e += omega_e * dt
        self.theta_e = self.theta_e % (2 * math.pi)

    def step_abc(self, va: float, vb: float, vc: float,
                 omega_e: float, tl: float = 0.0, dt: float = None) -> None:
        """Step with abc phase voltages (auto-transformed to dq).

        Args:
            va, vb, vc: Phase voltages [V]
            omega_e: Synchronous electrical angular velocity [rad/s]
            tl: Load torque [N·m]
            dt: Time step [s]
        """
        # Guard inputs
        va = _guard_numeric(va, 0.0)
        vb = _guard_numeric(vb, 0.0)
        vc = _guard_numeric(vc, 0.0)

        # Clarke transform: abc → αβ
        v_alpha = va
        v_beta = (va + 2 * vb) / math.sqrt(3)

        # Park transform: αβ → dq
        cos_t = math.cos(self.theta_e)
        sin_t = math.sin(self.theta_e)
        vsd = v_alpha * cos_t + v_beta * sin_t
        vsq = -v_alpha * sin_t + v_beta * cos_t

        # Step with dq voltages
        self.step(vsd, vsq, omega_e, tl, dt)

    def update_abc_currents(self) -> tuple[float, float, float]:
        """Update and return abc phase currents from dq currents."""
        theta = self.theta_e
        cos_t = math.cos(theta)
        sin_t = math.sin(theta)

        # Inverse Park: dq → αβ
        i_alpha = self.ids * cos_t - self.iqs * sin_t
        i_beta = self.ids * sin_t + self.iqs * cos_t

        # Inverse Clarke: αβ → abc
        self.ia = _guard_numeric(i_alpha, 0.0)
        self.ib = _guard_numeric(-0.5 * i_alpha + math.sqrt(3) / 2 * i_beta, 0.0)
        self.ic = _guard_numeric(-0.5 * i_alpha - math.sqrt(3) / 2 * i_beta, 0.0)

        return self.ia, self.ib, self.ic

    def set_flux_reference(self, flux_ref: float) -> None:
        """Set rotor flux reference (for field-oriented control).
        
        This initializes the rotor flux to the reference value.
        In steady-state, ψrd ≈ Lm * ids_ref
        
        Args:
            flux_ref: Desired rotor flux magnitude [Wb]
        """
        flux_ref = abs(_guard_numeric(flux_ref, 0.0))
        self.psi_rd = flux_ref
        self.psi_rq = 0.0

    def get_state(self) -> dict:
        """Get current state as dictionary."""
        self.update_abc_currents()
        return {
            "ids": self.ids,
            "iqs": self.iqs,
            "psi_rd": self.psi_rd,
            "psi_rq": self.psi_rq,
            "flux_mag": self.flux_rd_mag,
            "omega_m": self.omega_m,
            "omega_e": self.omega_e,
            "theta_e": self.theta_e,
            "torque": self.torque,
            "rpm": self.rpm,
            "slip_freq": self.slip_freq,
            "ia": self.ia,
            "ib": self.ib,
            "ic": self.ic,
        }

    def reset(self) -> None:
        """Reset all state variables."""
        self.ids = 0.0
        self.iqs = 0.0
        self.psi_rd = 0.0
        self.psi_rq = 0.0
        self.omega_m = 0.0
        self._omega_e = 0.0
        self.theta_e = 0.0
        self.torque = 0.0
        self.ia = 0.0
        self.ib = 0.0
        self.ic = 0.0


class IMVectorController:
    """Induction motor vector controller (Rotor Flux Oriented Control - RFOC).

    Implements:
    - Rotor flux estimation
    - Slip frequency calculation
    - Current control (d-axis: flux, q-axis: torque)
    - Speed control
    """

    def __init__(self, *, motor: IMdqModel,
                 kp_flux: float = 5.0, ki_flux: float = 500.0,
                 kp_torque: float = 5.0, ki_torque: float = 500.0,
                 kp_speed: float = 0.1, ki_speed: float = 1.0,
                 ts: float = _DEFAULT_DT_S):
        """
        Args:
            motor: Induction motor model
            kp_flux: Flux loop proportional gain
            ki_flux: Flux loop integral gain
            kp_torque: Torque loop proportional gain
            ki_torque: Torque loop integral gain
            kp_speed: Speed loop proportional gain
            ki_speed: Speed loop integral gain
            ts: Control loop time step [s]
        """
        self.motor = motor

        # Flux loop (d-axis current control)
        self.kp_flux = _guard_numeric(kp_flux, 5.0)
        self.ki_flux = _guard_numeric(ki_flux, 500.0)
        self._flux_integral = 0.0

        # Torque loop (q-axis current control)
        self.kp_torque = _guard_numeric(kp_torque, 5.0)
        self.ki_torque = _guard_numeric(ki_torque, 500.0)
        self._torque_integral = 0.0

        # Speed loop
        self.kp_speed = _guard_numeric(kp_speed, 0.1)
        self.ki_speed = _guard_numeric(ki_speed, 1.0)
        self._speed_integral = 0.0

        # Time step
        self.ts = max(_guard_numeric(ts, _DEFAULT_DT_S), 1e-6)

        # Outputs
        self.vsd_ref = 0.0
        self.vsq_ref = 0.0
        self.iq_ref = 0.0
        self.omega_slip = 0.0

    def _pi_controller(self, error: float, kp: float, ki: float,
                       integral: float, limit: float) -> tuple[float, float]:
        """PI controller with anti-windup."""
        error = _guard_numeric(error, 0.0)
        p_term = kp * error
        integral += error * self.ts
        integral = max(-limit, min(limit, integral))
        i_term = ki * integral
        output = p_term + i_term
        output = max(-limit, min(limit, output))
        return output, integral

    def update_speed(self, speed_ref: float, speed_meas: float,
                     flux_ref: float) -> tuple[float, float]:
        """Speed control loop.

        Args:
            speed_ref: Reference speed [rad/s]
            speed_meas: Measured speed [rad/s]
            flux_ref: Rotor flux reference [Wb]

        Returns:
            Tuple of (vsd_ref, vsq_ref) [V]
        """
        speed_ref = _guard_numeric(speed_ref, 0.0)
        speed_meas = _guard_numeric(speed_meas, 0.0)
        flux_ref = abs(_guard_numeric(flux_ref, 0.1))

        # Speed PI controller → q-axis current reference
        speed_error = speed_ref - speed_meas
        self.iq_ref, self._speed_integral = self._pi_controller(
            speed_error, self.kp_speed, self.ki_speed,
            self._speed_integral, limit=100.0
        )

        # Flux controller → d-axis current reference
        # ids_ref = flux_ref / Lm (in steady state)
        ids_ref = flux_ref / self.motor.Lm if self.motor.Lm > _MOTOR_EPS_L else 0.0

        # Current controllers
        vsd_ref, self._flux_integral = self._pi_controller(
            ids_ref - self.motor.ids,
            self.kp_flux, self.ki_flux,
            self._flux_integral, limit=200.0
        )

        vsq_ref, self._torque_integral = self._pi_controller(
            self.iq_ref - self.motor.iqs,
            self.kp_torque, self.ki_torque,
            self._torque_integral, limit=200.0
        )

        # Slip frequency calculation: ωslip = (Lm/Tr) * (iqs/ψrd)
        if self.motor.psi_rd > 1e-6 and self.motor.Tr > 1e-12:
            self.omega_slip = (self.motor.Lm / self.motor.Tr) * (
                self.motor.iqs / self.motor.psi_rd
            )
        else:
            self.omega_slip = 0.0

        # Synchronous frequency: ωe = ωr + ωslip
        omega_e = self.motor.Pp * self.motor.omega_m + self.omega_slip

        self.vsd_ref = vsd_ref
        self.vsq_ref = vsq_ref

        return vsd_ref, vsq_ref, omega_e

    def reset(self) -> None:
        """Reset controller state."""
        self._flux_integral = 0.0
        self._torque_integral = 0.0
        self._speed_integral = 0.0
        self.vsd_ref = 0.0
        self.vsq_ref = 0.0
        self.iq_ref = 0.0
        self.omega_slip = 0.0
