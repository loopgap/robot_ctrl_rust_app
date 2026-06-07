"""TUI shared utilities and constants."""

import math
import os

# ── Project path ──────────────────────────────────────────
# Use relative path from this file to project root (3 levels up from tools/tui/)
_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
OUTPUT_DIR = os.path.join(_PROJ, "output")
os.makedirs(OUTPUT_DIR, exist_ok=True)


def guard_float(val: str, fallback: float = 0.0) -> float:
    """Parse and validate a float string. Returns fallback on NaN/Inf."""
    try:
        v = float(val)
        if math.isnan(v) or math.isinf(v):
            return fallback
        return v
    except (ValueError, TypeError):
        return fallback


# ── Motor presets ─────────────────────────────────────────
MOTOR_PRESETS = {
    "Small PMSM (200W drone)": {
        "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
        "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4,
    },
    "Medium PMSM (2kW e-bike)": {
        "Rs": 0.05, "Ld": 0.2e-3, "Lq": 0.4e-3,
        "flux_pm": 0.05, "J": 0.005, "B": 0.0005, "Pp": 4,
    },
    "Large PMSM (20kW EV)": {
        "Rs": 0.01, "Ld": 0.1e-3, "Lq": 0.2e-3,
        "flux_pm": 0.12, "J": 0.05, "B": 0.001, "Pp": 3,
    },
}

# ── Scenarios ─────────────────────────────────────────────
SCENARIOS = {
    "Step Response": {"duration": 1.5, "speed_ref": 100, "profile": "step", "load": 0},
    "Ramp Test": {"duration": 1.5, "speed_ref": 100, "profile": "ramp", "load": 0},
    "Load Disturbance": {"duration": 2.0, "speed_ref": 100, "profile": "step", "load": 0.3},
    "Voltage Sag Ride-Through": {"duration": 2.0, "speed_ref": 100, "profile": "step", "load": 0},
}

# ── Scenario details (enhanced) ──────────────────────────
SCENARIO_DETAILS = {
    "Step Response": {
        "icon": "\u25b6",
        "description": "100 rad/s step input, measures settling time and overshoot",
    },
    "Ramp Test": {
        "icon": "\u25b2",
        "description": "Smooth acceleration 0 \u2192 100 rad/s over the simulation duration",
    },
    "Load Disturbance": {
        "icon": "\u26a1",
        "description": "0.3 N\u00b7m load torque applied at t=0.5s, tests disturbance rejection",
    },
    "Voltage Sag Ride-Through": {
        "icon": "\u26a0",
        "description": "20V bus voltage sag, tests controller recovery and ride-through",
    },
}

# ── Scan parameters ───────────────────────────────────────
SCAN_PARAMS = {
    "Speed Reference": ("speed", [30, 50, 100, 150, 200]),
    "FOC kp_id": ("kp_id", [1, 3, 5, 10, 20]),
    "FOC ki_id": ("ki_id", [100, 300, 500, 1000, 2000]),
    "Speed Loop Kp": ("spd_kp", [0.01, 0.03, 0.05, 0.1, 0.2]),
    "Load Torque": ("load", [0, 0.1, 0.3, 0.5, 1.0]),
}
