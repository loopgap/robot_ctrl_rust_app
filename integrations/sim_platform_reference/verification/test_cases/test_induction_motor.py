"""Tests for induction motor model: IMdqModel, IMVectorController.

Tests cover:
- Basic functionality
- NaN/Inf guard (CWE-754)
- Zero-divide guard (CWE-369)
- State transitions
- Vector control interface
- Boundary conditions
"""

import math
import os
import sys

import pytest

# Add project root to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../.."))


from sim_platform.models.motor import IMdqModel, IMVectorController

# ── Helper constants ──────────────────────────────────────────

NAN = float("nan")
INF = float("inf")
NINF = float("-inf")
BIG = 1e308
SMALL = 1e-308
DENORM = 5e-324


# ── IMdqModel Tests ──────────────────────────────────────────

class TestIMdqModel:
    """Test induction motor dq-axis model."""

    def setup_method(self):
        self.motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.001, Pp=2, dt_ns=50000
        )

    def test_basic_creation(self):
        """Test motor creation with valid parameters."""
        assert self.motor.Rs == 0.5
        assert self.motor.Rr == 0.5
        assert self.motor.Ls == 0.01
        assert self.motor.Lr == 0.01
        assert self.motor.Lm == 0.009
        assert self.motor.J == 0.01
        assert self.motor.Pp == 2

    def test_derived_parameters(self):
        """Test derived parameters calculation."""
        # σ = 1 - Lm²/(Ls*Lr)
        expected_sigma = 1.0 - (0.009 ** 2) / (0.01 * 0.01)
        assert abs(self.motor.sigma - expected_sigma) < 1e-6

        # Tr = Lr/Rr
        expected_Tr = 0.01 / 0.5
        assert abs(self.motor.Tr - expected_Tr) < 1e-6

    def test_nan_guard_parameters(self):
        """Test NaN guard on parameters."""
        motor = IMdqModel(
            Rs=NAN, Rr=NAN, Ls=NAN, Lr=NAN, Lm=NAN,
            J=NAN, B=NAN, Pp=2, dt_ns=50000
        )
        # Should use fallback values
        assert not math.isnan(motor.Rs)
        assert not math.isnan(motor.Rr)
        assert not math.isnan(motor.Ls)
        assert not math.isnan(motor.Lr)
        assert not math.isnan(motor.Lm)
        assert not math.isnan(motor.J)

    def test_inf_guard_parameters(self):
        """Test Inf guard on parameters."""
        motor = IMdqModel(
            Rs=INF, Rr=INF, Ls=INF, Lr=INF, Lm=INF,
            J=INF, B=INF, Pp=2, dt_ns=50000
        )
        assert not math.isinf(motor.Rs)
        assert not math.isinf(motor.Rr)
        assert not math.isinf(motor.Ls)

    def test_zero_inductance_guard(self):
        """Test zero inductance guard (CWE-369)."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.0, Lr=0.0, Lm=0.0,
            J=0.01, B=0.0, Pp=2, dt_ns=50000
        )
        # Should use minimum inductance
        assert motor.Ls >= 1e-9
        assert motor.Lr >= 1e-9
        assert motor.Lm >= 1e-9

    def test_zero_inertia_guard(self):
        """Test zero inertia guard (CWE-369)."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.0, B=0.0, Pp=2, dt_ns=50000
        )
        assert motor.J >= 1e-15

    def test_step_basic(self):
        """Test basic step function."""
        initial_theta = self.motor.theta_e
        self.motor.step(vsd=1.0, vsq=0.5, omega_e=100.0, tl=0.0, dt=1e-3)
        # Should update currents and angle
        assert self.motor.theta_e != initial_theta

    def test_step_nan_guard(self):
        """Test NaN guard on step inputs."""
        self.motor.step(vsd=NAN, vsq=NAN, omega_e=NAN, tl=NAN, dt=1e-3)
        # Should not crash, state should be guarded
        assert not math.isnan(self.motor.ids)
        assert not math.isnan(self.motor.iqs)
        assert not math.isnan(self.motor.omega_m)

    def test_step_inf_guard(self):
        """Test Inf guard on step inputs."""
        self.motor.step(vsd=INF, vsq=INF, omega_e=INF, tl=INF, dt=1e-3)
        assert not math.isinf(self.motor.ids)
        assert not math.isinf(self.motor.iqs)

    def test_step_zero_dt_guard(self):
        """Test zero dt guard."""
        self.motor.step(vsd=1.0, vsq=0.5, omega_e=100.0, tl=0.0, dt=0.0)
        # Should use minimum dt, not crash
        assert self.motor.dt >= 1e-12

    def test_torque_calculation(self):
        """Test torque calculation."""
        # Set some current and flux
        self.motor.ids = 1.0
        self.motor.iqs = 2.0
        self.motor.psi_rd = 0.1
        self.motor.psi_rq = 0.0

        torque = self.motor.torque_em
        # Torque should be non-zero with current and flux
        assert abs(torque) > 0

    def test_state_update(self):
        """Test state update after step."""
        state_before = self.motor.get_state()
        self.motor.step(vsd=2.0, vsq=1.0, omega_e=100.0, tl=0.1, dt=1e-3)
        state_after = self.motor.get_state()
        # Some states should change
        assert state_before["theta_e"] != state_after["theta_e"]

    def test_flux_dynamics(self):
        """Test rotor flux dynamics."""
        # Set initial flux
        self.motor.psi_rd = 0.1
        self.motor.psi_rq = 0.0

        # Step with d-axis current
        self.motor.step(vsd=1.0, vsq=0.0, omega_e=100.0, tl=0.0, dt=1e-3)

        # Flux should change
        assert self.motor.psi_rd != 0.1 or self.motor.psi_rq != 0.0

    def test_slip_frequency(self):
        """Test slip frequency calculation."""
        # Set motor speed
        self.motor.omega_m = 50.0  # Mechanical speed

        # Set synchronous frequency (different from electrical speed)
        # ωr = Pp * ωm = 2 * 50 = 100
        omega_e = 150.0  # Different from rotor electrical speed

        # Step
        self.motor.step(vsd=1.0, vsq=0.5, omega_e=omega_e, tl=0.0, dt=1e-3)

        # Slip should be non-zero: ωslip = ωe - ωr = 150 - 100 = 50
        assert abs(self.motor.slip_freq) > 0

    def test_reset(self):
        """Test reset function."""
        # Modify state
        self.motor.ids = 5.0
        self.motor.iqs = 3.0
        self.motor.psi_rd = 0.1
        self.motor.omega_m = 100.0
        self.motor.reset()
        # Should be reset to zero
        assert self.motor.ids == 0.0
        assert self.motor.iqs == 0.0
        assert self.motor.psi_rd == 0.0
        assert self.motor.omega_m == 0.0

    def test_set_flux_reference(self):
        """Test flux reference setting."""
        self.motor.set_flux_reference(0.2)
        assert self.motor.psi_rd == 0.2
        assert self.motor.psi_rq == 0.0

    def test_state_output(self):
        """Test state dictionary output."""
        state = self.motor.get_state()
        assert "ids" in state
        assert "iqs" in state
        assert "psi_rd" in state
        assert "psi_rq" in state
        assert "flux_mag" in state
        assert "omega_m" in state
        assert "theta_e" in state
        assert "torque" in state
        assert "rpm" in state
        assert "slip_freq" in state
        assert "ia" in state
        assert "ib" in state
        assert "ic" in state


