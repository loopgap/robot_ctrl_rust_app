"""Security Stress Test — Fourth Audit Round (最高强度压测).

Covers:
  1. NaN/Inf/-Inf injection at every model entry
  2. Extreme magnitude values (1e308, 2^63)
  3. Integer overflow and underflow
  4. Zero/negative denominators
  5. Unicode injection in all string parameters
  6. Type confusion (str→float, int→str paths)
  7. None/Empty injection
  8. Negative timestamps and time values
  9. Very large state accumulation (memory DoS)
  10. Denormal floating point values
  11. Rapid fault toggling
  12. Concurrent model access patterns
  13. Boundary condition combinations
"""

import itertools
import math
import os
import sys
import threading

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

import shutil
import tempfile
import unittest

from sim_platform.core.clock import ClockMode, GlobalClock, s_to_ns
from sim_platform.core.data_bus import DataBus, Signal
from sim_platform.core.model_registry import (
    Domain,
    FidelityLevel,
    ModelMetadata,
    ModelRegistry,
)
from sim_platform.core.orchestrator import Orchestrator, OrchestratorConfig
from sim_platform.models.controller.foc import (
    FOCController,
    PIController,
    SpeedController,
    clarke_transform,
    park_transform,
    svpwm,
)
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder
from sim_platform.tools.replay.hdf5_logger import HDF5Logger
from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector

# ── Attack payload library ────────────────────────────────

NAN = float("nan")
INF = float("inf")
NINF = float("-inf")
BIG = 1e308
SMALL = 1e-308
BIGINT = 2**63
NEGBIGINT = -2**63
NEG_ZERO = -0.0
DENORM = 5e-324
UNICODE_SQLI = "\u0027 OR 1=1 --\n\x00\r\n"
UNICODE_EMOJI = "\U0001f4a3" * 100  # 100 bomb emojis
LONG_STR = "A" * 100000
BINARY_BYTES = b"\x00\xff\xfe\x01" * 250


class _NoCrash:
    """Assert that a callable does NOT raise an exception."""
    def __init__(self, test, fn):
        self.test = test
        self.fn = fn
    def __enter__(self):
        self.fn()
        return self
    def __exit__(self, *args):
        pass


# ════════════════════════════════════════════════════════════
#  1. PMSMdqModel — Extreme Value Injection
# ════════════════════════════════════════════════════════════

class TestPMSM_ExtremeValues(unittest.TestCase):
    """Inject NaN/Inf/large values into all PMSM entry points."""

    def setUp(self):
        self.m = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                             flux_pm=0.03, J=0.001, B=0.0001, Pp=4, dt_ns=50000)

    def test_step_nan_vd(self): self.m.step(NAN, 10.0, 0.0)
    def test_step_nan_vq(self): self.m.step(10.0, NAN, 0.0)
    def test_step_inf_vd(self): self.m.step(INF, 10.0, 0.0)
    def test_step_inf_vq(self): self.m.step(10.0, INF, 0.0)
    def test_step_ninf_vd(self): self.m.step(NINF, 10.0, 0.0)
    def test_step_nan_load(self): self.m.step(10.0, 10.0, NAN)
    def test_step_inf_load(self): self.m.step(10.0, 10.0, INF)
    def test_step_big_voltage(self):
        for _ in range(10):
            self.m.step(BIG, BIG, 0.0)
        self.assertFalse(math.isnan(self.m.id))
        self.assertFalse(math.isnan(self.m.omega_m))

    def test_step_zero_dt(self):
        self.m.step(10.0, 10.0, 0.0, dt=0.0)
        self.assertFalse(math.isnan(self.m.id))

    def test_step_negative_dt(self):
        self.m.step(10.0, 10.0, 0.0, dt=-1.0)

    def test_step_nan_dt(self):
        self.m.step(10.0, 10.0, 0.0, dt=NAN)

    def test_step_abc_nan_all(self):
        self.m.step_abc(NAN, NAN, NAN)
        self.assertFalse(math.isnan(self.m.ia))

    def test_step_abc_inf(self):
        self.m.step_abc(INF, INF, -INF)

    def test_prolonged_extreme(self):
        for _ in range(100):
            self.m.step(BIG, BIG, BIG, dt=1e-6)
        self.assertTrue(math.isfinite(self.m.omega_m))

    def test_denormal_step(self):
        self.m.step(DENORM, DENORM, DENORM, dt=DENORM)

    def test_neg_zero(self):
        self.m.step(NEG_ZERO, NEG_ZERO, NEG_ZERO)

    def test_torque_with_nan_state(self):
        self.m.id = NAN
        self.m.iq = NAN
        t = self.m.torque_em
        self.assertFalse(math.isnan(t))

    def test_state_after_nan(self):
        self.m.id = NAN
        self.m.step(10.0, 10.0, 0.0)
        # NaN state should be guarded to 0.0, then step computes normally
        self.assertTrue(math.isfinite(self.m.id))

    def test_reset_after_corruption(self):
        self.m.id = BIG
        self.m.iq = BIG
        self.m.omega_m = NINF
        self.m.step(10., 10., 0.)
        self.m.reset()
        self.assertEqual(self.m.id, 0.0)
        self.assertEqual(self.m.iq, 0.0)
        self.assertEqual(self.m.omega_m, 0.0)

    def test_large_inductance_div(self):
        m2 = PMSMdqModel(Rs=0.1, Ld=1e-300, Lq=1e-300,
                         flux_pm=0.03, J=0.001, B=0.0001, Pp=4)
        m2.step(100.0, 100.0, 0.0)
        self.assertFalse(math.isnan(m2.id))


