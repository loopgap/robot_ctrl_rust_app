"""Tests for motor models: PMSMdqModel, PMSMAdvanced, BLDCModel.

Tests cover:
- Basic functionality
- NaN/Inf guard (CWE-754)
- Zero-divide guard (CWE-369)
- State transitions
- Boundary conditions
"""

import math
import os
import sys

import pytest

# Add project root to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../.."))


from sim_platform.models.motor import (
    BLDCController,
    BLDCModel,
    CommutationState,
    HallState,
    PMSMAdvanced,
    PMSMdqModel,
)

# ── Helper constants ──────────────────────────────────────────

NAN = float("nan")
INF = float("inf")
NINF = float("-inf")
BIG = 1e308
SMALL = 1e-308
DENORM = 5e-324


# ── PMSMdqModel Tests ─────────────────────────────────────────

class TestPMSMdqModel:
    """Test PMSM dq-axis model."""

    def setup_method(self):
        self.motor = PMSMdqModel(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            J=1e-3, B=0.0001, Pp=4, dt_ns=50000
        )

    def test_basic_creation(self):
        """Test motor creation with valid parameters."""
        assert self.motor.Rs == 0.1
        assert self.motor.Ld == 5e-4
        assert self.motor.Lq == 1e-3
        assert self.motor.flux_pm == 0.03
        assert self.motor.J == 1e-3
        assert self.motor.Pp == 4

    def test_nan_guard_parameters(self):
        """Test NaN guard on parameters."""
        motor = PMSMdqModel(
            Rs=NAN, Ld=NAN, Lq=NAN, flux_pm=NAN,
            J=NAN, B=NAN, Pp=4, dt_ns=50000
        )
        # Should use fallback values
        assert not math.isnan(motor.Rs)
        assert not math.isnan(motor.Ld)
        assert not math.isnan(motor.Lq)
        assert not math.isnan(motor.flux_pm)
        assert not math.isnan(motor.J)

    def test_inf_guard_parameters(self):
        """Test Inf guard on parameters."""
        motor = PMSMdqModel(
            Rs=INF, Ld=INF, Lq=INF, flux_pm=INF,
            J=INF, B=INF, Pp=4, dt_ns=50000
        )
        assert not math.isinf(motor.Rs)
        assert not math.isinf(motor.Ld)
        assert not math.isinf(motor.Lq)

    def test_zero_inductance_guard(self):
        """Test zero inductance guard (CWE-369)."""
        motor = PMSMdqModel(
            Rs=0.1, Ld=0.0, Lq=0.0, flux_pm=0.03,
            J=1e-3, B=0.0, Pp=4, dt_ns=50000
        )
        # Should use minimum inductance
        assert motor.Ld >= 1e-9
        assert motor.Lq >= 1e-9

    def test_zero_inertia_guard(self):
        """Test zero inertia guard (CWE-369)."""
        motor = PMSMdqModel(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            J=0.0, B=0.0, Pp=4, dt_ns=50000
        )
        assert motor.J >= 1e-15

    def test_step_basic(self):
        """Test basic step function."""
        initial_theta = self.motor.theta_e
        self.motor.step(vd=1.0, vq=0.5, tl=0.0, dt=1e-3)
        # Should update currents and angle
        assert self.motor.theta_e != initial_theta

    def test_step_nan_guard(self):
        """Test NaN guard on step inputs."""
        self.motor.step(vd=NAN, vq=NAN, tl=NAN, dt=1e-3)
        # Should not crash, state should be guarded
        assert not math.isnan(self.motor.id)
        assert not math.isnan(self.motor.iq)
        assert not math.isnan(self.motor.omega_m)

    def test_step_inf_guard(self):
        """Test Inf guard on step inputs."""
        self.motor.step(vd=INF, vq=INF, tl=INF, dt=1e-3)
        assert not math.isinf(self.motor.id)
        assert not math.isinf(self.motor.iq)

    def test_step_zero_dt_guard(self):
        """Test zero dt guard."""
        self.motor.step(vd=1.0, vq=0.5, tl=0.0, dt=0.0)
        # Should use minimum dt, not crash
        assert self.motor.dt >= 1e-12

    def test_torque_calculation(self):
        """Test torque calculation."""
        # Set some current
        self.motor.id = 0.0
        self.motor.iq = 1.0
        torque = self.motor.torque_em
        # Torque should be non-zero with iq current
        assert abs(torque) > 0

    def test_state_update(self):
        """Test state update after step."""
        state_before = self.motor.get_state()
        self.motor.step(vd=2.0, vq=1.0, tl=0.1, dt=1e-3)
        state_after = self.motor.get_state()
        # Some states should change
        assert state_before["theta_e"] != state_after["theta_e"]

    def test_reset(self):
        """Test reset function."""
        # Modify state
        self.motor.id = 5.0
        self.motor.iq = 3.0
        self.motor.omega_m = 100.0
        self.motor.reset()
        # Should be reset to zero
        assert self.motor.id == 0.0
        assert self.motor.iq == 0.0
        assert self.motor.omega_m == 0.0


