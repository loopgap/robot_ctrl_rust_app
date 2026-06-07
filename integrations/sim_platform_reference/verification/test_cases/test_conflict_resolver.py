"""Tests for conflict resolver and physics constraint validation."""

import pytest
from sim_platform.models.physics_constraints import (
    ConstraintViolation,
    PhysicsValidator,
)
from sim_platform.tools.gui.conflict_resolver import (
    ConflictDetector,
    ConflictImpact,
    ConflictResolutionEngine,
    ConflictResolution,
    ConflictRule,
    ConflictSeverity,
    ConflictDomain,
    ResolutionStrategy,
    generate_impact_heatmap,
)


class TestPhysicsValidator:
    """Existing PhysicsValidator tests."""

    def test_valid_config(self):
        validator = PhysicsValidator()
        config = {
            "motor_params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        violations = validator.validate(config)
        errors = [v for v in violations if v.severity == "error"]
        assert len(errors) == 0, f"Unexpected errors: {errors}"

    def test_error_on_negative_kp(self):
        validator = PhysicsValidator()
        config = {
            "motor_params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": -1.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        violations = validator.validate(config)
        errors = [v for v in violations if v.severity == "error"]
        assert len(errors) > 0

    def test_warning_on_high_bandwidth(self):
        validator = PhysicsValidator()
        config = {
            "motor_params": {"Rs": 0.01, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        violations = validator.validate(config)
        warnings = [v for v in violations if v.severity == "warning"]
        # Should have bandwidth warning
        assert any("奈奎斯特" in v.message for v in warnings)

    def test_summary_format(self):
        validator = PhysicsValidator()
        violations = [
            ConstraintViolation("test", 0, 1, "error msg", "error", "fix"),
        ]
        summary = validator.get_summary(violations)
        assert "错误" in summary
        assert "error msg" in summary


class TestConflictDetector:
    """Tests for enhanced conflict detector."""

    def test_detection(self):
        detector = ConflictDetector()
        config = {
            "motor_params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        results = detector.detect(config)
        assert isinstance(results, list)
        for v, impact in results:
            assert isinstance(v, ConstraintViolation)
            assert isinstance(impact, ConflictImpact)

    def test_detection_returns_impacts(self):
        detector = ConflictDetector()
        config = {
            "motor_params": {"Rs": 0, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        results = detector.detect(config)
        for v, impact in results:
            assert impact.domain in ConflictDomain
            assert isinstance(impact.result_accuracy_impact, float)
            assert 0.0 <= impact.result_accuracy_impact <= 1.0

    def test_impact_summary(self):
        detector = ConflictDetector()
        config = {
            "motor_params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        summary = detector.get_impact_summary(config)
        assert "total_conflicts" in summary
        assert "blocks" in summary or "blockers" in summary
        assert "domains_affected" in summary
        assert "can_proceed" in summary

    def test_detect_errors(self):
        detector = ConflictDetector()
        config = {
            "motor_params": {"Rs": 0, "Ld": 0, "Lq": 0, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 0},
            "foc": {"kp_id": -1, "ki_id": 500.0, "kp_iq": -1, "ki_iq": 500.0},
            "speed_pi": {"kp": -0.1, "ki": -0.5},
            "dt_c": 1.0, "dt_s": 0.01, "duration_s": 0.001,
            "speed_ref": 10000, "load_torque": 1000,
            "battery": {"voltage": 1.0},
        }
        errors = detector.detect_errors(config)
        assert len(errors) > 0


class TestConflictResolutionEngine:
    """Tests for conflict resolution engine."""

    def test_initialization(self):
        engine = ConflictResolutionEngine()
        rules = engine.get_rules()
        assert len(rules) > 0, "Should have default rules"

    def test_resolve_valid_config(self):
        engine = ConflictResolutionEngine()
        config = {
            "motor_params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        resolutions = engine.resolve(config)
        for r in resolutions:
            assert isinstance(r, ConflictResolution)
            assert r.strategy in ResolutionStrategy

    def test_rule_matching(self):
        rule = ConflictRule(
            rule_id="test.motor",
            name="Test motor rule",
            description="Test",
            parameter_pattern="Rs",
            default_strategy=ResolutionStrategy.AUTO_FIX,
        )
        violation = ConstraintViolation("Rs", 0, 1, "msg", "error", "fix")
        assert rule.matches(violation)

        violation2 = ConstraintViolation("Ld", 0, 1, "msg", "error", "fix")
        assert not rule.matches(violation2)

    def test_add_remove_rule(self):
        engine = ConflictResolutionEngine()
        initial_count = len(engine.get_rules())
        rule = ConflictRule(
            rule_id="test.custom",
            name="Custom rule",
            description="Test custom rule",
            parameter_pattern="test.*",
            default_strategy=ResolutionStrategy.IGNORE_THIS_RUN,
            priority=5,
        )
        engine.add_rule(rule)
        assert len(engine.get_rules()) == initial_count + 1

        engine.remove_rule("test.custom")
        assert len(engine.get_rules()) == initial_count

    def test_priority_ordering(self):
        engine = ConflictResolutionEngine()
        rule_low = ConflictRule(
            rule_id="test.low", name="Low", description="",
            parameter_pattern="test_low",
            default_strategy=ResolutionStrategy.IGNORE_THIS_RUN,
            priority=100,
        )
        rule_high = ConflictRule(
            rule_id="test.high", name="High", description="",
            parameter_pattern="test_high",
            default_strategy=ResolutionStrategy.AUTO_FIX,
            priority=1,
        )
        engine.add_rule(rule_low)
        engine.add_rule(rule_high)
        rules = engine.get_rules()
        # High priority (lower number) should come first
        assert rules[0].rule_id == "test.high"

    def test_audit_trail(self):
        engine = ConflictResolutionEngine()
        config = {
            "motor_params": {"Rs": 0.01, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        engine.resolve(config)
        trail = engine.get_audit_trail()
        assert len(trail) > 0

    def test_clear_audit(self):
        engine = ConflictResolutionEngine()
        engine.clear_audit()
        trail = engine.get_audit_trail()
        assert len(trail) == 0

    def test_apply_resolution(self):
        engine = ConflictResolutionEngine()
        config = {
            "motor_params": {"Rs": 0.01, "Ld": 0.5e-3, "Lq": 1.0e-3, "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "dt_c": 50e-6, "dt_s": 1e-3, "duration_s": 1.5,
            "speed_ref": 100.0, "load_torque": 0.0,
            "battery": {"voltage": 48.0},
        }
        violations = engine._detector._validator.validate(config)
        warnings = [v for v in violations if v.severity == "warning"]
        if warnings:
            resolution = ConflictResolution(
                violation=warnings[0],
                strategy=ResolutionStrategy.AUTO_FIX,
                auto_fix_value=0.5,
            )
            modified = engine.apply_resolution(config, resolution)
            assert isinstance(modified, dict)


class TestConflictSeverity:
    """Test severity enum values."""

    def test_severity_order(self):
        assert ConflictSeverity.BLOCKER.value == "blocker"
        assert ConflictSeverity.CRITICAL.value == "critical"
        assert ConflictSeverity.WARNING.value == "warning"
        assert ConflictSeverity.INFO.value == "info"


class TestConflictImpact:
    """Test impact model."""

    def test_impact_creation(self):
        impact = ConflictImpact(
            domain=ConflictDomain.MOTOR,
            affected_parameters=["Rs"],
            subsystem_effects=["Motor resistance"],
            severity=ConflictSeverity.WARNING,
            result_accuracy_impact=0.6,
            convergence_impact=0.3,
            stability_impact=0.5,
        )
        assert impact.domain == ConflictDomain.MOTOR
        assert "Rs" in impact.affected_parameters


class TestHeatmap:
    """Test impact heatmap generation."""

    def test_empty_heatmap(self):
        result = generate_impact_heatmap([], [])
        assert "No conflicts" in result

    def test_basic_heatmap(self):
        v = ConstraintViolation("Rs", 0.01, 1, "test", "warning", "fix")
        impact = ConflictImpact(
            domain=ConflictDomain.MOTOR,
            affected_parameters=["Rs"],
            subsystem_effects=["test"],
            severity=ConflictSeverity.WARNING,
        )
        heatmap = generate_impact_heatmap([v], [impact])
        assert "MOTOR" in heatmap.upper() or "motor" in heatmap.lower()


class TestResolutionStrategy:
    """Test resolution strategy enum."""

    def test_strategy_values(self):
        assert ResolutionStrategy.AUTO_FIX.value == "auto_fix"
        assert ResolutionStrategy.MANUAL_OVERRIDE.value == "manual"
        assert ResolutionStrategy.IGNORE_THIS_RUN.value == "ignore"
        assert ResolutionStrategy.IGNORE_ALWAYS.value == "ignore_always"
        assert ResolutionStrategy.ASK_EACH_TIME.value == "ask"
