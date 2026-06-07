"""Tests for configuration management system.

Tests:
- Schema validation
- Configuration loading/saving
- Template creation
- Error handling
"""

import json
import os
import sys
import tempfile

import pytest
import yaml

# Add project root to path
_PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJECT_ROOT)

from sim_platform.tools.config import (
    ConfigTemplates,
    ConfigurationManager,
    FaultConfig,
    FidelityLevel,
    ModelConfig,
    OutputConfig,
    OutputFormat,
    SimulationConfig,
    TimeConfig,
    config_to_json,
    config_to_yaml,
    create_default_pmsm_config,
)

# ── Schema Validation Tests ──────────────────────────────────

class TestModelConfig:
    """Test ModelConfig validation."""

    def test_valid_model_config(self):
        """Test valid model configuration."""
        config = ModelConfig(
            model_id="test_motor",
            model_type="motor",
            fidelity=FidelityLevel.L2,
            parameters={"Rs": 0.1, "Ld": 5e-4},
        )
        assert config.model_id == "test_motor"
        assert config.model_type == "motor"
        assert config.fidelity == FidelityLevel.L2

    def test_invalid_model_id(self):
        """Test invalid model ID."""
        with pytest.raises(ValueError, match="model_id"):
            ModelConfig(
                model_id="invalid@id",
                model_type="motor",
            )

    def test_empty_model_id(self):
        """Test empty model ID."""
        with pytest.raises(ValueError, match="model_id"):
            ModelConfig(
                model_id="",
                model_type="motor",
            )

    def test_nan_parameter(self):
        """Test NaN parameter value."""
        with pytest.raises(ValueError, match="NaN"):
            ModelConfig(
                model_id="test",
                model_type="motor",
                parameters={"Rs": float('nan')},
            )

    def test_inf_parameter(self):
        """Test Inf parameter value."""
        with pytest.raises(ValueError, match="Inf"):
            ModelConfig(
                model_id="test",
                model_type="motor",
                parameters={"Rs": float('inf')},
            )


class TestTimeConfig:
    """Test TimeConfig validation."""

    def test_valid_time_config(self):
        """Test valid time configuration."""
        config = TimeConfig(
            duration_s=1.0,
            dt_ns=50000,
        )
        assert config.duration_s == 1.0
        assert config.dt_ns == 50000

    def test_negative_duration(self):
        """Test negative duration."""
        with pytest.raises(ValueError, match="duration_s"):
            TimeConfig(duration_s=-1.0)

    def test_zero_duration(self):
        """Test zero duration."""
        with pytest.raises(ValueError, match="duration_s"):
            TimeConfig(duration_s=0.0)

    def test_nan_duration(self):
        """Test NaN duration."""
        with pytest.raises(ValueError, match="duration_s"):
            TimeConfig(duration_s=float('nan'))

    def test_inf_duration(self):
        """Test Inf duration."""
        with pytest.raises(ValueError, match="duration_s"):
            TimeConfig(duration_s=float('inf'))

    def test_negative_dt(self):
        """Test negative time step."""
        with pytest.raises(ValueError, match="dt_ns"):
            TimeConfig(dt_ns=-1)

    def test_zero_dt(self):
        """Test zero time step."""
        with pytest.raises(ValueError, match="dt_ns"):
            TimeConfig(dt_ns=0)


class TestFaultConfig:
    """Test FaultConfig validation."""

    def test_valid_fault_config(self):
        """Test valid fault configuration."""
        config = FaultConfig(
            enabled=True,
            fault_id="bias_fault",
            fault_type="bias",
            target_path="sensor://current",
            magnitude=0.1,
        )
        assert config.enabled is True
        assert config.fault_id == "bias_fault"

    def test_invalid_fault_id(self):
        """Test invalid fault ID."""
        with pytest.raises(ValueError, match="fault_id"):
            FaultConfig(fault_id="invalid@id")

    def test_nan_magnitude(self):
        """Test NaN magnitude."""
        with pytest.raises(ValueError, match="magnitude"):
            FaultConfig(magnitude=float('nan'))

    def test_inf_magnitude(self):
        """Test Inf magnitude."""
        with pytest.raises(ValueError, match="magnitude"):
            FaultConfig(magnitude=float('inf'))