# ── PMSMAdvanced Tests ────────────────────────────────────────

class TestPMSMAdvanced:
    """Test advanced PMSM model with saturation and temperature."""

    def setup_method(self):
        self.motor = PMSMAdvanced(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            J=1e-3, B=0.0001, Pp=4, dt_ns=50000,
            Ld_sat=3e-4, Lq_sat=6e-4, I_sat=10.0,
            Rs_temp_coeff=0.004, T_ref=25.0,
            kh=0.01, ke=0.001, alpha=2.0, beta=2.0
        )

    def test_basic_creation(self):
        """Test advanced motor creation."""
        assert self.motor.Ld0 == 5e-4
        assert self.motor.Lq0 == 1e-3
        assert self.motor.Ld_sat == 3e-4
        assert self.motor.I_sat == 10.0
        assert self.motor.T_ref == 25.0

    def test_saturation_effect(self):
        """Test inductance saturation with current."""
        # At zero current, inductance should be unsaturated
        Ld0, Lq0 = self.motor._get_saturated_inductance(0.0, 0.0)
        assert abs(Ld0 - 5e-4) < 1e-6
        assert abs(Lq0 - 1e-3) < 1e-6

        # At high current, inductance should decrease
        Ld_sat, Lq_sat = self.motor._get_saturated_inductance(20.0, 0.0)
        assert Ld_sat < Ld0
        assert Lq_sat < Lq0

    def test_temperature_resistance(self):
        """Test temperature-dependent resistance."""
        # At reference temperature
        Rs_ref = self.motor._get_temperature_resistance()
        assert abs(Rs_ref - 0.1) < 1e-6

        # At higher temperature
        self.motor.winding_temp = 50.0
        Rs_hot = self.motor._get_temperature_resistance()
        assert Rs_hot > Rs_ref

        # At lower temperature
        self.motor.winding_temp = 0.0
        Rs_cold = self.motor._get_temperature_resistance()
        assert Rs_cold < Rs_ref

    def test_iron_loss_calculation(self):
        """Test iron loss calculation."""
        # Zero frequency should give zero loss
        loss0 = self.motor._calculate_iron_loss(0.0, 0.0)
        assert loss0 == 0.0

        # Non-zero frequency should give non-zero loss
        loss1 = self.motor._calculate_iron_loss(100.0, 0.1)
        assert loss1 > 0.0

    def test_step_with_temperature(self):
        """Test step with temperature update."""
        self.motor.step(vd=1.0, vq=0.5, tl=0.0, dt=1e-3, winding_temp=50.0)
        assert self.motor.winding_temp == 50.0

    def test_loss_tracking(self):
        """Test loss tracking after step."""
        # Initial losses should be zero
        assert self.motor.copper_loss == 0.0
        assert self.motor.iron_loss == 0.0

        # After step with current, should have losses
        self.motor.step(vd=2.0, vq=1.0, tl=0.0, dt=1e-3)
        # Copper loss should be non-zero if current is non-zero
        if abs(self.motor.id) > 0 or abs(self.motor.iq) > 0:
            assert self.motor.copper_loss > 0.0

    def test_efficiency_calculation(self):
        """Test efficiency calculation."""
        # Set some operating point
        self.motor.id = 0.0
        self.motor.iq = 5.0
        self.motor.omega_m = 100.0

        efficiency = self.motor.get_efficiency(24.0)
        assert 0.0 <= efficiency <= 1.0

    def test_state_includes_advanced(self):
        """Test state includes advanced parameters."""
        state = self.motor.get_state()
        assert "winding_temp" in state
        assert "Rs_effective" in state
        assert "Ld_effective" in state
        assert "copper_loss" in state


# ── BLDCModel Tests ───────────────────────────────────────────

