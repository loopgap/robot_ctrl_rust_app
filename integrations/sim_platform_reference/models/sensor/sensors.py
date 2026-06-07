"""Sensor models with realistic imperfections.

Each model takes true physical values and outputs measured values
with noise, bias, quantization, delay, and saturation.

Security:
  - CWE-754: NaN/Inf guards on all inputs and outputs

Performance:
  - Noise pre-generated in batches (random.gauss amortized)

Numerical Limitations (IEEE 754 float64):
  - Noise generation: uses pre-filled 1024-sample buffer for performance.
    When noise_std is near zero (<1e-15), noise is skipped entirely
    (fast path). This avoids unnecessary random number generation.
  - Quantization: round(measured/quantization)*quantization may accumulate
    rounding errors for very small quantization steps (<1e-10).
  - Saturation: hard clamp to [-saturation, +saturation].
    No smooth transition — this models ADC clipping behavior.
  - Encoder: angle wrapping to [0, 2*pi) preserves precision.
    Speed estimation from angle differentiation is NOT implemented —
    use SpeedFusion from sensor_fusion.py for speed estimation.
"""

import math
import random

from sim_platform.core.utils import guard_numeric as _guard_num

# Pre-generate noise buffer to amortize random.gauss overhead
_NOISE_BUF_SIZE = 1024
_noise_buf: list = []
_noise_idx: int = 0


def _get_noise(std: float) -> float:
    """Get a Gaussian noise sample from pre-filled buffer (fast path)."""
    global _noise_idx, _noise_buf
    if _noise_idx >= len(_noise_buf):
        # Refill buffer
        _noise_buf = [random.gauss(0, 1) for _ in range(_NOISE_BUF_SIZE)]
        _noise_idx = 0
    val = _noise_buf[_noise_idx] * std
    _noise_idx += 1
    return val


class CurrentSensor:
    """L1: Phase current sensor with noise + bias.

    Parameters are in SI units (Amperes).
    """

    def __init__(self, *, noise_std: float = 0.05, bias: float = 0.0,
                 quantization: float = 0.0, saturation: float = float("inf")):
        self.noise_std = _guard_num(noise_std, 0.05)
        self.bias = _guard_num(bias, 0.0)
        self.quantization = _guard_num(quantization, 0.0)
        self.saturation = _guard_num(saturation, 1e6)
        # Cache for skip-noise fast path
        self._has_noise = abs(self.noise_std) > 1e-15

    def read(self, i_true: float) -> float:
        """Return measured current [A] with NaN/Inf guard."""
        if not math.isfinite(i_true):
            return 0.0
        if self._has_noise:
            measured = i_true + _get_noise(self.noise_std) + self.bias
        else:
            measured = i_true + self.bias
        measured = max(-self.saturation, min(self.saturation, measured))
        if self.quantization > 0:
            measured = round(measured / self.quantization) * self.quantization
        return measured

    def read_abc(self, ia: float, ib: float, ic: float) -> tuple:
        return (self.read(ia), self.read(ib), self.read(ic))


class Encoder:
    """L1: Rotary encoder with quantization + noise.

    Simulates angle measurement errors.
    """

    def __init__(self, *, noise_std: float = 0.0,
                 quantization: float = 2 * math.pi / 4096,
                 ppm: int = 4096):
        self.noise_std = _guard_num(noise_std, 0.0)
        self.quantization = _guard_num(quantization, 2 * math.pi / 4096)
        self.ppm = max(1, int(ppm))

    def read_angle(self, theta_e_true: float) -> float:
        """Return measured electrical angle [rad]."""
        if not math.isfinite(theta_e_true):
            return 0.0
        if self.noise_std > 1e-15:
            measured = theta_e_true + _get_noise(self.noise_std)
        else:
            measured = theta_e_true
        if self.quantization > 0:
            measured = round(measured / self.quantization) * self.quantization
        return measured % (2 * math.pi)

    def read_speed(self, omega_m_true: float) -> float:
        """Return measured mechanical speed [rad/s]."""
        if not math.isfinite(omega_m_true):
            return 0.0
        return omega_m_true + _get_noise(self.noise_std * 10)
