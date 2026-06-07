"""Extended Kalman Filter (EKF) for motor state estimation.

Implements:
- Nonlinear state estimation
- Parameter estimation (optional)
- Covariance management

Security:
  - CWE-754: NaN/Inf guards on all inputs and outputs
  - CWE-369: Zero-divide guard on denominators

Numerical Limitations (IEEE 754 float64):
  - Jacobian linearization: EKF uses first-order Taylor expansion.
    Accuracy degrades for highly nonlinear operating points or large dt.
    For dt > 1ms, consider using Unscented Kalman Filter (UKF) instead.
  - Covariance matrix: P must remain positive semi-definite.
    Numerical errors may cause P to become indefinite after many steps.
    The Joseph form is used for update to improve numerical stability.
  - State wrapping: theta_est is wrapped to [0, 2*pi) each step.
  - estimate() requires 6 parameters: (vd, vq, ia, ib, ic, omega_encoder).
    Missing omega_encoder will cause incorrect state estimation.
"""

import logging
import math
from collections.abc import Callable
from dataclasses import dataclass

import numpy as np

from sim_platform.core.constants import DEFAULT_DT_S as _DEFAULT_DT_S
from sim_platform.core.constants import NUMERIC_EPS as _EKF_EPS
from sim_platform.core.utils import guard_numeric as _guard_numeric

logger = logging.getLogger(__name__)


@dataclass
class EKFConfig:
    """EKF configuration parameters."""
    n_states: int = 4       # Number of states
    n_measurements: int = 2 # Number of measurements
    Q: np.ndarray | None = None  # Process noise covariance
    R: np.ndarray | None = None  # Measurement noise covariance
    P0: np.ndarray | None = None # Initial state covariance


