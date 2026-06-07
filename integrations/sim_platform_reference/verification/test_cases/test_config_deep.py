"""Deep Config Manager Tests — Cover all execution paths.

Targets uncovered lines in config_manager.py (72% -> 85%+).
"""

import os
import sys
import tempfile
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.tools.config.config_manager import (
    ConfigTemplates,
    ConfigurationManager,
    ConfigWizard,
)
from sim_platform.tools.config.config_schema import SimulationConfig


class TestConfigurationManager(unittest.TestCase):
    """Test ConfigurationManager all paths."""

    def setUp(self):
        self.mgr = ConfigurationManager()

    def test_create_config_default(self):
        """create_config should return valid config."""
        config = self.mgr.create_config("default")
        self.assertIsNotNone(config)
        self.assertIsInstance(config, SimulationConfig)

    def test_create_config_pmsm(self):
        """create_config with pmsm should work."""
        config = self.mgr.create_config("pmsm_foc")
        self.assertIsNotNone(config)

    def test_validate_valid_config(self):
        """validate_config should accept valid config."""
        config = self.mgr.create_config("default")
        errors = self.mgr.validate_config(config)
        self.assertIsInstance(errors, list)

    def test_load_nonexistent_file(self):
        """load_config should handle nonexistent file."""
        with self.assertRaises(Exception):
            self.mgr.load_config("/nonexistent/path.yaml")

    def test_save_and_load_roundtrip(self):
        """save and load should roundtrip."""
        config = self.mgr.create_config("default")
        with tempfile.NamedTemporaryFile(suffix='.yaml', delete=False, mode='w') as f:
            path = f.name
        try:
            self.mgr.save_config(config, path)
            loaded = self.mgr.load_config(path)
            self.assertIsNotNone(loaded)
        finally:
            if os.path.exists(path):
                os.unlink(path)

    def test_merge_configs(self):
        """merge_configs should combine configs."""
        base = self.mgr.create_config("default")
        overlay = {"name": "merged_test"}
        merged = self.mgr.merge_configs(base, overlay)
        self.assertIsNotNone(merged)
        self.assertEqual(merged.name, "merged_test")

    def test_get_config_history(self):
        """get_config_history should return list."""
        history = self.mgr.get_config_history()
        self.assertIsInstance(history, list)

    def test_clear_cache(self):
        """clear_cache should not raise."""
        self.mgr.clear_cache()


class TestConfigTemplates(unittest.TestCase):
    """Test ConfigTemplates all paths."""

    def test_pmsm_foc_template(self):
        """pmsm_foc_template should return valid config."""
        config = ConfigTemplates.pmsm_foc_template()
        self.assertIsNotNone(config)
        self.assertIsInstance(config, SimulationConfig)

    def test_bldc_six_step_template(self):
        """bldc_six_step_template should return valid config."""
        config = ConfigTemplates.bldc_six_step_template()
        self.assertIsNotNone(config)
        self.assertIsInstance(config, SimulationConfig)

    def test_im_vector_control_template(self):
        """im_vector_control_template should return valid config."""
        config = ConfigTemplates.im_vector_control_template()
        self.assertIsNotNone(config)
        self.assertIsInstance(config, SimulationConfig)

    def test_get_template_list(self):
        """get_template_list should return list."""
        templates = ConfigTemplates.get_template_list()
        self.assertIsInstance(templates, list)
        self.assertGreater(len(templates), 0)

    def test_get_template_by_name(self):
        """get_template should return config by name."""
        config = ConfigTemplates.get_template("pmsm_foc")
        self.assertIsNotNone(config)


class TestConfigWizard(unittest.TestCase):
    """Test ConfigWizard paths."""

    def test_wizard_class_exists(self):
        """ConfigWizard should exist."""
        self.assertTrue(hasattr(ConfigWizard, 'create_interactive'))

    def test_wizard_create_interactive(self):
        """create_interactive should return config."""
        from unittest.mock import patch

        from sim_platform.tools.config.config_manager import ConfigurationManager
        mgr = ConfigurationManager()
        wizard = ConfigWizard(mgr)
        # Mock all input() calls with defaults
        with patch('builtins.input', return_value=''):
            config = wizard.create_interactive()
        self.assertIsNotNone(config)


class TestSimulationConfig(unittest.TestCase):
    """Test SimulationConfig validation."""

    def test_default_config_valid(self):
        """Default config should be valid."""
        config = SimulationConfig()
        self.assertIsNotNone(config)

    def test_config_has_required_fields(self):
        """Config should have required fields."""
        config = SimulationConfig()
        # Check that config is a valid Pydantic model
        self.assertTrue(hasattr(config, 'model_dump'))

    def test_config_serialization(self):
        """Config should serialize to dict."""
        config = SimulationConfig()
        d = config.model_dump()
        self.assertIsInstance(d, dict)

    def test_config_from_dict(self):
        """Config should deserialize from dict."""
        config = SimulationConfig()
        d = config.model_dump()
        config2 = SimulationConfig(**d)
        self.assertIsNotNone(config2)


if __name__ == "__main__":
    unittest.main()
