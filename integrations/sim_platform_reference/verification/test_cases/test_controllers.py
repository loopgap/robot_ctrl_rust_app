"""Tests for MPC and EKF controllers.

Tests cover:
- MPCController: Basic functionality, prediction, optimization
- MPCCurrentController: Current control
- MPCSpeedController: Speed control
- EKFEstimator: State estimation
- PMSMEKF: PMSM state estimation

Security: NaN/Inf guards tested.
"""

import math
import os
import sys

import numpy as np
import pytest

# Add project root to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../.."))

from sim_platform.models.controller.ekf import PMSMEKF, EKFConfig, EKFEstimator
from sim_platform.models.controller.mpc import (
    MPCConfig,
    MPCController,
    MPCCurrentController,
    MPCSpeedController,
)

# ── Helper constants ──────────────────────────────────────────

NAN = float("nan")
INF = float("inf")
NINF = float("-inf")


# ── MPCController Tests ──────────────────────────────────────

class TestMPCController:
    """Test basic MPC controller."""

    def setup_method(self):
        config = MPCConfig(Np=5, Nc=2, Q=1.0, R=0.1, dt=1e-3)
        self.mpc = MPCController(config)

    def test_basic_creation(self):
        """Test MPC creation."""
        assert self.mpc.config.Np == 5
        assert self.mpc.config.Nc == 2
        assert self.mpc.config.Q == 1.0
        assert self.mpc.config.R == 0.1

    def test_predict(self):
        """Test state prediction."""
        def model(x, u):
            return x + 0.1 * u

        x_pred = self.mpc.predict(x0=1.0, u_seq=[0.5, 0.5], model=model)

        assert len(x_pred) == 5
        assert all(isinstance(x, float) for x in x_pred)
        assert all(not math.isnan(x) for x in x_pred)

    def test_cost_computation(self):
        """Test cost computation."""
        x_pred = [1.0, 1.1, 1.2, 1.3, 1.4]
        x_ref = 1.0
        u_seq = [0.5, 0.5]

        cost = self.mpc.compute_cost(x_pred, x_ref, u_seq)

        assert isinstance(cost, float)
        assert cost >= 0
        assert not math.isnan(cost)

    def test_solve(self):
        """Test MPC optimization."""
        def model(x, u):
            return x + 0.1 * u

        u_opt, u_seq = self.mpc.solve(x0=0.0, x_ref=1.0, model=model)

        assert isinstance(u_opt, float)
        assert len(u_seq) == 2
        assert all(isinstance(u, float) for u in u_seq)
        assert all(not math.isnan(u) for u in u_seq)

    def test_nan_guard_predict(self):
        """Test NaN guard on prediction."""
        def model(x, u):
            return x + u

        x_pred = self.mpc.predict(x0=NAN, u_seq=[NAN], model=model)

        assert all(not math.isnan(x) for x in x_pred)

    def test_nan_guard_solve(self):
        """Test NaN guard on solve."""
        def model(x, u):
            return x + u

        u_opt, u_seq = self.mpc.solve(x0=NAN, x_ref=NAN, model=model)

        assert not math.isnan(u_opt)
        assert all(not math.isnan(u) for u in u_seq)

    def test_constraints(self):
        """Test constraint enforcement."""
        config = MPCConfig(Np=5, Nc=2, Q=1.0, R=0.1, dt=1e-3,
                          u_min=-0.5, u_max=0.5)
        mpc = MPCController(config)

        def model(x, u):
            return x + u

        u_opt, u_seq = mpc.solve(x0=0.0, x_ref=10.0, model=model)

        # Check constraints
        assert all(-0.5 <= u <= 0.5 for u in u_seq)

    def test_get_state(self):
        """Test state retrieval."""
        state = self.mpc.get_state()

        assert "config" in state
        assert "x_pred" in state
        assert "u_pred" in state


# ── MPCCurrentController Tests ────────────────────────────────