class TestBLDCModel:
    """Test BLDC motor model."""

    def setup_method(self):
        self.motor = BLDCModel(
            Rs=0.5, Ls=1e-3, Ke=0.01, Kt=0.01,
            J=1e-4, B=0.0001, Pp=1, dt_ns=50000
        )

    def test_basic_creation(self):
        """Test BLDC motor creation."""
        assert self.motor.Rs == 0.5
        assert self.motor.Ls == 1e-3
        assert self.motor.Ke == 0.01
        assert self.motor.Kt == 0.01
        assert self.motor.J == 1e-4

    def test_nan_guard_parameters(self):
        """Test NaN guard on parameters."""
        motor = BLDCModel(
            Rs=NAN, Ls=NAN, Ke=NAN, Kt=NAN,
            J=NAN, B=NAN, Pp=1, dt_ns=50000
        )
        assert not math.isnan(motor.Rs)
        assert not math.isnan(motor.Ls)
        assert not math.isnan(motor.Ke)

    def test_zero_inductance_guard(self):
        """Test zero inductance guard."""
        motor = BLDCModel(
            Rs=0.5, Ls=0.0, Ke=0.01, Kt=0.01,
            J=1e-4, B=0.0, Pp=1, dt_ns=50000
        )
        assert motor.Ls >= 1e-9

    def test_hall_state_detection(self):
        """Test Hall state detection from angle."""
        # Test at different angles
        self.motor.theta_e = 0.0
        hall = self.motor.hall_state
        assert isinstance(hall, HallState)

        self.motor.theta_e = math.pi / 3
        hall = self.motor.hall_state
        assert isinstance(hall, HallState)

    def test_trapezoidal_emf(self):
        """Test trapezoidal back-EMF generation."""
        ea, eb, ec = self.motor._trapezoidal_emf(0.0)
        # At 0°, phase A should be at some defined point
        assert -1.0 <= ea <= 1.0
        assert -1.0 <= eb <= 1.0
        assert -1.0 <= ec <= 1.0

    def test_commutation_state(self):
        """Test commutation state update."""
        _initial_state = self.motor._commutation_state
        # Step with voltage
        self.motor.step(v_bus=12.0, tl=0.0, dt=1e-3)
        # Commutation state should be valid
        assert isinstance(self.motor._commutation_state, CommutationState)

    def test_step_basic(self):
        """Test basic step function."""
        initial_theta = self.motor.theta_e
        self.motor.step(v_bus=12.0, tl=0.0, dt=1e-3)
        # Should update angle
        assert self.motor.theta_e != initial_theta

    def test_step_nan_guard(self):
        """Test NaN guard on step inputs."""
        self.motor.step(v_bus=NAN, tl=NAN, dt=1e-3)
        assert not math.isnan(self.motor.ia)
        assert not math.isnan(self.motor.ib)
        assert not math.isnan(self.motor.ic)

    def test_step_inf_guard(self):
        """Test Inf guard on step inputs."""
        self.motor.step(v_bus=INF, tl=INF, dt=1e-3)
        assert not math.isinf(self.motor.ia)

    def test_torque_calculation(self):
        """Test torque calculation."""
        # Set some currents
        self.motor.ia = 1.0
        self.motor.ib = -0.5
        self.motor.ic = -0.5

        # Step to calculate torque
        self.motor.step(v_bus=12.0, tl=0.0, dt=1e-3)
        # Torque should be calculated
        assert isinstance(self.motor.torque, float)

    def test_mechanical_dynamics(self):
        """Test mechanical dynamics."""
        # Apply voltage to accelerate
        for _ in range(100):
            self.motor.step(v_bus=12.0, tl=0.0, dt=1e-3)

        # Should have some speed
        assert self.motor.omega_m > 0.0

    def test_load_torque(self):
        """Test load torque effect."""
        # Accelerate first
        for _ in range(100):
            self.motor.step(v_bus=12.0, tl=0.0, dt=1e-3)

        speed_no_load = self.motor.omega_m

        # Apply load
        for _ in range(100):
            self.motor.step(v_bus=12.0, tl=0.1, dt=1e-3)

        speed_with_load = self.motor.omega_m

        # Speed should be lower with load
        assert speed_with_load < speed_no_load

    def test_hall_sequence(self):
        """Test Hall sequence generation."""
        sequence = self.motor.get_hall_sequence(num_poles=1)
        # Should have 6 states per revolution
        assert len(sequence) == 6
        # All should be valid Hall states
        for hall in sequence:
            assert isinstance(hall, HallState)

    def test_reset(self):
        """Test reset function."""
        # Modify state
        self.motor.ia = 5.0
        self.motor.ib = 3.0
        self.motor.omega_m = 100.0
        self.motor.reset()
        # Should be reset
        assert self.motor.ia == 0.0
        assert self.motor.ib == 0.0
        assert self.motor.omega_m == 0.0

    def test_state_output(self):
        """Test state dictionary output."""
        state = self.motor.get_state()
        assert "ia" in state
        assert "ib" in state
        assert "ic" in state
        assert "omega_m" in state
        assert "theta_e" in state
        assert "torque" in state
        assert "rpm" in state
        assert "hall_state" in state


# ── BLDCController Tests ──────────────────────────────────────