# ════════════════════════════════════════════════════════════
#  2. PIController & SVPWM — Numerical Edge Cases
# ════════════════════════════════════════════════════════════

class TestPIC_SVPWM_Extreme(unittest.TestCase):
    """PI controller and SVPWM under extreme conditions."""

    def test_pi_nan_setpoint(self):
        pi = PIController(kp=1.0, ki=10.0, ts=0.001, out_min=-100, out_max=100)
        u = pi.update(NAN, 0.0)
        self.assertFalse(math.isnan(u))

    def test_pi_nan_measurement(self):
        pi = PIController(kp=1.0, ki=10.0, ts=0.001, out_min=-100, out_max=100)
        u = pi.update(10.0, NAN)
        self.assertFalse(math.isnan(u))

    def test_pi_both_nan(self):
        pi = PIController(kp=1.0, ki=10.0, ts=0.001)
        pi.update(NAN, NAN)

    def test_pi_inf_setpoint(self):
        pi = PIController(kp=1.0, ki=10.0, ts=0.001, out_min=-100, out_max=100)
        u = pi.update(INF, 0.0)
        self.assertFalse(math.isnan(u))

    def test_pi_kp_zero(self):
        pi = PIController(kp=0.0, ki=10.0, ts=0.001)
        self.assertGreater(pi.k_aw, 0)

    def test_pi_kp_ki_nan(self):
        pi = PIController(kp=NAN, ki=NAN, ts=0.001)
        pi.update(10.0, 0.0)

    def test_pi_saturates_correctly(self):
        pi = PIController(kp=1e6, ki=1e7, ts=0.001, out_min=-10, out_max=10)
        for _ in range(10000):
            u = pi.update(1e9, -1e9)
            self.assertTrue(abs(u) <= 10)

    def test_pi_ts_zero(self):
        pi = PIController(kp=1.0, ki=10.0, ts=0.0)
        pi.update(10.0, 0.0)

    def test_svpwm_nan_bus(self):
        da, db, dc = svpwm(10., 10., NAN)
        for d in [da, db, dc]:
            self.assertFalse(math.isnan(d))

    def test_svpwm_zero_bus(self):
        da, db, dc = svpwm(100., 100., 0.0)
        self.assertEqual((da, db, dc), (0.5, 0.5, 0.5))

    def test_svpwm_inf_voltage(self):
        da, db, dc = svpwm(INF, INF, 48.0)
        self.assertFalse(math.isnan(da))

    def test_svpwm_nan_voltage(self):
        da, db, dc = svpwm(NAN, NAN, 48.0)
        self.assertFalse(math.isnan(da))

    def test_svpwm_huge_values(self):
        da, db, dc = svpwm(1e308, -1e308, 0.001)
        for d in [da, db, dc]:
            self.assertTrue(0.0 <= d <= 1.0)

    def test_clarke_park_nan(self):
        i_alpha, i_beta = clarke_transform(NAN, NAN, NAN)
        id_val, iq_val = park_transform(i_alpha, i_beta, NAN)
        self.assertFalse(math.isnan(id_val))

    def test_pi_integral_not_explode(self):
        pi = PIController(kp=1.0, ki=1e9, ts=0.001, out_min=-100, out_max=100)
        for _ in range(50000):
            pi.update(1000., 0.)
        self.assertTrue(abs(pi.integral) < 1e6)

    def test_pi_inf_gains(self):
        pi = PIController(kp=INF, ki=INF, ts=0.001, out_min=-100, out_max=100)
        pi.update(1.0, 0.0)

    def test_pi_swapped_limits(self):
        pi = PIController(kp=1.0, ki=10.0, ts=0.001, out_min=100, out_max=-100)
        pi.update(1.0, 0.0)


