"""Unit tests for core modules — clock, data_bus, orchestrator, model_registry, utils, constants.

Fills the critical test coverage gap identified in code quality review.
Tests follow pytest style (AAA pattern).
"""

import os
import sys

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

import pytest

from sim_platform.core import constants
from sim_platform.core.clock import ClockMode, ClockState, GlobalClock, ns_to_s, s_to_ns
from sim_platform.core.data_bus import DataBus, DataValidity, Signal, SimEvent
from sim_platform.core.model_registry import (
    Domain,
    FidelityLevel,
    ModelMetadata,
    ModelRegistry,
    Port,
)
from sim_platform.core.orchestrator import EnergyAudit, Orchestrator, OrchestratorConfig
from sim_platform.core.utils import guard_in_range, guard_numeric, guard_positive, safe_divide

NAN = float("nan")
INF = float("inf")


# ══════════════════════════════════════════════════════════════
#  core/utils.py
# ══════════════════════════════════════════════════════════════

class TestGuardNumeric:
    def test_normal_value(self):
        assert guard_numeric(3.14) == 3.14

    def test_nan_returns_fallback(self):
        assert guard_numeric(NAN) == 0.0

    def test_inf_returns_fallback(self):
        assert guard_numeric(INF) == 0.0

    def test_neg_inf_returns_fallback(self):
        assert guard_numeric(-INF) == 0.0

    def test_custom_fallback(self):
        assert guard_numeric(NAN, fallback=99.0) == 99.0

    def test_zero_is_valid(self):
        assert guard_numeric(0.0) == 0.0

    def test_negative_is_valid(self):
        assert guard_numeric(-5.0) == -5.0


class TestGuardPositive:
    def test_positive_value(self):
        assert guard_positive(5.0) == 5.0

    def test_negative_clamped(self):
        assert guard_positive(-3.0, min_val=0.0) == 0.0

    def test_nan_returns_fallback(self):
        assert guard_positive(NAN, fallback=1.0) == 1.0


class TestGuardInRange:
    def test_in_range(self):
        assert guard_in_range(5.0, 0.0, 10.0) == 5.0

    def test_below_range(self):
        assert guard_in_range(-1.0, 0.0, 10.0) == 0.0

    def test_above_range(self):
        assert guard_in_range(15.0, 0.0, 10.0) == 10.0

    def test_nan_returns_fallback(self):
        assert guard_in_range(NAN, 0.0, 10.0, fallback=5.0) == 5.0


class TestSafeDivide:
    def test_normal_division(self):
        assert safe_divide(10.0, 2.0) == 5.0

    def test_zero_denominator(self):
        assert safe_divide(10.0, 0.0) == 0.0

    def test_near_zero_denominator(self):
        assert safe_divide(10.0, 1e-15) == 0.0

    def test_nan_numerator(self):
        assert safe_divide(NAN, 2.0) == 0.0

    def test_inf_numerator(self):
        result = safe_divide(INF, 2.0)
        assert result == 0.0


# ══════════════════════════════════════════════════════════════
#  core/constants.py
# ══════════════════════════════════════════════════════════════

class TestConstants:
    def test_motor_eps_l_positive(self):
        assert constants.MOTOR_EPS_L > 0

    def test_motor_eps_j_positive(self):
        assert constants.MOTOR_EPS_J > 0

    def test_default_v_bus(self):
        assert constants.DEFAULT_V_BUS == 48.0

    def test_max_total_steps(self):
        assert constants.MAX_TOTAL_STEPS == 1_000_000_000

    def test_numeric_eps(self):
        assert constants.NUMERIC_EPS == 1e-12


# ══════════════════════════════════════════════════════════════
#  core/clock.py
# ══════════════════════════════════════════════════════════════

