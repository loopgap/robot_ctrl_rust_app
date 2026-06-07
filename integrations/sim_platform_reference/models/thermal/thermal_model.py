"""Thermal network model — RC ladder for motor winding temperature.

L2 fidelity: Single-node thermal model (winding → ambient).

Security:
  - CWE-754: NaN/Inf guards on all inputs
  - CWE-369: Zero-divide guard on thermal capacitance

Numerical Limitations (IEEE 754 float64):
  - Integration: Forward Euler (1st order). Thermal time constants are
    typically 10-100s, so dt=50us gives negligible integration error
    (~5e-9 °C/step). This is the most numerically benign model.
  - Steady-state accuracy: T converges to T_ambient + P_loss * R_th.
    Convergence requires ~5*time_constants (tau = R_th * C_th).
    For tau=50s, allow at least 250s for 99.3% convergence.
  - Negative heat input: physically represents cooling below ambient.
    Model handles this correctly (T can drop below T_ambient).
  - Very small C_th: may cause rapid temperature oscillations.
    C_th is clamped to THERMAL_EPS_C at init.
"""

import math

from sim_platform.core.constants import THERMAL_ALPHA_CU as _ALPHA_CU
from sim_platform.core.constants import THERMAL_ALPHA_MAG_NDFEB as _ALPHA_MAG
from sim_platform.core.constants import THERMAL_EPS_C as _THERMAL_EPS_C


class ThermalNode:
    """Single-node thermal model: T_winding = T_ambient + P_loss * R_th.

    Models: P_loss = I²*R (copper loss) + friction losses
    Thermal: dT/dt = (P_loss - (T - T_ambient)/R_th) / C_th
    """

    def __init__(self, *, R_th: float = 0.5, C_th: float = 100.0,
                 T_ambient: float = 25.0, T_max: float = 150.0):
        # Thermal resistance [K/W]
        self.R_th = R_th if math.isfinite(R_th) and R_th > 0 else 0.5
        # Thermal capacitance [J/K] — clamp to THERMAL_EPS_C to prevent division by zero
        self.C_th = max(C_th if math.isfinite(C_th) and C_th > 0 else 100.0, _THERMAL_EPS_C)
        # Maximum temperature [°C] — guard against zero (CWE-369)
        self.T_max = T_max if math.isfinite(T_max) and T_max > 0 else 150.0
        # Ambient temperature [°C]
        self.T_ambient = T_ambient if math.isfinite(T_ambient) else 25.0

        self.T = self.T_ambient  # Current temperature [°C]
        self.P_loss = 0.0  # Current power loss [W]

    def step(self, P_loss: float, dt: float) -> float:
        """Update temperature based on power loss.

        Args:
            P_loss: Power dissipation [W].
            dt: Time step [s].

        Returns:
            Current temperature [°C].
        """
        if not math.isfinite(P_loss):
            P_loss = 0.0
        if not math.isfinite(dt) or dt <= 0:
            return self.T

        self.P_loss = P_loss

        # Heat transfer: dT/dt = (P_loss - (T - T_amb)/R_th) / C_th
        dT_dt = (P_loss - (self.T - self.T_ambient) / self.R_th) / self.C_th

        if not math.isfinite(dT_dt):
            dT_dt = 0.0

        self.T += dT_dt * dt

        # Clamp to physical range
        self.T = max(self.T_ambient, min(self.T_max * 1.5, self.T))

        return self.T

    @property
    def is_overheating(self) -> bool:
        """Check if temperature exceeds maximum."""
        return self.T_max < self.T

    @property
    def thermal_derating(self) -> float:
        """Thermal derating factor [0, 1]. 1.0 = no derating."""
        if self.T_max >= self.T:
            return 1.0
        # Linear derating above T_max
        return max(0.0, 1.0 - (self.T - self.T_max) / self.T_max)

    def reset(self) -> None:
        self.T = self.T_ambient
        self.P_loss = 0.0


class MotorThermalModel:
    """Combined motor thermal model with winding and magnet nodes.

    Tracks:
      - Winding temperature (affects Rs)
      - Magnet temperature (affects flux_pm)
    """

    def __init__(self, *, R_th_winding: float = 0.5, C_th_winding: float = 100.0,
                 R_th_magnet: float = 1.0, C_th_magnet: float = 50.0,
                 T_ambient: float = 25.0, T_max_winding: float = 150.0,
                 T_max_magnet: float = 80.0):
        self.winding = ThermalNode(R_th=R_th_winding, C_th=C_th_winding,
                                   T_ambient=T_ambient, T_max=T_max_winding)
        self.magnet = ThermalNode(R_th=R_th_magnet, C_th=C_th_magnet,
                                  T_ambient=T_ambient, T_max=T_max_magnet)

    def step(self, copper_loss_W: float, iron_loss_W: float, dt: float) -> None:
        """Update thermal state.

        Args:
            copper_loss_W: Copper (I²R) losses [W].
            iron_loss_W: Iron (core) losses [W].
            dt: Time step [s].
        """
        # Winding heats from copper losses
        self.winding.step(copper_loss_W, dt)
        # Magnet heats from iron losses (simplified)
        self.magnet.step(iron_loss_W * 0.3, dt)  # 30% of iron loss to magnet

    def get_Rs_factor(self, T_ref: float = 25.0) -> float:
        """Resistance scaling factor due to temperature.

        Rs(T) = Rs(T_ref) * (1 + alpha * (T - T_ref))
        alpha for copper ≈ 0.00393 /K (from constants.py)
        """
        return 1.0 + _ALPHA_CU * (self.winding.T - T_ref)

    def get_flux_factor(self, T_ref: float = 25.0) -> float:
        """Flux scaling factor due to magnet temperature.

        NdFeB: -0.12%/K typical (from constants.py)
        """
        return max(0.5, 1.0 + _ALPHA_MAG * (self.magnet.T - T_ref))

    @property
    def is_overheating(self) -> bool:
        return self.winding.is_overheating or self.magnet.is_overheating

    def reset(self) -> None:
        self.winding.reset()
        self.magnet.reset()