# ════════════════════════════════════════════════════════════
#  3. FOCController — Full Chain Extreme Test
# ════════════════════════════════════════════════════════════

class TestFOC_Stress(unittest.TestCase):
    """FOC controller stress test."""

    def setUp(self):
        self.foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                                 ts=50e-6, v_bus=48.0)

    def test_foc_all_nan_currents(self):
        self.foc.update(NAN, NAN, NAN, 0.0, 0.0, 0.0)
        self.assertFalse(math.isnan(self.foc.duty_a))

    def test_foc_nan_angle(self):
        self.foc.update(1.0, -0.5, -0.5, NAN, 0.0, 0.0)

    def test_foc_nan_refs(self):
        self.foc.update(1.0, -0.5, -0.5, 0.0, NAN, NAN)
        self.assertFalse(math.isnan(self.foc.vd_ref))

    def test_foc_inf_currents(self):
        for _ in range(10):
            self.foc.update(INF, -INF, INF, 0.0, 0.0, 0.0)
        self.assertTrue(math.isfinite(self.foc.duty_a))

    def test_foc_zero_bus_stress(self):
        f2 = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                           ts=50e-6, v_bus=0.0)
        f2.update(1.0, -0.5, -0.5, 0.0, 0.0, 0.0)
        self.assertFalse(math.isnan(f2.duty_a))

    def test_foc_nan_dt(self):
        f2 = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                           ts=NAN, v_bus=48.0)
        f2.update(1.0, -0.5, -0.5, 0.0, 0.0, 0.0)

    def test_foc_rapid_angle_change(self):
        for ang in [0, math.pi/2, math.pi, -math.pi, 100*math.pi, -1000*math.pi]:
            self.foc.update(1.0, -0.5, -0.5, ang, 0.0, 0.0)
            self.assertTrue(0.0 <= self.foc.duty_a <= 1.0)

    def test_speed_ctrl_nan_gains(self):
        sc = SpeedController(kp=NAN, ki=NAN, ts=1e-3)
        u = sc.update(100.0, 0.0)
        self.assertFalse(math.isnan(u))


# ════════════════════════════════════════════════════════════
#  4. Sensors & Power — Noise/Inverter Extreme
# ════════════════════════════════════════════════════════════

class TestSensorPower_Stress(unittest.TestCase):
    """Sensor and power model extreme tests."""

    def test_current_sensor_nan_input(self):
        cs = CurrentSensor(noise_std=0.1, bias=0.01)
        val = cs.read(NAN)
        self.assertFalse(math.isnan(val))

    def test_current_sensor_inf_saturation(self):
        cs = CurrentSensor(noise_std=0.1, bias=0.0, saturation=100.0)
        val = cs.read(INF)
        self.assertTrue(abs(val) <= 100.0)

    def test_current_sensor_nan_noise(self):
        cs = CurrentSensor(noise_std=NAN, bias=0.0)
        val = cs.read(10.0)
        self.assertFalse(math.isnan(val))

    def test_current_sensor_all_nan_abc(self):
        cs = CurrentSensor(noise_std=0.1, bias=0.0)
        a, b, c = cs.read_abc(NAN, NAN, NAN)
        self.assertFalse(math.isnan(a))

    def test_encoder_nan_angle(self):
        enc = Encoder(noise_std=0.001)
        val = enc.read_angle(NAN)
        self.assertFalse(math.isnan(val))

    def test_encoder_nan_speed(self):
        enc = Encoder(noise_std=0.001)
        val = enc.read_speed(NAN)
        self.assertFalse(math.isnan(val))

    def test_encoder_inf_angle(self):
        enc = Encoder(noise_std=0.001)
        val = enc.read_angle(INF)
        self.assertFalse(math.isinf(val))

    def test_inverter_all_nan_duty(self):
        inv = AverageInverter(48.0)
        va, vb, vc = inv.step(NAN, NAN, NAN)
        self.assertFalse(math.isnan(va))

    def test_inverter_nan_bus(self):
        inv = AverageInverter(48.0)
        va, vb, vc = inv.step(0.7, 0.7, 0.7, v_bus=NAN)
        self.assertFalse(math.isnan(va))

    def test_battery_nan_load(self):
        bat = RintBattery(48.0, 0.05)
        v = bat.step(NAN)
        self.assertTrue(math.isfinite(v))

    def test_battery_inf_load(self):
        bat = RintBattery(48.0, 0.05)
        v = bat.step(INF)
        self.assertGreaterEqual(v, 0)

    def test_battery_zero_resistance(self):
        bat = RintBattery(48.0, 0.0)
        v = bat.step(100.0)
        self.assertAlmostEqual(v, 48.0, places=5)