class TestGlobalClock:
    def test_initial_state(self):
        c = GlobalClock()
        assert c.sim_time_ns == 0
        assert c.step_count == 0
        assert c.mode == ClockMode.OFFLINE

    def test_advance(self):
        c = GlobalClock()
        c.advance(1000)
        assert c.sim_time_ns == 1000
        assert c.step_count == 1

    def test_advance_negative_raises(self):
        c = GlobalClock()
        with pytest.raises(ValueError):
            c.advance(-100)

    def test_advance_type_check(self):
        c = GlobalClock()
        with pytest.raises(TypeError):
            c.advance(1.5)

    def test_advance_while_paused(self):
        c = GlobalClock()
        c.pause()
        c.advance(1000)
        assert c.sim_time_ns == 0

    def test_sim_time_s(self):
        c = GlobalClock()
        c.advance(1_000_000_000)
        assert abs(c.sim_time_s - 1.0) < 1e-9

    def test_pause_resume(self):
        c = GlobalClock()
        c.pause()
        assert c.paused
        c.resume()
        assert not c.paused

    def test_resume_not_paused_noop(self):
        c = GlobalClock()
        c.advance(1000)
        c.resume()  # Should not raise
        assert c.sim_time_ns == 1000

    def test_diverged(self):
        c = GlobalClock()
        assert not c.diverged
        c.mark_diverged()
        assert c.diverged

    def test_snapshot_restore(self):
        c = GlobalClock()
        c.advance(5000)
        state = c.snapshot()
        assert state.sim_time_ns == 5000
        c.reset()
        assert c.sim_time_ns == 0
        c.restore(state)
        assert c.sim_time_ns == 5000

    def test_restore_validates(self):
        c = GlobalClock()
        bad_state = ClockState(sim_time_ns=-100)
        with pytest.raises(ValueError):
            c.restore(bad_state)

    def test_reset(self):
        c = GlobalClock()
        c.advance(1000)
        c.reset()
        assert c.sim_time_ns == 0
        assert c.step_count == 0


class TestSToNs:
    def test_normal(self):
        assert s_to_ns(1.0) == 1_000_000_000

    def test_nan(self):
        assert s_to_ns(NAN) == 0

    def test_inf(self):
        assert s_to_ns(INF) == 0

    def test_zero(self):
        assert s_to_ns(0.0) == 0


class TestNsToS:
    def test_normal(self):
        assert abs(ns_to_s(1_000_000_000) - 1.0) < 1e-9


# ══════════════════════════════════════════════════════════════
#  core/data_bus.py
# ══════════════════════════════════════════════════════════════

class TestSignal:
    def test_basic_creation(self):
        sig = Signal(source="test://s1", signal_type="current", value=5.0)
        assert sig.value == 5.0

    def test_nan_value_marked_invalid(self):
        sig = Signal(source="test://s1", signal_type="t", value=NAN)
        assert DataValidity.INVALID in sig.validity
        assert sig.quality == 0.0

    def test_inf_value_marked_invalid(self):
        sig = Signal(source="test://s1", signal_type="t", value=INF)
        assert DataValidity.INVALID in sig.validity

    def test_negative_timestamp(self):
        sig = Signal(source="test://s1", signal_type="t", timestamp_ns=-100, value=1.0)
        # Should log warning but not raise
        assert sig.timestamp_ns == -100

    def test_quality_clamped(self):
        sig = Signal(source="test://s1", signal_type="t", quality=5.0)
        assert sig.quality == 1.0
        sig2 = Signal(source="test://s1", signal_type="t", quality=-1.0)
        assert sig2.quality == 0.0

    def test_source_normalized(self):
        sig = Signal(source="bare", signal_type="t", value=0.0)
        assert "://" in sig.source

    def test_path_traversal_rejected(self):
        with pytest.raises(ValueError):
            Signal(source="../../etc/passwd", signal_type="t", value=0.0)


class TestSimEvent:
    def test_valid_event(self):
        e = SimEvent(event_type="FAULT", source="test://s1")
        assert e.event_type == "FAULT"

    def test_invalid_event_type(self):
        with pytest.raises(ValueError):
            SimEvent(event_type="INVALID_TYPE", source="test://s1")

    def test_negative_timestamp(self):
        with pytest.raises(ValueError):
            SimEvent(event_type="FAULT", source="test://s1", timestamp_ns=-1)