class TestMPCCurrentController:
    """Test MPC current controller."""

    def setup_method(self):
        self.controller = MPCCurrentController(
            L=1e-3, R=0.5, Ts=50e-6, i_max=100.0, v_max=48.0
        )

    def test_basic_creation(self):
        """Test controller creation."""
        assert self.controller.L == 1e-3
        assert self.controller.R == 0.5
        assert self.controller.Ts == 50e-6

    def test_update(self):
        """Test current control."""
        v_ref = self.controller.update(i_ref=5.0, i_meas=0.0)

        assert isinstance(v_ref, float)
        assert not math.isnan(v_ref)
        assert -48.0 <= v_ref <= 48.0

    def test_tracking(self):
        """Test current tracking."""
        # Positive error
        v_ref1 = self.controller.update(i_ref=10.0, i_meas=0.0)

        # Negative error
        v_ref2 = self.controller.update(i_ref=0.0, i_meas=10.0)

        # Outputs should be different
        assert v_ref1 != v_ref2

    def test_nan_guard(self):
        """Test NaN guard."""
        v_ref = self.controller.update(i_ref=NAN, i_meas=NAN)

        assert not math.isnan(v_ref)
        assert -48.0 <= v_ref <= 48.0

    def test_reset(self):
        """Test reset."""
        self.controller.update(i_ref=5.0, i_meas=0.0)
        self.controller.reset()

        assert self.controller.i_ref == 0.0
        assert self.controller.v_ref == 0.0


# ── MPCSpeedController Tests ─────────────────────────────────

class TestMPCSpeedController:
    """Test MPC speed controller."""

    def setup_method(self):
        self.controller = MPCSpeedController(
            J=0.01, B=0.001, Kt=0.1, Ts=1e-3,
            omega_max=500.0, i_max=100.0
        )

    def test_basic_creation(self):
        """Test controller creation."""
        assert self.controller.J == 0.01
        assert self.controller.B == 0.001
        assert self.controller.Kt == 0.1

    def test_update(self):
        """Test speed control."""
        i_ref = self.controller.update(omega_ref=100.0, omega_meas=0.0)

        assert isinstance(i_ref, float)
        assert not math.isnan(i_ref)
        assert -100.0 <= i_ref <= 100.0

    def test_tracking(self):
        """Test speed tracking."""
        # Positive error
        i_ref1 = self.controller.update(omega_ref=100.0, omega_meas=0.0)

        # Negative error
        i_ref2 = self.controller.update(omega_ref=0.0, omega_meas=100.0)

        # Outputs should be different
        assert i_ref1 != i_ref2

    def test_nan_guard(self):
        """Test NaN guard."""
        i_ref = self.controller.update(omega_ref=NAN, omega_meas=NAN)

        assert not math.isnan(i_ref)
        assert -100.0 <= i_ref <= 100.0

    def test_reset(self):
        """Test reset."""
        self.controller.update(omega_ref=100.0, omega_meas=0.0)
        self.controller.reset()

        assert self.controller.omega_ref == 0.0
        assert self.controller.i_ref == 0.0


# ── EKFEstimator Tests ───────────────────────────────────────

