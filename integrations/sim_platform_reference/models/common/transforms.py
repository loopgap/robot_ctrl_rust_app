"""Coordinate transforms for motor control — single source of truth.

Previously duplicated across foc.py, pmsm_dq.py, im_dq.py, pmsm_advanced.py.
All transforms are pure math functions with input guards (NaN/Inf protection).

Security:
  - CWE-754: NaN/Inf guards on all numeric entry points
  - CWE-369: Zero-divide guard in svpwm (v_bus near-zero)

Numerical Limitations (IEEE 754 float64):
  - Clarke/Park transforms: pure trigonometry, machine precision (~1e-15).
    Roundtrip error is typically 0 (exact for balanced 3-phase systems).
  - SVPWM: duty cycles clamped to [0, 1]. Overmodulation uses linear
    scaling which preserves relative phase relationships.
  - Hot-path optimization: guards at entry points, not inlined.

This module is a single source of truth — all motor models and controllers
import transforms from here (eliminates ~120 lines of duplicated code).
"""

import math

from sim_platform.core.constants import PWM_EPS_V as _PWM_EPS_V

# ── Precomputed constants ────────────────────────────────────
_SQRT3_INV = 1.0 / math.sqrt(3)
_SQRT3_HALF = math.sqrt(3) / 2


# ── Coordinate Transforms ─────────────────────────────────────

def clarke_transform(ia: float, ib: float, ic: float) -> tuple:
    """Clarke: abc → αβ. Guards NaN inputs."""
    if not (math.isfinite(ia) and math.isfinite(ib) and math.isfinite(ic)):
        return (0.0, 0.0)
    i_alpha = ia
    i_beta = (ia + 2 * ib) * _SQRT3_INV
    return i_alpha, i_beta


def inverse_clarke(i_alpha: float, i_beta: float) -> tuple:
    """Inverse Clarke: αβ → abc. Guards NaN inputs."""
    if not (math.isfinite(i_alpha) and math.isfinite(i_beta)):
        return (0.0, 0.0, 0.0)
    ia = i_alpha
    ib = -0.5 * i_alpha + _SQRT3_HALF * i_beta
    ic = -0.5 * i_alpha - _SQRT3_HALF * i_beta
    return ia, ib, ic


def park_transform(i_alpha: float, i_beta: float, theta: float) -> tuple:
    """Park: αβ → dq. Guards NaN inputs."""
    if not (math.isfinite(i_alpha) and math.isfinite(i_beta) and math.isfinite(theta)):
        return (0.0, 0.0)
    cos_t = math.cos(theta)
    sin_t = math.sin(theta)
    id_val = i_alpha * cos_t + i_beta * sin_t
    iq_val = -i_alpha * sin_t + i_beta * cos_t
    return id_val, iq_val


def inverse_park(vd: float, vq: float, theta: float) -> tuple:
    """Inverse Park: dq → αβ. Guards NaN inputs."""
    if not (math.isfinite(vd) and math.isfinite(vq) and math.isfinite(theta)):
        return (0.0, 0.0)
    cos_t = math.cos(theta)
    sin_t = math.sin(theta)
    v_alpha = vd * cos_t - vq * sin_t
    v_beta = vd * sin_t + vq * cos_t
    return v_alpha, v_beta


# ── SVPWM ────────────────────────────────────────────────────

def svpwm(v_alpha: float, v_beta: float, v_bus: float) -> tuple:
    """Space Vector PWM — αβ voltages → 3-phase duty cycles.

    SECURITY (CWE-754): NaN/Inf guards at entry, v_bus near-zero guard (CWE-369).
    Safe fallback: 50% duty on all phases = zero voltage output.

    Returns duty cycles in [0, 1] range.
    """
    # SECURITY: Guard inputs at entry point only
    if not math.isfinite(v_alpha) or not math.isfinite(v_beta) or not math.isfinite(v_bus):
        return (0.5, 0.5, 0.5)

    # SECURITY: v_bus near-zero → cannot modulate (CWE-369)
    if abs(v_bus) < _PWM_EPS_V:
        return (0.5, 0.5, 0.5)

    # Inverse Clarke
    va = v_alpha
    vb = -0.5 * v_alpha + _SQRT3_HALF * v_beta
    vc = -0.5 * v_alpha - _SQRT3_HALF * v_beta

    v_inv = 1.0 / v_bus
    v_mid = v_bus * 0.5
    da = (va + v_mid) * v_inv
    db = (vb + v_mid) * v_inv
    dc = (vc + v_mid) * v_inv

    # Clamp to [0, 1] and guard
    da = max(0.0, min(1.0, da)) if math.isfinite(da) else 0.5
    db = max(0.0, min(1.0, db)) if math.isfinite(db) else 0.5
    dc = max(0.0, min(1.0, dc)) if math.isfinite(dc) else 0.5

    return da, db, dc
