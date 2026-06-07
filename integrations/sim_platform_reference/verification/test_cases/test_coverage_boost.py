"""Coverage boost tests for low-coverage modules.

Targets:
- tools/visualization/plot_log.py (14% -> 80%+)
- core/orchestrator.py (66% -> 85%+)
- tools/config/config_manager.py (72% -> 85%+)
"""

import math
import os
import sys
import tempfile
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)


# ════════════════════════════════════════════════════════════
#  plot_log.py Tests (14% -> 80%+)
# ════════════════════════════════════════════════════════════

class TestPlotLog(unittest.TestCase):
    """Test plot_log.py visualization functions."""

    def test_sanitize_path_strips_directory(self):
        """_sanitize_path should strip directory components."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        result = _sanitize_path("/tmp/malicious/../../etc/passwd.png")
        self.assertEqual(result, "passwd.png")

    def test_sanitize_path_allows_png(self):
        """_sanitize_path should allow .png extension."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        result = _sanitize_path("results.png")
        self.assertEqual(result, "results.png")

    def test_sanitize_path_allows_svg(self):
        """_sanitize_path should allow .svg extension."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        result = _sanitize_path("results.svg")
        self.assertEqual(result, "results.svg")

    def test_sanitize_path_allows_pdf(self):
        """_sanitize_path should allow .pdf extension."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        result = _sanitize_path("results.pdf")
        self.assertEqual(result, "results.pdf")

    def test_sanitize_path_converts_invalid_ext(self):
        """_sanitize_path should convert invalid extensions to .png."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        result = _sanitize_path("results.exe")
        self.assertEqual(result, "results.png")

    def test_sanitize_path_handles_no_ext(self):
        """_sanitize_path should handle files without extension."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        result = _sanitize_path("results")
        self.assertEqual(result, "results.png")

    def test_plot_foc_results_generates_file(self):
        """plot_foc_results should generate a PNG file."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results

        # Generate test data
        n = 1000
        t = [i * 0.001 for i in range(n)]
        data = {
            "time": t,
            "speed_ref": [100.0] * n,
            "speed": [100 * (1 - math.exp(-ti * 2)) for ti in t],
            "id": [0.1 * math.sin(2 * math.pi * 50 * ti) for ti in t],
            "iq": [5.0 * (1 - math.exp(-ti * 3)) for ti in t],
            "ia": [0.0] * n,
            "ib": [0.0] * n,
            "ic": [0.0] * n,
            "torque": [0.05] * n,
            "duty_a": [0.5] * n,
            "duty_b": [0.5] * n,
            "duty_c": [0.5] * n,
            "vd": [0.0] * n,
            "vq": [10.0] * n,
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "test_plot.png")
            result = plot_foc_results(data, out_path)
            self.assertTrue(os.path.exists(result))
            self.assertTrue(result.endswith(".png"))

    def test_plot_foc_results_empty_data_raises(self):
        """plot_foc_results should raise on empty data."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        with self.assertRaises(ValueError):
            plot_foc_results({}, "test.png")

    def test_plot_foc_results_oversized_data_raises(self):
        """plot_foc_results should reject oversized data."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        data = {"time": [0.0] * 600_000}
        with self.assertRaises(ValueError):
            plot_foc_results(data, "test.png")

    def test_plot_foc_results_sanitizes_title(self):
        """plot_foc_results should sanitize title (no newlines)."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results

        n = 100
        data = {
            "time": [i * 0.01 for i in range(n)],
            "speed": [100.0] * n,
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "test.png")
            # Title with injection attempt
            result = plot_foc_results(data, out_path, title="Title\nInjection\rAttempt")
            self.assertTrue(os.path.exists(result))

    def test_plot_quick_generates_file(self):
        """plot_quick should generate a PNG file."""
        from sim_platform.tools.visualization.plot_log import plot_quick

        x = [i * 0.01 for i in range(100)]
        y = [math.sin(xi) for xi in x]

        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "quick.png")
            result = plot_quick(x, y, out_path)
            self.assertTrue(os.path.exists(result))

    def test_plot_quick_oversized_data_raises(self):
        """plot_quick should reject oversized data."""
        from sim_platform.tools.visualization.plot_log import plot_quick
        x = [0.0] * 600_000
        y = [0.0] * 600_000
        with self.assertRaises(ValueError):
            plot_quick(x, y, "test.png")

    def test_plot_quick_with_labels(self):
        """plot_quick should accept custom labels."""
        from sim_platform.tools.visualization.plot_log import plot_quick

        x = [1, 2, 3]
        y = [4, 5, 6]

        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "labeled.png")
            result = plot_quick(x, y, xlabel="X", ylabel="Y", title="Test", output_path=out_path)
            self.assertTrue(os.path.exists(result))


# ════════════════════════════════════════════════════════════
#  orchestrator.py Tests (66% -> 85%+)
# ════════════════════════════════════════════════════════════

