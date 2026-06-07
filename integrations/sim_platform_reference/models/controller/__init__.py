"""Controller models for sim_platform.

Available controllers:
- PIController: PI controller with anti-windup
- FOCController: Field-Oriented Control with dual PI current loops
- SpeedController: Speed PI controller
- MPCController: Model Predictive Controller
- MPCCurrentController: MPC-based current controller
- MPCSpeedController: MPC-based speed controller
- EKFEstimator: Extended Kalman Filter
- PMSMEKF: EKF for PMSM state estimation
"""

from .ekf import PMSMEKF, EKFConfig, EKFEstimator
from .foc import FOCController, PIController, SpeedController, svpwm
from .mpc import MPCConfig, MPCController, MPCCurrentController, MPCSpeedController

__all__ = [
    "PIController",
    "FOCController",
    "SpeedController",
    "svpwm",
    "MPCController",
    "MPCConfig",
    "MPCCurrentController",
    "MPCSpeedController",
    "EKFEstimator",
    "EKFConfig",
    "PMSMEKF",
]
