"""Tests for solver presets module."""

import pytest
from sim_platform.tools.gui.solver_presets import (
    SolverParameters,
    SolverPreset,
    SolverPresetManager,
    SolverType,
    IntegrationMode,
    BUILTIN_PRESETS,
)


class TestSolverParameters:
    """Tests for SolverParameters model."""

    def test_default_values(self):
        params = SolverParameters()
        assert params.dt_current == 50e-6
        assert params.dt_speed == 1e-3
        assert params.solver_type == SolverType.FORWARD_EULER
        assert params.integration_mode == IntegrationMode.FIXED_STEP
        assert params.nan_guard_enabled is True
        assert params.clamp_outputs is True

    def test_to_dict(self):
        params = SolverParameters(dt_current=100e-6, dt_speed=2e-3)
        d = params.to_dict()
        assert d["dt_current"] == 100e-6
        assert d["dt_speed"] == 2e-3
        assert d["solver_type"] == "forward_euler"

    def test_from_dict(self):
        data = {"dt_current": 25e-6, "dt_speed": 500e-6, "solver_type": "rk4"}
        params = SolverParameters.from_dict(data)
        assert params.dt_current == 25e-6
        assert params.solver_type == SolverType.RK4

    def test_from_dict_defaults(self):
        params = SolverParameters.from_dict({})
        assert params.dt_current == 50e-6
        assert params.nan_guard_enabled is True

    def test_roundtrip(self):
        params = SolverParameters(
            dt_current=10e-6,
            solver_type=SolverType.RK45,
            integration_mode=IntegrationMode.ADAPTIVE,
            rel_tolerance=1e-8,
        )
        d = params.to_dict()
        restored = SolverParameters.from_dict(d)
        assert restored.dt_current == params.dt_current
        assert restored.solver_type == params.solver_type
        assert restored.rel_tolerance == params.rel_tolerance


class TestSolverPreset:
    """Tests for SolverPreset."""

    def test_creation(self):
        params = SolverParameters()
        preset = SolverPreset(
            name="Test",
            description="Test preset",
            version="1.0.0",
            parameters=params,
            frozen_fields=["dt_current"],
        )
        assert preset.name == "Test"
        assert "dt_current" in preset.frozen_fields

    def test_to_dict(self):
        params = SolverParameters(dt_current=10e-6)
        preset = SolverPreset(
            name="Test", description="Desc", version="2.0",
            parameters=params, frozen_fields=["dt_current"],
            metadata={"tag": "test"},
        )
        d = preset.to_dict()
        assert d["name"] == "Test"
        assert d["version"] == "2.0"
        assert d["parameters"]["dt_current"] == 10e-6
        assert d["frozen_fields"] == ["dt_current"]
        assert d["metadata"]["tag"] == "test"

    def test_from_dict(self):
        data = {
            "name": "Restored",
            "description": "Restored preset",
            "version": "1.0",
            "parameters": {"dt_current": 50e-6},
            "frozen_fields": [],
            "metadata": {},
        }
        preset = SolverPreset.from_dict(data)
        assert preset.name == "Restored"
        assert preset.parameters.dt_current == 50e-6