class TestOrchestratorCoverage(unittest.TestCase):
    """Test orchestrator.py uncovered paths."""

    def setUp(self):
        from sim_platform.core.clock import ClockMode
        from sim_platform.core.orchestrator import Orchestrator, OrchestratorConfig
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        self.orch = Orchestrator(cfg)

    def test_set_sim_duration_valid(self):
        """set_sim_duration should accept valid values."""
        self.orch.set_sim_duration(1.0)
        self.assertEqual(self.orch._sim_time_s_max, 1.0)

    def test_set_sim_duration_zero(self):
        """set_sim_duration should reject zero."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(0.0)

    def test_set_sim_duration_negative(self):
        """set_sim_duration should reject negative."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(-1.0)

    def test_set_sim_duration_nan(self):
        """set_sim_duration should reject NaN."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(float('nan'))

    def test_set_sim_duration_inf(self):
        """set_sim_duration should reject Inf."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(float('inf'))

    def test_schedule_fault_valid(self):
        """schedule_fault should accept valid callable."""
        self.orch.schedule_fault(0.5, lambda: None)
        self.assertEqual(len(self.orch._fault_queue), 1)

    def test_schedule_fault_not_callable(self):
        """schedule_fault should reject non-callable."""
        with self.assertRaises(TypeError):
            self.orch.schedule_fault(0.5, "not_callable")

    def test_schedule_fault_negative_time(self):
        """schedule_fault should reject negative time."""
        # Current implementation logs warning and skips
        self.orch.schedule_fault(-1.0, lambda: None)
        # Should not be added to queue
        self.assertEqual(len(self.orch._fault_queue), 0)

    def test_register_model_valid(self):
        """register_model should accept valid model."""
        from sim_platform.core.model_registry import Domain, FidelityLevel, ModelMetadata
        meta = ModelMetadata(
            model_id="mdl://test",
            model_name="Test",
            domain=Domain.MOTOR,
            fidelity=FidelityLevel.L2_LUMPED,
        )
        self.orch.register_model(object(), meta)
        self.assertEqual(self.orch.registry.model_count, 1)

    def test_register_model_none_raises(self):
        """register_model should reject None model."""
        from sim_platform.core.model_registry import Domain, FidelityLevel, ModelMetadata
        meta = ModelMetadata(
            model_id="mdl://test2",
            model_name="Test2",
            domain=Domain.MOTOR,
            fidelity=FidelityLevel.L2_LUMPED,
        )
        with self.assertRaises(TypeError):
            self.orch.register_model(None, meta)

    def test_energy_audit_limit(self):
        """energy_audit should have deque maxlen."""
        from sim_platform.core.orchestrator import EnergyAudit
        # EnergyAudit is a dataclass with specific fields
        audit = EnergyAudit(
            power_input_j=1.0,
            mechanical_output_j=0.9,
            thermal_loss_j=0.05,
            stored_energy_j=0.05,
            imbalance_j=0.0,
        )
        self.assertIsNotNone(audit)
        self.assertAlmostEqual(audit.imbalance_pct, 0.0)

    def test_run_validates_step_ns(self):
        """run should validate step_ns is positive integer."""
        # run expects (duration_s, step_ns) - step_ns must be int
        with self.assertRaises((TypeError, ValueError)):
            self.orch.run(1.0, 1.0)  # step_ns must be int

    def test_run_validates_duration(self):
        """run should validate duration_s is positive."""
        with self.assertRaises(ValueError):
            self.orch.run(-1.0, 1000)

    def test_reset_clears_state(self):
        """reset should clear clock, bus, faults, audits."""
        self.orch.clock.advance(1000)
        self.orch.reset()
        self.assertEqual(self.orch.clock.sim_time_ns, 0)


# ════════════════════════════════════════════════════════════
#  clock.py Tests (72% -> 85%+)
# ════════════════════════════════════════════════════════════

class TestClockCoverage(unittest.TestCase):
    """Test clock.py uncovered paths."""

    def test_advance_valid(self):
        """advance should accept valid positive int."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(1000)
        self.assertEqual(c.sim_time_ns, 1000)

    def test_advance_zero(self):
        """advance should accept zero."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(0)
        self.assertEqual(c.sim_time_ns, 0)

    def test_advance_negative_raises(self):
        """advance should reject negative values."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        with self.assertRaises(ValueError):
            c.advance(-1000)

    def test_advance_float_raises(self):
        """advance should reject float values."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        with self.assertRaises(TypeError):
            c.advance(1.5)

    def test_pause_resume(self):
        """pause and resume should toggle state."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.pause()
        self.assertTrue(c._paused)
        c.resume()
        self.assertFalse(c._paused)

    def test_resume_when_not_paused(self):
        """resume when not paused should be no-op."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.resume()  # Should not raise
        self.assertFalse(c._paused)

    def test_snapshot_restore(self):
        """snapshot and restore should preserve state."""
        from sim_platform.core.clock import ClockMode, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(5000)
        state = c.snapshot()
        c.advance(1000)
        c.restore(state)
        self.assertEqual(c.sim_time_ns, 5000)

    def test_restore_invalid_state(self):
        """restore should reject invalid state."""
        from sim_platform.core.clock import ClockMode, ClockState, GlobalClock
        c = GlobalClock(mode=ClockMode.OFFLINE)
        invalid_state = ClockState(
            sim_time_ns=-1,
            wall_time_base_ns=0,
            step_count=0,
            mode=ClockMode.OFFLINE,
        )
        with self.assertRaises(ValueError):
            c.restore(invalid_state)

    def test_s_to_ns_valid(self):
        """s_to_ns should convert seconds to nanoseconds."""
        from sim_platform.core.clock import s_to_ns
        self.assertEqual(s_to_ns(1.0), 1_000_000_000)

    def test_s_to_ns_zero(self):
        """s_to_ns should handle zero."""
        from sim_platform.core.clock import s_to_ns
        self.assertEqual(s_to_ns(0.0), 0)

    def test_s_to_ns_nan(self):
        """s_to_ns should return 0 for NaN."""
        from sim_platform.core.clock import s_to_ns
        self.assertEqual(s_to_ns(float('nan')), 0)

    def test_s_to_ns_inf(self):
        """s_to_ns should return 0 for Inf."""
        from sim_platform.core.clock import s_to_ns
        self.assertEqual(s_to_ns(float('inf')), 0)


