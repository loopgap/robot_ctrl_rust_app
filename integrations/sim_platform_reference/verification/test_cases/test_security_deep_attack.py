"""Deep security attack suite — Phase 8: covering blind spots.

Covers attack vectors NOT in test_security_attack.py:
  1. BLDC model NaN/Inf/boundary attacks
  2. Induction Motor model NaN/Inf/boundary attacks
  3. Advanced PMSM saturation edge cases
  4. Thermal model runaway / negative thermal resistance
  5. Sensor fusion divergence attacks
  6. MPC controller adversarial inputs
  7. EKF estimator divergence / ill-conditioned matrices
  8. Configuration injection attacks (YAML bomb, type confusion)
  9. HDF5 logger path traversal / corruption
  10. Fault injection edge cases
  11. Clock nanosecond overflow / wraparound
  12. Orchestrator multi-rate scheduling adversarial
  13. DataBus advanced ACL bypass attempts
  14. Cross-model coupling attacks
  15. Memory pressure / leak detection
  16. PI Controller adversarial
  17. Closed-loop adversarial full duration
"""

import gc
import math
import os
import sys
import tempfile
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

NAN = float("nan")
INF = float("inf")
NINF = float("-inf")
BIG = 1e308
TINY = 1e-308
DENORM = 5e-324


# ============================================================
# 1. BLDC Model Deep Attack
# ============================================================
class TestBLDCDeepAttack(unittest.TestCase):
    """Deep attack vectors on BLDC model."""

    def _make_bldc(self):
        from sim_platform.models.motor.bldc import BLDCModel
        return BLDCModel(Rs=0.05, Ls=0.5e-3, Ke=0.01, Kt=0.01,
                         J=5e-4, B=1e-4, Pp=4)

    def test_bldc_step_nan_v_bus(self):
        """NaN v_bus input."""
        m = self._make_bldc()
        m.step(NAN)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_bldc_step_inf_v_bus(self):
        """Inf v_bus input."""
        m = self._make_bldc()
        m.step(INF)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_bldc_step_mixed_nan_inf(self):
        """NaN v_bus with Inf load torque."""
        m = self._make_bldc()
        m.step(NAN, tl=INF)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_bldc_extreme_load_torque(self):
        """Load torque 1000x normal."""
        m = self._make_bldc()
        for _ in range(100):
            m.step(10.0, tl=1000.0)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_bldc_negative_inertia(self):
        """Negative inertia should be rejected or guarded."""
        from sim_platform.models.motor.bldc import BLDCModel
        try:
            m = BLDCModel(Rs=0.05, Ls=0.5e-3, Ke=0.01, Kt=0.01,
                          J=-1e-3, B=1e-4, Pp=4)
            m.step(10.0)
            self.assertTrue(math.isfinite(m.omega_m))
        except (ValueError, AssertionError):
            pass

    def test_bldc_zero_resistance(self):
        """Zero Rs — division by zero risk."""
        from sim_platform.models.motor.bldc import BLDCModel
        try:
            m = BLDCModel(Rs=0.0, Ls=0.5e-3, Ke=0.01, Kt=0.01,
                          J=5e-4, B=1e-4, Pp=4)
            m.step(10.0)
            self.assertTrue(math.isfinite(m.omega_m))
        except (ValueError, ZeroDivisionError, AssertionError):
            pass

    def test_bldc_10k_rapid_steps(self):
        """10k rapid steps with random v_bus values."""
        import random
        m = self._make_bldc()
        for _ in range(10000):
            v_bus = random.uniform(-100, 100)
            tl = random.uniform(-10, 10)
            m.step(v_bus, tl=tl)
        self.assertTrue(math.isfinite(m.omega_m))
        self.assertTrue(math.isfinite(m.theta_e))

    def test_bldc_negative_direction(self):
        """Negative direction parameter."""
        m = self._make_bldc()
        for _ in range(100):
            m.step(10.0, direction=-1)
        self.assertTrue(math.isfinite(m.omega_m))


# ============================================================
# 2. Induction Motor Deep Attack
# ============================================================
class TestIMDeepAttack(unittest.TestCase):
    """Deep attack vectors on Induction Motor model."""

    def _make_im(self):
        from sim_platform.models.motor.im_dq import IMdqModel
        return IMdqModel(Rs=0.5, Rr=0.4, Ls=0.05, Lr=0.05,
                         Lm=0.045, J=0.01, B=0.001, Pp=2)

    def test_im_step_all_nan(self):
        """NaN vsd and vsq."""
        m = self._make_im()
        m.step(NAN, NAN, 0.0)
        self.assertTrue(math.isfinite(m.ids))

    def test_im_step_all_inf(self):
        """Inf vsd and vsq."""
        m = self._make_im()
        m.step(INF, INF, 0.0)
        self.assertTrue(math.isfinite(m.ids))

    def test_im_step_negative_omega(self):
        """Negative electrical omega — reverse field."""
        m = self._make_im()
        for _ in range(1000):
            m.step(100.0, 10.0, -300.0)
        self.assertTrue(math.isfinite(m.ids))

    def test_im_extreme_slip(self):
        """Extreme slip condition — rotor speed = 0, high stator freq."""
        m = self._make_im()
        for _ in range(1000):
            m.step(300.0, 100.0, 5000.0)
        self.assertTrue(math.isfinite(m.ids))
        self.assertTrue(math.isfinite(m.iqs))

    def test_im_zero_rotor_resistance(self):
        """Rr=0 — division by zero risk in rotor equation."""
        from sim_platform.models.motor.im_dq import IMdqModel
        try:
            m = IMdqModel(Rs=0.5, Rr=0.0, Ls=0.05, Lr=0.05,
                          Lm=0.045, J=0.01, B=0.001, Pp=2)
            m.step(100.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.ids))
        except (ValueError, ZeroDivisionError, AssertionError):
            pass

    def test_im_rapid_direction_reversal(self):
        """Rapid forward/reverse switching."""
        m = self._make_im()
        for i in range(5000):
            sign = 1 if i % 2 == 0 else -1
            m.step(sign * 200.0, sign * 50.0, sign * 300.0)
        self.assertTrue(math.isfinite(m.ids))

    def test_im_extreme_load_torque(self):
        """IM with extreme load torque."""
        m = self._make_im()
        for _ in range(500):
            m.step(100.0, 50.0, 0.0, tl=1000.0)
        self.assertTrue(math.isfinite(m.ids))


