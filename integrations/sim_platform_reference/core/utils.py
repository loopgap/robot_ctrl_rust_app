"""Core utilities — shared numerical safety functions.

Centralizes _guard_numeric and other numeric helpers to eliminate
code duplication across motor models, controllers, and sensors.

Security:
  - CWE-754: NaN/Inf guards (single source of truth)
  - CWE-369: Zero-divide helpers
"""

import math


def guard_numeric(value: float, fallback: float = 0.0) -> float:
    """Guard against NaN/Inf, return fallback.

    This is the SINGLE SOURCE OF TRUTH for NaN/Inf checking.
    All modules should import from here instead of defining their own.

    Optimized: uses math.isfinite() which is faster than isnan()+isinf().

    Args:
        value: Input value to check.
        fallback: Value to return if input is NaN or Inf.

    Returns:
        value if finite, fallback otherwise.
    """
    if math.isfinite(value):
        return value
    return fallback


def guard_positive(value: float, fallback: float = 0.0,
                   min_val: float = 0.0) -> float:
    """Guard against NaN/Inf and enforce minimum value.

    Args:
        value: Input value to check.
        fallback: Value to return if input is NaN or Inf.
        min_val: Minimum allowed value (clamped).

    Returns:
        Clamped finite value.
    """
    v = guard_numeric(value, fallback)
    return max(v, min_val)


def guard_in_range(value: float, low: float, high: float,
                   fallback: float = 0.0) -> float:
    """Guard against NaN/Inf and clamp to [low, high].

    Args:
        value: Input value to check.
        low: Minimum allowed value.
        high: Maximum allowed value.
        fallback: Value to return if input is NaN or Inf.

    Returns:
        Clamped finite value.
    """
    v = guard_numeric(value, fallback)
    return max(low, min(high, v))


def safe_divide(numerator: float, denominator: float,
                eps: float = 1e-12, fallback: float = 0.0) -> float:
    """Divide with zero-denominator guard (CWE-369).

    Args:
        numerator: Dividend.
        denominator: Divisor.
        eps: Minimum absolute denominator value.
        fallback: Value to return if denominator is near-zero or result is NaN/Inf.

    Returns:
        numerator / denominator if safe, fallback otherwise.
    """
    if abs(denominator) < eps:
        return fallback
    result = numerator / denominator
    return guard_numeric(result, fallback)