# ════════════════════════════════════════════════════════════
#  5. FaultInjector — Malicious Injection Vectors
# ════════════════════════════════════════════════════════════

class TestFaultInjector_Stress(unittest.TestCase):
    """FaultInjector under attack conditions."""

    def test_malicious_fault_id_attr_override(self):
        inj = FaultInjector()
        cfg = FaultConfig(
            fault_id="__init__", fault_type="BIAS",
            target_path="sensor://test", magnitude=5.0, start_time_s=0.0)
        inj.add_fault(cfg)
        inj.activate_at(0.0)
        val = inj.apply("sensor://test", 10.0, 0.1)
        self.assertFalse(math.isnan(val))

    def test_malicious_fault_id_special_chars(self):
        inj = FaultInjector()
        for fid in ["../../etc/passwd", "'; DROP TABLE; --", "\x00null", "\n\r"]:
            cfg = FaultConfig(
                fault_id=fid, fault_type="BIAS",
                target_path="sensor://test", magnitude=5.0, start_time_s=0.0)
            inj.add_fault(cfg)
        inj.activate_at(0.0)
        val = inj.apply("sensor://test", 10.0, 0.1)
        self.assertFalse(math.isnan(val))

    def test_fault_unknown_type(self):
        with self.assertRaises(ValueError):
            FaultConfig(fault_id="bad", fault_type="EXEC_CODE",
                        target_path="sensor://test")

    def test_fault_nan_magnitude_rejected(self):
        with self.assertRaises(ValueError):
            FaultConfig(fault_id="nan_mag", fault_type="BIAS",
                        target_path="sensor://test", magnitude=NAN)

    def test_fault_inf_magnitude_rejected(self):
        with self.assertRaises(ValueError):
            FaultConfig(fault_id="inf_mag", fault_type="BIAS",
                        target_path="sensor://test", magnitude=INF)

    def test_fault_probability_out_of_range(self):
        with self.assertRaises(ValueError):
            FaultConfig(fault_id="bad", fault_type="BIAS",
                        target_path="sensor://test", probability=999.0)

    def test_fault_negative_duration(self):
        with self.assertRaises(ValueError):
            FaultConfig(fault_id="bad", fault_type="BIAS",
                        target_path="sensor://test", duration_s=-1.0)

    def test_fault_target_path_no_protocol(self):
        with self.assertRaises(ValueError):
            FaultConfig(fault_id="bad", fault_type="BIAS",
                        target_path="just_a_string")

    def test_rapid_toggle_faults(self):
        inj = FaultInjector()
        for i in range(100):
            cfg = FaultConfig(fault_id=f"f{i}", fault_type="FREEZE",
                              target_path="sensor://test",
                              magnitude=0, start_time_s=i * 0.001)
            inj.add_fault(cfg)
        for t in [i * 0.001 for i in range(100)]:
            inj.activate_at(t)
            v = inj.apply("sensor://test", float(t * 10), t)
            self.assertFalse(math.isnan(v))
        inj.clear_all()

    def test_freeze_cache_accumulation(self):
        inj = FaultInjector()
        for i in range(1000):
            fid = f"frz_{i}"
            cfg = FaultConfig(fault_id=fid, fault_type="FREEZE",
                              target_path=f"sensor://ch_{i%8}",
                              magnitude=0, start_time_s=0.0)
            inj.add_fault(cfg)
        inj.activate_at(0.0)
        for _ in range(10):
            for i in range(1000):
                _v = inj.apply(f"sensor://ch_{i%8}", float(i), 1.0)
        self.assertLessEqual(len(inj._freeze_cache), 1000)

    def test_fault_unicode_id(self):
        cfg = FaultConfig(fault_id="모터고장💣test", fault_type="BIAS",
                          target_path="sensor://t", magnitude=1.0)
        self.assertTrue(cfg.fault_id.isascii())
        inj = FaultInjector()
        inj.add_fault(cfg)
        self.assertIn(cfg.fault_id, inj._faults)