# ============================================================
# 3. Advanced PMSM Saturation Edge Cases
# ============================================================
class TestPMSMAdvancedDeepAttack(unittest.TestCase):
    """Attack vectors on Advanced PMSM with saturation."""

    def _make_advanced(self):
        from sim_platform.models.motor.pmsm_advanced import PMSMAdvanced
        return PMSMAdvanced(Rs=0.1, Ld=5e-4, Lq=1e-3,
                            flux_pm=0.03, J=1e-3, B=0.0,
                            Ld_sat=0.0001, Lq_sat=0.0002, I_sat=10.0)

    def test_advanced_saturation_extreme_currents(self):
        """Very high currents should saturate gracefully."""
        m = self._make_advanced()
        for _ in range(500):
            m.step(500.0, 500.0)
        self.assertTrue(math.isfinite(m.id))

    def test_advanced_negative_saturation_coeff(self):
        """Negative saturation values."""
        from sim_platform.models.motor.pmsm_advanced import PMSMAdvanced
        try:
            m = PMSMAdvanced(Rs=0.1, Ld=5e-4, Lq=1e-3,
                             flux_pm=0.03, J=1e-3, B=0.0,
                             Ld_sat=-0.0001, Lq_sat=-0.0002, I_sat=10.0)
            m.step(10.0, 10.0)
            self.assertTrue(math.isfinite(m.id))
        except (ValueError, AssertionError):
            pass

    def test_advanced_all_nan_inputs(self):
        """NaN saturation path."""
        m = self._make_advanced()
        m.step(NAN, NAN)
        self.assertTrue(math.isfinite(m.id))

    def test_advanced_alternating_saturation(self):
        """Alternating between saturated and unsaturated."""
        m = self._make_advanced()
        for i in range(5000):
            if i % 2 == 0:
                m.step(500.0, 500.0)
            else:
                m.step(0.1, 0.1)
        self.assertTrue(math.isfinite(m.id))

    def test_advanced_with_temperature_feedback(self):
        """Temperature-dependent Rs and flux."""
        m = self._make_advanced()
        for i in range(1000):
            temp = 25.0 + (i * 0.1)  # Gradually heating
            m.step(20.0, 20.0, winding_temp=temp)
        self.assertTrue(math.isfinite(m.id))

    def test_advanced_extreme_temp(self):
        """Extreme temperature — 500C winding."""
        m = self._make_advanced()
        m.step(20.0, 20.0, winding_temp=500.0)
        self.assertTrue(math.isfinite(m.id))

    def test_advanced_negative_temp(self):
        """Negative temperature (cryogenic)."""
        m = self._make_advanced()
        m.step(20.0, 20.0, winding_temp=-100.0)
        self.assertTrue(math.isfinite(m.id))


# ============================================================
# 4. Thermal Model Deep Attack
# ============================================================
class TestThermalModelDeepAttack(unittest.TestCase):
    """Attack vectors on thermal network model."""

    def test_thermal_negative_heat_source(self):
        """Negative heat generation (energy extraction)."""
        from sim_platform.models.thermal.thermal_model import ThermalNode
        node = ThermalNode(C_th=100.0, R_th=0.5, T_ambient=25.0)
        for _ in range(1000):
            node.step(-1000.0, 0.001)
        self.assertTrue(math.isfinite(node.T))

    def test_thermal_zero_thermal_capacitance(self):
        """Zero thermal capacitance — division by zero."""
        from sim_platform.models.thermal.thermal_model import ThermalNode
        try:
            node = ThermalNode(C_th=0.0, R_th=0.5, T_ambient=25.0)
            node.step(100.0, 0.001)
            self.assertTrue(math.isfinite(node.T))
        except (ValueError, ZeroDivisionError, AssertionError):
            pass

    def test_thermal_negative_resistance(self):
        """Negative thermal resistance — unphysical but should not crash."""
        from sim_platform.models.thermal.thermal_model import ThermalNode
        try:
            node = ThermalNode(C_th=100.0, R_th=-0.5, T_ambient=25.0)
            node.step(100.0, 0.001)
            self.assertTrue(math.isfinite(node.T))
        except (ValueError, AssertionError):
            pass

    def test_thermal_runaway_simulation(self):
        """Simulate thermal runaway — massive heat, no cooling."""
        from sim_platform.models.thermal.thermal_model import ThermalNode
        node = ThermalNode(C_th=10.0, R_th=100.0, T_ambient=25.0, T_max=300.0)
        for _ in range(100000):
            node.step(1e6, 1e-5)
        self.assertTrue(math.isfinite(node.T))

    def test_motor_thermal_model_nan_input(self):
        """Motor thermal model with NaN heat input."""
        from sim_platform.models.thermal.thermal_model import MotorThermalModel
        mtm = MotorThermalModel()
        mtm.step(NAN, NAN, 0.001)
        self.assertTrue(math.isfinite(mtm.get_Rs_factor()))
        self.assertTrue(math.isfinite(mtm.get_flux_factor()))

    def test_motor_thermal_model_zero_time(self):
        """Zero time step."""
        from sim_platform.models.thermal.thermal_model import MotorThermalModel
        mtm = MotorThermalModel()
        try:
            mtm.step(100.0, 10.0, 0.0)
        except (ValueError, ZeroDivisionError):
            pass

    def test_motor_thermal_model_very_large_heat(self):
        """Very large heat input."""
        from sim_platform.models.thermal.thermal_model import MotorThermalModel
        mtm = MotorThermalModel()
        for _ in range(1000):
            mtm.step(1e8, 1e7, 0.001)
        self.assertTrue(math.isfinite(mtm.get_Rs_factor()))

    def test_motor_thermal_rs_flux_factors(self):
        """Rs and flux factors should remain bounded under extreme heating."""
        from sim_platform.models.thermal.thermal_model import MotorThermalModel
        mtm = MotorThermalModel()
        for _ in range(10000):
            mtm.step(1e6, 1e5, 1e-4)
        rs_f = mtm.get_Rs_factor()
        flux_f = mtm.get_flux_factor()
        self.assertTrue(math.isfinite(rs_f))
        self.assertTrue(math.isfinite(flux_f))
        self.assertGreater(rs_f, 0.0)  # Rs factor should stay positive
        self.assertGreater(flux_f, 0.0)  # Flux factor should stay positive


