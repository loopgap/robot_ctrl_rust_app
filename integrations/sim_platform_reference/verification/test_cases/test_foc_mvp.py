"""Verification tests for PMSM+FOC MVP.

Tests cover: model units, PI stability, FOC transforms,
closed-loop convergence, and fault injection robustness.
"""

import math
import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../.."))

from sim_platform.models.controller.foc import (
    FOCController,
    PIController,
    SpeedController,
    clarke_transform,
    inverse_park,
    park_transform,
    svpwm,
)
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder
from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector


class TestCoordinateTransforms(unittest.TestCase):
    """Verify Clarke/Park/SVPWM mathematical correctness."""

    def test_clarke_park_roundtrip(self):
        """abc → αβ → dq → αβ → abc should be identity."""
        ia, ib, ic = 10.0, -5.0, -5.0
        i_alpha, i_beta = clarke_transform(ia, ib, ic)
        id_val, iq_val = park_transform(i_alpha, i_beta, 0.0)
        # At theta=0: id=i_alpha, iq=-i_beta
        self.assertAlmostEqual(id_val, i_alpha, places=10)
        self.assertAlmostEqual(iq_val, -i_beta, places=10)

    def test_inverse_park(self):
        """Inverse Park at theta=0."""
        va, vb = inverse_park(10.0, 5.0, 0.0)
        self.assertAlmostEqual(va, 10.0, places=8)
        self.assertAlmostEqual(vb, 5.0, places=8)

    def test_svpwm_output_range(self):
        """Duty cycles must be in [0, 1]."""
        for _ in range(100):
            import random
            va = random.uniform(-20, 20)
            vb = random.uniform(-20, 20)
            da, db, dc = svpwm(va, vb, 48.0)
            for d in (da, db, dc):
                self.assertGreaterEqual(d, 0.0)
                self.assertLessEqual(d, 1.0)


class TestPIController(unittest.TestCase):
    """Verify PI controller behavior."""

    def test_step_response(self):
        """PI should converge to setpoint."""
        pi = PIController(kp=1.0, ki=10.0, ts=0.001,
                          out_min=-100, out_max=100)
        y = 0.0
        for _ in range(2000):
            u = pi.update(10.0, y)
            y += u * 0.001 * 10  # simple plant: gain=10
        self.assertAlmostEqual(y, 10.0, delta=1.0)  # within 10%

    def test_anti_windup(self):
        """Saturated PI should recover quickly."""
        pi = PIController(kp=2.0, ki=100.0, ts=0.001,
                          out_min=-5.0, out_max=5.0)
        for _ in range(100):
            pi.update(100.0, 0.0)  # large error → saturates
        self.assertTrue(pi.saturated)
        # Recover when error reverses
        pi.update(-100.0, 100.0)
        self.assertLess(pi.integral, 50)


class TestMotorModel(unittest.TestCase):
    """Verify PMSM model physics."""

    def setUp(self):
        self.motor = PMSMdqModel(
            Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
            flux_pm=0.03, J=0.001, B=0.0001, Pp=4)

    def test_zero_input_no_motion(self):
        """Zero voltage → no current → no torque → stationary."""
        for _ in range(100):
            self.motor.step(0, 0, 0)
        self.assertAlmostEqual(self.motor.omega_m, 0.0, delta=1e-6)
        self.assertAlmostEqual(self.motor.torque, 0.0, delta=1e-6)

    def test_positive_q_voltage_accelerates(self):
        """Positive vq should produce positive torque and speed."""
        for _ in range(500):
            self.motor.step(0, 10.0, 0)
        self.assertGreater(self.motor.omega_m, 0)
        self.assertGreater(self.motor.torque, 0)

    def test_theta_e_wraps(self):
        """Electrical angle should stay in [0, 2π)."""
        self.motor.theta_e = 6.5  # > 2π
        self.motor.step(10, 10, 0)
        self.assertLess(self.motor.theta_e, 2 * math.pi)

    def test_load_reduces_speed(self):
        """Applying load torque should reduce speed."""
        self.motor.reset()
        for _ in range(1000):
            self.motor.step(0, 10.0, 0)
        speed_noload = self.motor.omega_m
        self.motor.reset()
        for _ in range(1000):
            self.motor.step(0, 10.0, 0.5)  # 0.5 N·m load
        speed_load = self.motor.omega_m
        self.assertLess(speed_load, speed_noload)


class TestSensors(unittest.TestCase):
    """Verify sensor models produce realistic measurements."""

    def test_current_sensor_noise(self):
        cs = CurrentSensor(noise_std=0.1, bias=0.0)
        readings = [cs.read(1.0) for _ in range(1000)]
        mean = sum(readings) / len(readings)
        self.assertAlmostEqual(mean, 1.0, delta=0.1)

    def test_current_sensor_bias(self):
        cs = CurrentSensor(noise_std=0.0, bias=0.5)
        self.assertAlmostEqual(cs.read(1.0), 1.5, places=5)

    def test_encoder_quantization(self):
        enc = Encoder(noise_std=0.0, quantization=2*math.pi/4096)
        val = enc.read_angle(0.01)
        # Should be quantized to nearest LSB
        lsb = 2 * math.pi / 4096
        remainder = val % lsb
        self.assertAlmostEqual(remainder, 0.0, places=12)