class TestDataBus:
    def setUp(self):
        self.bus = DataBus()
        self.bus.register_module("module://test")

    def test_register_module(self):
        bus = DataBus()
        bus.register_module("test")
        assert bus.registered_module_count == 1

    def test_publish_requires_module_id(self):
        bus = DataBus()
        bus.register_module("test")
        sig = Signal(source="test://s1", signal_type="t", value=1.0)
        with pytest.raises(PermissionError):
            bus.publish("topic", sig)

    def test_publish_unregistered_raises(self):
        bus = DataBus()
        sig = Signal(source="test://s1", signal_type="t", value=1.0)
        with pytest.raises(PermissionError):
            bus.publish("topic", sig, module_id="module://unknown")

    def test_publish_registered_works(self):
        bus = DataBus()
        bus.register_module("test")
        sig = Signal(source="test://s1", signal_type="t", value=1.0)
        bus.publish("topic", sig, module_id="module://test")
        assert bus.read_latest("topic") is not None

    def test_subscribe_requires_module_id(self):
        bus = DataBus()
        bus.register_module("test")
        with pytest.raises(PermissionError):
            bus.subscribe("topic", lambda s: None)

    def test_subscribe_unregistered_raises(self):
        bus = DataBus()
        with pytest.raises(PermissionError):
            bus.subscribe("topic", lambda s: None, module_id="module://unknown")

    def test_subscribe_registered_works(self):
        bus = DataBus()
        bus.register_module("test")
        bus.subscribe("topic", lambda s: None, module_id="module://test")

    def test_topic_acl_restrict(self):
        bus = DataBus()
        bus.register_module("a")
        bus.register_module("b")
        bus.restrict_topic("secret", ["module://a"])
        sig = Signal(source="a://s", signal_type="t", value=1.0)
        bus.publish("secret", sig, module_id="module://a")
        with pytest.raises(PermissionError):
            bus.publish("secret", sig, module_id="module://b")

    def test_clear_security_requires_token(self):
        bus = DataBus()
        bus.register_module("test")
        bus.set_admin_token("secret123")
        with pytest.raises(PermissionError):
            bus.clear_security()
        with pytest.raises(PermissionError):
            bus.clear_security(admin_token="wrong")
        bus.clear_security(admin_token="secret123")
        assert bus.registered_module_count == 0

    def test_snapshot_deep_copy(self):
        bus = DataBus()
        bus.register_module("test")
        sig = Signal(source="test://s1", signal_type="t", value=42.0)
        bus.publish("topic", sig, module_id="module://test")
        snap = bus.snapshot()
        snap["latest"]["topic"].value = 999.0
        assert bus.read_latest("topic").value == 42.0

    def test_read_history_max_count_validation(self):
        bus = DataBus()
        assert bus.read_history("nonexistent", max_count=-1) == []

    def test_event_limit(self):
        bus = DataBus()
        for i in range(60000):
            bus.publish_event(SimEvent(event_type="FAULT", source="test://s1"))
        assert len(bus._events) <= 50000


# ══════════════════════════════════════════════════════════════
#  core/model_registry.py
# ══════════════════════════════════════════════════════════════

