"""Advanced PMSM model with saturation, temperature, and iron loss.

Extends PMSMdqModel with:
- Magnetic saturation (Ld, Lq vs current)
- Temperature-dependent resistance
- Iron loss model
- Detailed torque calculation

Security:
  - CWE-369: Zero-divide guard on all denominators
  - CWE-754: NaN/Inf guards on all inputs and state transitions
  - CWE-190: Overflow prevention on current squaring (id^2 + iq^2)

Numerical Limitations (IEEE 754 float64):
  - Current squaring: id/iq are clamped to +/-DEFAULT_I_MAX (200A) before
    squaring to prevent OverflowError. If your motor exceeds 200A, adjust
    DEFAULT_I_MAX in core/constants.py.
  - Iron loss: frequency clamped to 1e6 Hz, B_peak clamped to 10T.
  - Mechanical loss: omega_m clamped to +/-1e6 rad/s.
  - Saturation exp(): exp(-i_mag/I_sat) may underflow to 0 for very large
    i_mag/I_sat ratios. This is physically correct (full saturation).
  - Thermal coupling: temperature-dependent Rs may amplify numerical errors
    if thermal model drifts. Monitor T_winding in long runs.
"""

import math

from sim_platform.core.constants import DEFAULT_I_MAX as _I_MAX
from sim_platform.core.constants import MOTOR_EPS_L as _MOTOR_EPS_L
from sim_platform.core.utils import guard_numeric as _guard_numeric

from .pmsm_dq import PMSMdqModel