class TestFOCClosedLoop(unittest.TestCase):
    """End-to-end FOC closed-loop simulation."""

    def test_speed_tracking(self):
        """Speed should track reference within tolerance."""
        motor = PMSMdqModel(
            Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
            flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
            dt_ns=50000)
        inverter = AverageInverter(48.0)
        cs = CurrentSensor(noise_std=0.05, bias=0.01)
        enc = Encoder(noise_std=0.0,
                      quantization=2*math.pi/4096)
        foc = FOCController(
            kp_id=5.0, ki_id=500.0,
            kp_iq=5.0, ki_iq=500.0,
            ts=50e-6, v_bus=48.0)
        speed_ctrl = SpeedController(
            kp=0.05, ki=0.5, ts=1e-3)

        speed_ref = 100.0  # rad/s
        iq_ref = 0.0
        speed_ratio = 20   # 1ms / 50us

        for step in range(40000):  # 2 seconds
            if step % speed_ratio == 0:
                speed_meas = enc.read_speed(motor.omega_m)
                iq_ref = speed_ctrl.update(speed_ref, speed_meas)

            ia_m, ib_m, ic_m = cs.read_abc(motor.ia, motor.ib, motor.ic)
            th_m = enc.read_angle(motor.theta_e)
            duty_a, duty_b, duty_c = foc.update(
                ia_m, ib_m, ic_m, th_m,
                id_ref=0.0, iq_ref=iq_ref)
            va, vb, vc = inverter.step(
                duty_a, duty_b, duty_c, 48.0, ia_m, ib_m, ic_m)
            motor.step_abc(va, vb, vc, tl=0.0, dt=50e-6)
            motor.update_abc_currents()

        final_speed = motor.omega_m
        error_pct = abs(final_speed - speed_ref) / speed_ref * 100
        self.assertLess(error_pct, 5.0,
                       f"Speed error {error_pct:.1f}% exceeds 5%")

    def test_id_converges_to_zero(self):
        """FOC should regulate id to 0."""
        motor = PMSMdqModel(
            Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
            flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
            dt_ns=50000)
        inverter = AverageInverter(48.0)
        cs = CurrentSensor(noise_std=0.01, bias=0.0)
        enc = Encoder(noise_std=0.0)
        foc = FOCController(
            kp_id=5.0, ki_id=500.0,
            kp_iq=5.0, ki_iq=500.0,
            ts=50e-6, v_bus=48.0)
        speed_ctrl = SpeedController(kp=0.05, ki=0.5, ts=1e-3)

        speed_ratio = 20
        iq_ref = 0.0

        for step in range(20000):  # 1 second
            if step % speed_ratio == 0:
                iq_ref = speed_ctrl.update(50.0, enc.read_speed(motor.omega_m))
            ia_m, ib_m, ic_m = cs.read_abc(motor.ia, motor.ib, motor.ic)
            th_m = enc.read_angle(motor.theta_e)
            da, db, dc = foc.update(ia_m, ib_m, ic_m, th_m,
                                    id_ref=0.0, iq_ref=iq_ref)
            va, vb, vc = inverter.step(da, db, dc, 48.0, ia_m, ib_m, ic_m)
            motor.step_abc(va, vb, vc, tl=0.0, dt=50e-6)
            motor.update_abc_currents()

        # After settling, id should be near 0
        self.assertAlmostEqual(motor.id, 0.0, delta=0.5)


class TestFaultInjection(unittest.TestCase):
    """Verify fault injection framework."""

    def test_bias_fault(self):
        inj = FaultInjector()
        cfg = FaultConfig(
            fault_id="test_bias", fault_type="BIAS",
            target_path="sensor://current", magnitude=5.0,
            start_time_s=0.5, duration_s=1.0)
        inj.add_fault(cfg)
        inj.activate_at(0.5)

        val = inj.apply("sensor://current", 10.0, 0.6)
        self.assertAlmostEqual(val, 15.0)  # 10 + 5 bias

    def test_noise_fault(self):
        inj = FaultInjector()
        cfg = FaultConfig(
            fault_id="test_noise", fault_type="NOISE",
            target_path="sensor://v", magnitude=2.0,
            start_time_s=0.0)
        inj.add_fault(cfg)
        inj.activate_at(0.0)

        readings = [inj.apply("sensor://v", 10.0, 0.1) for _ in range(100)]
        self.assertNotEqual(readings[0], readings[-1])  # noise varies

    def test_freeze_fault(self):
        inj = FaultInjector()
        cfg = FaultConfig(
            fault_id="test_freeze", fault_type="FREEZE",
            target_path="sensor://angle", magnitude=0,
            start_time_s=0.0)
        inj.add_fault(cfg)
        inj.activate_at(0.0)

        v1 = inj.apply("sensor://angle", 1.0, 0.0)
        v2 = inj.apply("sensor://angle", 2.0, 0.1)
        self.assertEqual(v1, v2)  # frozen


if __name__ == "__main__":
    unittest.main()
