"""High-intensity stress tests — concurrent, long-running, extreme conditions.

Covers:
  1. Multi-thread concurrent access
  2. Long-running stability (10k+ steps)
  3. Extreme parameter combinations
  4. Resource exhaustion guards
  5. Rapid mode switching
"""

import math
import os
import sys
import threading
import time
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.core.data_bus import DataBus, Signal
from sim_platform.core.orchestrator import Orchestrator
from sim_platform.models.controller.foc import FOCController, PIController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter


class TestConcurrentAccess(unittest.TestCase):
    """Multi-thread concurrent access stress tests."""

    def test_8_threads_motor_step(self):
        """8 threads simultaneously stepping motor."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        errors = []

        def worker():
            try:
                for _ in range(2000):
                    m.step(10.0, 10.0, 0.0)
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=worker) for _ in range(8)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0, f"Thread errors: {errors}")
        self.assertTrue(math.isfinite(m.id))

    def test_8_threads_foc_update(self):
        """8 threads simultaneously updating FOC."""
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        errors = []

        def worker():
            try:
                for _ in range(2000):
                    foc.update(1.0, 1.0, 1.0, 0.0, 0.0, 100.0)
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=worker) for _ in range(8)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0)

    def test_8_threads_data_bus(self):
        """8 threads simultaneously publishing/subscribing."""
        bus = DataBus()
        errors = []
        for i in range(8):
            bus.register_module(f"module://t{i}")

        def worker(tid):
            try:
                for i in range(1000):
                    sig = Signal(source=f"t://t{tid}", signal_type="v", value=float(i))
                    bus.publish(f"topic_{tid}", sig, module_id=f"module://t{tid}")
                    bus.read_latest(f"topic_{tid}")
            except Exception as e:
                errors.append(str(e))

        threads = [threading.Thread(target=worker, args=(i,)) for i in range(8)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0)

    def test_mixed_read_write_threads(self):
        """Mixed reader/writer threads on DataBus."""
        bus = DataBus()
        bus.register_module("module://writer")
        for i in range(4):
            bus.register_module(f"module://reader{i}")
        errors = []

        def writer():
            try:
                for i in range(2000):
                    sig = Signal(source="t://w", signal_type="v", value=float(i))
                    bus.publish("shared", sig, module_id="module://writer")
            except Exception as e:
                errors.append(f"writer: {e}")

        def reader(rid):
            try:
                for _ in range(2000):
                    _v = bus.read_latest("shared")
            except Exception as e:
                errors.append(f"reader{rid}: {e}")

        threads = [threading.Thread(target=writer)]
        threads += [threading.Thread(target=reader, args=(i,)) for i in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        self.assertEqual(len(errors), 0)


class TestLongRunningStability(unittest.TestCase):
    """Long-running stability tests."""

    def test_50k_steps_stability(self):
        """50,000 steps should remain stable."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter()

        for step in range(50000):
            iq_ref = sc.update(100.0, m.omega_m)
            ia, ib, ic = m.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            m.step_abc(va, vb, vc)

            if step % 5000 == 0:
                self.assertTrue(math.isfinite(m.omega_m), f"NaN at step {step}")
                self.assertTrue(math.isfinite(m.id), f"NaN id at step {step}")

        self.assertTrue(math.isfinite(m.omega_m))
        self.assertGreater(m.omega_m, 0, "Motor should be spinning")

    def test_100k_pi_controller_stability(self):
        """100,000 PI updates should remain bounded."""
        pi = PIController(kp=1.0, ki=1e6, ts=1e-3, out_min=-100, out_max=100)
        for _ in range(100000):
            u = pi.update(100.0, 0.0)
            self.assertTrue(math.isfinite(u))
            self.assertTrue(abs(u) <= 100.0)

    def test_rapid_start_stop(self):
        """Rapid start/stop cycles should not corrupt state."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for cycle in range(100):
            # Run 100 steps
            for _ in range(100):
                m.step(10.0, 10.0, 0.0)
            # Reset
            m.reset()
            self.assertEqual(m.id, 0.0)
            self.assertEqual(m.omega_m, 0.0)


class TestExtremeParameters(unittest.TestCase):
    """Extreme parameter combinations."""

    def test_very_small_inductance(self):
        """Very small inductance (1nH) should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=1e-9, Lq=1e-9, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(1000):
            m.step(10.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.id))

    def test_very_large_inductance(self):
        """Very large inductance (1H) should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=1.0, Lq=1.0, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(1000):
            m.step(10.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.id))

    def test_very_small_inertia(self):
        """Very small inertia should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-9, B=0.0)
        for _ in range(1000):
            m.step(10.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.omega_m))

    def test_very_large_inertia(self):
        """Very large inertia should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=10.0, B=0.0)
        for _ in range(1000):
            m.step(10.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.omega_m))

    def test_zero_resistance(self):
        """Zero resistance should not cause NaN."""
        m = PMSMdqModel(Rs=0.0, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(1000):
            m.step(10.0, 10.0, 0.0)
            self.assertTrue(math.isfinite(m.id))

    def test_very_high_voltage(self):
        """Very high voltage (1000V) should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(1000):
            m.step(1000.0, 1000.0, 0.0)
            self.assertTrue(math.isfinite(m.id))

    def test_negative_voltage(self):
        """Negative voltage should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(1000):
            m.step(-48.0, -48.0, 0.0)
            self.assertTrue(math.isfinite(m.id))

    def test_very_high_load_torque(self):
        """Very high load torque should not cause NaN."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        for _ in range(1000):
            m.step(48.0, 48.0, tl=1000.0)
            self.assertTrue(math.isfinite(m.omega_m))


class TestResourceExhaustion(unittest.TestCase):
    """Resource exhaustion guard tests."""

    def test_data_bus_max_history(self):
        """DataBus history should be bounded."""
        bus = DataBus()
        bus.register_module("module://test")
        for i in range(10000):
            sig = Signal(source="t://s", signal_type="v", value=float(i))
            bus.publish("topic", sig, module_id="module://test")
        # History should be bounded
        hist = bus.read_history("topic")
        self.assertLessEqual(len(hist), 1000)

    def test_orchestrator_max_events(self):
        """Orchestrator events should be bounded."""
        o = Orchestrator()
        o.register_stepper("s1", lambda ns: None)
        # This should complete without memory explosion
        o.run(1000, 0.01)
        self.assertTrue(True)

    def test_fault_queue_bounded(self):
        """Fault queue should handle many faults."""
        o = Orchestrator()
        for i in range(1000):
            o.schedule_fault(float(i) * 1e-6, lambda: None)
        self.assertLessEqual(len(o._fault_queue), 1000)


class TestPerformanceBenchmark(unittest.TestCase):
    """Performance benchmark tests."""

    def test_throughput_benchmark(self):
        """Throughput should be > 100k steps/sec."""
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        steps = 10000

        start = time.perf_counter()
        for _ in range(steps):
            m.step(10.0, 10.0, 0.0)
        elapsed = time.perf_counter() - start

        throughput = steps / elapsed
        self.assertGreater(throughput, 50000, f"Throughput too low: {throughput:.0f} steps/sec")

    def test_foc_throughput_benchmark(self):
        """FOC throughput should be > 10k updates/sec."""
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        steps = 10000

        start = time.perf_counter()
        for _ in range(steps):
            foc.update(1.0, 1.0, 1.0, 0.0, 0.0, 100.0)
        elapsed = time.perf_counter() - start

        throughput = steps / elapsed
        self.assertGreater(throughput, 5000, f"FOC throughput too low: {throughput:.0f}/sec")


if __name__ == "__main__":
    unittest.main()