class EKFEstimator:
    """Extended Kalman Filter for nonlinear state estimation.

    Implements:
    - State prediction using nonlinear model
    - Measurement update with linearized observation
    - Covariance propagation and update

    Security: All inputs guarded against NaN/Inf (CWE-754).
    """

    def __init__(self, config: EKFConfig):
        """
        Args:
            config: EKF configuration parameters
        """
        self.config = config

        # Validate dimensions
        self.n = max(1, config.n_states)
        self.m = max(1, config.n_measurements)

        # Initialize covariance matrices
        if config.Q is None:
            self.Q = np.eye(self.n) * 0.01
        else:
            self.Q = np.array(config.Q, dtype=float)
            if self.Q.shape != (self.n, self.n):
                self.Q = np.eye(self.n) * 0.01

        if config.R is None:
            self.R = np.eye(self.m) * 0.1
        else:
            self.R = np.array(config.R, dtype=float)
            if self.R.shape != (self.m, self.m):
                self.R = np.eye(self.m) * 0.1

        if config.P0 is None:
            self.P = np.eye(self.n) * 1.0
        else:
            self.P = np.array(config.P0, dtype=float)
            if self.P.shape != (self.n, self.n):
                self.P = np.eye(self.n) * 1.0

        # State estimate
        self.x = np.zeros(self.n)

        # Guard all matrices
        self._guard_matrix(self.Q, "Q")
        self._guard_matrix(self.R, "R")
        self._guard_matrix(self.P, "P")

    def _guard_matrix(self, M: np.ndarray, name: str) -> np.ndarray:
        """Guard matrix against NaN/Inf."""
        if M is None:
            return np.eye(self.n) * 0.01

        # Replace NaN/Inf with safe values
        mask_nan = np.isnan(M)
        mask_inf = np.isinf(M)

        if np.any(mask_nan) or np.any(mask_inf):
            M[mask_nan] = 0.0
            M[mask_inf] = 1e6  # Large but finite
            # Ensure diagonal is positive
            for i in range(min(M.shape)):
                if M[i, i] <= 0:
                    M[i, i] = 0.01

        return M

    def predict(self, x: np.ndarray, u: np.ndarray,
                f: Callable[[np.ndarray, np.ndarray], np.ndarray],
                F: Callable[[np.ndarray, np.ndarray], np.ndarray]) -> tuple[np.ndarray, np.ndarray]:
        """Predict step: propagate state and covariance.
        
        Args:
            x: Current state estimate
            u: Control input
            f: State transition function x(k+1) = f(x(k), u(k))
            F: Jacobian of state transition ∂f/∂x
            
        Returns:
            Tuple of (predicted state, predicted covariance)
        """
        # Guard inputs
        x = np.array([_guard_numeric(xi, 0.0) for xi in x])
        u = np.array([_guard_numeric(ui, 0.0) for ui in u])

        try:
            # State prediction
            x_pred = f(x, u)
            x_pred = np.array([_guard_numeric(xi, 0.0) for xi in x_pred])

            # Jacobian
            F_k = F(x, u)
            F_k = self._guard_matrix(F_k, "F")

            # Covariance prediction: P = F*P*F' + Q
            P_pred = F_k @ self.P @ F_k.T + self.Q
            P_pred = self._guard_matrix(P_pred, "P_pred")

        except Exception as e:
            # Fallback: identity prediction
            logger.warning("EKF predict failed: %s, using fallback", e)
            x_pred = x.copy()
            P_pred = self.P + self.Q

        self.x = x_pred
        self.P = P_pred

        return x_pred, P_pred

    def update(self, z: np.ndarray, x_pred: np.ndarray, P_pred: np.ndarray,
               h: Callable[[np.ndarray], np.ndarray],
               H: Callable[[np.ndarray], np.ndarray]) -> tuple[np.ndarray, np.ndarray]:
        """Update step: correct state estimate with measurement.
        
        Args:
            z: Measurement vector
            x_pred: Predicted state
            P_pred: Predicted covariance
            h: Observation function z = h(x)
            H: Jacobian of observation ∂h/∂x
            
        Returns:
            Tuple of (updated state, updated covariance)
        """
        # Guard inputs
        z = np.array([_guard_numeric(zi, 0.0) for zi in z])
        x_pred = np.array([_guard_numeric(xi, 0.0) for xi in x_pred])
        P_pred = self._guard_matrix(P_pred, "P_pred")

        try:
            # Predicted measurement
            z_pred = h(x_pred)
            z_pred = np.array([_guard_numeric(zi, 0.0) for zi in z_pred])

            # Measurement residual
            y = z - z_pred
            y = np.array([_guard_numeric(yi, 0.0) for yi in y])

            # Jacobian
            H_k = H(x_pred)
            H_k = self._guard_matrix(H_k, "H")

            # Innovation covariance: S = H*P*H' + R
            S = H_k @ P_pred @ H_k.T + self.R
            S = self._guard_matrix(S, "S")

            # Kalman gain: K = P*H'*S^(-1)
            try:
                S_inv = np.linalg.inv(S)
            except np.linalg.LinAlgError:
                S_inv = np.linalg.pinv(S)

            K = P_pred @ H_k.T @ S_inv
            K = self._guard_matrix(K, "K")

            # State update: x = x_pred + K*y
            x_upd = x_pred + K @ y
            x_upd = np.array([_guard_numeric(xi, 0.0) for xi in x_upd])

            # Covariance update: Joseph form for numerical stability
            # P = (I-KH)P(I-KH)' + KRK'  (guarantees P stays symmetric positive-definite)
            I = np.eye(self.n)
            IKH = I - K @ H_k
            P_upd = IKH @ P_pred @ IKH.T + K @ self.R @ K.T
            P_upd = self._guard_matrix(P_upd, "P_upd")

        except Exception as e:
            # Fallback: keep prediction
            logger.warning("EKF update failed: %s, using prediction", e)
            x_upd = x_pred.copy()
            P_upd = P_pred

        self.x = x_upd
        self.P = P_upd

        return x_upd, P_upd

    def get_state(self) -> dict:
        """Get EKF state."""
        return {
            "x": self.x.tolist(),
            "P": self.P.tolist(),
            "Q": self.Q.tolist(),
            "R": self.R.tolist(),
        }

    def reset(self, x0: np.ndarray | None = None) -> None:
        """Reset EKF state."""
        if x0 is not None:
            self.x = np.array([_guard_numeric(xi, 0.0) for xi in x0])
        else:
            self.x = np.zeros(self.n)
        self.P = np.eye(self.n) * 1.0