class PMSMAdvanced(PMSMdqModel):
    """Advanced PMSM model with saturation and temperature effects.

    Extends PMSMdqModel with:
    - Magnetic saturation: Ld(id, iq), Lq(id, iq)
    - Temperature: Rs(T), flux_pm(T)
    - Iron loss: P_iron = kh*f*B^2 + ke*f^2*B^2
    - Detailed torque: T = 1.5*Pp*(flux_pm*iq + (Ld-Lq)*id*iq)

    Security: All input paths guarded against NaN/Inf (CWE-754).
    """

    def __init__(self, *, Rs: float, Ld: float, Lq: float,
                 flux_pm: float, J: float, B: float = 0.0,
                 Pp: int = 4, dt_ns: int = 50000,
                 # Saturation parameters
                 Ld_sat: float = 0.0, Lq_sat: float = 0.0,
                 I_sat: float = 10.0,
                 # Temperature parameters
                 Rs_temp_coeff: float = 0.004,  # [1/°C]
                 T_ref: float = 25.0,  # [°C]
                 # Iron loss parameters
                 kh: float = 0.0,  # Hysteresis loss coefficient
                 ke: float = 0.0,  # Eddy current loss coefficient
                 alpha: float = 2.0,  # Steinmetz exponent
                 beta: float = 2.0):  # Frequency exponent
        """
        Args:
            Rs: Stator resistance at T_ref [Ω]
            Ld: d-axis inductance at zero current [H]
            Lq: q-axis inductance at zero current [H]
            flux_pm: PM flux linkage at T_ref [Wb]
            J: Rotor inertia [kg·m²]
            B: Viscous friction [N·m·s/rad]
            Pp: Pole pairs
            dt_ns: Default time step [ns]
            Ld_sat: d-axis saturation inductance [H]
            Lq_sat: q-axis saturation inductance [H]
            I_sat: Saturation current [A]
            Rs_temp_coeff: Resistance temperature coefficient [1/°C]
            T_ref: Reference temperature [°C]
            kh: Hysteresis loss coefficient
            ke: Eddy current loss coefficient
            alpha: Steinmetz exponent
            beta: Frequency exponent
        """
        # Initialize base model
        super().__init__(Rs=Rs, Ld=Ld, Lq=Lq, flux_pm=flux_pm,
                         J=J, B=B, Pp=Pp, dt_ns=dt_ns)

        # Saturation parameters
        self.Ld0 = Ld  # Unsaturated inductance
        self.Lq0 = Lq
        self.Ld_sat = _guard_numeric(Ld_sat, Ld * 0.7)  # Saturated inductance
        self.Lq_sat = _guard_numeric(Lq_sat, Lq * 0.7)
        self.I_sat = abs(_guard_numeric(I_sat, 10.0))

        # Temperature parameters
        self.Rs0 = Rs  # Resistance at T_ref
        self.Rs_temp_coeff = _guard_numeric(Rs_temp_coeff, 0.004)
        self.T_ref = _guard_numeric(T_ref, 25.0)
        self.winding_temp = self.T_ref  # Current temperature

        # Iron loss parameters
        self.kh = _guard_numeric(kh, 0.0)
        self.ke = _guard_numeric(ke, 0.0)
        self.alpha = _guard_numeric(alpha, 2.0)
        self.beta = _guard_numeric(beta, 2.0)

        # Loss tracking
        self.copper_loss = 0.0
        self.iron_loss = 0.0
        self.mechanical_loss = 0.0
        self.total_loss = 0.0

    def _get_saturated_inductance(self, id_val: float, iq_val: float) -> tuple[float, float]:
        """Calculate saturated inductances based on current magnitude.
        
        Uses exponential saturation model:
        L = L_sat + (L0 - L_sat) * exp(-|I|/I_sat)
        """
        # Clamp to prevent OverflowError on squaring (CWE-190)
        id_c = max(min(id_val, _I_MAX), -_I_MAX)
        iq_c = max(min(iq_val, _I_MAX), -_I_MAX)
        i_mag = math.sqrt(id_c**2 + iq_c**2)

        # Exponential saturation
        if self.I_sat > 1e-12:
            sat_factor = math.exp(-i_mag / self.I_sat)
        else:
            sat_factor = 0.0

        # Saturated inductances
        Ld = self.Ld_sat + (self.Ld0 - self.Ld_sat) * sat_factor
        Lq = self.Lq_sat + (self.Lq0 - self.Lq_sat) * sat_factor

        # Guard against zero
        Ld = max(Ld, _MOTOR_EPS_L)
        Lq = max(Lq, _MOTOR_EPS_L)

        return Ld, Lq

    def _get_temperature_resistance(self) -> float:
        """Calculate resistance at current temperature.
        
        Rs(T) = Rs0 * (1 + alpha*(T - T_ref))
        """
        delta_T = self.winding_temp - self.T_ref
        Rs = self.Rs0 * (1.0 + self.Rs_temp_coeff * delta_T)
        return max(Rs, 1e-6)  # Guard against zero

    def _calculate_iron_loss(self, freq: float, B_peak: float) -> float:
        """Calculate iron loss using Steinmetz equation.
        
        P_iron = kh * f^alpha * B^beta + ke * f^2 * B^2
        """
        # Guard inputs
        freq = abs(_guard_numeric(freq, 0.0))
        B_peak = abs(_guard_numeric(B_peak, 0.0))

        # Clamp to prevent OverflowError on exponentiation (CWE-190)
        freq = min(freq, 1e6)      # Max 1 MHz
        B_peak = min(B_peak, 10.0) # Max 10 T

        # Hysteresis loss
        if freq > 0 and B_peak > 0:
            hysteresis = self.kh * (freq ** self.alpha) * (B_peak ** self.beta)
        else:
            hysteresis = 0.0

        # Eddy current loss
        eddy = self.ke * (freq ** 2) * (B_peak ** 2)

        return _guard_numeric(hysteresis + eddy, 0.0)

    @property
    def torque_em(self) -> float:
        """Electromagnetic torque with saturation effects."""
        id_s = _guard_numeric(self.id, 0.0)
        iq_s = _guard_numeric(self.iq, 0.0)

        # Get saturated inductances
        Ld, Lq = self._get_saturated_inductance(id_s, iq_s)

        # Torque calculation
        torque = 1.5 * self.Pp * (
            self.flux_pm * iq_s +
            (Ld - Lq) * id_s * iq_s
        )

        return _guard_numeric(torque, 0.0)

    def step(self, vd: float, vq: float, tl: float = 0.0,
             dt: float = None, winding_temp: float = None) -> None:
        """Step with advanced effects.

        Args:
            vd: d-axis voltage [V]
            vq: q-axis voltage [V]
            tl: Load torque [N·m]
            dt: Time step [s]
            winding_temp: Winding temperature [°C] (None = use current)
        """
        # Guard inputs
        vd = _guard_numeric(vd, 0.0)
        vq = _guard_numeric(vq, 0.0)
        tl = _guard_numeric(tl, 0.0)
        if dt is None:
            dt = self.dt
        else:
            dt = max(_guard_numeric(dt, self.dt), 1e-12)

        # Update temperature if provided
        if winding_temp is not None:
            self.winding_temp = _guard_numeric(winding_temp, self.T_ref)

        # Get temperature-dependent resistance
        Rs = self._get_temperature_resistance()

        # Get saturated inductances
        Ld, Lq = self._get_saturated_inductance(self.id, self.iq)

        we = self.omega_e
        id_p, iq_p = self.id, self.iq

        # Electrical dynamics with saturation
        did = (vd - Rs * id_p + we * Lq * iq_p) / Ld
        diq = (vq - Rs * iq_p - we * (Ld * id_p + self.flux_pm)) / Lq

        # Guard derivatives
        did = _guard_numeric(did, 0.0)
        diq = _guard_numeric(diq, 0.0)

        self.id += did * dt
        self.iq += diq * dt

        # Guard state
        self.id = _guard_numeric(self.id, 0.0)
        self.iq = _guard_numeric(self.iq, 0.0)

        # Clamp currents before squaring to prevent OverflowError (CWE-190)
        id_c = max(min(self.id, _I_MAX), -_I_MAX)
        iq_c = max(min(self.iq, _I_MAX), -_I_MAX)

        # Calculate losses
        self.copper_loss = Rs * (id_c**2 + iq_c**2)

        # Iron loss (simplified)
        freq = abs(self.omega_e) / (2 * math.pi)
        B_peak = abs(self.flux_pm)  # Simplified
        self.iron_loss = self._calculate_iron_loss(freq, B_peak)

        # Mechanical loss (clamp omega to prevent overflow)
        omega_c = max(min(self.omega_m, 1e6), -1e6)
        self.mechanical_loss = self.B * omega_c**2

        # Total loss
        self.total_loss = _guard_numeric(
            self.copper_loss + self.iron_loss + self.mechanical_loss, 0.0)

        # Torque calculation
        self.torque = self.torque_em
        self.torque = _guard_numeric(self.torque, 0.0)

        # Mechanical dynamics
        dw = (self.torque - tl - self.B * self.omega_m) / self.J
        dw = _guard_numeric(dw, 0.0)
        self.omega_m += dw * dt
        self.omega_m = _guard_numeric(self.omega_m, 0.0)

        # Update angles
        self.theta_e += self.Pp * self.omega_m * dt
        self.theta_e = self.theta_e % (2 * math.pi)

    def get_state(self) -> dict:
        """Get current state including advanced parameters."""
        state = super().get_state()
        state.update({
            "winding_temp": self.winding_temp,
            "Rs_effective": self._get_temperature_resistance(),
            "Ld_effective": self._get_saturated_inductance(self.id, self.iq)[0],
            "Lq_effective": self._get_saturated_inductance(self.id, self.iq)[1],
            "copper_loss": self.copper_loss,
            "iron_loss": self.iron_loss,
            "mechanical_loss": self.mechanical_loss,
            "total_loss": self.total_loss,
        })
        return state

    def get_efficiency(self, v_bus: float) -> float:
        """Calculate efficiency.
        
        Args:
            v_bus: DC bus voltage [V]
            
        Returns:
            Efficiency [0, 1]
        """
        v_bus = abs(_guard_numeric(v_bus, 1.0))

        # Input power (simplified)
        P_in = v_bus * (abs(self.ia) + abs(self.ib) + abs(self.ic))

        # Output power
        P_out = self.torque * self.omega_m

        # Efficiency
        if P_in > 1e-6:
            efficiency = P_out / P_in
            return max(0.0, min(1.0, efficiency))
        else:
            return 0.0

    def reset(self) -> None:
        """Reset all state variables."""
        super().reset()
        self.winding_temp = self.T_ref
        self.copper_loss = 0.0
        self.iron_loss = 0.0
        self.mechanical_loss = 0.0
        self.total_loss = 0.0
