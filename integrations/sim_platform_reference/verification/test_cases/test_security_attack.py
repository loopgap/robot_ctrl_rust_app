"""Final security attack test suite — comprehensive, no-stone-unturned.

Covers:
  1. Regression from performance optimizations
  2. NaN/Inf propagation through all code paths
  3. Numerical stability under extreme conditions
  4. State corruption recovery
  5. Boundary conditions (zero, negative, overflow)
  6. Resource exhaustion guards
  7. Thread safety basics
  8. Cross-module integration security
"""

import math
import os
import sys
import threading
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

NAN = float("nan")
INF = float("inf")
NINF = float("-inf")
BIG = 1e308
TINY = 1e-308
DENORM = 5e-324


class TestNaNInfPropagation(unittest.TestCase):
    """Verify NaN/Inf cannot propagate through any code path."""

    def test_clarke_nan_ia(self):
        from sim_platform.models.controller.foc import clarke_transform
        a, b = clarke_transform(NAN, 1.0, 1.0)
        self.assertTrue(math.isfinite(a))
        self.assertTrue(math.isfinite(b))

    def test_clarke_nan_ib(self):
        from sim_platform.models.controller.foc import clarke_transform
        a, b = clarke_transform(1.0, NAN, 1.0)
        self.assertTrue(math.isfinite(a))
        self.assertTrue(math.isfinite(b))

    def test_clarke_nan_ic(self):
        from sim_platform.models.controller.foc import clarke_transform
        a, b = clarke_transform(1.0, 1.0, NAN)
        self.assertTrue(math.isfinite(a))
        self.assertTrue(math.isfinite(b))

    def test_clarke_inf(self):
        from sim_platform.models.controller.foc import clarke_transform
        a, b = clarke_transform(INF, 1.0, 1.0)
        self.assertTrue(math.isfinite(a))
        self.assertTrue(math.isfinite(b))

    def test_park_nan_alpha(self):
        from sim_platform.models.controller.foc import park_transform
        d, q = park_transform(NAN, 1.0, 0.0)
        self.assertTrue(math.isfinite(d))
        self.assertTrue(math.isfinite(q))

    def test_park_nan_theta(self):
        from sim_platform.models.controller.foc import park_transform
        d, q = park_transform(1.0, 1.0, NAN)
        self.assertTrue(math.isfinite(d))
        self.assertTrue(math.isfinite(q))

    def test_inverse_park_nan(self):
        from sim_platform.models.controller.foc import inverse_park
        a, b = inverse_park(NAN, 1.0, 0.0)
        self.assertTrue(math.isfinite(a))
        self.assertTrue(math.isfinite(b))

    def test_svpwm_nan_alpha(self):
        from sim_platform.models.controller.foc import svpwm
        da, db, dc = svpwm(NAN, 0.0, 48.0)
        self.assertEqual((da, db, dc), (0.5, 0.5, 0.5))

    def test_svpwm_inf_vbus(self):
        from sim_platform.models.controller.foc import svpwm
        da, db, dc = svpwm(1.0, 0.0, INF)
        self.assertTrue(all(math.isfinite(d) for d in (da, db, dc)))

    def test_svpwm_zero_vbus(self):
        from sim_platform.models.controller.foc import svpwm
        da, db, dc = svpwm(1.0, 0.0, 0.0)
        self.assertEqual((da, db, dc), (0.5, 0.5, 0.5))

    def test_svpwm_negative_vbus(self):
        from sim_platform.models.controller.foc import svpwm
        da, db, dc = svpwm(1.0, 0.0, -48.0)
        self.assertTrue(all(0.0 <= d <= 1.0 for d in (da, db, dc)))

    def test_svpwm_tiny_vbus(self):
        from sim_platform.models.controller.foc import svpwm
        da, db, dc = svpwm(1.0, 0.0, 1e-15)
        self.assertEqual((da, db, dc), (0.5, 0.5, 0.5))


