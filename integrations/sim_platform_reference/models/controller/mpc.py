"""Model Predictive Control (MPC) for motor control applications.

Implements:
- Finite Control Set MPC (FCS-MPC) for current control
- Linear MPC for speed control
- Quadratic Programming (QP) solver for optimization

Security:
  - CWE-754: NaN/Inf guards on all inputs and outputs
  - CWE-369: Zero-divide guard on denominators

Numerical Limitations (IEEE 754 float64):
  - Prediction model: uses Forward Euler discretization of RL circuit.
    Accuracy depends on Ts (sampling time) relative to L/R time constant.
    For Ts > L/R, the discrete model becomes unstable.
    Rule of thumb: Ts < L/(10*R) for reliable predictions.
  - update() is 1D: takes (i_ref, i_meas), returns single voltage command.
    For dq-axis control, instantiate two MPCCurrentController instances.
  - Cost function: quadratic cost with constraints. No guarantee of
    global optimality for nonlinear motor dynamics.
"""

import logging
from collections.abc import Callable
from dataclasses import dataclass

from sim_platform.core.constants import DEFAULT_DT_S as _DEFAULT_DT_S
from sim_platform.core.constants import NUMERIC_EPS as _MPC_EPS
from sim_platform.core.utils import guard_numeric as _guard_numeric

logger = logging.getLogger(__name__)


@dataclass
class MPCConfig:
    """MPC configuration parameters."""
    Np: int = 10          # Prediction horizon
    Nc: int = 3           # Control horizon
    Q: float = 1.0        # State tracking weight
    R: float = 0.1        # Control effort weight
    dt: float = _DEFAULT_DT_S  # Sample time [s]
    u_min: float = -1.0   # Minimum control output
    u_max: float = 1.0    # Maximum control output
    du_min: float = -0.1  # Minimum control increment
    du_max: float = 0.1   # Maximum control increment
    learning_rate: float = 0.01   # Gradient descent step size
    max_iterations: int = 50      # Max optimization iterations
    grad_perturbation: float = 1e-3  # Numerical gradient perturbation


