"""BLDC (Brushless DC) motor model with trapezoidal back-EMF.

Implements:
- Trapezoidal back-EMF waveform
- Six-step commutation logic
- Phase current dynamics
- Electromagnetic torque calculation
- Mechanical dynamics

Suitable for: BLDC motor control design, HIL simulation, real-time use.
Not suitable for: Detailed magnetic saturation, cogging torque analysis.

Security:
  - CWE-369: Zero-divide guard on L, J
  - CWE-754: NaN/Inf guards on all inputs and state transitions

Numerical Limitations (IEEE 754 float64):
  - Integration: Forward Euler (1st order). Same accuracy bounds as PMSM.
  - Theta wrapping: both theta_e and theta_m wrapped to [0, 2*pi) each step.
  - step() takes v_bus (single DC voltage), NOT phase voltages.
    The six-step commutation logic automatically generates phase voltages.
  - Trapezoidal back-EMF: uses piecewise linear approximation.
    Transition regions (60-degree sectors) have discontinuous derivatives
    which may cause small torque spikes at commutation boundaries.
  - Without friction (B=0) or load, speed grows without bound.
"""

import math
from enum import Enum

from sim_platform.core.constants import DEFAULT_DT_S as _DEFAULT_DT_S
from sim_platform.core.constants import DEFAULT_I_MAX as _DEFAULT_I_MAX
from sim_platform.core.constants import MOTOR_EPS_J as _MOTOR_EPS_J
from sim_platform.core.constants import MOTOR_EPS_L as _MOTOR_EPS_L
from sim_platform.core.utils import guard_numeric as _guard_numeric


class HallState(Enum):
    """Hall sensor states for six-step commutation."""
    H1_H2 = 0b101  # 5
    H1_H3 = 0b100  # 4
    H2_H3 = 0b110  # 6
    H2_H1 = 0b010  # 2
    H3_H1 = 0b011  # 3
    H3_H2 = 0b001  # 1


class CommutationState(Enum):
    """Six-step commutation states."""
    AB = 0   # Phase A+, B-
    AC = 1   # Phase A+, C-
    BC = 2   # Phase B+, C-
    BA = 3   # Phase B+, A-
    CA = 4   # Phase C+, A-
    CB = 5   # Phase C+, B-


# Mapping from Hall state to commutation state
HALL_TO_COMMUTATION = {
    HallState.H1_H2: CommutationState.AB,
    HallState.H1_H3: CommutationState.AC,
    HallState.H2_H3: CommutationState.BC,
    HallState.H2_H1: CommutationState.BA,
    HallState.H3_H1: CommutationState.CA,
    HallState.H3_H2: CommutationState.CB,
}