class TestPIControllerSecurity(unittest.TestCase):
    """PI controller security under extreme conditions."""

    def test_pi_nan_setpoint(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=10.0, ts=1e-3)
        u = pi.update(NAN, 0.0)
        self.assertTrue(math.isfinite(u))

    def test_pi_inf_setpoint(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=10.0, ts=1e-3)
        u = pi.update(INF, 0.0)
        self.assertTrue(math.isfinite(u))

    def test_pi_nan_measurement(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=10.0, ts=1e-3)
        u = pi.update(10.0, NAN)
        self.assertTrue(math.isfinite(u))

    def test_pi_integral_bounded(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=1e9, ts=1e-3, out_min=-100, out_max=100)
        for _ in range(100000):
            pi.update(1000.0, 0.0)
        self.assertTrue(abs(pi.integral) < 1e6)
        self.assertTrue(abs(pi.prev_output) <= 100.0)

    def test_pi_zero_kp(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=0.0, ki=10.0, ts=1e-3)
        u = pi.update(10.0, 0.0)
        self.assertTrue(math.isfinite(u))

    def test_pi_nan_kp(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=NAN, ki=10.0, ts=1e-3)
        self.assertTrue(math.isfinite(pi.kp))
        u = pi.update(10.0, 0.0)
        self.assertTrue(math.isfinite(u))

    def test_pi_inf_kp(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=INF, ki=INF, ts=1e-3)
        u = pi.update(10.0, 0.0)
        self.assertTrue(math.isfinite(u))

    def test_pi_swapped_limits(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=10.0, ts=1e-3, out_min=100, out_max=-100)
        u = pi.update(10.0, 0.0)
        self.assertTrue(math.isfinite(u))

    def test_pi_negative_ts(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=10.0, ts=-1e-3)
        self.assertGreater(pi.ts, 0)

    def test_pi_zero_ts(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=10.0, ts=0.0)
        self.assertGreater(pi.ts, 0)