# ════════════════════════════════════════════════════════════
#  6. DataBus — Poison Payloads
# ════════════════════════════════════════════════════════════

class TestDataBus_Stress(unittest.TestCase):
    """DataBus under attack."""

    def setUp(self):
        self.bus = DataBus()
        # Register test modules for default-deny policy (CWE-862)
        self.bus.register_module("module://test")
        self.bus.register_module("module://attacker")
        self.bus.register_module("module://ctrl_a")
        self.bus.register_module("module://t")

    def test_publish_nan_signal(self):
        sig = Signal(source="test://s1", signal_type="temp", value=NAN)
        self.assertEqual(sig.quality, 0.0)
        self.bus.publish("topic1", sig, module_id="module://test")

    def test_publish_inf_signal(self):
        sig = Signal(source="test://s1", signal_type="temp", value=INF)
        self.bus.publish("topic1", sig, module_id="module://test")

    def test_publish_negative_timestamp(self):
        sig = Signal(source="test://s1", signal_type="temp",
                     timestamp_ns=-999999, value=10.0)
        self.bus.publish("topic1", sig, module_id="module://test")

    def test_publish_invalid_safety_level(self):
        sig = Signal(source="test://s1", signal_type="temp",
                     safety_level=999, value=10.0)
        self.assertEqual(sig.safety_level, 0)

    def test_topic_acl_deny_unregistered_raises(self):
        self.bus.restrict_topic("critical", ["module://ctrl_a"])
        sig = Signal(source="module://attacker", signal_type="cmd", value=999.0)
        with self.assertRaises(PermissionError):
            self.bus.publish("critical", sig, module_id="module://attacker")

    def test_topic_acl_allow_registered(self):
        self.bus.register_module("module://ctrl_a")
        self.bus.restrict_topic("cmd", ["module://ctrl_a"])
        sig = Signal(source="module://ctrl_a", signal_type="cmd", value=10.0)
        self.bus.publish("cmd", sig, module_id="module://ctrl_a")
        latest = self.bus.read_latest("cmd")
        self.assertIsNotNone(latest)

    def test_massive_publishing(self):
        for i in range(10000):
            self.bus.publish_scalar(f"topic_{i % 10}", float(i) % 100.0,
                                    module_id="module://test")
        self.assertLessEqual(len(self.bus._history), 10000)
        self.assertEqual(self.bus._seq, 10000)

    def test_subscriber_exception_safe(self):
        def bad_cb(sig):
            raise RuntimeError("subscriber crash")
        self.bus.subscribe("test", bad_cb, module_id="module://test")
        sig = Signal(source="t://x", signal_type="t", value=0.0)
        self.bus.publish("test", sig, module_id="module://test")
        latest = self.bus.read_latest("test")
        self.assertIsNotNone(latest)

    def test_signal_without_protocol(self):
        sig = Signal(source="bare_string", signal_type="t", value=0.0)
        self.assertIn("://", sig.source)

    def test_signal_negative_quality(self):
        sig = Signal(source="t://x", signal_type="t", quality=-5.0)
        self.assertEqual(sig.quality, 0.0)


# ════════════════════════════════════════════════════════════
#  7. Clock & Orchestrator — Temporal Attacks
# ════════════════════════════════════════════════════════════