class BLDCModel:
    """BLDC motor model with trapezoidal back-EMF.

    Implements six-step commutation control with:
    - Trapezoidal back-EMF waveform
    - Phase current dynamics (L-R circuit)
    - Electromagnetic torque calculation
    - Mechanical dynamics (inertia + friction)

    Security: All input paths guarded against NaN/Inf (CWE-754).
    Zero-divide on L/J prevented (CWE-369).
    """

    def __init__(self, *, Rs: float, Ls: float, Ke: float, Kt: float,
                 J: float, B: float = 0.0, Pp: int = 1,
                 dt_ns: int = 50000):
        """
        Args:
            Rs: Phase resistance [Ω]
            Ls: Phase inductance [H]
            Ke: Back-EMF constant [V·s/rad]
            Kt: Torque constant [N·m/A]
            J: Rotor inertia [kg·m²]
            B: Viscous friction [N·m·s/rad]
            Pp: Pole pairs
            dt_ns: Default time step [ns]
        """
        # Guard parameters
        self.Rs = _guard_numeric(Rs, 0.1)
        self.Ls = max(_guard_numeric(Ls, 1e-3), _MOTOR_EPS_L)
        self.Ke = _guard_numeric(Ke, 0.01)
        self.Kt = _guard_numeric(Kt, 0.01)
        self.J = max(_guard_numeric(J, 1e-4), _MOTOR_EPS_J)
        self.B = _guard_numeric(B, 0.0)
        self.Pp = max(1, int(Pp))

        # Time step
        if dt_ns > 0:
            self.dt = dt_ns * 1e-9
        else:
            self.dt = _DEFAULT_DT_S
        self.dt = max(self.dt, 1e-12)

        # State variables
        self.ia = 0.0
        self.ib = 0.0
        self.ic = 0.0
        self.omega_m = 0.0      # Mechanical speed [rad/s]
        self.theta_e = 0.0      # Electrical angle [rad]
        self.theta_m = 0.0      # Mechanical angle [rad]
        self.torque = 0.0       # Electromagnetic torque [N·m]

        # Commutation state
        self._commutation_state = CommutationState.AB
        self._hall_state = HallState.H1_H2

        # Back-EMF shape parameters (trapezoidal)
        self._emf_flat_top = 120.0  # degrees (electrical)
        self._emf_slope = 60.0      # degrees transition region

    @property
    def omega_e(self) -> float:
        """Electrical angular velocity [rad/s]."""
        return self.Pp * self.omega_m

    @property
    def rpm(self) -> float:
        """Mechanical speed in RPM."""
        return self.omega_m * 60.0 / (2 * math.pi)

    @property
    def hall_state(self) -> HallState:
        """Current Hall sensor state based on electrical angle."""
        return self._get_hall_state(self.theta_e)

    def _get_hall_state(self, theta_e: float) -> HallState:
        """Determine Hall state from electrical angle.
        
        Each Hall sensor covers 180° electrical, offset by 120°.
        """
        # Normalize to [0, 2π)
        theta = theta_e % (2 * math.pi)
        theta_deg = math.degrees(theta)

        # Hall sensor thresholds (simplified model)
        # H1: 0-180°, H2: 120-300°, H3: 240-60° (wrapped)
        h1 = 0 <= theta_deg < 180
        h2 = 120 <= theta_deg < 300 or theta_deg < 60
        h3 = (240 <= theta_deg < 360) or (0 <= theta_deg < 60)

        # Convert to HallState enum
        hall_val = (h1 << 2) | (h2 << 1) | h3

        # Map to valid Hall states
        hall_mapping = {
            0b101: HallState.H1_H2,
            0b100: HallState.H1_H3,
            0b110: HallState.H2_H3,
            0b010: HallState.H2_H1,
            0b011: HallState.H3_H1,
            0b001: HallState.H3_H2,
        }

        return hall_mapping.get(hall_val, HallState.H1_H2)

    def _trapezoidal_emf(self, theta_e: float) -> tuple[float, float, float]:
        """Generate trapezoidal back-EMF waveform.
        
        Returns back-EMF coefficients for phases A, B, C in [-1, 1].
        """
        # Normalize angle to [0, 2π)
        theta = theta_e % (2 * math.pi)
        theta_deg = math.degrees(theta)

        def emf_shape(angle_deg: float) -> float:
            """Trapezoidal waveform: flat top, linear transitions."""
            # Normalize to [0, 360)
            a = angle_deg % 360.0

            # Flat top region (120°)
            if 0 <= a < 60:
                return a / 60.0  # Rising edge
            elif 60 <= a < 180:
                return 1.0  # Flat top
            elif 180 <= a < 240:
                return 1.0 - (a - 180) / 60.0  # Falling edge
            elif 240 <= a < 300:
                return -1.0  # Flat bottom
            else:  # 300-360
                return -1.0 + (a - 300) / 60.0  # Rising edge from bottom

        # Phase offsets: 0°, 120°, 240°
        ea = emf_shape(theta_deg)
        eb = emf_shape(theta_deg - 120.0)
        ec = emf_shape(theta_deg - 240.0)

        return ea, eb, ec

    def _get_phase_voltages(self, v_bus: float, direction: int = 1) -> tuple[float, float, float]:
        """Get phase voltages based on commutation state.
        
        Args:
            v_bus: DC bus voltage [V]
            direction: 1 for forward, -1 for reverse
            
        Returns:
            Phase voltages (va, vb, vc) [V]
        """
        # Commutation table: (active phases, voltage polarity)
        comm_table = {
            CommutationState.AB: (1, -1, 0),  # A+, B-
            CommutationState.AC: (1, 0, -1),  # A+, C-
            CommutationState.BC: (0, 1, -1),  # B+, C-
            CommutationState.BA: (-1, 1, 0),  # B+, A-
            CommutationState.CA: (-1, 0, 1),  # C+, A-
            CommutationState.CB: (0, -1, 1),  # C+, B-
        }

        polarity = comm_table.get(self._commutation_state, (0, 0, 0))
        va = polarity[0] * v_bus * 0.5 * direction
        vb = polarity[1] * v_bus * 0.5 * direction
        vc = polarity[2] * v_bus * 0.5 * direction

        return va, vb, vc

    def step(self, v_bus: float, tl: float = 0.0,
             dt: float = None, direction: int = 1) -> None:
        """Step the BLDC model with six-step commutation.

        Args:
            v_bus: DC bus voltage [V]
            tl: Load torque [N·m]
            dt: Time step [s]
            direction: 1 for forward, -1 for reverse
        """
        # Guard inputs
        v_bus = _guard_numeric(v_bus, 0.0)
        tl = _guard_numeric(tl, 0.0)
        if dt is None:
            dt = self.dt
        else:
            dt = max(_guard_numeric(dt, self.dt), 1e-12)

        # Get back-EMF coefficients
        ea, eb, ec = self._trapezoidal_emf(self.theta_e)

        # Back-EMF voltages
        v_emf_a = self.Ke * self.omega_e * ea
        v_emf_b = self.Ke * self.omega_e * eb
        v_emf_c = self.Ke * self.omega_e * ec

        # Get applied voltages from commutation
        va, vb, vc = self._get_phase_voltages(v_bus, direction)

        # Phase current dynamics: L * di/dt = v_applied - v_emf - R*i
        # Using Forward Euler
        dia = (va - v_emf_a - self.Rs * self.ia) / self.Ls
        dib = (vb - v_emf_b - self.Rs * self.ib) / self.Ls
        dic = (vc - v_emf_c - self.Rs * self.ic) / self.Ls

        # Guard derivatives
        dia = _guard_numeric(dia, 0.0)
        dib = _guard_numeric(dib, 0.0)
        dic = _guard_numeric(dic, 0.0)

        # Update currents
        self.ia += dia * dt
        self.ib += dib * dt
        self.ic += dic * dt

        # Guard currents (CWE-754: NaN/Inf)
        self.ia = _guard_numeric(self.ia, 0.0)
        self.ib = _guard_numeric(self.ib, 0.0)
        self.ic = _guard_numeric(self.ic, 0.0)

        # Clamp currents to prevent overflow in torque calculation (CWE-190)
        self.ia = max(-_DEFAULT_I_MAX, min(_DEFAULT_I_MAX, self.ia))
        self.ib = max(-_DEFAULT_I_MAX, min(_DEFAULT_I_MAX, self.ib))
        self.ic = max(-_DEFAULT_I_MAX, min(_DEFAULT_I_MAX, self.ic))

        # Electromagnetic torque: T = Kt * (ia*ea + ib*eb + ic*ec)
        self.torque = self.Kt * (self.ia * ea + self.ib * eb + self.ic * ec)
        self.torque = _guard_numeric(self.torque, 0.0)

        # Mechanical dynamics
        dw = (self.torque - tl - self.B * self.omega_m) / self.J
        dw = _guard_numeric(dw, 0.0)
        self.omega_m += dw * dt
        self.omega_m = _guard_numeric(self.omega_m, 0.0)

        # Update angles
        self.theta_e += self.Pp * self.omega_m * dt
        self.theta_m += self.omega_m * dt

        # Wrap angles
        self.theta_e = self.theta_e % (2 * math.pi)
        self.theta_m = self.theta_m % (2 * math.pi)

        # Update commutation state based on Hall sensors
        self._hall_state = self._get_hall_state(self.theta_e)
        self._commutation_state = HALL_TO_COMMUTATION.get(
            self._hall_state, CommutationState.AB)

    def step_with_hall(self, v_bus: float, hall_state: HallState,
                       tl: float = 0.0, dt: float = None) -> None:
        """Step with explicit Hall state (for sensor-based control).

        Args:
            v_bus: DC bus voltage [V]
            hall_state: Current Hall sensor state
            tl: Load torque [N·m]
            dt: Time step [s]
        """
        # Update commutation from Hall state
        self._hall_state = hall_state
        self._commutation_state = HALL_TO_COMMUTATION.get(
            hall_state, CommutationState.AB)

        # Step with normal dynamics
        self.step(v_bus, tl, dt)

    def reset(self) -> None:
        """Reset all state variables."""
        self.ia = 0.0
        self.ib = 0.0
        self.ic = 0.0
        self.omega_m = 0.0
        self.theta_e = 0.0
        self.theta_m = 0.0
        self.torque = 0.0
        self._commutation_state = CommutationState.AB
        self._hall_state = HallState.H1_H2

    def get_state(self) -> dict:
        """Get current state as dictionary."""
        return {
            "ia": self.ia,
            "ib": self.ib,
            "ic": self.ic,
            "omega_m": self.omega_m,
            "omega_e": self.omega_e,
            "theta_e": self.theta_e,
            "theta_m": self.theta_m,
            "torque": self.torque,
            "rpm": self.rpm,
            "hall_state": self._hall_state.value,
            "commutation_state": self._commutation_state.value,
        }

    def get_hall_sequence(self, num_poles: int = 1) -> list[HallState]:
        """Get expected Hall sequence for one mechanical revolution.
        
        Args:
            num_poles: Number of pole pairs
            
        Returns:
            List of Hall states for one revolution
        """
        sequence = []
        steps_per_rev = 6 * num_poles
        for i in range(steps_per_rev):
            theta_e = 2 * math.pi * i / steps_per_rev
            hall = self._get_hall_state(theta_e)
            sequence.append(hall)
        return sequence