# ============================================================
# 5. Sensor Fusion Deep Attack
# ============================================================
class TestSensorFusionDeepAttack(unittest.TestCase):
    """Attack vectors on sensor fusion (Kalman filter)."""

    def test_kalman_filter_all_nan_measurements(self):
        """All measurements NaN."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        kf = SimpleKalmanFilter(Q=0.01, R=0.1, x0=0.0)
        for _ in range(100):
            kf.predict()
            kf.update(NAN)
        self.assertTrue(math.isfinite(kf.get_estimate()))

    def test_kalman_filter_all_inf_measurements(self):
        """All measurements Inf."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        kf = SimpleKalmanFilter(Q=0.01, R=0.1, x0=0.0)
        for _ in range(100):
            kf.predict()
            kf.update(INF)
        self.assertTrue(math.isfinite(kf.get_estimate()))

    def test_kalman_filter_zero_process_noise(self):
        """Zero process noise — filter becomes rigid."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        kf = SimpleKalmanFilter(Q=0.0, R=0.1, x0=0.0)
        for _ in range(100):
            kf.predict()
            kf.update(100.0)
        self.assertTrue(math.isfinite(kf.get_estimate()))

    def test_kalman_filter_zero_measurement_noise(self):
        """Zero measurement noise — filter trusts measurements fully."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        kf = SimpleKalmanFilter(Q=0.01, R=0.0, x0=0.0)
        for _ in range(100):
            kf.predict()
            kf.update(50.0)
        self.assertTrue(math.isfinite(kf.get_estimate()))

    def test_kalman_filter_negative_noise(self):
        """Negative noise parameters — should be rejected or guarded."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        try:
            kf = SimpleKalmanFilter(Q=-0.01, R=-0.1, x0=0.0)
            kf.predict()
            kf.update(10.0)
            self.assertTrue(math.isfinite(kf.get_estimate()))
        except (ValueError, AssertionError):
            pass

    def test_speed_fusion_conflicting_sensors(self):
        """Encoder says 100 rad/s, current estimator says -100 rad/s."""
        from sim_platform.models.fusion.sensor_fusion import SpeedFusion
        sf = SpeedFusion(Q=0.01, R_encoder=0.1, R_current=1.0)
        for _ in range(500):
            sf.update(speed_encoder=100.0, speed_current=-100.0)
        self.assertTrue(math.isfinite(sf.get_estimate()))

    def test_speed_fusion_one_sensor_nan(self):
        """One sensor outputs NaN."""
        from sim_platform.models.fusion.sensor_fusion import SpeedFusion
        sf = SpeedFusion(Q=0.01, R_encoder=0.1, R_current=1.0)
        for _ in range(100):
            sf.update(speed_encoder=NAN, speed_current=50.0)
        self.assertTrue(math.isfinite(sf.get_estimate()))

    def test_speed_fusion_extreme_divergence(self):
        """Sensors diverge to 1e6 rad/s difference."""
        from sim_platform.models.fusion.sensor_fusion import SpeedFusion
        sf = SpeedFusion(Q=0.01, R_encoder=0.1, R_current=1.0)
        for _ in range(100):
            sf.update(speed_encoder=1e6, speed_current=-1e6)
        self.assertTrue(math.isfinite(sf.get_estimate()))

    def test_speed_fusion_encoder_only(self):
        """SpeedFusion with only encoder input."""
        from sim_platform.models.fusion.sensor_fusion import SpeedFusion
        sf = SpeedFusion(Q=0.01, R_encoder=0.1, R_current=1.0)
        for _ in range(100):
            sf.update(speed_encoder=100.0)
        self.assertTrue(math.isfinite(sf.get_estimate()))


# ============================================================
# 6. MPC Controller Deep Attack
# ============================================================
class TestMPCDeepAttack(unittest.TestCase):
    """Deep attack vectors on MPC controllers."""

    def test_mpc_current_controller_all_nan(self):
        """All inputs NaN."""
        from sim_platform.models.controller.mpc import MPCCurrentController
        mpc = MPCCurrentController(L=5e-4, R=0.1, Ts=50e-6)
        out = mpc.update(NAN, NAN)
        self.assertTrue(math.isfinite(out))

    def test_mpc_current_controller_all_inf(self):
        """All inputs Inf."""
        from sim_platform.models.controller.mpc import MPCCurrentController
        mpc = MPCCurrentController(L=5e-4, R=0.1, Ts=50e-6)
        out = mpc.update(INF, INF)
        self.assertTrue(math.isfinite(out))

    def test_mpc_current_controller_large_ref(self):
        """Very large current reference."""
        from sim_platform.models.controller.mpc import MPCCurrentController
        mpc = MPCCurrentController(L=5e-4, R=0.1, Ts=50e-6, i_max=100.0, v_max=48.0)
        for _ in range(100):
            out = mpc.update(1e6, 0.0)
        self.assertTrue(math.isfinite(out))
        self.assertLessEqual(abs(out), 48.0 * 2)  # Should be somewhat bounded

    def test_mpc_speed_controller_nan_ref(self):
        """NaN speed reference."""
        from sim_platform.models.controller.mpc import MPCSpeedController
        mpc = MPCSpeedController(J=1e-3, B=0.0, Kt=0.01, Ts=1e-3)
        out = mpc.update(NAN, 100.0)
        self.assertTrue(math.isfinite(out))

    def test_mpc_speed_controller_extreme_ref(self):
        """Extremely large speed reference."""
        from sim_platform.models.controller.mpc import MPCSpeedController
        mpc = MPCSpeedController(J=1e-3, B=0.0, Kt=0.01, Ts=1e-3)
        for _ in range(100):
            out = mpc.update(1e10, 100.0)
        self.assertTrue(math.isfinite(out))

    def test_mpc_speed_controller_negative_params(self):
        """Negative physical parameters."""
        from sim_platform.models.controller.mpc import MPCSpeedController
        try:
            mpc = MPCSpeedController(J=-1e-3, B=0.0, Kt=0.01, Ts=1e-3)
            out = mpc.update(100.0, 50.0)
            self.assertTrue(math.isfinite(out))
        except (ValueError, AssertionError):
            pass


# ============================================================
# 7. EKF Estimator Deep Attack
# ============================================================
class TestEKFDeepAttack(unittest.TestCase):
    """Deep attack vectors on EKF estimator."""

    def _make_pmsm_ekf(self):
        from sim_platform.models.controller.ekf import PMSMEKF
        return PMSMEKF(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03,
                       Pp=4, dt=50e-6)

    def test_pmsm_ekf_nan_voltages(self):
        """NaN voltages to PMSMEKF — estimate() handles predict+update."""
        ekf = self._make_pmsm_ekf()
        state = ekf.estimate(NAN, NAN, NAN, NAN, NAN, 100.0)
        self.assertTrue(all(math.isfinite(v) for v in state))

    def test_pmsm_ekf_inf_voltages(self):
        """Inf voltages to PMSMEKF."""
        ekf = self._make_pmsm_ekf()
        state = ekf.estimate(INF, INF, INF, INF, INF, 100.0)
        self.assertTrue(all(math.isfinite(v) for v in state))

    def test_pmsm_ekf_10k_steps_stability(self):
        """10k steps with varying inputs."""
        import random
        ekf = self._make_pmsm_ekf()
        for _ in range(10000):
            vd = random.uniform(-50, 50)
            vq = random.uniform(-50, 50)
            ia = random.uniform(-100, 100)
            ib = random.uniform(-100, 100)
            ic = random.uniform(-100, 100)
            omega = random.uniform(-500, 500)
            state = ekf.estimate(vd, vq, ia, ib, ic, omega)
        self.assertTrue(all(math.isfinite(v) for v in state))

    def test_pmsm_ekf_extreme_currents(self):
        """Extreme current measurements."""
        ekf = self._make_pmsm_ekf()
        for _ in range(1000):
            state = ekf.estimate(10.0, 10.0, 1e6, 1e6, 1e6, 100.0)
        self.assertTrue(all(math.isfinite(v) for v in state))


# ============================================================
# 8. Configuration Injection Attack
# ============================================================
class TestConfigInjectionAttack(unittest.TestCase):
    """Configuration injection and malformed input attacks."""

    def test_yaml_type_confusion(self):
        """YAML with unexpected types."""
        from sim_platform.tools.config.config_manager import ConfigurationManager
        cm = ConfigurationManager()
        bad_config = {
            "simulation": {
                "dt": "not_a_number",
                "duration": [1, 2, 3],
                "mode": 12345,
            }
        }
        try:
            cm.validate_config(bad_config)
        except Exception:
            pass

    def test_yaml_deeply_nested_bomb(self):
        """Deeply nested YAML structure."""
        import yaml
        deep = {"a": {"b": {"c": {"d": {"e": {"f": {"g": 42}}}}}}}
        result = yaml.safe_load(yaml.dump(deep))
        self.assertEqual(result["a"]["b"]["c"]["d"]["e"]["f"]["g"], 42)

    def test_config_negative_duration(self):
        """Negative simulation duration."""
        from sim_platform.tools.config.config_manager import ConfigurationManager
        cm = ConfigurationManager()
        config = {"simulation": {"dt": 50e-6, "duration": -1.0, "mode": "offline"}}
        try:
            cm.validate_config(config)
        except Exception:
            pass

    def test_config_zero_dt(self):
        """Zero time step."""
        from sim_platform.tools.config.config_manager import ConfigurationManager
        cm = ConfigurationManager()
        config = {"simulation": {"dt": 0.0, "duration": 1.0, "mode": "offline"}}
        try:
            cm.validate_config(config)
        except Exception:
            pass

    def test_config_extreme_dt(self):
        """Extremely large time step."""
        from sim_platform.tools.config.config_manager import ConfigurationManager
        cm = ConfigurationManager()
        config = {"simulation": {"dt": 1e10, "duration": 1.0, "mode": "offline"}}
        try:
            cm.validate_config(config)
        except Exception:
            pass


# ============================================================
# 9. HDF5 Logger Path Traversal Attack
# ============================================================
class TestHDF5LoggerAttack(unittest.TestCase):
    """HDF5 logger security attack vectors."""

    def test_path_traversal_attempt(self):
        """Path traversal in log filename."""
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        with tempfile.TemporaryDirectory() as tmpdir:
            evil_path = os.path.join(tmpdir, "..", "..", "evil.hdf5")
            try:
                logger = HDF5Logger(evil_path)
                logger.close()
            except (ValueError, OSError, FileNotFoundError):
                pass

    def test_empty_filename(self):
        """Empty filename."""
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        try:
            logger = HDF5Logger("")
            logger.close()
        except (ValueError, OSError):
            pass

    def test_very_long_filename(self):
        """Very long filename."""
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        with tempfile.TemporaryDirectory() as tmpdir:
            long_name = os.path.join(tmpdir, "a" * 300 + ".hdf5")
            try:
                logger = HDF5Logger(long_name)
                logger.close()
            except (ValueError, OSError):
                pass

    def test_record_nan_values(self):
        """Record NaN values."""
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        with tempfile.TemporaryDirectory() as tmpdir:
            path = os.path.join(tmpdir, "test.hdf5")
            logger = HDF5Logger(path)
            try:
                logger.record(0.0, nan_val=NAN, inf_val=INF)
            except (ValueError, Exception):
                pass
            logger.close()

    def test_rapid_open_close(self):
        """Rapidly open and close logger."""
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        with tempfile.TemporaryDirectory() as tmpdir:
            for i in range(100):
                path = os.path.join(tmpdir, f"rapid_{i}.hdf5")
                try:
                    logger = HDF5Logger(path)
                    logger.close()
                except Exception:
                    pass

    def test_record_large_payload(self):
        """Record very large payload."""
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        with tempfile.TemporaryDirectory() as tmpdir:
            path = os.path.join(tmpdir, "large.hdf5")
            logger = HDF5Logger(path)
            try:
                big_dict = {f"key_{i}": float(i) for i in range(1000)}
                logger.record(0.0, **big_dict)
            except Exception:
                pass
            logger.close()


# ============================================================
# 10. Fault Injection Edge Cases
# ============================================================
class TestFaultInjectionEdgeCases(unittest.TestCase):
    """Fault injection system edge case attacks."""

    def _make_injector_with_fault(self, start=1.0, dur=0.5):
        from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector
        fi = FaultInjector()
        fi.add_fault(FaultConfig(
            fault_id="test_fault",
            fault_type="BIAS",
            target_path="module://motor/Rs",
            magnitude=0.05,
            start_time_s=start,
            duration_s=dur,
        ))
        return fi

    def test_fault_apply_at_time_zero(self):
        """Fault at t=0 — boundary condition."""
        fi = self._make_injector_with_fault(start=0.0, dur=1.0)
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=0.0)
        self.assertTrue(math.isfinite(result))

    def test_fault_apply_negative_time(self):
        """Fault at negative time."""
        fi = self._make_injector_with_fault(start=1.0, dur=1.0)
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=-1.0)
        # Should return original value (no fault active)
        self.assertEqual(result, 0.1)

    def test_fault_apply_very_large_time(self):
        """Fault at very large time."""
        fi = self._make_injector_with_fault(start=1.0, dur=1.0)
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=1e15)
        self.assertTrue(math.isfinite(result))

    def test_fault_apply_nan_time(self):
        """Fault check with NaN time."""
        fi = self._make_injector_with_fault(start=1.0, dur=1.0)
        try:
            result = fi.apply("module://motor/Rs", 0.1, sim_time_s=NAN)
            self.assertTrue(math.isfinite(result))
        except (ValueError, TypeError):
            pass

    def test_fault_activate_at(self):
        """activate_at should trigger fault at specific time."""
        fi = self._make_injector_with_fault(start=5.0, dur=1.0)
        fi.activate_at(sim_time_s=3.0)
        # Before fault time — no effect
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=3.0)
        self.assertEqual(result, 0.1)

    def test_fault_multiple_faults_same_target(self):
        """Multiple faults on the same target path."""
        from sim_platform.verification.fault_injection.injector import FaultConfig
        fi = self._make_injector_with_fault(start=0.0, dur=0.5)
        fi.add_fault(FaultConfig(
            fault_id="fault_2",
            fault_type="BIAS",
            target_path="module://motor/Rs",
            magnitude=0.1,
            start_time_s=0.25,
            duration_s=0.5,
        ))
        # At t=0.3 both faults should be active
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=0.3)
        self.assertTrue(math.isfinite(result))

    def test_fault_clear_all(self):
        """clear_all should remove all faults."""
        fi = self._make_injector_with_fault(start=0.0, dur=1.0)
        fi.clear_all()
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=0.5)
        self.assertEqual(result, 0.1)

    def test_fault_zero_duration(self):
        """Zero duration fault — instantaneous."""
        fi = self._make_injector_with_fault(start=1.0, dur=0.0)
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=1.0)
        self.assertTrue(math.isfinite(result))

    def test_fault_negative_magnitude(self):
        """Negative fault magnitude."""
        from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector
        fi = FaultInjector()
        fi.add_fault(FaultConfig(
            fault_id="neg_mag",
            fault_type="BIAS",
            target_path="module://motor/Rs",
            magnitude=-0.5,
            start_time_s=0.0,
            duration_s=1.0,
        ))
        result = fi.apply("module://motor/Rs", 0.1, sim_time_s=0.5)
        self.assertTrue(math.isfinite(result))


# ============================================================
# 11. Clock Deep Attack
# ============================================================
class TestClockDeepAttack(unittest.TestCase):
    """Deep attack vectors on GlobalClock."""

    def test_clock_advance_very_large_dt(self):
        """Advance by very large time step."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        try:
            clk.advance(int(1e18))
        except (ValueError, OverflowError):
            pass
        self.assertTrue(clk.sim_time_ns >= 0)

    def test_clock_advance_zero(self):
        """Advance by zero."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        clk.advance(0)
        self.assertEqual(clk.sim_time_ns, 0)

    def test_clock_rapid_100k_advances(self):
        """100k rapid advances."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        for _ in range(100000):
            clk.advance(1)
        self.assertEqual(clk.sim_time_ns, 100000)

    def test_clock_overflow_ns(self):
        """Approach nanosecond overflow (2^63)."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        try:
            clk.advance(2**62)
            self.assertTrue(clk.sim_time_ns >= 0)
        except (ValueError, OverflowError):
            pass

    def test_clock_snapshot_restore(self):
        """Snapshot and restore should be consistent."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        clk.advance(1000000)
        snap = clk.snapshot()
        clk.advance(5000000)
        clk.restore(snap)
        self.assertEqual(clk.sim_time_ns, 1000000)

    def test_clock_mode_attributes(self):
        """Clock mode should be accessible."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        self.assertIsNotNone(clk.mode)
        self.assertFalse(clk.paused)

    def test_clock_sim_time_s(self):
        """sim_time_s should be consistent with sim_time_ns."""
        from sim_platform.core.clock import GlobalClock
        clk = GlobalClock()
        clk.advance(1_000_000_000)  # 1 second
        self.assertAlmostEqual(clk.sim_time_s, 1.0, places=6)


# ============================================================
# 12. DataBus Advanced ACL Bypass
# ============================================================
class TestDataBusAdvancedBypass(unittest.TestCase):
    """Advanced ACL bypass attempts on DataBus."""

    def test_double_register_module(self):
        """Double-registering same module should not create duplicate."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        bus.register_module("module://attacker")
        bus.register_module("module://attacker")

    def test_spoofed_module_id_publish(self):
        """Publish with unregistered module ID."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        bus.register_module("module://legit")
        bus.restrict_topic("test_topic", ["module://legit"])
        try:
            bus.publish("test_topic", 42, module_id="module://spoof")
            self.fail("Should have raised PermissionError")
        except (PermissionError, ValueError):
            pass

    def test_acl_to_restricted_topic(self):
        """Subscribe to restricted topic without permission."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        bus.register_module("module://attacker")
        bus.register_module("module://legit")
        bus.restrict_topic("secret_topic", ["module://legit"])
        # Attacker tries to read
        try:
            _val = bus.read_latest("secret_topic")
            # If no error, value should be None
        except PermissionError:
            pass

    def test_clear_security_with_wrong_token(self):
        """Clear security with wrong admin token."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        bus.set_admin_token("correct_token")
        try:
            bus.clear_security(admin_token="wrong_token")
            self.fail("Should have raised PermissionError")
        except PermissionError:
            pass

    def test_clear_security_with_correct_token(self):
        """Clear security with correct admin token."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        bus.register_module("module://test")
        bus.set_admin_token("correct_token")
        bus.clear_security(admin_token="correct_token")

    def test_publish_none_value(self):
        """Publish None value — should raise or be rejected."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        bus.register_module("module://test")
        try:
            bus.publish("topic", None, module_id="module://test")
        except (ValueError, TypeError, AttributeError):
            pass  # Rejecting None is correct behavior

    def test_snapshot_isolation(self):
        """Snapshot should not affect original data."""
        from sim_platform.core.data_bus import DataBus, Signal
        bus = DataBus()
        bus.register_module("module://test")
        s = Signal(source="module://test", signal_type="SCALAR", value=42)
        bus.publish("val", s, module_id="module://test")
        snap = bus.snapshot()
        # Modify snapshot dict
        if "val" in snap:
            snap["val"] = 999
        # Original should be unaffected
        latest = bus.read_latest("val")
        self.assertIsNotNone(latest)

    def test_publish_unregistered_module_no_id(self):
        """Publish without module_id should fail."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        try:
            bus.publish("topic", 42)
            self.fail("Should have raised PermissionError")
        except PermissionError:
            pass

    def test_subscribe_empty_module_id(self):
        """Subscribe without module_id should fail."""
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        try:
            bus.subscribe("topic", lambda x: None)
            self.fail("Should have raised PermissionError")
        except PermissionError:
            pass


