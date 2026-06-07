"""Sensor fusion — simple Kalman filter for speed estimation.

L2 fidelity: 1D Kalman filter fusing encoder + current-based speed estimate.

Security:
  - CWE-754: NaN/Inf guards on all inputs
  - CWE-369: Zero-divide guard on innovation covariance

Numerical Limitations (IEEE 754 float64):
  - SimpleKalmanFilter: 1D scalar filter. P (covariance) must stay non-negative.
    Uses max(0, P) guard to prevent negative variance from numerical errors.
  - Q (process noise) and R (measurement noise): if both are zero, the filter
    becomes deterministic and ignores measurements after the first update.
    Always set Q > 0 for tracking applications.
  - SpeedFusion: fuses encoder speed (high precision, quantized) with
    current-based speed (continuous, noisy). When encoder and current
    estimates diverge significantly, the filter may take many steps to converge.
"""

import math


class SimpleKalmanFilter:
    """1D Kalman filter for speed estimation.

    State: x = [speed]
    Measurement: z = speed_measured
    """

    def __init__(self, *, Q: float = 0.01, R: float = 1.0, x0: float = 0.0):
        # Process noise covariance
        self.Q = Q if math.isfinite(Q) and Q > 0 else 0.01
        # Measurement noise covariance
        self.R = R if math.isfinite(R) and R > 0 else 1.0
        # State estimate
        self.x = x0 if math.isfinite(x0) else 0.0
        # Error covariance
        self.P = 1.0

    def predict(self, u: float = 0.0) -> float:
        """Predict step (constant velocity model).

        Args:
            u: Control input (acceleration estimate). Default 0.

        Returns:
            Predicted state.
        """
        if not math.isfinite(u):
            u = 0.0
        # x_pred = x + u*dt (simplified: dt=1)
        self.x += u
        self.P += self.Q
        return self.x

    def update(self, z: float) -> float:
        """Update step with measurement.

        Args:
            z: Measurement value.

        Returns:
            Updated state estimate.
        """
        if not math.isfinite(z):
            return self.x

        # Innovation
        y = z - self.x
        # Innovation covariance
        S = self.P + self.R
        if S < 1e-15:
            return self.x  # Guard against zero divide

        # Kalman gain
        K = self.P / S

        # Update state
        self.x += K * y
        # Update covariance (Joseph form for numerical stability)
        self.P = (1 - K)**2 * self.P + K**2 * self.R

        # Guard against NaN
        if not math.isfinite(self.x):
            self.x = 0.0
        if not math.isfinite(self.P):
            self.P = 1.0

        return self.x

    def get_estimate(self) -> float:
        """Get current state estimate."""
        return self.x

    def get_uncertainty(self) -> float:
        """Get current estimation uncertainty (std dev)."""
        return math.sqrt(max(0, self.P))

    def reset(self, x0: float = 0.0) -> None:
        self.x = x0 if math.isfinite(x0) else 0.0
        self.P = 1.0


class SpeedFusion:
    """Fuse multiple speed estimates using Kalman filter.

    Combines:
      - Encoder-based speed (high precision, quantized)
      - Current-based speed estimate (noisy, continuous)
    """

    def __init__(self, *, Q: float = 0.01, R_encoder: float = 0.5,
                 R_current: float = 2.0):
        self.kf = SimpleKalmanFilter(Q=Q, R=R_encoder)
        self.R_encoder = R_encoder
        self.R_current = R_current

    def update(self, speed_encoder: float, speed_current: float = None) -> float:
        """Fuse speed estimates.

        Args:
            speed_encoder: Speed from encoder [rad/s].
            speed_current: Speed from current model [rad/s]. Optional.

        Returns:
            Fused speed estimate [rad/s].
        """
        # Predict (constant velocity)
        self.kf.predict()

        # Save original R to avoid side effects
        original_R = self.kf.R

        # Update with encoder (primary)
        if math.isfinite(speed_encoder):
            self.kf.R = self.R_encoder
            self.kf.update(speed_encoder)

        # Update with current-based estimate (secondary)
        if speed_current is not None and math.isfinite(speed_current):
            self.kf.R = self.R_current
            self.kf.update(speed_current)

        # Restore original R
        self.kf.R = original_R

        return self.kf.get_estimate()

    def get_estimate(self) -> float:
        return self.kf.get_estimate()

    def get_uncertainty(self) -> float:
        return self.kf.get_uncertainty()

    def reset(self) -> None:
        self.kf.reset()