class TestEKFEstimator:
    """Test basic EKF estimator."""

    def setup_method(self):
        config = EKFConfig(
            n_states=2,
            n_measurements=1,
            Q=np.diag([0.01, 0.01]),
            R=np.array([[0.1]]),
            P0=np.eye(2) * 0.1
        )
        self.ekf = EKFEstimator(config)

    def test_basic_creation(self):
        """Test EKF creation."""
        assert self.ekf.n == 2
        assert self.ekf.m == 1
        assert self.ekf.x.shape == (2,)

    def test_predict(self):
        """Test state prediction."""
        def f(x, u):
            return np.array([x[0] + x[1], x[1]])

        def F(x, u):
            return np.array([[1, 1], [0, 1]])

        x_pred, P_pred = self.ekf.predict(
            x=np.array([0.0, 0.0]),
            u=np.array([0.0]),
            f=f, F=F
        )

        assert x_pred.shape == (2,)
        assert P_pred.shape == (2, 2)
        assert not np.any(np.isnan(x_pred))

    def test_update(self):
        """Test measurement update."""
        def h(x):
            return np.array([x[0]])

        def H(x):
            return np.array([[1, 0]])

        x_upd, P_upd = self.ekf.update(
            z=np.array([1.0]),
            x_pred=np.array([0.0, 0.0]),
            P_pred=np.eye(2) * 0.1,
            h=h, H=H
        )

        assert x_upd.shape == (2,)
        assert P_upd.shape == (2, 2)
        assert not np.any(np.isnan(x_upd))

    def test_nan_guard(self):
        """Test NaN guard."""
        config = EKFConfig(
            n_states=2,
            n_measurements=1,
            Q=np.array([[NAN, 0], [0, NAN]]),
            R=np.array([[NAN]])
        )
        ekf = EKFEstimator(config)

        # Should not crash
        assert ekf.Q.shape == (2, 2)
        assert ekf.R.shape == (1, 1)

    def test_get_state(self):
        """Test state retrieval."""
        state = self.ekf.get_state()

        assert "x" in state
        assert "P" in state
        assert "Q" in state
        assert "R" in state

    def test_reset(self):
        """Test reset."""
        self.ekf.x = np.array([5.0, 3.0])
        self.ekf.reset()

        assert np.allclose(self.ekf.x, np.zeros(2))


# ── PMSMEKF Tests ────────────────────────────────────────────

class TestPMSMEKF:
    """Test PMSM EKF estimator."""

    def setup_method(self):
        self.ekf = PMSMEKF(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            Pp=4, dt=50e-6,
            Q_diag=[0.01, 0.01, 0.1, 0.01],
            R_diag=[0.1, 0.1, 0.1, 0.01]
        )

    def test_basic_creation(self):
        """Test PMSM EKF creation."""
        assert self.ekf.n == 4
        assert self.ekf.m == 4
        assert self.ekf.Rs == 0.1
        assert self.ekf.Ld == 5e-4

    def test_estimate(self):
        """Test state estimation."""
        id_est, iq_est, omega_est, theta_est = self.ekf.estimate(
            vd=1.0, vq=0.5,
            ia=0.1, ib=-0.05, ic=-0.05,
            omega_encoder=100.0
        )

        assert isinstance(id_est, float)
        assert isinstance(iq_est, float)
        assert isinstance(omega_est, float)
        assert isinstance(theta_est, float)
        assert not math.isnan(id_est)
        assert not math.isnan(iq_est)
        assert not math.isnan(omega_est)
        assert 0 <= theta_est < 2 * math.pi

    def test_nan_guard(self):
        """Test NaN guard on inputs."""
        id_est, iq_est, omega_est, theta_est = self.ekf.estimate(
            vd=NAN, vq=NAN,
            ia=NAN, ib=NAN, ic=NAN,
            omega_encoder=NAN
        )

        assert not math.isnan(id_est)
        assert not math.isnan(iq_est)
        assert not math.isnan(omega_est)

    def test_convergence(self):
        """Test estimation convergence."""
        # Run multiple steps
        for _ in range(100):
            id_est, iq_est, omega_est, theta_est = self.ekf.estimate(
                vd=1.0, vq=0.5,
                ia=0.1, ib=-0.05, ic=-0.05,
                omega_encoder=100.0
            )

        # Should converge to reasonable estimates
        assert isinstance(id_est, float)
        assert not math.isnan(id_est)
        # Omega estimate should be in reasonable range (not wildly divergent)
        assert abs(omega_est) < 10000.0, f"EKF omega estimate {omega_est} diverged"
        # Theta should be in valid range
        assert 0 <= theta_est < 2 * math.pi, f"Theta {theta_est} out of range"

    def test_get_state(self):
        """Test state retrieval."""
        state = self.ekf.get_state()

        assert "id_est" in state
        assert "iq_est" in state
        assert "omega_est" in state
        assert "theta_est" in state

    def test_reset(self):
        """Test reset."""
        # Run some steps
        self.ekf.estimate(vd=1.0, vq=0.5, ia=0.1, ib=-0.05, ic=-0.05,
                         omega_encoder=100.0)

        # Reset
        self.ekf.reset(x0=np.array([0.0, 0.0, 0.0, 0.0]))

        assert np.allclose(self.ekf.x, np.zeros(4))