class TestBLDCController:
    """Test BLDC controller."""

    def setup_method(self):
        self.controller = BLDCController(
            kp_speed=0.1, ki_speed=1.0,
            i_max=10.0, v_bus=24.0, dt=1e-3
        )

    def test_basic_creation(self):
        """Test controller creation."""
        assert self.controller.kp_speed == 0.1
        assert self.controller.ki_speed == 1.0
        assert self.controller.i_max == 10.0

    def test_speed_control(self):
        """Test speed control output."""
        # No error
        output = self.controller.update(speed_ref=100.0, speed_meas=100.0)
        assert output == pytest.approx(0.0, abs=1e-6)

        # Positive error
        output = self.controller.update(speed_ref=100.0, speed_meas=50.0)
        assert output > 0.0

        # Negative error
        output = self.controller.update(speed_ref=50.0, speed_meas=100.0)
        assert output < 0.0

    def test_output_limiting(self):
        """Test output limiting to [-1, 1]."""
        # Large error
        output = self.controller.update(speed_ref=1000.0, speed_meas=0.0)
        assert -1.0 <= output <= 1.0

    def test_nan_guard(self):
        """Test NaN guard on inputs."""
        output = self.controller.update(speed_ref=NAN, speed_meas=NAN)
        assert not math.isnan(output)

    def test_reset(self):
        """Test reset function."""
        # Build up integral
        for _ in range(10):
            self.controller.update(speed_ref=100.0, speed_meas=0.0)

        self.controller.reset()
        assert self.controller._speed_error_integral == 0.0


# ── Integration Tests ─────────────────────────────────────────

class TestMotorIntegration:
    """Integration tests for motor models."""

    def test_bldc_with_controller(self):
        """Test BLDC motor with controller integration."""
        motor = BLDCModel(
            Rs=0.5, Ls=1e-3, Ke=0.01, Kt=0.01,
            J=1e-4, B=0.0001, Pp=1, dt_ns=50000
        )
        controller = BLDCController(
            kp_speed=0.1, ki_speed=1.0,
            i_max=10.0, v_bus=24.0, dt=1e-3
        )

        # Control loop
        speed_ref = 100.0
        for _ in range(1000):
            # Get control output
            duty = controller.update(speed_ref, motor.omega_m)

            # Apply to motor (simplified: v_bus * duty)
            v_applied = 24.0 * abs(duty)
            motor.step(v_bus=v_applied, tl=0.0, dt=1e-3)

        # Should approach reference speed
        assert motor.omega_m > 0.0

    def test_pmsm_advanced_temperature_effect(self):
        """Test PMSM advanced with temperature effect."""
        motor = PMSMAdvanced(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            J=1e-3, B=0.0001, Pp=4, dt_ns=50000,
            Rs_temp_coeff=0.004, T_ref=25.0
        )

        # Step at different temperatures
        motor.step(vd=1.0, vq=0.5, tl=0.0, dt=1e-3, winding_temp=25.0)
        state_25 = motor.get_state()

        motor.reset()
        motor.step(vd=1.0, vq=0.5, tl=0.0, dt=1e-3, winding_temp=100.0)
        state_100 = motor.get_state()

        # Resistance should be higher at higher temperature
        assert state_100["Rs_effective"] > state_25["Rs_effective"]


# ── Extreme Value Tests ───────────────────────────────────────

class TestMotorExtremeValues:
    """Test motor models with extreme values."""

    def test_pmsm_big_voltage(self):
        """Test PMSM with very large voltage."""
        motor = PMSMdqModel(
            Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
            J=1e-3, B=0.0, Pp=4, dt_ns=50000
        )
        motor.step(vd=BIG, vq=BIG, tl=0.0, dt=1e-3)
        assert not math.isnan(motor.id)
        assert not math.isinf(motor.id)

    def test_bldc_zero_resistance(self):
        """Test BLDC with zero resistance."""
        motor = BLDCModel(
            Rs=0.0, Ls=1e-3, Ke=0.01, Kt=0.01,
            J=1e-4, B=0.0, Pp=1, dt_ns=50000
        )
        motor.step(v_bus=12.0, tl=0.0, dt=1e-3)
        assert not math.isnan(motor.ia)

    def test_pmsm_denormal_values(self):
        """Test PMSM with denormalized floats."""
        motor = PMSMdqModel(
            Rs=DENORM, Ld=DENORM, Lq=DENORM, flux_pm=DENORM,
            J=DENORM, B=0.0, Pp=4, dt_ns=50000
        )
        motor.step(vd=1.0, vq=0.5, tl=0.0, dt=1e-3)
        assert not math.isnan(motor.id)


# ── Run tests ─────────────────────────────────────────────────

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