class TestModelRegistry:
    def test_register_and_get(self):
        reg = ModelRegistry()
        meta = ModelMetadata(model_id="mdl://test/v1", model_name="Test",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        reg.register("model_obj", meta)
        assert reg.get("mdl://test/v1") == "model_obj"
        assert reg.model_count == 1

    def test_duplicate_raises(self):
        reg = ModelRegistry()
        meta = ModelMetadata(model_id="mdl://dup", model_name="Dup",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        reg.register("obj1", meta)
        with pytest.raises(ValueError):
            reg.register("obj2", meta)

    def test_get_not_found_generic_error(self):
        reg = ModelRegistry()
        with pytest.raises(KeyError, match="not found"):
            reg.get("nonexistent")

    def test_empty_model_id_raises(self):
        reg = ModelRegistry()
        meta = ModelMetadata(model_id="", model_name="Bad",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        with pytest.raises(ValueError):
            reg.register("obj", meta)

    def test_list_by_domain(self):
        reg = ModelRegistry()
        m1 = ModelMetadata(model_id="mdl://m1", model_name="M1",
                           domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        m2 = ModelMetadata(model_id="mdl://s1", model_name="S1",
                           domain=Domain.SENSOR, fidelity=FidelityLevel.L1_EMPIRICAL)
        reg.register("m1", m1)
        reg.register("s1", m2)
        motors = reg.list_by_domain(Domain.MOTOR)
        assert len(motors) == 1

    def test_validate_dependencies(self):
        reg = ModelRegistry()
        m1 = ModelMetadata(model_id="mdl://a", model_name="A",
                           domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED,
                           dependencies=["mdl://b"])
        m2 = ModelMetadata(model_id="mdl://b", model_name="B",
                           domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        reg.register("a", m1)
        reg.register("b", m2)
        assert len(reg.validate_dependencies()) == 0


class TestPort:
    def test_valid_port(self):
        p = Port(name="voltage", unit="V")
        assert p.name == "voltage"

    def test_empty_name_raises(self):
        with pytest.raises(ValueError):
            Port(name="", unit="V")

    def test_path_traversal_name_raises(self):
        with pytest.raises(ValueError):
            Port(name="../etc", unit="V")


# ══════════════════════════════════════════════════════════════
#  core/orchestrator.py
# ══════════════════════════════════════════════════════════════

class TestEnergyAudit:
    def test_imbalance_pct_normal(self):
        a = EnergyAudit(power_input_j=100.0, imbalance_j=5.0)
        assert abs(a.imbalance_pct - 5.0) < 0.1

    def test_imbalance_pct_nan_input(self):
        a = EnergyAudit(power_input_j=NAN, imbalance_j=5.0)
        assert a.imbalance_pct == 0.0


class TestOrchestrator:
    def test_basic_run(self):
        o = Orchestrator(OrchestratorConfig(mode=ClockMode.OFFLINE))
        step_count = [0]
        def step_fn():
            step_count[0] += 1
        o.run_simple(step_fn, step_ns=1000000, duration_s=0.001)
        assert step_count[0] > 0

    def test_run_zero_step_raises(self):
        o = Orchestrator()
        with pytest.raises(ValueError):
            o.run(0, 1.0)

    def test_run_negative_duration_raises(self):
        o = Orchestrator()
        with pytest.raises(ValueError):
            o.run(50000, -1.0)

    def test_register_none_model_raises(self):
        o = Orchestrator()
        meta = ModelMetadata(model_id="mdl://t", model_name="T",
                             domain=Domain.MOTOR, fidelity=FidelityLevel.L2_LUMPED)
        with pytest.raises(TypeError):
            o.register_model(None, meta)

    def test_schedule_fault_non_callable_raises(self):
        o = Orchestrator()
        with pytest.raises(TypeError):
            o.schedule_fault(1.0, "not_callable")

    def test_schedule_fault_nan_time_skipped(self):
        o = Orchestrator()
        o.schedule_fault(NAN, lambda: None)
        assert len(o._fault_queue) == 0

    def test_set_sim_duration_validation(self):
        o = Orchestrator()
        with pytest.raises(ValueError):
            o.set_sim_duration(-1.0)
        with pytest.raises(ValueError):
            o.set_sim_duration(NAN)
        o.set_sim_duration(1.0)

    def test_total_steps_cap(self):
        o = Orchestrator()
        with pytest.raises(ValueError, match="exceeds maximum"):
            o.run(1, 1e10)  # Would be 10^19 steps

    def test_reset(self):
        o = Orchestrator()
        o.set_sim_duration(1.0)
        o.reset()
        # reset clears clock/bus/faults/audits but preserves config
        assert o.clock.sim_time_ns == 0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