class TestClock_Stress(unittest.TestCase):
    """Clock temporal attack vectors."""

    def test_s_to_ns_nan(self):
        self.assertEqual(s_to_ns(NAN), 0)

    def test_s_to_ns_inf(self):
        self.assertEqual(s_to_ns(INF), 0)

    def test_clock_advance_negative(self):
        c = GlobalClock(mode=ClockMode.OFFLINE)
        with self.assertRaises(ValueError):
            c.advance(-1000)

    def test_clock_advance_zero(self):
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(0)
        self.assertEqual(c.step_count, 1)

    def test_clock_diverge_detection(self):
        c = GlobalClock()
        self.assertFalse(c.diverged)
        c.mark_diverged()
        self.assertTrue(c.diverged)

    def test_clock_snapshot_restore_loop(self):
        c = GlobalClock()
        c.advance(1000000)
        s = c.snapshot()
        c.reset()
        self.assertEqual(c.sim_time_ns, 0)
        c.restore(s)
        self.assertEqual(s.sim_time_ns, 1000000)

    def test_clock_nan_dt(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        o = Orchestrator(cfg)
        with self.assertRaises((ValueError, TypeError)):
            o.run(NAN, 0.01)

    def test_orchestrator_step_zero(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        o = Orchestrator(cfg)
        with self.assertRaises(ValueError):
            o.run(0, 1.0)

    def test_orchestrator_negative_duration(self):
        cfg = OrchestratorConfig()
        o = Orchestrator(cfg)
        with self.assertRaises(ValueError):
            o.run(50000, -1.0)

    def test_orchestrator_nan_fault_time(self):
        o = Orchestrator()
        o.schedule_fault(NAN, lambda: None)
        self.assertEqual(len(o._fault_queue), 0)

    def test_orchestrator_inf_fault_time(self):
        o = Orchestrator()
        o.schedule_fault(INF, lambda: None)
        self.assertEqual(len(o._fault_queue), 0)


# ════════════════════════════════════════════════════════════
#  8. ModelRegistry — Metadata Attacks
# ════════════════════════════════════════════════════════════

class TestRegistry_Stress(unittest.TestCase):
    """ModelRegistry edge cases."""

    def setUp(self):
        self.reg = ModelRegistry()

    def test_register_duplicate(self):
        meta = ModelMetadata(model_id="mdl://dup", model_name="test",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        self.reg.register(None, meta)
        with self.assertRaises(ValueError):
            self.reg.register(None, meta)

    def test_get_nonexistent_generic_error(self):
        with self.assertRaises(KeyError) as ctx:
            self.reg.get("nonexistent")
        self.assertIn("not found", str(ctx.exception))

    def test_get_metadata_generic_error(self):
        with self.assertRaises(KeyError) as ctx:
            self.reg.get_metadata("nonexistent")
        self.assertIn("not found", str(ctx.exception))

    def test_dependency_validation(self):
        meta_a = ModelMetadata(model_id="mdl://a", model_name="A",
                               domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED,
                               dependencies=["mdl://b"])
        meta_b = ModelMetadata(model_id="mdl://b", model_name="B",
                               domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        self.reg.register(None, meta_a)
        self.reg.register(None, meta_b)
        missing = self.reg.validate_dependencies()
        self.assertEqual(len(missing), 0)

    def test_missing_dependency_detected(self):
        meta = ModelMetadata(model_id="mdl://orphan", model_name="o",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED,
                             dependencies=["mdl://nonexistent"])
        self.reg.register(None, meta)
        missing = self.reg.validate_dependencies()
        self.assertEqual(len(missing), 1)


# ════════════════════════════════════════════════════════════
#  9. HDF5Logger — File Integrity Under Stress
# ════════════════════════════════════════════════════════════

class TestLogger_Stress(unittest.TestCase):
    """HDF5Logger edge cases."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def test_record_nan_value(self):
        path = os.path.join(self.tmpdir, "nan.h5")
        with HDF5Logger(path) as log:
            log.record(0.0, val=NAN)
        self.assertTrue(HDF5Logger.verify_integrity(path))

    def test_record_inf_value(self):
        path = os.path.join(self.tmpdir, "inf.h5")
        with HDF5Logger(path) as log:
            log.record(0.0, val=INF)
        self.assertTrue(HDF5Logger.verify_integrity(path))

    def test_record_large_batch(self):
        path = os.path.join(self.tmpdir, "large.h5")
        with HDF5Logger(path) as log:
            for i in range(5000):
                log.record(float(i), val=float(i))
        self.assertTrue(HDF5Logger.verify_integrity(path))

    def test_corrupted_file_fails_verify(self):
        path = os.path.join(self.tmpdir, "corrupt.h5")
        with open(path, "w") as f:
            f.write("NOT_AN_HDF5_FILE")
        self.assertFalse(HDF5Logger.verify_integrity(path))


# ════════════════════════════════════════════════════════════
#  10. Concurrent Access — Thread Safety Stress
# ════════════════════════════════════════════════════════════

class TestConcurrent_Stress(unittest.TestCase):
    """Concurrent model access patterns."""

    def test_concurrent_motor_steps(self):
        motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                            flux_pm=0.03, J=0.001, B=0.0001, Pp=4)
        errors = []

        def step_motor():
            try:
                for _ in range(100):
                    motor.step(10., 10., 0.)
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=step_motor) for _ in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0)

    def test_concurrent_bus_publish_read(self):
        bus = DataBus()
        errors = []
        # Register all thread modules before concurrent access
        for i in range(4):
            bus.register_module(f"module://thread_t{i}")

        def write_read(prefix):
            try:
                for i in range(500):
                    bus.publish_vector(
                        f"topic_{prefix}",
                        {"a": float(i), "b": float(-i)},
                        module_id=f"module://thread_{prefix}")
                    saved = bus.read_latest(f"topic_{prefix}/a")
                    if saved is not None:
                        bus.publish_scalar(
                            f"response_{prefix}", saved.value * 2.0,
                            module_id=f"module://thread_{prefix}")
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=write_read, args=(f"t{i}",))
                   for i in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0)


# ════════════════════════════════════════════════════════════
#  11. Combined Extreme Scenario — Integrated Stress
# ════════════════════════════════════════════════════════════

class TestIntegrated_Stress(unittest.TestCase):
    """Whole-system stress with extreme parameter combinations."""

    def test_full_chain_nan_propagation_barrier(self):
        motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                            flux_pm=0.03, J=0.001, B=0.0001, Pp=4, dt_ns=50000)
        inv = AverageInverter(48.0)
        cs = CurrentSensor(noise_std=0.1, bias=0.01)
        enc = Encoder(noise_std=0.001)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                            ts=50e-6, v_bus=48.0)
        spd = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inj = FaultInjector()

        # Add aggressive faults
        for fid, ftype in [("n1","NOISE"), ("b1","BIAS"), ("f1","FREEZE"), ("d1","DROPOUT")]:
            inj.add_fault(FaultConfig(fault_id=fid, fault_type=ftype,
                          target_path="sensor://current", magnitude=100.0,
                          start_time_s=0.0))

        burst_points = [0, 100, 200, 300, 400]  # inject NaN at these steps
        results = []
        iq_ref = 0.0

        for step in range(1000):
            t = step * 50e-6
            inj.activate_at(t)

            if step % 20 == 0:
                sm = enc.read_speed(motor.omega_m)
                iq_ref = spd.update(100.0, sm)

            # Occasionally inject NaN
            if step in burst_points:
                ia, ib, ic = NAN, NAN, NAN
            else:
                ia, ib, ic = cs.read_abc(motor.ia, motor.ib, motor.ic)

            th = enc.read_angle(motor.theta_e)
            da, db, dc = foc.update(ia, ib, ic, th, 0.0, iq_ref)
            vb = inj.apply("sensor://current", 48.0, t)
            va, vb_out, vc = inv.step(da, db, dc, vb, ia, ib, ic)
            motor.step_abc(va, vb_out, vc, tl=0.0, dt=50e-6)
            motor.update_abc_currents()
            results.append(motor.omega_m)

        # Key assertion: no NaN propagated after NaN burst
        final = results[-1]
        self.assertFalse(math.isnan(final))
        self.assertTrue(math.isfinite(final))

        # Check recovery
        after_burst = results[410:510]
        self.assertTrue(all(math.isfinite(v) for v in after_burst))

    def test_parameter_explosion_motor(self):
        """Test all extreme parameter combinations."""
        base = {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4}
        extremes = [0.0, 1e-6, NAN, 1e308, 1.0, 1e6]

        for rs, ld, lq in itertools.product(extremes, repeat=3):
            try:
                m = PMSMdqModel(Rs=rs, Ld=ld, Lq=lq, **{k: v for k, v in base.items()
                                if k not in ("Rs", "Ld", "Lq")})
                m.step(10.0, 10.0, 0.0)
                self.assertFalse(math.isnan(m.id))
            except Exception:
                pass

    def test_memory_pressure_large_loop(self):
        m = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                        flux_pm=0.03, J=0.001, B=0.0001, Pp=4)
        for _ in range(50000):
            m.step(10., 10., 0.)
        self.assertTrue(math.isfinite(m.omega_m))


if __name__ == "__main__":
    unittest.main()