class PMSMEKF(EKFEstimator):
    """EKF for PMSM state estimation.

    States: [id, iq, ωm, θe]
    Measurements: [ia, ib, ic, ωm_encoder]
    """

    def __init__(self, *, Rs: float, Ld: float, Lq: float,
                 flux_pm: float, Pp: int, dt: float,
                 B: float = 1e-4, J: float = 1e-3,
                 Q_diag: list[float] | None = None,
                 R_diag: list[float] | None = None):
        """
        Args:
            Rs: Stator resistance [Ω]
            Ld: d-axis inductance [H]
            Lq: q-axis inductance [H]
            flux_pm: PM flux linkage [Wb]
            Pp: Pole pairs
            dt: Sample time [s]
            B: Viscous friction [N·m·s/rad] (default 1e-4)
            J: Rotor inertia [kg·m²] (default 1e-3)
            Q_diag: Process noise diagonal
            R_diag: Measurement noise diagonal
        """
        # Motor parameters
        self.Rs = _guard_numeric(Rs, 0.1)
        self.Ld = max(_guard_numeric(Ld, 5e-4), _EKF_EPS)
        self.Lq = max(_guard_numeric(Lq, 1e-3), _EKF_EPS)
        self.flux_pm = _guard_numeric(flux_pm, 0.03)
        self.Pp = max(1, int(Pp))
        self.dt = max(_guard_numeric(dt, _DEFAULT_DT_S), _EKF_EPS)
        self.B = _guard_numeric(B, 1e-4)
        self.J = max(_guard_numeric(J, 1e-3), _EKF_EPS)

        # EKF configuration
        n_states = 4  # [id, iq, ωm, θe]
        n_meas = 4    # [ia, ib, ic, ωm]

        # Process noise
        if Q_diag is not None and len(Q_diag) == n_states:
            Q = np.diag(Q_diag)
        else:
            Q = np.diag([0.01, 0.01, 0.1, 0.01])

        # Measurement noise
        if R_diag is not None and len(R_diag) == n_meas:
            R = np.diag(R_diag)
        else:
            R = np.diag([0.1, 0.1, 0.1, 0.01])

        config = EKFConfig(
            n_states=n_states,
            n_measurements=n_meas,
            Q=Q,
            R=R,
            P0=np.eye(n_states) * 0.1
        )

        super().__init__(config)

    def _state_transition(self, x: np.ndarray, u: np.ndarray) -> np.ndarray:
        """PMSM state transition function.
        
        x = [id, iq, ωm, θe]
        u = [vd, vq]
        """
        id_, iq, omega, theta = x
        vd, vq = u

        # Electrical dynamics (PMSM dq-axis model)
        we = self.Pp * omega  # electrical angular velocity
        did = (vd - self.Rs * id_ + we * self.Lq * iq) / self.Ld
        diq = (vq - self.Rs * iq - we * (self.Ld * id_ + self.flux_pm)) / self.Lq

        # Electromagnetic torque: Te = 1.5 * Pp * (flux_pm * iq + (Ld - Lq) * id * iq)
        torque_em = 1.5 * self.Pp * (self.flux_pm * iq + (self.Ld - self.Lq) * id_ * iq)

        # Mechanical dynamics: J * dω/dt = Te - B * ω
        dw = (torque_em - self.B * omega) / self.J

        # Angle dynamics
        dtheta = we

        # Forward Euler
        x_next = np.array([
            id_ + self.dt * did,
            iq + self.dt * diq,
            omega + self.dt * dw,
            theta + self.dt * dtheta
        ])

        return x_next

    def _state_jacobian(self, x: np.ndarray, u: np.ndarray) -> np.ndarray:
        """Jacobian of state transition ∂f/∂x."""
        id_, iq, omega, theta = x
        we = self.Pp * omega

        # ∂f/∂x
        F = np.array([
            [-self.Rs / self.Ld, we * self.Lq / self.Ld, self.Pp * self.Lq * iq / self.Ld, 0],
            [-we * self.Ld / self.Lq, -self.Rs / self.Lq, -self.Pp * (self.Ld * id_ + self.flux_pm) / self.Lq, 0],
            [1.5 * self.Pp * (self.Ld - self.Lq) * iq / self.J,
             1.5 * self.Pp * (self.flux_pm + (self.Ld - self.Lq) * id_) / self.J,
             -self.B / self.J, 0],
            [0, 0, self.Pp, 0]
        ])

        return F

    def _observation(self, x: np.ndarray) -> np.ndarray:
        """Observation function z = h(x).
        
        Measurements: [ia, ib, ic, ωm]
        """
        id_, iq, omega, theta = x

        # Inverse Park transform: dq -> abc
        cos_t = math.cos(theta)
        sin_t = math.sin(theta)

        i_alpha = id_ * cos_t - iq * sin_t
        i_beta = id_ * sin_t + iq * cos_t

        ia = i_alpha
        ib = -0.5 * i_alpha + math.sqrt(3) / 2 * i_beta
        ic = -0.5 * i_alpha - math.sqrt(3) / 2 * i_beta

        return np.array([ia, ib, ic, omega])

    def _observation_jacobian(self, x: np.ndarray) -> np.ndarray:
        """Jacobian of observation ∂h/∂x."""
        id_, iq, omega, theta = x

        cos_t = math.cos(theta)
        sin_t = math.sin(theta)

        # ∂h/∂x
        H = np.array([
            [cos_t, -sin_t, 0, -id_ * sin_t - iq * cos_t],
            [-0.5 * cos_t + math.sqrt(3)/2 * sin_t, 0.5 * sin_t + math.sqrt(3)/2 * cos_t, 0,
             0.5 * id_ * sin_t - 0.5 * iq * cos_t + math.sqrt(3)/2 * (id_ * cos_t + iq * sin_t)],
            [-0.5 * cos_t - math.sqrt(3)/2 * sin_t, 0.5 * sin_t - math.sqrt(3)/2 * cos_t, 0,
             0.5 * id_ * sin_t + 0.5 * iq * cos_t - math.sqrt(3)/2 * (id_ * cos_t + iq * sin_t)],
            [0, 0, 1, 0]
        ])

        return H

    def estimate(self, vd: float, vq: float,
                 ia: float, ib: float, ic: float,
                 omega_encoder: float) -> tuple[float, float, float, float]:
        """Estimate PMSM states.
        
        Args:
            vd, vq: dq voltages [V]
            ia, ib, ic: Phase currents [A]
            omega_encoder: Encoder speed [rad/s]
            
        Returns:
            Tuple of (id_est, iq_est, omega_est, theta_est)
        """
        # Guard inputs
        vd = _guard_numeric(vd, 0.0)
        vq = _guard_numeric(vq, 0.0)
        ia = _guard_numeric(ia, 0.0)
        ib = _guard_numeric(ib, 0.0)
        ic = _guard_numeric(ic, 0.0)
        omega_encoder = _guard_numeric(omega_encoder, 0.0)

        # Control input
        u = np.array([vd, vq])

        # Measurement
        z = np.array([ia, ib, ic, omega_encoder])

        # Predict
        x_pred, P_pred = self.predict(
            self.x, u,
            self._state_transition,
            self._state_jacobian
        )

        # Update
        x_upd, P_upd = self.update(
            z, x_pred, P_pred,
            self._observation,
            self._observation_jacobian
        )

        # Extract estimates
        id_est = _guard_numeric(x_upd[0], 0.0)
        iq_est = _guard_numeric(x_upd[1], 0.0)
        omega_est = _guard_numeric(x_upd[2], 0.0)
        theta_est = _guard_numeric(x_upd[3], 0.0)

        # Wrap angle
        theta_est = theta_est % (2 * math.pi)

        return id_est, iq_est, omega_est, theta_est

    def get_state(self) -> dict:
        """Get EKF state with motor estimates."""
        state = super().get_state()
        state.update({
            "id_est": self.x[0],
            "iq_est": self.x[1],
            "omega_est": self.x[2],
            "theta_est": self.x[3],
        })
        return state