class TestOutputConfig:
    """Test OutputConfig validation."""

    def test_valid_output_config(self):
        """Test valid output configuration."""
        config = OutputConfig(
            filename="test_output",
            format=OutputFormat.HDF5,
        )
        assert config.filename == "test_output"
        assert config.format == OutputFormat.HDF5

    def test_path_traversal_filename(self):
        """Test path traversal in filename."""
        with pytest.raises(ValueError, match="path separators"):
            OutputConfig(filename="../etc/passwd")

    def test_slash_in_filename(self):
        """Test slash in filename."""
        with pytest.raises(ValueError, match="path separators"):
            OutputConfig(filename="path/to/file")


# ── SimulationConfig Tests ──────────────────────────────────

class TestSimulationConfig:
    """Test SimulationConfig validation."""

    def test_valid_config(self):
        """Test valid simulation configuration."""
        config = SimulationConfig(
            name="test",
            time=TimeConfig(duration_s=1.0, dt_ns=50000),
            models=[
                ModelConfig(
                    model_id="motor",
                    model_type="motor",
                    parameters={"Rs": 0.1},
                ),
            ],
        )
        assert config.name == "test"
        assert len(config.models) == 1

    def test_duplicate_model_ids(self):
        """Test duplicate model IDs."""
        with pytest.raises(ValueError, match="Duplicate"):
            SimulationConfig(
                models=[
                    ModelConfig(model_id="motor", model_type="motor"),
                    ModelConfig(model_id="motor", model_type="motor"),
                ],
            )

    def test_circular_dependency(self):
        """Test circular dependency detection."""
        with pytest.raises(ValueError, match="Circular"):
            SimulationConfig(
                models=[
                    ModelConfig(
                        model_id="a",
                        model_type="motor",
                        depends_on=["b"],
                    ),
                    ModelConfig(
                        model_id="b",
                        model_type="motor",
                        depends_on=["a"],
                    ),
                ],
            )

    def test_get_model_by_id(self):
        """Test getting model by ID."""
        config = SimulationConfig(
            models=[
                ModelConfig(model_id="motor", model_type="motor"),
                ModelConfig(model_id="sensor", model_type="sensor"),
            ],
        )
        motor = config.get_model_by_id("motor")
        assert motor is not None
        assert motor.model_id == "motor"

        missing = config.get_model_by_id("missing")
        assert missing is None

    def test_get_total_steps(self):
        """Test total steps calculation."""
        config = SimulationConfig(
            time=TimeConfig(duration_s=1.0, dt_ns=50000),
        )
        total_steps = config.get_total_steps()
        assert total_steps == 20000  # 1.0 / 50e-6 = 20000


# ── Configuration Manager Tests ──────────────────────────────

class TestConfigurationManager:
    """Test ConfigurationManager."""

    def test_create_config(self):
        """Test creating configuration from template."""
        manager = ConfigurationManager()
        config = manager.create_config("test", "pmsm_foc")

        assert config.name == "test"
        assert len(config.models) > 0

    def test_load_save_yaml(self):
        """Test loading and saving YAML configuration."""
        manager = ConfigurationManager()
        config = create_default_pmsm_config()

        with tempfile.NamedTemporaryFile(suffix='.yaml', delete=False) as f:
            temp_path = f.name

        try:
            # Save configuration
            manager.save_config(config, temp_path)

            # Load configuration
            loaded = manager.load_config(temp_path)

            assert loaded.name == config.name
            assert len(loaded.models) == len(config.models)
            assert loaded.time.duration_s == config.time.duration_s
        finally:
            os.unlink(temp_path)

    def test_load_save_json(self):
        """Test loading and saving JSON configuration."""
        manager = ConfigurationManager()
        config = create_default_pmsm_config()

        with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
            temp_path = f.name

        try:
            # Save configuration
            manager.save_config(config, temp_path)

            # Load configuration
            loaded = manager.load_config(temp_path)

            assert loaded.name == config.name
            assert len(loaded.models) == len(config.models)
        finally:
            os.unlink(temp_path)

    def test_validate_config(self):
        """Test configuration validation."""
        manager = ConfigurationManager()
        config = create_default_pmsm_config()

        errors = manager.validate_config(config)
        assert len(errors) == 0

    def test_merge_configs(self):
        """Test configuration merging."""
        manager = ConfigurationManager()
        base = create_default_pmsm_config()

        override = {
            'time': {'duration_s': 2.0},
            'scenario': {'speed_ref_value': 200.0},
        }

        merged = manager.merge_configs(base, override)

        assert merged.time.duration_s == 2.0
        assert merged.scenario.speed_ref_value == 200.0

    def test_invalid_yaml(self):
        """Test loading invalid YAML."""
        manager = ConfigurationManager()

        with tempfile.NamedTemporaryFile(suffix='.yaml', mode='w', delete=False) as f:
            f.write("invalid: yaml: content: [")
            temp_path = f.name

        try:
            with pytest.raises(ValueError, match="Invalid YAML"):
                manager.load_config(temp_path)
        finally:
            os.unlink(temp_path)

    def test_missing_file(self):
        """Test loading missing file."""
        manager = ConfigurationManager()

        with pytest.raises(FileNotFoundError):
            manager.load_config("nonexistent.yaml")

    def test_unsupported_format(self):
        """Test loading unsupported format."""
        manager = ConfigurationManager()

        with tempfile.NamedTemporaryFile(suffix='.xyz', delete=False) as f:
            temp_path = f.name

        try:
            with pytest.raises(ValueError, match="Unsupported"):
                manager.load_config(temp_path)
        finally:
            os.unlink(temp_path)