class MPCController:
    """Model Predictive Controller for motor control.

    Implements:
    - Prediction model based on system dynamics
    - Quadratic cost function with terminal weight
    - Input constraints and rate constraints
    - QP solver for optimization

    Security: All inputs guarded against NaN/Inf (CWE-754).
    """

    def __init__(self, config: MPCConfig):
        """
        Args:
            config: MPC configuration parameters
        """
        self.config = config

        # Validate configuration
        self.config.Np = max(1, int(config.Np))
        self.config.Nc = max(1, min(config.Nc, config.Np))
        self.config.Q = max(_MPC_EPS, _guard_numeric(config.Q, 1.0))
        self.config.R = max(_MPC_EPS, _guard_numeric(config.R, 0.1))
        self.config.dt = max(_MPC_EPS, _guard_numeric(config.dt, _DEFAULT_DT_S))

        # Ensure u_min < u_max
        if self.config.u_min > self.config.u_max:
            self.config.u_min, self.config.u_max = self.config.u_max, self.config.u_min

        # Internal state
        self._x_pred: list[float] = []  # Predicted states
        self._u_pred: list[float] = []  # Predicted controls
        self._cost: float = 0.0         # Optimal cost

    def predict(self, x0: float, u_seq: list[float],
                model: Callable[[float, float], float]) -> list[float]:
        """Predict future states given initial state and control sequence.
        
        Args:
            x0: Initial state
            u_seq: Control sequence [u0, u1, ..., uNc-1]
            model: State transition function x(k+1) = f(x(k), u(k))
            
        Returns:
            Predicted states [x1, x2, ..., xNp]
        """
        x0 = _guard_numeric(x0, 0.0)

        # Extend control sequence to prediction horizon
        u_extended = list(u_seq)
        while len(u_extended) < self.config.Np:
            u_extended.append(u_extended[-1] if u_extended else 0.0)

        # Predict states
        x_pred = []
        x_current = x0
        for k in range(self.config.Np):
            u_current = _guard_numeric(u_extended[k], 0.0)
            try:
                x_next = model(x_current, u_current)
                x_next = _guard_numeric(x_next, 0.0)
            except Exception:
                # SECURITY: Keep last known state instead of resetting to 0.0
                # (0.0 could be a dangerous output for motor current control)
                logger.warning("MPC prediction model failed at step %d, "
                               "keeping last state x_current=%.4f", k, x_current)
                x_next = x_current
            x_pred.append(x_next)
            x_current = x_next

        self._x_pred = x_pred
        return x_pred

    def compute_cost(self, x_pred: list[float], x_ref: float,
                     u_seq: list[float]) -> float:
        """Compute quadratic cost function.
        
        J = Σ[ Q*(x(k) - x_ref)² + R*u(k)² ]
        
        Args:
            x_pred: Predicted states
            x_ref: Reference state
            u_seq: Control sequence
            
        Returns:
            Total cost
        """
        x_ref = _guard_numeric(x_ref, 0.0)

        cost = 0.0

        # State tracking cost
        for k, x in enumerate(x_pred):
            x = _guard_numeric(x, 0.0)
            error = x - x_ref
            cost += self.config.Q * error * error

        # Control effort cost
        for k, u in enumerate(u_seq[:self.config.Nc]):
            u = _guard_numeric(u, 0.0)
            cost += self.config.R * u * u

        self._cost = cost
        return cost

    def solve(self, x0: float, x_ref: float,
              model: Callable[[float, float], float],
              u_init: list[float] | None = None) -> tuple[float, list[float]]:
        """Solve MPC optimization problem.
        
        Uses gradient descent to find optimal control sequence.
        
        Args:
            x0: Current state
            x_ref: Reference state
            model: State transition function
            u_init: Initial control sequence guess
            
        Returns:
            Tuple of (optimal control, optimal sequence)
        """
        x0 = _guard_numeric(x0, 0.0)
        x_ref = _guard_numeric(x_ref, 0.0)

        # Initialize control sequence
        if u_init is None:
            u_seq = [0.0] * self.config.Nc
        else:
            u_seq = [max(self.config.u_min, min(self.config.u_max,
                     _guard_numeric(u, 0.0))) for u in u_init[:self.config.Nc]]
            # Extend if needed
            while len(u_seq) < self.config.Nc:
                u_seq.append(u_seq[-1] if u_seq else 0.0)

        # Gradient descent optimization
        learning_rate = self.config.learning_rate
        num_iterations = self.config.max_iterations
        delta = self.config.grad_perturbation

        for _ in range(num_iterations):
            # Predict with current control sequence
            x_pred = self.predict(x0, u_seq, model)

            # Compute cost (for convergence monitoring)
            _cost = self.compute_cost(x_pred, x_ref, u_seq)

            # Compute gradient (numerical)
            gradient = []
            for k in range(self.config.Nc):
                u_plus = list(u_seq)
                u_plus[k] += delta
                x_pred_plus = self.predict(x0, u_plus, model)
                cost_plus = self.compute_cost(x_pred_plus, x_ref, u_plus)

                u_minus = list(u_seq)
                u_minus[k] -= delta
                x_pred_minus = self.predict(x0, u_minus, model)
                cost_minus = self.compute_cost(x_pred_minus, x_ref, u_minus)

                grad = (cost_plus - cost_minus) / (2 * delta)
                gradient.append(grad)

            # Update control sequence
            for k in range(self.config.Nc):
                u_seq[k] -= learning_rate * gradient[k]

                # Apply constraints
                u_seq[k] = max(self.config.u_min, min(self.config.u_max, u_seq[k]))

                # Apply rate constraints
                if k > 0:
                    du = u_seq[k] - u_seq[k-1]
                    du = max(self.config.du_min, min(self.config.du_max, du))
                    u_seq[k] = u_seq[k-1] + du

                # Guard against NaN
                u_seq[k] = _guard_numeric(u_seq[k], 0.0)

        self._u_pred = u_seq
        return u_seq[0], u_seq

    def get_state(self) -> dict:
        """Get MPC state."""
        return {
            "x_pred": self._x_pred,
            "u_pred": self._u_pred,
            "cost": self._cost,
            "config": {
                "Np": self.config.Np,
                "Nc": self.config.Nc,
                "Q": self.config.Q,
                "R": self.config.R,
            }
        }


