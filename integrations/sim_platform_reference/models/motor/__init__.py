"""Motor models for sim_platform.

Available models:
- PMSMdqModel: PMSM dq-axis model (L2 fidelity)
- PMSMAdvanced: PMSM with saturation, temperature, iron loss
- BLDCModel: BLDC motor model with trapezoidal back-EMF
- BLDCController: Six-step commutation controller
- IMdqModel: Induction motor dq-axis model (L2 fidelity)
- IMVectorController: Induction motor vector controller (RFOC)
"""

from .bldc import BLDCController, BLDCModel, CommutationState, HallState
from .im_dq import IMdqModel, IMVectorController
from .pmsm_advanced import PMSMAdvanced
from .pmsm_dq import PMSMdqModel

__all__ = [
    "PMSMdqModel",
    "PMSMAdvanced",
    "BLDCModel",
    "BLDCController",
    "HallState",
    "CommutationState",
    "IMdqModel",
    "IMVectorController",
]