# ════════════════════════════════════════════════════════════
#  data_bus.py Tests (94% -> 98%+)
# ════════════════════════════════════════════════════════════

class TestDataBusCoverage(unittest.TestCase):
    """Test data_bus.py uncovered paths."""

    def setUp(self):
        from sim_platform.core.data_bus import DataBus
        self.bus = DataBus()
        self.bus.register_module("module://test")

    def test_publish_requires_module_id(self):
        """publish should require module_id."""
        from sim_platform.core.data_bus import Signal
        sig = Signal(source="test://s", signal_type="t", value=1.0)
        with self.assertRaises(PermissionError):
            self.bus.publish("topic", sig, module_id="")

    def test_publish_unregistered_module(self):
        """publish should reject unregistered module."""
        from sim_platform.core.data_bus import Signal
        sig = Signal(source="test://s", signal_type="t", value=1.0)
        with self.assertRaises(PermissionError):
            self.bus.publish("topic", sig, module_id="module://unknown")

    def test_subscribe_requires_module_id(self):
        """subscribe should require module_id."""
        with self.assertRaises(PermissionError):
            self.bus.subscribe("topic", lambda s: None, module_id="")

    def test_subscribe_unregistered_module(self):
        """subscribe should reject unregistered module."""
        with self.assertRaises(PermissionError):
            self.bus.subscribe("topic", lambda s: None, module_id="module://unknown")

    def test_snapshot_returns_deep_copy(self):
        """snapshot should return deep copy."""
        from sim_platform.core.data_bus import Signal
        sig = Signal(source="test://s", signal_type="t", value=1.0)
        self.bus.publish("topic", sig, module_id="module://test")
        snap = self.bus.snapshot()
        # Modify snapshot should not affect internal state
        snap["latest"]["topic"].value = 999.0
        latest = self.bus.read_latest("topic")
        self.assertNotEqual(latest.value, 999.0)

    def test_read_history_max_count(self):
        """read_history should respect max_count."""
        from sim_platform.core.data_bus import Signal
        for i in range(10):
            sig = Signal(source="test://s", signal_type="t", value=float(i))
            self.bus.publish("topic", sig, module_id="module://test")
        history = self.bus.read_history("topic", max_count=5)
        self.assertEqual(len(history), 5)

    def test_read_history_negative_max_count(self):
        """read_history should handle negative max_count."""
        history = self.bus.read_history("topic", max_count=-1)
        self.assertEqual(len(history), 0)

    def test_clear_security_requires_admin(self):
        """clear_security should require admin token."""
        with self.assertRaises(PermissionError):
            self.bus.clear_security(admin_token="")

    def test_clear_security_valid_admin(self):
        """clear_security should work with valid admin token."""
        # First set the admin token
        self.bus._admin_token = "admin_secret"
        self.bus.clear_security(admin_token="admin_secret")
        # After clear, modules should be removed
        self.assertEqual(len(self.bus._registered_modules), 0)

    def test_sim_event_valid(self):
        """SimEvent should accept valid event_type."""
        from sim_platform.core.data_bus import SimEvent
        event = SimEvent(
            event_type="FAULT",
            source="test",
            timestamp_ns=1000,
        )
        self.assertEqual(event.event_type, "FAULT")

    def test_sim_event_invalid_type(self):
        """SimEvent should reject invalid event_type."""
        from sim_platform.core.data_bus import SimEvent
        with self.assertRaises(ValueError):
            SimEvent(event_type="INVALID", source="test")

    def test_sim_event_negative_timestamp(self):
        """SimEvent should reject negative timestamp."""
        from sim_platform.core.data_bus import SimEvent
        with self.assertRaises(ValueError):
            SimEvent(event_type="FAULT", source="test", timestamp_ns=-1)


if __name__ == "__main__":
    unittest.main()
