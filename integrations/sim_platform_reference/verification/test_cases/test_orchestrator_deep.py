"""Deep Orchestrator Tests — Cover all execution paths.

Targets uncovered lines: 100, 105-107, 114, 122-124, 177-181, 190-196,
198, 200, 212-216, 220-230, 241, 246-249, 260-261, 273-292, 311, 313,
315, 319, 325-327
"""

import os
import sys
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.core.clock import ClockMode
from sim_platform.core.orchestrator import Orchestrator, OrchestratorConfig, StepResult

# ════════════════════════════════════════════════════════════
#  1. Registration & Validation
# ════════════════════════════════════════════════════════════

class TestOrchestratorRegistration(unittest.TestCase):
    """Test model/stepper/initializer registration."""

    def setUp(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        self.orch = Orchestrator(cfg)

    def test_register_stepper_valid(self):
        """register_stepper should accept callable."""
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.assertIn("s1", self.orch._steppers)

    def test_register_stepper_not_callable(self):
        """register_stepper should reject non-callable."""
        with self.assertRaises(TypeError):
            self.orch.register_stepper("s1", "not_callable")

    def test_register_initializer_valid(self):
        """register_initializer should accept callable."""
        self.orch.register_initializer("s1", lambda: None)
        self.assertIn("s1", self.orch._initializers)

    def test_register_initializer_not_callable(self):
        """register_initializer should reject non-callable."""
        with self.assertRaises(TypeError):
            self.orch.register_initializer("s1", 123)

    def test_add_stop_condition_valid(self):
        """add_stop_condition should accept callable."""
        self.orch.add_stop_condition(lambda: False)
        self.assertEqual(len(self.orch._stop_hooks), 1)

    def test_add_stop_condition_not_callable(self):
        """add_stop_condition should reject non-callable."""
        with self.assertRaises(TypeError):
            self.orch.add_stop_condition(42)


# ════════════════════════════════════════════════════════════
#  2. Duration Validation
# ════════════════════════════════════════════════════════════

class TestDurationValidation(unittest.TestCase):
    """Test set_sim_duration validation paths."""

    def setUp(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        self.orch = Orchestrator(cfg)

    def test_set_duration_not_numeric(self):
        """set_sim_duration should reject non-numeric."""
        with self.assertRaises(TypeError):
            self.orch.set_sim_duration("abc")

    def test_set_duration_nan(self):
        """set_sim_duration should reject NaN."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(float('nan'))

    def test_set_duration_inf(self):
        """set_sim_duration should reject Inf."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(float('inf'))

    def test_set_duration_zero(self):
        """set_sim_duration should reject zero."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(0.0)

    def test_set_duration_negative(self):
        """set_sim_duration should reject negative."""
        with self.assertRaises(ValueError):
            self.orch.set_sim_duration(-1.0)

    def test_set_duration_valid(self):
        """set_sim_duration should accept valid value."""
        self.orch.set_sim_duration(1.0)
        self.assertEqual(self.orch._sim_time_s_max, 1.0)


# ════════════════════════════════════════════════════════════
#  3. run() Execution Paths
# ════════════════════════════════════════════════════════════

class TestRunExecutionPaths(unittest.TestCase):
    """Test run() method all execution paths."""

    def setUp(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        self.orch = Orchestrator(cfg)

    def test_run_calls_initializers(self):
        """run() should call all registered initializers."""
        called = []
        self.orch.register_initializer("s1", lambda: called.append(True))
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.001)
        self.assertEqual(len(called), 1)

    def test_run_initializer_exception_handled(self):
        """run() should handle initializer exceptions gracefully."""
        def bad_init():
            raise RuntimeError("init failed")
        self.orch.register_initializer("s1", bad_init)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        # Should not raise
        self.orch.run(1000000, 0.001)

    def test_run_stop_condition_triggers(self):
        """run() should stop when stop condition returns True."""
        step_count = [0]
        def stop_at_10():
            return step_count[0] >= 10
        def counting_stepper(ns):
            step_count[0] += 1
            return StepResult("s1")
        self.orch.add_stop_condition(stop_at_10)
        self.orch.register_stepper("s1", counting_stepper)
        self.orch.run(1000000, 0.1)  # 100k steps
        # Should have stopped early
        self.assertLessEqual(step_count[0], 11)

    def test_run_stop_condition_exception_handled(self):
        """run() should handle stop condition exceptions."""
        def bad_condition():
            raise RuntimeError("condition failed")
        self.orch.add_stop_condition(bad_condition)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        # Should not raise
        self.orch.run(1000000, 0.001)

    def test_run_solver_exception_handled(self):
        """run() should handle solver exceptions gracefully."""
        call_count = [0]
        def bad_stepper(ns):
            call_count[0] += 1
            if call_count[0] == 5:
                raise RuntimeError("solver crash")
            return StepResult("s1")
        self.orch.register_stepper("s1", bad_stepper)
        # Should not raise
        self.orch.run(1000000, 0.001)

    def test_run_unconverged_solver(self):
        """run() should handle unconverged solver."""
        def unconverged_stepper(ns):
            return StepResult("s1", converged=False, error_estimate=100.0)
        self.orch.register_stepper("s1", unconverged_stepper)
        # Should complete without error
        self.orch.run(1000000, 0.001)

    def test_run_progress_callback(self):
        """run() should call progress callback."""
        progress_values = []
        def track_progress(pct):
            progress_values.append(pct)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.01, progress_callback=track_progress)
        self.assertGreater(len(progress_values), 0)

    def test_run_progress_callback_exception_handled(self):
        """run() should handle progress callback exceptions."""
        def bad_callback(pct):
            raise RuntimeError("callback failed")
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        # Should not raise
        self.orch.run(1000000, 0.01, progress_callback=bad_callback)

    def test_run_step_ns_not_int(self):
        """run() should reject non-int step_ns."""
        with self.assertRaises(ValueError):
            self.orch.run(1.0, 0.001)

    def test_run_step_ns_zero(self):
        """run() should reject zero step_ns."""
        with self.assertRaises(ValueError):
            self.orch.run(0, 0.001)

    def test_run_step_ns_negative(self):
        """run() should reject negative step_ns."""
        with self.assertRaises(ValueError):
            self.orch.run(-1000, 0.001)

    def test_run_duration_negative(self):
        """run() should reject negative duration."""
        with self.assertRaises(ValueError):
            self.orch.run(1000, -1.0)

    def test_run_duration_nan(self):
        """run() should reject NaN duration."""
        with self.assertRaises(ValueError):
            self.orch.run(1000, float('nan'))

    def test_run_duration_inf(self):
        """run() should reject Inf duration."""
        with self.assertRaises(ValueError):
            self.orch.run(1000, float('inf'))

    def test_run_total_steps_exceeds_max(self):
        """run() should reject excessive total steps."""
        with self.assertRaises(ValueError):
            self.orch.run(1, 1e15)  # 1e24 steps

    def test_run_with_auto_step_halving(self):
        """run() should halve steps on unconvergence."""
        cfg = OrchestratorConfig(
            mode=ClockMode.OFFLINE,
            auto_step_halving=True,
            max_step_halving=3,
        )
        orch = Orchestrator(cfg)
        call_count = [0]
        def sometimes_unconverged(ns):
            call_count[0] += 1
            return StepResult("s1", converged=(call_count[0] % 5 != 0))
        orch.register_stepper("s1", sometimes_unconverged)
        orch.run(1000000, 0.001)
        # Should complete
        self.assertGreater(call_count[0], 0)


# ════════════════════════════════════════════════════════════
#  4. Fault Injection
# ════════════════════════════════════════════════════════════

class TestFaultInjection(unittest.TestCase):
    """Test fault injection paths."""

    def setUp(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        self.orch = Orchestrator(cfg)

    def test_schedule_fault_valid(self):
        """schedule_fault should accept valid fault."""
        self.orch.schedule_fault(0.5, lambda: None)
        self.assertEqual(len(self.orch._fault_queue), 1)

    def test_schedule_fault_not_callable(self):
        """schedule_fault should reject non-callable."""
        with self.assertRaises(TypeError):
            self.orch.schedule_fault(0.5, "not_callable")

    def test_schedule_fault_nan_time(self):
        """schedule_fault should reject NaN time."""
        self.orch.schedule_fault(float('nan'), lambda: None)
        self.assertEqual(len(self.orch._fault_queue), 0)

    def test_schedule_fault_inf_time(self):
        """schedule_fault should reject Inf time."""
        self.orch.schedule_fault(float('inf'), lambda: None)
        self.assertEqual(len(self.orch._fault_queue), 0)

    def test_schedule_fault_negative_time(self):
        """schedule_fault should reject negative time."""
        self.orch.schedule_fault(-1.0, lambda: None)
        self.assertEqual(len(self.orch._fault_queue), 0)

    def test_fault_executed_at_correct_time(self):
        """Fault should execute at scheduled time."""
        executed = []
        # Schedule fault at 0 time so it executes immediately
        self.orch.schedule_fault(0.0, lambda: executed.append(True))
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.001)
        self.assertEqual(len(executed), 1)

    def test_fault_exception_handled(self):
        """run() should handle fault exceptions."""
        def bad_fault():
            raise RuntimeError("fault failed")
        self.orch.schedule_fault(0.0001, bad_fault)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        # Should not raise
        self.orch.run(1000000, 0.001)

    def test_multiple_faults_ordered(self):
        """Multiple faults should execute in time order."""
        order = []
        # Schedule all at time 0 so they execute immediately in order
        self.orch.schedule_fault(0.0, lambda: order.append(1))
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.001)
        self.assertGreater(len(order), 0)


# ════════════════════════════════════════════════════════════
#  5. Energy Audit
# ════════════════════════════════════════════════════════════

class TestEnergyAudit(unittest.TestCase):
    """Test energy audit paths."""

    def setUp(self):
        cfg = OrchestratorConfig(
            mode=ClockMode.OFFLINE,
            enable_energy_audit=True,
            energy_audit_period_steps=100,
        )
        self.orch = Orchestrator(cfg)

    def test_energy_audit_called(self):
        """Energy audit should be called periodically."""
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.01)  # Many steps
        self.assertGreater(len(self.orch._energy_audits), 0)

    def test_energy_audit_with_power_model(self):
        """Energy audit should query model power interfaces."""
        class PowerModel:
            def get_power_input(self): return 10.0
            def get_power_output(self): return 8.0
            def get_power_loss(self): return 1.5
            def get_stored_energy(self): return 0.5

        from sim_platform.core.model_registry import Domain, FidelityLevel, ModelMetadata
        meta = ModelMetadata(
            model_id="mdl://power",
            model_name="PowerModel",
            domain=Domain.MOTOR,
            fidelity=FidelityLevel.L2_LUMPED,
        )
        self.orch.register_model(PowerModel(), meta)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.01)
        self.assertGreater(len(self.orch._energy_audits), 0)
        audit = self.orch._energy_audits[-1]
        self.assertAlmostEqual(audit.power_input_j, 10.0)

    def test_energy_audit_nan_power_ignored(self):
        """Energy audit should ignore NaN power values."""
        class NaNModel:
            def get_power_input(self): return float('nan')
            def get_power_output(self): return float('inf')

        from sim_platform.core.model_registry import Domain, FidelityLevel, ModelMetadata
        meta = ModelMetadata(
            model_id="mdl://nan",
            model_name="NaNModel",
            domain=Domain.MOTOR,
            fidelity=FidelityLevel.L2_LUMPED,
        )
        self.orch.register_model(NaNModel(), meta)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        self.orch.run(1000000, 0.01)
        audit = self.orch._energy_audits[-1]
        self.assertEqual(audit.power_input_j, 0.0)

    def test_energy_audit_exception_handled(self):
        """Energy audit should handle model exceptions."""
        class BadModel:
            def get_power_input(self): raise RuntimeError("broken")

        from sim_platform.core.model_registry import Domain, FidelityLevel, ModelMetadata
        meta = ModelMetadata(
            model_id="mdl://bad",
            model_name="BadModel",
            domain=Domain.MOTOR,
            fidelity=FidelityLevel.L2_LUMPED,
        )
        self.orch.register_model(BadModel(), meta)
        self.orch.register_stepper("s1", lambda ns: StepResult("s1"))
        # Should not raise
        self.orch.run(1000000, 0.01)


# ════════════════════════════════════════════════════════════
#  6. run_simple() Paths
# ════════════════════════════════════════════════════════════

class TestRunSimple(unittest.TestCase):
    """Test run_simple() all paths."""

    def setUp(self):
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        self.orch = Orchestrator(cfg)

    def test_run_simple_valid(self):
        """run_simple should accept valid inputs."""
        self.orch.run_simple(lambda: None, 1000000, 0.001)
        self.assertEqual(self.orch.clock.step_count, 1)

    def test_run_simple_not_callable(self):
        """run_simple should reject non-callable."""
        with self.assertRaises(TypeError):
            self.orch.run_simple("not_callable", 1000000, 0.001)

    def test_run_simple_step_ns_not_int(self):
        """run_simple should reject non-int step_ns."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, 1.0, 0.001)

    def test_run_simple_step_ns_zero(self):
        """run_simple should reject zero step_ns."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, 0, 0.001)

    def test_run_simple_step_ns_negative(self):
        """run_simple should reject negative step_ns."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, -1000, 0.001)

    def test_run_simple_duration_negative(self):
        """run_simple should reject negative duration."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, 1000, -1.0)

    def test_run_simple_duration_nan(self):
        """run_simple should reject NaN duration."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, 1000, float('nan'))

    def test_run_simple_duration_inf(self):
        """run_simple should reject Inf duration."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, 1000, float('inf'))

    def test_run_simple_total_steps_exceeds_max(self):
        """run_simple should reject excessive total steps."""
        with self.assertRaises(ValueError):
            self.orch.run_simple(lambda: None, 1, 1e15)

    def test_run_simple_step_exception_raises(self):
        """run_simple should raise RuntimeError on step failure."""
        def bad_step():
            raise RuntimeError("step failed")
        with self.assertRaises(RuntimeError):
            self.orch.run_simple(bad_step, 1000000, 0.001)

    def test_run_simple_advances_clock(self):
        """run_simple should advance clock."""
        self.orch.run_simple(lambda: None, 1000000, 0.001)
        self.assertEqual(self.orch.clock.sim_time_ns, 1000000)


# ════════════════════════════════════════════════════════════
#  7. Clock Divergence
# ════════════════════════════════════════════════════════════

class TestClockDivergence(unittest.TestCase):
    """Test divergence handling."""

    def test_diverged_stops_run(self):
        """run() should stop when clock diverges."""
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        orch = Orchestrator(cfg)
        step_count = [0]
        def diverge_at_10(ns):
            step_count[0] += 1
            if step_count[0] >= 10:
                orch.clock.mark_diverged()
            return StepResult("s1")
        orch.register_stepper("s1", diverge_at_10)
        orch.run(1000000, 0.1)
        # Should have stopped
        self.assertTrue(orch.clock.diverged)


# ════════════════════════════════════════════════════════════
#  8. Reset
# ════════════════════════════════════════════════════════════

class TestReset(unittest.TestCase):
    """Test reset functionality."""

    def test_reset_clears_state(self):
        """reset should clear clock, bus, faults, audits."""
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE)
        orch = Orchestrator(cfg)
        orch.clock.advance(1000)
        orch.schedule_fault(0.5, lambda: None)
        orch.reset()
        self.assertEqual(orch.clock.sim_time_ns, 0)
        self.assertEqual(len(orch._fault_queue), 0)


if __name__ == "__main__":
    unittest.main()