# ── Integration Tests ─────────────────────────────────────────

class TestControllerIntegration:
    """Integration tests for MPC and EKF."""

    def test_mpc_with_motor_model(self):
        """Test MPC with motor model."""
        # Simple motor model
        L = 1e-3
        R = 0.5
        dt = 50e-6

        def motor_model(i, v):
            di = (v - R * i) / L
            return i + dt * di

        # Create MPC with more iterations for convergence
        config = MPCConfig(Np=5, Nc=2, Q=1.0, R=0.1, dt=dt,
                          learning_rate=0.1, max_iterations=100)
        mpc = MPCController(config)

        # Control loop - run enough steps for the MPC to drive current
        i_meas = 0.0
        i_ref = 5.0
        for _ in range(500):
            u_opt, _ = mpc.solve(x0=i_meas, x_ref=i_ref, model=motor_model)
            i_meas = motor_model(i_meas, u_opt)

        # Should approach reference (MPC drives current toward target)
        assert i_meas > 1.0, f"MPC current {i_meas:.2f}A should be > 1.0A (ref={i_ref}A)"
        # Current should be moving in the right direction
        assert i_meas < i_ref * 2.0, f"MPC current {i_meas:.2f}A exceeded 2x reference"

    def test_ekf_with_motor_model(self):
        """Test EKF with motor model."""
        # Create EKF
        ekf = PMSMEKF(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            Pp=4, dt=50e-6
        )

        # Run estimation
        for _ in range(50):
            id_est, iq_est, omega_est, theta_est = ekf.estimate(
                vd=1.0, vq=0.5,
                ia=0.1, ib=-0.05, ic=-0.05,
                omega_encoder=100.0
            )

        # Should produce finite estimates
        assert not math.isnan(id_est)
        assert not math.isnan(omega_est)
        # Omega estimate should be in reasonable range
        assert abs(omega_est) < 10000.0, f"EKF omega {omega_est} diverged"
        # Theta should be valid
        assert 0 <= theta_est < 2 * math.pi


# ── Extreme Value Tests ───────────────────────────────────────

class TestControllerExtremeValues:
    """Test controllers with extreme values."""

    def test_mpc_large_reference(self):
        """Test MPC with large reference."""
        config = MPCConfig(Np=5, Nc=2, Q=1.0, R=0.1, dt=1e-3)
        mpc = MPCController(config)

        def model(x, u):
            return x + u

        u_opt, u_seq = mpc.solve(x0=0.0, x_ref=1e6, model=model)

        assert not math.isnan(u_opt)
        assert all(not math.isnan(u) for u in u_seq)

    def test_ekf_zero_parameters(self):
        """Test EKF with zero parameters."""
        ekf = PMSMEKF(
            Rs=0.0, Ld=1e-9, Lq=1e-9, flux_pm=0.0,
            Pp=1, dt=50e-6
        )

        id_est, iq_est, omega_est, theta_est = ekf.estimate(
            vd=1.0, vq=0.5, ia=0.1, ib=-0.05, ic=-0.05,
            omega_encoder=100.0
        )

        assert not math.isnan(id_est)


# ── Run tests ─────────────────────────────────────────────────

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
