"""Tests for thermal model and sensor fusion."""

import math
import os
import sys
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter, SpeedFusion
from sim_platform.models.thermal.thermal_model import MotorThermalModel, ThermalNode


class TestThermalNode(unittest.TestCase):
    """ThermalNode basic tests."""

    def test_init_defaults(self):
        t = ThermalNode()
        self.assertEqual(t.T, 25.0)
        self.assertEqual(t.T_ambient, 25.0)

    def test_heating(self):
        t = ThermalNode(R_th=1.0, C_th=100.0, T_ambient=25.0)
        # Apply 10W for 1000 seconds (10000 steps * 0.1s)
        for _ in range(10000):
            t.step(10.0, 0.1)
        # Temperature should rise toward T_ambient + P*R_th = 35°C
        self.assertGreater(t.T, 30.0)
        self.assertAlmostEqual(t.T, 35.0, delta=1.0)

    def test_cooling(self):
        t = ThermalNode(R_th=1.0, C_th=100.0, T_ambient=25.0)
        t.T = 100.0  # Start hot
        # No power loss, should cool toward ambient
        for _ in range(10000):
            t.step(0.0, 0.1)
        self.assertLess(t.T, 30.0)

    def test_nan_power(self):
        t = ThermalNode()
        t.step(float('nan'), 0.1)
        self.assertTrue(math.isfinite(t.T))

    def test_inf_power(self):
        t = ThermalNode()
        t.step(float('inf'), 0.1)
        self.assertTrue(math.isfinite(t.T))

    def test_negative_dt(self):
        t = ThermalNode()
        T_before = t.T
        t.step(10.0, -0.1)
        self.assertEqual(t.T, T_before)

    def test_overheating_detection(self):
        t = ThermalNode(T_max=50.0)
        t.T = 60.0
        self.assertTrue(t.is_overheating)

    def test_thermal_derating(self):
        t = ThermalNode(T_max=100.0)
        t.T = 80.0
        self.assertEqual(t.thermal_derating, 1.0)
        t.T = 120.0
        self.assertLess(t.thermal_derating, 1.0)
        self.assertGreater(t.thermal_derating, 0.0)

    def test_reset(self):
        t = ThermalNode()
        t.T = 100.0
        t.P_loss = 50.0
        t.reset()
        self.assertEqual(t.T, 25.0)
        self.assertEqual(t.P_loss, 0.0)


class TestMotorThermalModel(unittest.TestCase):
    """MotorThermalModel tests."""

    def test_init(self):
        m = MotorThermalModel()
        self.assertEqual(m.winding.T, 25.0)
        self.assertEqual(m.magnet.T, 25.0)

    def test_heating(self):
        m = MotorThermalModel()
        # Apply copper loss for 100 seconds
        for _ in range(1000):
            m.step(copper_loss_W=10.0, iron_loss_W=5.0, dt=0.1)
        self.assertGreater(m.winding.T, 25.0)
        self.assertGreater(m.magnet.T, 25.0)

    def test_rs_factor(self):
        m = MotorThermalModel()
        m.winding.T = 125.0  # Hot winding
        factor = m.get_Rs_factor(T_ref=25.0)
        # Rs should increase ~39% for 100K rise
        self.assertGreater(factor, 1.3)
        self.assertLess(factor, 1.5)

    def test_flux_factor(self):
        m = MotorThermalModel()
        m.magnet.T = 80.0  # Hot magnet
        factor = m.get_flux_factor(T_ref=25.0)
        # Flux should decrease
        self.assertLess(factor, 1.0)
        self.assertGreater(factor, 0.5)

    def test_overheating(self):
        m = MotorThermalModel(T_max_winding=50.0)
        m.winding.T = 60.0
        self.assertTrue(m.is_overheating)

    def test_reset(self):
        m = MotorThermalModel()
        m.winding.T = 100.0
        m.magnet.T = 80.0
        m.reset()
        self.assertEqual(m.winding.T, 25.0)
        self.assertEqual(m.magnet.T, 25.0)


class TestSimpleKalmanFilter(unittest.TestCase):
    """SimpleKalmanFilter tests."""

    def test_init(self):
        kf = SimpleKalmanFilter()
        self.assertEqual(kf.x, 0.0)

    def test_predict(self):
        kf = SimpleKalmanFilter()
        kf.predict(u=1.0)
        self.assertEqual(kf.x, 1.0)

    def test_update(self):
        kf = SimpleKalmanFilter(Q=0.01, R=1.0)
        kf.update(10.0)
        # With P=1.0, R=1.0, K=0.5, x = 0 + 0.5*(10-0) = 5.0
        self.assertAlmostEqual(kf.x, 5.0, delta=0.1)

    def test_convergence(self):
        """Repeated measurements should converge."""
        kf = SimpleKalmanFilter(Q=0.01, R=1.0)
        for _ in range(100):
            kf.update(10.0)
        self.assertAlmostEqual(kf.x, 10.0, delta=0.1)

    def test_nan_measurement(self):
        kf = SimpleKalmanFilter()
        kf.update(float('nan'))
        self.assertTrue(math.isfinite(kf.x))

    def test_inf_measurement(self):
        kf = SimpleKalmanFilter()
        kf.update(float('inf'))
        self.assertTrue(math.isfinite(kf.x))

    def test_uncertainty_decreases(self):
        kf = SimpleKalmanFilter()
        P_before = kf.P
        kf.update(5.0)
        self.assertLess(kf.P, P_before)

    def test_reset(self):
        kf = SimpleKalmanFilter()
        kf.update(10.0)
        kf.reset(5.0)
        self.assertEqual(kf.x, 5.0)


class TestSpeedFusion(unittest.TestCase):
    """SpeedFusion tests."""

    def test_basic_fusion(self):
        sf = SpeedFusion()
        est = sf.update(speed_encoder=100.0)
        self.assertTrue(math.isfinite(est))

    def test_fusion_with_current(self):
        sf = SpeedFusion()
        est = sf.update(speed_encoder=100.0, speed_current=95.0)
        self.assertTrue(math.isfinite(est))

    def test_fusion_convergence(self):
        sf = SpeedFusion()
        for _ in range(100):
            est = sf.update(speed_encoder=100.0)
        self.assertAlmostEqual(est, 100.0, delta=1.0)

    def test_nan_encoder(self):
        sf = SpeedFusion()
        est = sf.update(speed_encoder=float('nan'))
        self.assertTrue(math.isfinite(est))

    def test_inf_encoder(self):
        sf = SpeedFusion()
        est = sf.update(speed_encoder=float('inf'))
        self.assertTrue(math.isfinite(est))

    def test_nan_current(self):
        sf = SpeedFusion()
        est = sf.update(speed_encoder=100.0, speed_current=float('nan'))
        self.assertTrue(math.isfinite(est))

    def test_reset(self):
        sf = SpeedFusion()
        sf.update(100.0)
        sf.reset()
        self.assertEqual(sf.get_estimate(), 0.0)

    def test_uncertainty(self):
        sf = SpeedFusion()
        u1 = sf.get_uncertainty()
        sf.update(100.0)
        u2 = sf.get_uncertainty()
        self.assertLess(u2, u1)


if __name__ == "__main__":
    unittest.main()
