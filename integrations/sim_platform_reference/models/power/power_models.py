"""Power electronics models — battery and inverter.

Security:
  - CWE-754: NaN/Inf guards on model entry points
"""

import math


class IdealBattery:
    """L0: Ideal voltage source."""

    def __init__(self, v_nom: float = 48.0):
        self.v_nom = v_nom if math.isfinite(v_nom) else 48.0
        self.v = self.v_nom

    def step(self, i_load: float = 0.0) -> float:
        return self.v


class RintBattery:
    """L1: Battery with internal resistance (OCV + Rint)."""

    def __init__(self, v_oc: float = 48.0, r_int: float = 0.05):
        self.v_oc = v_oc if math.isfinite(v_oc) else 48.0
        self.r_int = max(r_int if math.isfinite(r_int) else 0.05, 1e-9)
        self.v = self.v_oc

    def step(self, i_load: float = 0.0) -> float:
        if not math.isfinite(i_load):
            return self.v
        self.v = max(0.0, self.v_oc - i_load * self.r_int)
        return self.v


class AverageInverter:
    """L2: Three-phase inverter averaged model.

    Converts duty cycles and DC bus voltage to phase voltages.
    """

    def __init__(self, v_bus: float = 48.0, dead_time_ns: float = 200.0):
        self.v_bus = v_bus if math.isfinite(v_bus) else 48.0
        self.dead_time_ns = dead_time_ns if math.isfinite(dead_time_ns) else 200.0

    def step(self, duty_a: float, duty_b: float, duty_c: float,
             v_bus: float = None, ia: float = 0.0, ib: float = 0.0,
             ic: float = 0.0) -> tuple:
        """Compute three-phase output voltages with NaN/Inf guards."""
        # Guard duty cycles at entry
        if not (math.isfinite(duty_a) and math.isfinite(duty_b) and math.isfinite(duty_c)):
            return (0.0, 0.0, 0.0)

        da = max(0.0, min(1.0, duty_a))
        db = max(0.0, min(1.0, duty_b))
        dc = max(0.0, min(1.0, duty_c))

        if v_bus is None:
            v_bus = self.v_bus
        else:
            if not math.isfinite(v_bus):
                v_bus = self.v_bus
            else:
                self.v_bus = v_bus

        va = v_bus * (da - 0.5)
        vb = v_bus * (db - 0.5)
        vc = v_bus * (dc - 0.5)

        return (va, vb, vc)