class BLDCController:
    """Simple six-step commutation controller for BLDC motors.

    Implements:
    - Speed PI controller
    - Current limiting
    - Commutation sequencing
    """

    def __init__(self, *, kp_speed: float, ki_speed: float,
                 i_max: float = 10.0, v_bus: float = 24.0,
                 dt: float = 1e-3):
        """
        Args:
            kp_speed: Speed proportional gain
            ki_speed: Speed integral gain
            i_max: Maximum current [A]
            v_bus: DC bus voltage [V]
            dt: Control loop time step [s]
        """
        self.kp_speed = _guard_numeric(kp_speed, 0.1)
        self.ki_speed = _guard_numeric(ki_speed, 1.0)
        self.i_max = max(abs(_guard_numeric(i_max, 10.0)), 1e-6)
        self.v_bus = abs(_guard_numeric(v_bus, 24.0))
        self.dt = max(_guard_numeric(dt, 1e-3), 1e-6)

        # PI state
        self._speed_error_integral = 0.0
        self._output = 0.0

    def update(self, speed_ref: float, speed_meas: float) -> float:
        """Speed PI controller output.
        
        Args:
            speed_ref: Reference speed [rad/s]
            speed_meas: Measured speed [rad/s]
            
        Returns:
            Duty cycle [-1, 1] (direction + magnitude)
        """
        speed_ref = _guard_numeric(speed_ref, 0.0)
        speed_meas = _guard_numeric(speed_meas, 0.0)

        error = speed_ref - speed_meas

        # PI controller
        p_term = self.kp_speed * error
        self._speed_error_integral += error * self.dt
        i_term = self.ki_speed * self._speed_error_integral

        # Anti-windup
        self._speed_error_integral = max(-self.i_max, min(self.i_max,
                                         self._speed_error_integral))

        # Output
        output = p_term + i_term

        # Limit output to [-1, 1] for duty cycle
        self._output = max(-1.0, min(1.0, output / self.i_max))

        return self._output

    def reset(self) -> None:
        """Reset controller state."""
        self._speed_error_integral = 0.0
        self._output = 0.0