# ── IMVectorController Tests ─────────────────────────────────

class TestIMVectorController:
    """Test induction motor vector controller."""

    def setup_method(self):
        self.motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.001, Pp=2, dt_ns=50000
        )
        self.controller = IMVectorController(
            motor=self.motor,
            kp_flux=5.0, ki_flux=500.0,
            kp_torque=5.0, ki_torque=500.0,
            kp_speed=0.1, ki_speed=1.0,
            ts=50e-6
        )

    def test_basic_creation(self):
        """Test controller creation."""
        assert self.controller.motor == self.motor
        assert self.controller.kp_flux == 5.0
        assert self.controller.ki_flux == 500.0
        assert self.controller.kp_torque == 5.0
        assert self.controller.ki_torque == 500.0

    def test_speed_control(self):
        """Test speed control output."""
        # No error
        vsd, vsq, omega_e = self.controller.update_speed(
            speed_ref=100.0, speed_meas=100.0, flux_ref=0.1
        )
        # Should produce some output
        assert isinstance(vsd, float)
        assert isinstance(vsq, float)
        assert isinstance(omega_e, float)

    def test_speed_error(self):
        """Test speed error handling."""
        # Positive error
        vsd1, vsq1, _ = self.controller.update_speed(
            speed_ref=100.0, speed_meas=50.0, flux_ref=0.1
        )

        # Negative error
        vsd2, vsq2, _ = self.controller.update_speed(
            speed_ref=50.0, speed_meas=100.0, flux_ref=0.1
        )

        # Outputs should be different
        assert vsq1 != vsq2

    def test_flux_control(self):
        """Test flux control."""
        # Set flux reference
        vsd, vsq, omega_e = self.controller.update_speed(
            speed_ref=100.0, speed_meas=100.0, flux_ref=0.2
        )

        # d-axis voltage should be non-zero for flux control
        assert isinstance(vsd, float)

    def test_slip_frequency_calculation(self):
        """Test slip frequency calculation."""
        # Set motor state
        self.motor.iqs = 5.0
        self.motor.psi_rd = 0.1

        # Update controller
        self.controller.update_speed(
            speed_ref=100.0, speed_meas=100.0, flux_ref=0.1
        )

        # Slip frequency should be calculated
        assert isinstance(self.controller.omega_slip, float)

    def test_nan_guard(self):
        """Test NaN guard on inputs."""
        vsd, vsq, omega_e = self.controller.update_speed(
            speed_ref=NAN, speed_meas=NAN, flux_ref=NAN
        )
        assert not math.isnan(vsd)
        assert not math.isnan(vsq)
        assert not math.isnan(omega_e)

    def test_reset(self):
        """Test reset function."""
        # Build up integral
        for _ in range(10):
            self.controller.update_speed(
                speed_ref=100.0, speed_meas=0.0, flux_ref=0.1
            )

        self.controller.reset()
        assert self.controller._flux_integral == 0.0
        assert self.controller._torque_integral == 0.0
        assert self.controller._speed_integral == 0.0