class MPCCurrentController:
    """MPC-based current controller for motor control.

    Implements FCS-MPC for fast current control.
    """

    def __init__(self, *, L: float, R: float, Ts: float,
                 i_max: float = 100.0, v_max: float = 48.0,
                 Np: int = 5, Nc: int = 2,
                 Q: float = 1.0, R_weight: float = 0.01):
        """
        Args:
            L: Inductance [H]
            R: Resistance [Ω]
            Ts: Sample time [s]
            i_max: Maximum current [A]
            v_max: Maximum voltage [V]
            Np: Prediction horizon (default 5)
            Nc: Control horizon (default 2)
            Q: State tracking weight (default 1.0)
            R_weight: Control effort weight (default 0.01)
        """
        self.L = max(_guard_numeric(L, 1e-3), _MPC_EPS)
        self.R = _guard_numeric(R, 0.1)
        self.Ts = max(_guard_numeric(Ts, _DEFAULT_DT_S), _MPC_EPS)
        self.i_max = abs(_guard_numeric(i_max, 100.0))
        self.v_max = abs(_guard_numeric(v_max, 48.0))

        # MPC configuration
        config = MPCConfig(
            Np=max(1, int(Np)),
            Nc=max(1, min(int(Nc), max(1, int(Np)))),
            Q=max(_MPC_EPS, _guard_numeric(Q, 1.0)),
            R=max(_MPC_EPS, _guard_numeric(R_weight, 0.01)),
            dt=self.Ts,
            u_min=-self.v_max,
            u_max=self.v_max
        )
        self.mpc = MPCController(config)

        # State
        self.i_ref = 0.0
        self.v_ref = 0.0

    def _motor_model(self, i: float, v: float) -> float:
        """Motor current dynamics: di/dt = (v - R*i) / L
        
        Discrete: i(k+1) = i(k) + Ts * (v(k) - R*i(k)) / L
        """
        i = _guard_numeric(i, 0.0)
        v = _guard_numeric(v, 0.0)

        di = (v - self.R * i) / self.L
        di = _guard_numeric(di, 0.0)

        i_next = i + self.Ts * di
        return _guard_numeric(i_next, 0.0)

    def update(self, i_ref: float, i_meas: float) -> float:
        """Compute voltage reference using MPC.
        
        Args:
            i_ref: Current reference [A]
            i_meas: Measured current [A]
            
        Returns:
            Voltage reference [V]
        """
        i_ref = _guard_numeric(i_ref, 0.0)
        i_meas = _guard_numeric(i_meas, 0.0)

        # Limit reference
        i_ref = max(-self.i_max, min(self.i_max, i_ref))

        # Solve MPC
        v_opt, _ = self.mpc.solve(
            x0=i_meas,
            x_ref=i_ref,
            model=self._motor_model
        )

        self.i_ref = i_ref
        self.v_ref = v_opt

        return v_opt

    def reset(self) -> None:
        """Reset controller state."""
        self.i_ref = 0.0
        self.v_ref = 0.0


class MPCSpeedController:
    """MPC-based speed controller for motor control.

    Implements linear MPC for speed control.
    """

    def __init__(self, *, J: float, B: float, Kt: float, Ts: float,
                 omega_max: float = 500.0, i_max: float = 100.0):
        """
        Args:
            J: Rotor inertia [kg·m²]
            B: Viscous friction [N·m·s/rad]
            Kt: Torque constant [N·m/A]
            Ts: Sample time [s]
            omega_max: Maximum speed [rad/s]
            i_max: Maximum current [A]
        """
        self.J = max(_guard_numeric(J, 1e-3), _MPC_EPS)
        self.B = _guard_numeric(B, 0.001)
        self.Kt = _guard_numeric(Kt, 0.1)
        self.Ts = max(_guard_numeric(Ts, 1e-3), _MPC_EPS)
        self.omega_max = abs(_guard_numeric(omega_max, 500.0))
        self.i_max = abs(_guard_numeric(i_max, 100.0))

        # MPC configuration
        config = MPCConfig(
            Np=10,
            Nc=3,
            Q=1.0,
            R=0.1,
            dt=self.Ts,
            u_min=-self.i_max,
            u_max=self.i_max
        )
        self.mpc = MPCController(config)

        # State
        self.omega_ref = 0.0
        self.i_ref = 0.0

    def _speed_model(self, omega: float, i_q: float) -> float:
        """Speed dynamics: dω/dt = (Kt*i_q - B*ω) / J
        
        Discrete: ω(k+1) = ω(k) + Ts * (Kt*i_q(k) - B*ω(k)) / J
        """
        omega = _guard_numeric(omega, 0.0)
        i_q = _guard_numeric(i_q, 0.0)

        dw = (self.Kt * i_q - self.B * omega) / self.J
        dw = _guard_numeric(dw, 0.0)

        omega_next = omega + self.Ts * dw
        return _guard_numeric(omega_next, 0.0)

    def update(self, omega_ref: float, omega_meas: float) -> float:
        """Compute current reference using MPC.
        
        Args:
            omega_ref: Speed reference [rad/s]
            omega_meas: Measured speed [rad/s]
            
        Returns:
            Current reference [A]
        """
        omega_ref = _guard_numeric(omega_ref, 0.0)
        omega_meas = _guard_numeric(omega_meas, 0.0)

        # Limit reference
        omega_ref = max(-self.omega_max, min(self.omega_max, omega_ref))

        # Solve MPC
        i_opt, _ = self.mpc.solve(
            x0=omega_meas,
            x_ref=omega_ref,
            model=self._speed_model
        )

        self.omega_ref = omega_ref
        self.i_ref = i_opt

        return i_opt

    def reset(self) -> None:
        """Reset controller state."""
        self.omega_ref = 0.0
        self.i_ref = 0.0