# ── Template Tests ──────────────────────────────────────────

class TestConfigTemplates:
    """Test configuration templates."""

    def test_pmsm_foc_template(self):
        """Test PMSM FOC template."""
        config = ConfigTemplates.pmsm_foc_template()

        assert config.name == "PMSM FOC MVP"
        assert len(config.models) > 0
        assert config.time.duration_s > 0

    def test_bldc_six_step_template(self):
        """Test BLDC six-step template."""
        config = ConfigTemplates.bldc_six_step_template()

        assert config.name == "BLDC Six-Step"
        assert len(config.models) > 0

    def test_im_vector_control_template(self):
        """Test IM vector control template."""
        config = ConfigTemplates.im_vector_control_template()

        assert config.name == "IM Vector Control"
        assert len(config.models) > 0

    def test_get_template_list(self):
        """Test getting template list."""
        templates = ConfigTemplates.get_template_list()

        assert "pmsm_foc" in templates
        assert "bldc_six_step" in templates
        assert "im_vector_control" in templates

    def test_get_template(self):
        """Test getting template by name."""
        config = ConfigTemplates.get_template("pmsm_foc")
        assert config.name == "PMSM FOC MVP"

    def test_invalid_template(self):
        """Test getting invalid template."""
        with pytest.raises(ValueError, match="Unknown template"):
            ConfigTemplates.get_template("invalid")


# ── Utility Function Tests ──────────────────────────────────

class TestUtilityFunctions:
    """Test utility functions."""

    def test_config_to_yaml(self):
        """Test converting config to YAML."""
        config = create_default_pmsm_config()
        yaml_str = config_to_yaml(config)

        # Parse back
        parsed = yaml.safe_load(yaml_str)
        assert parsed['name'] == config.name

    def test_config_to_json(self):
        """Test converting config to JSON."""
        config = create_default_pmsm_config()
        json_str = config_to_json(config)

        # Parse back
        parsed = json.loads(json_str)
        assert parsed['name'] == config.name

    def test_create_default_config(self):
        """Test creating default configuration."""
        from sim_platform.tools.config import create_default_config

        config = create_default_config()
        assert config.name == "PMSM FOC MVP"


# ── Integration Tests ──────────────────────────────────────

class TestConfigIntegration:
    """Test configuration integration."""

    def test_full_workflow(self):
        """Test full configuration workflow."""
        manager = ConfigurationManager()

        # Create from template
        config = manager.create_config("test_sim", "pmsm_foc")

        # Modify configuration
        config.time.duration_s = 2.0
        config.scenario.speed_ref_value = 150.0

        # Validate
        errors = manager.validate_config(config)
        assert len(errors) == 0

        # Save to file
        with tempfile.NamedTemporaryFile(suffix='.yaml', delete=False) as f:
            temp_path = f.name

        try:
            manager.save_config(config, temp_path)

            # Load from file
            loaded = manager.load_config(temp_path)

            # Verify
            assert loaded.name == "test_sim"
            assert loaded.time.duration_s == 2.0
            assert loaded.scenario.speed_ref_value == 150.0
        finally:
            os.unlink(temp_path)

    def test_config_history(self):
        """Test configuration history."""
        manager = ConfigurationManager()

        # Create and save configuration
        config = manager.create_config("test", "pmsm_foc")

        with tempfile.NamedTemporaryFile(suffix='.yaml', delete=False) as f:
            temp_path = f.name

        try:
            manager.save_config(config, temp_path)
            manager.load_config(temp_path)

            # Check history
            history = manager.get_config_history()
            assert len(history) >= 2  # create + save + load
        finally:
            os.unlink(temp_path)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