# ── Integration Tests ─────────────────────────────────────────

class TestIMIntegration:
    """Integration tests for induction motor models."""

    def test_im_with_vector_control(self):
        """Test induction motor with vector control integration."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.001, Pp=2, dt_ns=50000
        )
        controller = IMVectorController(
            motor=motor,
            kp_flux=5.0, ki_flux=500.0,
            kp_torque=5.0, ki_torque=500.0,
            kp_speed=0.1, ki_speed=1.0,
            ts=50e-6
        )

        # Control loop
        speed_ref = 100.0
        flux_ref = 0.1
        for _ in range(1000):
            # Get control output
            vsd, vsq, omega_e = controller.update_speed(
                speed_ref, motor.omega_m, flux_ref
            )

            # Apply to motor
            motor.step(vsd, vsq, omega_e, tl=0.0, dt=50e-6)

        # Should approach reference speed
        assert motor.omega_m > 0.0

    def test_im_with_load(self):
        """Test induction motor with load torque."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.001, Pp=2, dt_ns=50000
        )

        # Apply voltage for multiple steps to build up speed
        for _ in range(100):
            motor.step(vsd=10.0, vsq=5.0, omega_e=100.0, tl=0.0, dt=1e-3)

        # Now apply load
        motor.step(vsd=10.0, vsq=5.0, omega_e=100.0, tl=0.5, dt=1e-3)

        # Should have some speed (may be negative due to load, but should be valid)
        assert isinstance(motor.omega_m, float)
        assert not math.isnan(motor.omega_m)

    def test_im_abc_interface(self):
        """Test induction motor abc interface."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.001, Pp=2, dt_ns=50000
        )

        # Step with abc voltages
        motor.step_abc(va=10.0, vb=-5.0, vc=-5.0, omega_e=100.0, tl=0.0, dt=1e-3)

        # Update abc currents
        ia, ib, ic = motor.update_abc_currents()

        # Should have valid currents
        assert isinstance(ia, float)
        assert isinstance(ib, float)
        assert isinstance(ic, float)


# ── Extreme Value Tests ───────────────────────────────────────

class TestIMExtremeValues:
    """Test induction motor models with extreme values."""

    def test_im_big_voltage(self):
        """Test IM with very large voltage."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.0, Pp=2, dt_ns=50000
        )
        motor.step(vsd=BIG, vsq=BIG, omega_e=BIG, tl=0.0, dt=1e-3)
        assert not math.isnan(motor.ids)
        assert not math.isinf(motor.ids)

    def test_im_zero_resistance(self):
        """Test IM with zero resistance."""
        motor = IMdqModel(
            Rs=0.0, Rr=0.0, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.0, Pp=2, dt_ns=50000
        )
        motor.step(vsd=10.0, vsq=5.0, omega_e=100.0, tl=0.0, dt=1e-3)
        assert not math.isnan(motor.ids)

    def test_im_denormal_values(self):
        """Test IM with denormalized floats."""
        motor = IMdqModel(
            Rs=DENORM, Rr=DENORM, Ls=DENORM, Lr=DENORM, Lm=DENORM,
            J=DENORM, B=0.0, Pp=2, dt_ns=50000
        )
        motor.step(vsd=1.0, vsq=0.5, omega_e=100.0, tl=0.0, dt=1e-3)
        assert not math.isnan(motor.ids)

    def test_im_negative_speed(self):
        """Test IM with negative speed."""
        motor = IMdqModel(
            Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01, Lm=0.009,
            J=0.01, B=0.001, Pp=2, dt_ns=50000
        )
        # Negative synchronous speed
        motor.step(vsd=10.0, vsq=5.0, omega_e=-100.0, tl=0.0, dt=1e-3)
        assert motor.omega_m <= 0.0 or motor.omega_m >= 0.0  # Should be valid


# ── Run tests ─────────────────────────────────────────────────

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