class TestSolverPresetManager:
    """Tests for SolverPresetManager."""

    def test_default_presets(self):
        mgr = SolverPresetManager()
        presets = mgr.list_presets()
        assert "standard" in presets
        assert "high_precision" in presets
        assert "adaptive" in presets
        assert "realtime_hil" in presets

    def test_apply_preset(self):
        mgr = SolverPresetManager()
        params = mgr.apply_preset("standard")
        assert params.solver_type == SolverType.FORWARD_EULER
        assert mgr.current_preset_name == "standard"

    def test_apply_invalid_preset(self):
        mgr = SolverPresetManager()
        with pytest.raises(KeyError):
            mgr.apply_preset("nonexistent")

    def test_frozen_parameters(self):
        mgr = SolverPresetManager()
        mgr.apply_preset("standard")
        frozen = mgr.get_frozen_fields()
        assert "nan_guard_enabled" in frozen

        # Should not be able to modify frozen parameter
        result = mgr.update_parameter("nan_guard_enabled", False)
        assert result is False
        # Value should not change
        assert mgr.current_parameters.nan_guard_enabled is True

    def test_unfrozen_parameter(self):
        mgr = SolverPresetManager()
        mgr.apply_preset("standard")
        result = mgr.update_parameter("dt_current", 100e-6)
        assert result is True
        assert mgr.current_parameters.dt_current == 100e-6

    def test_is_frozen(self):
        mgr = SolverPresetManager()
        mgr.apply_preset("standard")
        assert mgr.is_frozen("nan_guard_enabled") is True
        assert mgr.is_frozen("dt_current") is False

    def test_no_preset_applied(self):
        mgr = SolverPresetManager()
        assert mgr.current_preset_name is None
        assert mgr.is_frozen("any_param") is False
        assert mgr.get_frozen_fields() == []

    def test_invalid_parameter(self):
        mgr = SolverPresetManager()
        with pytest.raises(AttributeError):
            mgr.update_parameter("nonexistent_param", 0)

    def test_change_log(self):
        mgr = SolverPresetManager()
        mgr.apply_preset("standard")
        log = mgr.get_change_log()
        assert len(log) > 0
        assert log[0]["action"] == "apply_preset"
        assert log[0]["target"] == "standard"

        mgr.update_parameter("dt_current", 25e-6)
        log = mgr.get_change_log()
        assert log[-1]["action"] == "update_param"

    def test_config_hash(self):
        mgr = SolverPresetManager()
        mgr.apply_preset("standard")
        h1 = mgr.get_config_hash()
        assert len(h1) == 16
        assert h1.isalnum()

        # Hash should change after parameter update
        mgr.update_parameter("dt_current", 25e-6)
        h2 = mgr.get_config_hash()
        assert h1 != h2

    def test_add_custom_preset(self):
        mgr = SolverPresetManager()
        initial = len(mgr.list_presets())
        preset = SolverPreset(
            name="custom_test",
            description="Custom preset for testing",
            version="1.0",
            parameters=SolverParameters(dt_current=10e-6),
            frozen_fields=[],
        )
        mgr.add_preset(preset)
        assert len(mgr.list_presets()) == initial + 1

        # Should be retrievable
        retrieved = mgr.get_preset("custom_test")
        assert retrieved.parameters.dt_current == 10e-6

    def test_remove_custom_preset(self):
        mgr = SolverPresetManager()
        preset = SolverPreset(
            name="removable",
            description="To be removed",
            version="1.0",
            parameters=SolverParameters(),
        )
        mgr.add_preset(preset)
        assert mgr.remove_preset("removable") is True
        assert mgr.remove_preset("removable") is False

    def test_cannot_remove_builtin(self):
        mgr = SolverPresetManager()
        assert mgr.remove_preset("standard") is False

    def test_reset_to_defaults(self):
        mgr = SolverPresetManager()
        mgr.apply_preset("high_precision")
        mgr.reset_to_defaults()
        assert mgr.current_preset_name == "standard"
        assert mgr.current_parameters.solver_type == SolverType.FORWARD_EULER

    def test_get_preset(self):
        mgr = SolverPresetManager()
        preset = mgr.get_preset("standard")
        assert preset is not None
        assert preset.name == "Standard (Forward Euler)"

        assert mgr.get_preset("nonexistent") is None


class TestSolverType:
    """Test solver type enum."""

    def test_solver_types(self):
        assert SolverType.FORWARD_EULER.value == "forward_euler"
        assert SolverType.RK4.value == "rk4"
        assert SolverType.RK45.value == "rk45"
        assert SolverType.IMPLICIT_EULER.value == "implicit_euler"
        assert SolverType.TRAPEZOIDAL.value == "trapezoidal"


class TestIntegrationMode:
    """Test integration mode enum."""

    def test_integration_modes(self):
        assert IntegrationMode.FIXED_STEP.value == "fixed_step"
        assert IntegrationMode.ADAPTIVE.value == "adaptive"
        assert IntegrationMode.MULTI_RATE.value == "multi_rate"


class TestBuiltinPresets:
    """Test built-in preset definitions."""

    def test_all_presets_valid(self):
        for name, preset in BUILTIN_PRESETS.items():
            assert preset.name
            assert preset.version
            assert isinstance(preset.parameters, SolverParameters)
            assert isinstance(preset.frozen_fields, list)
            assert isinstance(preset.metadata, dict)

    def test_standard_preset(self):
        preset = BUILTIN_PRESETS["standard"]
        assert preset.parameters.dt_current == 50e-6
        assert preset.parameters.solver_type == SolverType.FORWARD_EULER

    def test_realtime_preset(self):
        preset = BUILTIN_PRESETS["realtime_hil"]
        assert preset.parameters.realtime_target is True
        assert preset.parameters.miss_deadline_policy == "abort"
