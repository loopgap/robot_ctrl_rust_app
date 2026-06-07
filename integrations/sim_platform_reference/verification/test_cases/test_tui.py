"""Tests for sim_platform Textual TUI.

Covers:
- Shared helper function validation (_guard_float)
- Configuration data structures (MOTOR_PRESETS, SCENARIOS, SCAN_PARAMS)
- Screen composition and widget presence
- Navigation between screens
- Security: input bounds, NaN/Inf rejection
"""

import math
import os
import sys
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.tools.tui.app import SimPlatformTUI
from sim_platform.tools.tui.screens import (
    ConfigScreen,
    MainScreen,
    ResultsScreen,
    RunScreen,
    ScanScreen,
)
from sim_platform.tools.tui.utils import MOTOR_PRESETS, SCAN_PARAMS, SCENARIOS, guard_float

# Backward compat alias
_guard_float = guard_float


class TestGuardFloat(unittest.TestCase):
    """_guard_float: parse and validate float input with safety."""

    def test_normal_float(self):
        self.assertEqual(_guard_float("3.14"), 3.14)

    def test_integer_string(self):
        self.assertEqual(_guard_float("100"), 100.0)

    def test_nan_string_returns_fallback(self):
        self.assertEqual(_guard_float("nan"), 0.0)

    def test_inf_string_returns_fallback(self):
        self.assertEqual(_guard_float("inf"), 0.0)

    def test_negative_inf_returns_fallback(self):
        self.assertEqual(_guard_float("-inf"), 0.0)

    def test_garbage_returns_fallback(self):
        self.assertEqual(_guard_float("not_a_number"), 0.0)

    def test_empty_returns_fallback(self):
        self.assertEqual(_guard_float(""), 0.0)

    def test_custom_fallback(self):
        self.assertEqual(_guard_float("nan", 42.0), 42.0)

    def test_float_with_spaces(self):
        self.assertEqual(_guard_float("  3.14  "), 3.14)

    def test_none_type_returns_fallback(self):
        self.assertEqual(_guard_float(None), 0.0)

    def test_zero_input(self):
        self.assertEqual(_guard_float("0"), 0.0)

    def test_negative_float(self):
        self.assertEqual(_guard_float("-5.0"), -5.0)


class TestMotorPresets(unittest.TestCase):
    """MOTOR_PRESETS configuration validity."""

    def test_all_presets_have_required_keys(self):
        required = {"Rs", "Ld", "Lq", "flux_pm", "J", "B", "Pp"}
        for name, params in MOTOR_PRESETS.items():
            with self.subTest(preset=name):
                for key in required:
                    self.assertIn(key, params, f"{name} missing {key}")

    def test_all_params_finite_and_positive(self):
        for name, params in MOTOR_PRESETS.items():
            with self.subTest(preset=name):
                for key, val in params.items():
                    if key in ("Rs", "Ld", "Lq", "flux_pm", "J", "B"):
                        self.assertGreater(
                            val, 0,
                            f"{name}.{key}={val} must be positive")
                    if key == "Pp":
                        self.assertGreaterEqual(
                            val, 1,
                            f"{name}.{key}={val} must be >= 1")
                    self.assertFalse(
                        math.isnan(val) or math.isinf(val),
                        f"{name}.{key} is NaN or Inf")

    def test_three_presets(self):
        self.assertEqual(len(MOTOR_PRESETS), 3)


class TestScenarios(unittest.TestCase):
    """SCENARIOS configuration validity."""

    def test_all_scenarios_have_required_keys(self):
        required = {"duration", "speed_ref", "profile", "load"}
        for name, params in SCENARIOS.items():
            with self.subTest(scenario=name):
                for key in required:
                    self.assertIn(key, params, f"{name} missing {key}")

    def test_all_durations_reasonable(self):
        for name, params in SCENARIOS.items():
            self.assertGreater(params["duration"], 0.0)
            self.assertLess(params["duration"], 60.0)

    def test_speed_refs_reasonable(self):
        for name, params in SCENARIOS.items():
            self.assertGreater(params["speed_ref"], 0)
            self.assertLess(params["speed_ref"], 1000)

    def test_loads_non_negative(self):
        for name, params in SCENARIOS.items():
            self.assertGreaterEqual(params["load"], 0)

    def test_four_scenarios(self):
        self.assertEqual(len(SCENARIOS), 4)