class TestFOCSecurity(unittest.TestCase):
    """FOC controller security under attack conditions."""

    def test_foc_all_nan_inputs(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        da, db, dc = foc.update(NAN, NAN, NAN, NAN, NAN, NAN)
        self.assertTrue(all(math.isfinite(d) for d in (da, db, dc)))

    def test_foc_all_inf_inputs(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        da, db, dc = foc.update(INF, INF, INF, INF, INF, INF)
        self.assertTrue(all(math.isfinite(d) for d in (da, db, dc)))

    def test_foc_mixed_nan_inf(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        da, db, dc = foc.update(NAN, INF, 0.0, NAN, INF, 0.0)
        self.assertTrue(all(math.isfinite(d) for d in (da, db, dc)))

    def test_foc_large_currents(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        da, db, dc = foc.update(1e6, -1e6, 0.0, 0.0, 100.0, 200.0)
        self.assertTrue(all(0.0 <= d <= 1.0 for d in (da, db, dc)))

    def test_foc_nan_init_params(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=NAN, ki_id=NAN, kp_iq=NAN, ki_iq=NAN, ts=NAN, v_bus=NAN)
        self.assertTrue(math.isfinite(foc.v_bus))
        da, db, dc = foc.update(1.0, 1.0, 1.0, 0.0, 0.0, 0.0)
        self.assertTrue(all(math.isfinite(d) for d in (da, db, dc)))

    def test_foc_reset_clears_state(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        foc.update(10.0, 10.0, 10.0, 1.0, 50.0, 100.0)
        foc.reset()
        self.assertEqual(foc.vd_ref, 0.0)
        self.assertEqual(foc.vq_ref, 0.0)
        self.assertEqual(foc.duty_a, 0.5)


class TestPMSMSecurity(unittest.TestCase):
    """PMSM model security under attack conditions."""

    def test_pmsm_step_nan_vd(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(NAN, 0.0, 0.0)
        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))

    def test_pmsm_step_inf_vd(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(INF, 0.0, 0.0)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_step_nan_state_recovery(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.id = NAN
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_step_inf_state_recovery(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.omega_m = INF
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_pmsm_torque_nan_state(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.id = NAN
        m.iq = NAN
        t = m.torque_em
        self.assertTrue(math.isfinite(t))

    def test_pmsm_step_abc_nan(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step_abc(NAN, 0.0, 0.0)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_step_abc_inf(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step_abc(INF, INF, INF)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_step_nan_dt(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(10.0, 10.0, 0.0, dt=NAN)
        # Should not crash
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_step_negative_dt(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(10.0, 10.0, 0.0, dt=-1e-3)
        # Should not update state
        self.assertEqual(m.id, 0.0)

    def test_pmsm_step_zero_dt(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(10.0, 10.0, 0.0, dt=0.0)
        self.assertEqual(m.id, 0.0)

    def test_pmsm_nan_init_params(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=NAN, Ld=NAN, Lq=NAN, flux_pm=NAN, J=NAN, B=NAN)
        self.assertTrue(math.isfinite(m.Rs))
        self.assertTrue(math.isfinite(m.Ld))
        self.assertTrue(math.isfinite(m.J))
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_zero_inductance(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=0.0, Lq=0.0, flux_pm=0.03, J=1e-3, B=0.0)
        # Ld/Lq should be guarded to minimum
        self.assertGreater(m.Ld, 0)
        self.assertGreater(m.Lq, 0)
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_zero_inertia(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=0.0, B=0.0)
        self.assertGreater(m.J, 0)
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_pmsm_10k_steps_stability(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(10000):
            m.step(10.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.id), "NaN at step")
            self.assertTrue(math.isfinite(m.iq), "NaN at step")
            self.assertTrue(math.isfinite(m.omega_m), "NaN at step")


class TestSensorSecurity(unittest.TestCase):
    """Sensor models security under attack conditions."""

    def test_current_sensor_nan(self):
        from sim_platform.models.sensor.sensors import CurrentSensor
        s = CurrentSensor()
        v = s.read(NAN)
        self.assertTrue(math.isfinite(v))

    def test_current_sensor_inf(self):
        from sim_platform.models.sensor.sensors import CurrentSensor
        s = CurrentSensor()
        v = s.read(INF)
        self.assertTrue(math.isfinite(v))

    def test_current_sensor_abc_nan(self):
        from sim_platform.models.sensor.sensors import CurrentSensor
        s = CurrentSensor()
        a, b, c = s.read_abc(NAN, NAN, NAN)
        self.assertTrue(all(math.isfinite(v) for v in (a, b, c)))

    def test_encoder_nan(self):
        from sim_platform.models.sensor.sensors import Encoder
        e = Encoder()
        v = e.read_angle(NAN)
        self.assertTrue(math.isfinite(v))

    def test_encoder_inf(self):
        from sim_platform.models.sensor.sensors import Encoder
        e = Encoder()
        v = e.read_angle(INF)
        self.assertTrue(math.isfinite(v))

    def test_encoder_speed_nan(self):
        from sim_platform.models.sensor.sensors import Encoder
        e = Encoder()
        v = e.read_speed(NAN)
        self.assertTrue(math.isfinite(v))


class TestPowerModelSecurity(unittest.TestCase):
    """Power model security under attack conditions."""

    def test_battery_nan_init(self):
        from sim_platform.models.power.power_models import RintBattery
        b = RintBattery(v_oc=NAN, r_int=NAN)
        self.assertTrue(math.isfinite(b.v_oc))
        self.assertTrue(math.isfinite(b.r_int))

    def test_battery_nan_load(self):
        from sim_platform.models.power.power_models import RintBattery
        b = RintBattery()
        v = b.step(NAN)
        self.assertTrue(math.isfinite(v))

    def test_battery_inf_load(self):
        from sim_platform.models.power.power_models import RintBattery
        b = RintBattery()
        v = b.step(INF)
        self.assertTrue(math.isfinite(v))

    def test_inverter_nan_duty(self):
        from sim_platform.models.power.power_models import AverageInverter
        inv = AverageInverter()
        va, vb, vc = inv.step(NAN, NAN, NAN)
        self.assertTrue(all(math.isfinite(v) for v in (va, vb, vc)))

    def test_inverter_inf_duty(self):
        from sim_platform.models.power.power_models import AverageInverter
        inv = AverageInverter()
        va, vb, vc = inv.step(INF, INF, INF)
        self.assertTrue(all(math.isfinite(v) for v in (va, vb, vc)))

    def test_inverter_nan_vbus(self):
        from sim_platform.models.power.power_models import AverageInverter
        inv = AverageInverter()
        va, vb, vc = inv.step(0.5, 0.5, 0.5, v_bus=NAN)
        self.assertTrue(all(math.isfinite(v) for v in (va, vb, vc)))

    def test_inverter_inf_vbus(self):
        from sim_platform.models.power.power_models import AverageInverter
        inv = AverageInverter()
        va, vb, vc = inv.step(0.5, 0.5, 0.5, v_bus=INF)
        self.assertTrue(all(math.isfinite(v) for v in (va, vb, vc)))

    def test_inverter_negative_duty(self):
        from sim_platform.models.power.power_models import AverageInverter
        inv = AverageInverter()
        va, vb, vc = inv.step(-1.0, -1.0, -1.0)
        self.assertTrue(all(math.isfinite(v) for v in (va, vb, vc)))

    def test_inverter_large_duty(self):
        from sim_platform.models.power.power_models import AverageInverter
        inv = AverageInverter()
        va, vb, vc = inv.step(100.0, 100.0, 100.0)
        self.assertTrue(all(math.isfinite(v) for v in (va, vb, vc)))


class TestNumericalStability(unittest.TestCase):
    """Numerical stability under extreme conditions."""

    def test_denorm_values(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(DENORM, DENORM, DENORM)
        self.assertTrue(math.isfinite(m.id))

    def test_tiny_values(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(TINY, TINY, TINY)
        self.assertTrue(math.isfinite(m.id))

    def test_big_values(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(BIG, BIG, BIG)
        self.assertTrue(math.isfinite(m.id))

    def test_negative_big_values(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.step(-BIG, -BIG, -BIG)
        self.assertTrue(math.isfinite(m.id))

    def test_alternating_sign(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for i in range(1000):
            v = BIG if i % 2 == 0 else -BIG
            m.step(v, v, 0.0)
            self.assertTrue(math.isfinite(m.id))

    def test_pi_alternating_setpoint(self):
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=1e6, ts=1e-3, out_min=-100, out_max=100)
        for i in range(10000):
            sp = 1000.0 if i % 2 == 0 else -1000.0
            u = pi.update(sp, 0.0)
            self.assertTrue(math.isfinite(u))
            self.assertTrue(abs(u) <= 100.0)


class TestStateCorruptionRecovery(unittest.TestCase):
    """Recovery from externally corrupted state."""

    def test_pmsm_corrupt_id_recovery(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.id = NAN
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.id))

    def test_pmsm_corrupt_iq_recovery(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.iq = INF
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.iq))

    def test_pmsm_corrupt_omega_recovery(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.omega_m = NINF
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.omega_m))

    def test_pmsm_corrupt_theta_recovery(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.theta_e = BIG
        m.step(10.0, 10.0, 0.0)
        self.assertTrue(math.isfinite(m.theta_e))

    def test_pmsm_reset_after_corruption(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.id = BIG
        m.iq = NINF
        m.omega_m = NAN
        m.step(10.0, 10.0, 0.0)
        m.reset()
        self.assertEqual(m.id, 0.0)
        self.assertEqual(m.iq, 0.0)
        self.assertEqual(m.omega_m, 0.0)


class TestCrossModuleIntegration(unittest.TestCase):
    """Cross-module integration security."""

    def test_foc_pmsm_loop_nan_input(self):
        from sim_platform.models.controller.foc import FOCController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter
        from sim_platform.models.sensor.sensors import CurrentSensor, Encoder

        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        _cs = CurrentSensor()
        _enc = Encoder()
        inv = AverageInverter()

        # Inject NaN at various points
        for _ in range(100):
            da, db, dc = foc.update(NAN, NAN, NAN, 0.0, 0.0, 100.0)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)
            self.assertTrue(math.isfinite(motor.id))

    def test_foc_pmsm_loop_inf_input(self):
        from sim_platform.models.controller.foc import FOCController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        inv = AverageInverter()

        for _ in range(100):
            da, db, dc = foc.update(INF, INF, INF, 0.0, 0.0, 100.0)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)
            self.assertTrue(math.isfinite(motor.id))

    def test_full_loop_stability(self):
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter, RintBattery
        from sim_platform.models.sensor.sensors import CurrentSensor, Encoder

        ts = 50e-6
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=ts)
        speed_ctrl = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0, dt_ns=int(ts*1e9))
        cs = CurrentSensor(noise_std=0.01)
        enc = Encoder()
        inv = AverageInverter()
        batt = RintBattery()

        for step in range(5000):
            speed_ref = 100.0
            iq_ref = speed_ctrl.update(speed_ref, motor.omega_m)

            ia_m, ib_m, ic_m = cs.read_abc(motor.ia, motor.ib, motor.ic)
            theta_m = enc.read_angle(motor.theta_e)

            da, db, dc = foc.update(ia_m, ib_m, ic_m, theta_m, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc, v_bus=batt.step())
            motor.step_abc(va, vb, vc)
            motor.update_abc_currents()

            # All values must remain finite
            self.assertTrue(math.isfinite(motor.id), f"NaN id at step {step}")
            self.assertTrue(math.isfinite(motor.iq), f"NaN iq at step {step}")
            self.assertTrue(math.isfinite(motor.omega_m), f"NaN omega at step {step}")
            self.assertTrue(math.isfinite(motor.torque), f"NaN torque at step {step}")


class TestThreadSafety(unittest.TestCase):
    """Basic thread safety checks."""

    def test_concurrent_motor_step(self):
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        errors = []

        def worker():
            try:
                for _ in range(1000):
                    m.step(10.0, 10.0, 0.0)
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=worker) for _ in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0, f"Thread errors: {errors}")
        self.assertTrue(math.isfinite(m.id))

    def test_concurrent_foc_update(self):
        from sim_platform.models.controller.foc import FOCController
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        errors = []

        def worker():
            try:
                for _ in range(1000):
                    foc.update(1.0, 1.0, 1.0, 0.0, 0.0, 100.0)
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=worker) for _ in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0, f"Thread errors: {errors}")


class TestDataBusSecurity(unittest.TestCase):
    """DataBus security under attack conditions."""

    def test_publish_unregistered_module(self):
        from sim_platform.core.data_bus import DataBus, Signal
        bus = DataBus()
        sig = Signal(source="test://s1", signal_type="t", value=1.0)
        with self.assertRaises(PermissionError):
            bus.publish("topic", sig, module_id="module://evil")

    def test_subscribe_unregistered_module(self):
        from sim_platform.core.data_bus import DataBus
        bus = DataBus()
        with self.assertRaises(PermissionError):
            bus.subscribe("topic", lambda s: None, module_id="module://evil")

    def test_publish_no_module_id(self):
        from sim_platform.core.data_bus import DataBus, Signal
        bus = DataBus()
        sig = Signal(source="test://s1", signal_type="t", value=1.0)
        with self.assertRaises(PermissionError):
            bus.publish("topic", sig)

    def test_acl_default_deny(self):
        from sim_platform.core.data_bus import DataBus, Signal
        bus = DataBus()
        bus.register_module("module://good")
        bus.restrict_topic("secret", ["module://good"])
        sig = Signal(source="test://s1", signal_type="t", value=1.0)
        # Unregistered module should fail
        with self.assertRaises(PermissionError):
            bus.publish("secret", sig, module_id="module://evil")

    def test_snapshot_deep_copy(self):
        from sim_platform.core.data_bus import DataBus, Signal
        bus = DataBus()
        bus.register_module("module://test")
        sig = Signal(source="test://s1", signal_type="t", value=42.0)
        bus.publish("topic", sig, module_id="module://test")
        snap = bus.snapshot()
        # Modify snapshot should not affect internal state
        snap["latest"]["topic"].value = 999.0
        internal = bus.read_latest("topic")
        self.assertEqual(internal.value, 42.0)


class TestOrchestratorSecurity(unittest.TestCase):
    """Orchestrator security under attack conditions."""

    def test_run_zero_duration_rejected(self):
        from sim_platform.core.orchestrator import Orchestrator
        o = Orchestrator()
        # Zero duration should be rejected
        with self.assertRaises(ValueError):
            o.run(1000, 0.0)

    def test_run_very_small_duration(self):
        from sim_platform.core.orchestrator import Orchestrator
        o = Orchestrator()
        o.register_stepper("s1", lambda ns: None)
        # 1us duration, 1us step (1 step)
        o.run(1000, 1e-6)
        self.assertTrue(True)

    def test_run_total_steps_limit(self):
        from sim_platform.core.orchestrator import Orchestrator
        o = Orchestrator()
        # Should raise ValueError for excessive steps
        with self.assertRaises(ValueError):
            o.run(1e10, 1)  # 10^19 steps

    def test_schedule_fault_not_callable(self):
        from sim_platform.core.orchestrator import Orchestrator
        o = Orchestrator()
        with self.assertRaises(TypeError):
            o.schedule_fault(0.0, "not_callable")

    def test_schedule_fault_negative_time(self):
        from sim_platform.core.orchestrator import Orchestrator
        o = Orchestrator()
        o.schedule_fault(-1.0, lambda: None)
        # Should not be added to queue
        self.assertEqual(len(o._fault_queue), 0)

    def test_register_model_none(self):
        from sim_platform.core.model_registry import Domain, FidelityLevel, ModelMetadata
        from sim_platform.core.orchestrator import Orchestrator
        o = Orchestrator()
        meta = ModelMetadata(model_id="mdl://test", model_name="Test",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        with self.assertRaises(TypeError):
            o.register_model(None, meta)


if __name__ == "__main__":
    unittest.main()