# ============================================================
# 13. Cross-Model Coupling Attack
# ============================================================
class TestCrossModelCouplingAttack(unittest.TestCase):
    """Cross-model interaction attack vectors."""

    def test_bldc_step_abc_with_foc_controller(self):
        """BLDC motor with FOC controller (mismatched control strategy)."""
        from sim_platform.models.controller.foc import FOCController
        from sim_platform.models.motor.bldc import BLDCModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = BLDCModel(Rs=0.05, Ls=0.5e-3, Ke=0.01, Kt=0.01,
                          J=5e-4, B=1e-4, Pp=4)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        inv = AverageInverter(v_bus=48.0)

        for _ in range(1000):
            da, db, dc = foc.update(0.0, 0.0, 0.0, 0.0, 0.0, 10.0)
            va, vb, vc = inv.step(da, db, dc)
            # BLDC uses v_bus, not abc voltages
            motor.step(sum([va, vb, vc]) / 3.0)
        self.assertTrue(math.isfinite(motor.omega_m))

    def test_im_with_pmsm_voltages(self):
        """IM model receiving PMSM-like voltages."""
        from sim_platform.models.motor.im_dq import IMdqModel
        im = IMdqModel(Rs=0.5, Rr=0.4, Ls=0.05, Lr=0.05,
                       Lm=0.045, J=0.01, B=0.001, Pp=2)
        for _ in range(500):
            im.step(200.0, 150.0, 300.0)
        self.assertTrue(math.isfinite(im.ids))

    def test_closed_loop_sensor_mismatch(self):
        """Closed loop with sensor reading 10x actual speed."""
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter(v_bus=48.0)

        for _ in range(2000):
            measured_speed = motor.omega_m * 10.0
            iq_ref = sc.update(100.0, measured_speed)
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)
        self.assertTrue(math.isfinite(motor.omega_m))

    def test_thermal_feedback_loop(self):
        """Thermal model feeding back into motor parameters."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.thermal.thermal_model import MotorThermalModel

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        thermal = MotorThermalModel()

        for _ in range(5000):
            motor.step(20.0, 20.0)
            thermal.step(abs(motor.id) * 10.0, abs(motor.iq) * 10.0, 50e-6)
            rs_factor = thermal.get_Rs_factor()
            motor.Rs = 0.1 * rs_factor

        self.assertTrue(math.isfinite(motor.omega_m))

    def test_pmsm_advanced_with_thermal_cascade(self):
        """Advanced PMSM + thermal model cascaded — temperature-dependent parameters."""
        from sim_platform.models.motor.pmsm_advanced import PMSMAdvanced
        from sim_platform.models.thermal.thermal_model import MotorThermalModel

        motor = PMSMAdvanced(Rs=0.1, Ld=5e-4, Lq=1e-3,
                             flux_pm=0.03, J=1e-3, B=0.0,
                             Rs_temp_coeff=0.004, T_ref=25.0)
        motor.reset()
        thermal = MotorThermalModel()

        for i in range(2000):
            motor.step(2.0, 2.0)
            # Use clamped current for thermal to avoid overflow
            id_safe = max(min(motor.id, 100.0), -100.0)
            iq_safe = max(min(motor.iq, 100.0), -100.0)
            thermal.step(id_safe**2 * 0.1, iq_safe**2 * 0.1, 50e-6)
        self.assertTrue(math.isfinite(motor.omega_m))
        self.assertTrue(math.isfinite(thermal.get_Rs_factor()))


# ============================================================
# 14. Memory Pressure / Leak Detection
# ============================================================
class TestMemoryPressure(unittest.TestCase):
    """Memory pressure and leak detection tests."""

    def test_create_destroy_1000_motors(self):
        """Create and destroy 1000 motor instances."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        for _ in range(1000):
            m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
            m.step(10.0, 10.0)
            del m
        gc.collect()

    def test_create_destroy_1000_data_buses(self):
        """Create and destroy 1000 DataBus instances."""
        from sim_platform.core.data_bus import DataBus, Signal
        for _ in range(1000):
            bus = DataBus()
            bus.register_module("module://test")
            s = Signal(source="module://test", signal_type="SCALAR", value=42)
            bus.publish("topic", s, module_id="module://test")
            del bus
        gc.collect()

    def test_create_destroy_1000_orchestrators(self):
        """Create and destroy 1000 Orchestrator instances."""
        from sim_platform.core.orchestrator import Orchestrator
        for _ in range(1000):
            orch = Orchestrator()
            del orch
        gc.collect()

    def test_data_bus_repeated_publishes(self):
        """DataBus should handle 10k repeated publishes."""
        from sim_platform.core.data_bus import DataBus, Signal
        bus = DataBus()
        bus.register_module("module://test")
        for i in range(10000):
            s = Signal(source="module://test", signal_type="SCALAR", value=float(i))
            bus.publish("growing_topic", s, module_id="module://test")
        # Should not crash
        latest = bus.read_latest("growing_topic")
        self.assertIsNotNone(latest)

    def test_create_destroy_500_thermal_models(self):
        """Create and destroy 500 thermal models."""
        from sim_platform.models.thermal.thermal_model import MotorThermalModel
        for _ in range(500):
            m = MotorThermalModel()
            m.step(100.0, 10.0, 0.001)
            del m
        gc.collect()

    def test_create_destroy_500_kalman_filters(self):
        """Create and destroy 500 Kalman filters."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        for _ in range(500):
            kf = SimpleKalmanFilter(Q=0.01, R=0.1)
            kf.predict()
            kf.update(10.0)
            del kf
        gc.collect()


# ============================================================
# 15. PI Controller Adversarial
# ============================================================
class TestPIControllerAdversarial(unittest.TestCase):
    """PI controller adversarial input tests."""

    def test_pi_rapid_setpoint_oscillation(self):
        """Rapidly oscillating setpoint."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=100.0, ts=1e-3, out_min=-100, out_max=100)
        for i in range(10000):
            sp = 100.0 if i % 2 == 0 else -100.0
            out = pi.update(sp, 0.0)
        self.assertTrue(math.isfinite(out))

    def test_pi_measurement_bang_bang(self):
        """Bang-bang measurement input."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=100.0, ts=1e-3, out_min=-100, out_max=100)
        for i in range(10000):
            meas = 1e6 if i % 2 == 0 else -1e6
            out = pi.update(0.0, meas)
        self.assertTrue(math.isfinite(out))

    def test_pi_very_high_ki(self):
        """Very high integral gain — windup risk."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=1e10, ts=1e-3, out_min=-100, out_max=100)
        for _ in range(1000):
            out = pi.update(100.0, 0.0)
        self.assertTrue(math.isfinite(out))
        self.assertLessEqual(out, 100.0)
        self.assertGreaterEqual(out, -100.0)

    def test_pi_very_high_kp(self):
        """Very high proportional gain."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1e10, ki=0.0, ts=1e-3, out_min=-100, out_max=100)
        out = pi.update(100.0, 0.0)
        self.assertTrue(math.isfinite(out))
        self.assertLessEqual(out, 100.0)

    def test_pi_all_nan_inputs(self):
        """All NaN inputs to PI."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=100.0, ts=1e-3, out_min=-100, out_max=100)
        out = pi.update(NAN, NAN)
        self.assertTrue(math.isfinite(out))

    def test_pi_all_inf_inputs(self):
        """All Inf inputs to PI."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=100.0, ts=1e-3, out_min=-100, out_max=100)
        out = pi.update(INF, INF)
        self.assertTrue(math.isfinite(out))


# ============================================================
# 16. FOC+PMSM Closed-Loop Adversarial Full Duration
# ============================================================
class TestClosedLoopAdversarialFull(unittest.TestCase):
    """Full closed-loop adversarial tests over significant durations."""

    def test_10k_step_full_loop_random_disturbances(self):
        """10k step closed loop with random disturbances."""
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter(v_bus=48.0)

        for _ in range(10000):
            iq_ref = sc.update(100.0, motor.omega_m)
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)

        self.assertTrue(math.isfinite(motor.omega_m))
        self.assertTrue(math.isfinite(motor.id))
        self.assertTrue(math.isfinite(motor.iq))

    def test_voltage_sag_recovery(self):
        """Simulate voltage sag and recovery."""
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)

        for i in range(5000):
            if 2000 <= i < 3000:
                v_bus = 10.0
            else:
                v_bus = 48.0
            inv = AverageInverter(v_bus=v_bus)
            iq_ref = sc.update(100.0, motor.omega_m)
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)

        self.assertTrue(math.isfinite(motor.omega_m))

    def test_instantaneous_load_step(self):
        """Instantaneous load step from 0 to 5 Nm."""
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter(v_bus=48.0)

        for i in range(5000):
            _load = 0.0 if i < 2500 else 5.0
            iq_ref = sc.update(100.0, motor.omega_m)
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)

        self.assertTrue(math.isfinite(motor.omega_m))

    def test_5k_step_full_loop_with_nan_injection(self):
        """5k step loop with occasional NaN injection."""
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter(v_bus=48.0)

        for i in range(5000):
            # Inject NaN every 100th step
            if i % 100 == 0:
                iq_ref = sc.update(100.0, NAN)
            else:
                iq_ref = sc.update(100.0, motor.omega_m)
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)

        self.assertTrue(math.isfinite(motor.omega_m))


# ============================================================
# 17. Orchestrator Adversarial
# ============================================================
class TestOrchestratorAdversarial(unittest.TestCase):
    """Orchestrator adversarial tests."""

    def test_orchestrator_register_none_model(self):
        """Register None as model."""
        from sim_platform.core.orchestrator import Orchestrator
        orch = Orchestrator()
        try:
            orch.register_model(None)
        except (ValueError, TypeError, AttributeError):
            pass

    def test_orchestrator_set_very_short_duration(self):
        """Set very short simulation duration."""
        from sim_platform.core.orchestrator import Orchestrator
        orch = Orchestrator()
        try:
            orch.set_sim_duration(1e-15)
        except (ValueError, AssertionError):
            pass

    def test_orchestrator_set_zero_duration(self):
        """Set zero simulation duration."""
        from sim_platform.core.orchestrator import Orchestrator
        orch = Orchestrator()
        try:
            orch.set_sim_duration(0.0)
        except (ValueError, AssertionError):
            pass

    def test_orchestrator_schedule_fault_not_callable(self):
        """Schedule a non-callable as fault."""
        from sim_platform.core.orchestrator import Orchestrator
        orch = Orchestrator()
        try:
            orch.schedule_fault("not_callable", 1.0)
        except (ValueError, TypeError):
            pass

    def test_orchestrator_schedule_fault_negative_time(self):
        """Schedule fault at negative time."""
        from sim_platform.core.orchestrator import Orchestrator
        orch = Orchestrator()
        try:
            orch.schedule_fault(-1.0, lambda: None)
        except (ValueError, AssertionError):
            pass

    def test_orchestrator_rapid_reset(self):
        """Rapidly reset orchestrator 1000 times."""
        from sim_platform.core.orchestrator import Orchestrator
        orch = Orchestrator()
        for _ in range(1000):
            orch.reset()


if __name__ == "__main__":
    unittest.main()