class TestScanParams(unittest.TestCase):
    """SCAN_PARAMS configuration validity."""

    def test_all_scan_params_have_two_elements(self):
        for name, (key, values) in SCAN_PARAMS.items():
            with self.subTest(param=name):
                self.assertIsInstance(key, str)
                self.assertIsInstance(values, (list, tuple))
                self.assertGreaterEqual(len(values), 2)

    def test_all_values_finite(self):
        for name, (key, values) in SCAN_PARAMS.items():
            for v in values:
                self.assertFalse(math.isnan(v) or math.isinf(v),
                                 f"{name}:{v} is NaN/Inf")

    def test_five_scan_params(self):
        self.assertEqual(len(SCAN_PARAMS), 5)


class TestTUIStructure(unittest.TestCase):
    """TUI app structure and screen definitions."""

    def test_app_has_screen_attributes(self):
        """App stores screen instances."""
        attrs = ["main_screen", "config_screen", "run_screen",
                 "results_screen", "scan_screen"]
        for attr in attrs:
            self.assertTrue(hasattr(SimPlatformTUI, attr),
                            f"Missing attribute: {attr}")

    def test_app_has_bindings(self):
        self.assertGreater(len(SimPlatformTUI.BINDINGS), 0)

    def test_app_binding_keys_unique(self):
        keys = [b.key for b in SimPlatformTUI.BINDINGS]
        self.assertEqual(len(keys), len(set(keys)),
                         "Duplicate binding keys found")

    def test_goto_method_exists(self):
        self.assertTrue(hasattr(SimPlatformTUI, "goto"))

    def test_screens_inherit_from_screen(self):
        from textual.screen import Screen
        for screen_cls in [MainScreen, ConfigScreen, RunScreen,
                           ResultsScreen, ScanScreen]:
            self.assertTrue(issubclass(screen_cls, Screen),
                            f"{screen_cls.__name__} is not a Screen")

    def test_screens_have_compose(self):
        for screen_cls in [MainScreen, ConfigScreen, RunScreen,
                           ResultsScreen, ScanScreen]:
            self.assertTrue(hasattr(screen_cls, "compose"),
                            f"{screen_cls.__name__} missing compose()")


class TestTUIWidgetPresence(unittest.TestCase):
    """Verify that screens contain expected widgets via compose()."""

    def _widget_ids_from_compose(self, screen_cls) -> set:
        """Extract widget IDs from compose() without running the app."""
        import inspect
        source = inspect.getsource(screen_cls.compose)
        import re
        # Find all `id="..."` occurrences
        ids = set(re.findall(r'id="([^"]+)"', source))
        return ids

    def test_main_screen_has_buttons(self):
        ids = self._widget_ids_from_compose(MainScreen)
        # Modern UI uses InfoCard instead of buttons
        self.assertTrue(
            "btn-run" in ids or "card-run" in ids,
            "Expected btn-run or card-run in widget IDs"
        )
        self.assertTrue(
            "btn-config" in ids or "card-config" in ids,
            "Expected btn-config or card-config in widget IDs"
        )
        self.assertTrue(
            "btn-scan" in ids or "card-scan" in ids,
            "Expected btn-scan or card-scan in widget IDs"
        )

    def test_config_screen_has_inputs(self):
        ids = self._widget_ids_from_compose(ConfigScreen)
        self.assertIn("speed_ref", ids)
        self.assertIn("duration", ids)
        self.assertIn("load_torque", ids)
        self.assertIn("motor_preset", ids)
        self.assertIn("scenario", ids)

    def test_config_screen_has_run_buttons(self):
        ids = self._widget_ids_from_compose(ConfigScreen)
        self.assertIn("run-now", ids)
        self.assertIn("run-plot", ids)

    def test_run_screen_has_progress_and_log(self):
        ids = self._widget_ids_from_compose(RunScreen)
        self.assertIn("progress", ids)
        self.assertIn("run-log", ids)
        self.assertIn("view-results", ids)

    def test_results_screen_has_table(self):
        ids = self._widget_ids_from_compose(ResultsScreen)
        self.assertIn("metrics-table", ids)
        self.assertIn("rerun", ids)
        self.assertIn("plot", ids)

    def test_scan_screen_has_param_select(self):
        ids = self._widget_ids_from_compose(ScanScreen)
        self.assertIn("scan-param", ids)
        self.assertIn("scan-values", ids)
        self.assertIn("scan-progress", ids)
        self.assertIn("start-scan", ids)


if __name__ == "__main__":
    unittest.main()
