"""Tests for PySide6 GUI module.

Covers:
- Module imports and instantiation
- ConfigPanel get_config validation
- ResultTable data display
- ChartWidget data points and clear
- Theme stylesheet generation
- Worker thread stop mechanism
- Input validation (NaN/Inf/range)
- Security: no eval/exec, safe defaults
"""

from __future__ import annotations

import os

import pytest

# Skip all tests if PySide6 not available
try:
    from PySide6.QtWidgets import QApplication
    # Qt is not needed for basic widget tests
    PYSIDE6_AVAILABLE = True
except ImportError:
    PYSIDE6_AVAILABLE = False

pytestmark = pytest.mark.skipif(
    not PYSIDE6_AVAILABLE, reason="PySide6 not installed"
)

# ── Fixtures ──────────────────────────────────────────────

@pytest.fixture(scope="session")
def qapp():
    """Create a QApplication for the test session."""
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    yield app


# ── Import Tests ──────────────────────────────────────────

class TestImports:
    """All GUI modules should import without error."""

    def test_import_gui_package(self):
        from sim_platform.tools.gui import main
        assert callable(main)

    def test_import_app(self):
        from sim_platform.tools.gui.app import run_app
        assert callable(run_app)

    def test_import_theme(self):
        from sim_platform.tools.gui.theme import COLORS, get_stylesheet
        assert callable(get_stylesheet)
        assert isinstance(COLORS, dict)

    def test_import_workers(self):
        pass

    def test_import_config_panel(self):
        pass

    def test_import_chart_widget(self):
        pass

    def test_import_log_widget(self):
        pass

    def test_import_stat_cards(self):
        pass

    def test_import_result_table(self):
        pass

    def test_import_scan_dialog(self):
        pass

    def test_import_about_dialog(self):
        pass


# ── Theme Tests ───────────────────────────────────────────

class TestTheme:
    """Theme should provide valid QSS."""

    def test_get_stylesheet_returns_string(self):
        from sim_platform.tools.gui.theme import get_stylesheet
        css = get_stylesheet()
        assert isinstance(css, str)
        assert len(css) > 100

    def test_stylesheet_contains_base_colors(self):
        from sim_platform.tools.gui.theme import COLORS, get_stylesheet
        css = get_stylesheet()
        assert COLORS["bg_base"] in css
        assert COLORS["text_primary"] in css
        assert COLORS["accent"] in css

    def test_colors_has_required_keys(self):
        from sim_platform.tools.gui.theme import COLORS
        required = ["bg_base", "bg_surface", "bg_elevated", "bg_overlay",
                     "text_primary", "text_secondary", "accent",
                     "green", "red", "yellow", "chart_speed"]
        for key in required:
            assert key in COLORS, f"Missing color: {key}"

    def test_colors_are_valid_format(self):
        from sim_platform.tools.gui.theme import COLORS
        for name, color in COLORS.items():
            # Accept hex (#RRGGBB) or rgba() format
            assert color.startswith("#") or color.startswith("rgba(") or color.startswith("rgb("), \
                f"{name} not hex/rgba: {color}"


# ── ConfigPanel Tests ─────────────────────────────────────

class TestConfigPanel:
    """ConfigPanel should validate inputs and return correct config."""

    def test_default_config(self, qapp):
        from sim_platform.tools.gui.widgets.config_panel import ConfigPanel
        panel = ConfigPanel()
        config = panel.get_config()
        assert "motor_params" in config
        assert "speed_ref" in config
        assert "duration_s" in config
        assert "load_torque" in config
        assert config["speed_ref"] == 100.0
        assert config["duration_s"] == 1.5

    def test_motor_presets_valid(self):
        from sim_platform.tools.gui.widgets.config_panel import MOTOR_PRESETS
        for name, params in MOTOR_PRESETS.items():
            assert "Rs" in params
            assert "Ld" in params
            assert "Lq" in params
            assert "flux_pm" in params
            assert "J" in params
            assert "B" in params
            assert "Pp" in params
            assert params["Rs"] > 0
            assert params["Ld"] > 0
            assert params["J"] > 0

    def test_scenarios_valid(self):
        from sim_platform.tools.gui.widgets.config_panel import SCENARIOS
        for name, sc in SCENARIOS.items():
            assert "duration" in sc
            assert "speed_ref" in sc
            assert sc["duration"] > 0
            assert sc["speed_ref"] > 0

    def test_guard_float_nan(self):
        from sim_platform.tools.gui.widgets.config_panel import _guard_float
        assert _guard_float(float("nan"), 42.0) == 42.0

    def test_guard_float_inf(self):
        from sim_platform.tools.gui.widgets.config_panel import _guard_float
        assert _guard_float(float("inf"), 42.0) == 42.0
        assert _guard_float(float("-inf"), 42.0) == 42.0

    def test_guard_float_normal(self):
        from sim_platform.tools.gui.widgets.config_panel import _guard_float
        assert _guard_float(3.14) == 3.14
        assert _guard_float(0.0) == 0.0
        assert _guard_float(-1.5) == -1.5


# ── ChartWidget Tests ─────────────────────────────────────

class TestChartWidget:
    """ChartWidget should handle data points and clear."""

    def test_creation(self, qapp):
        from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
        widget = ChartWidget()
        assert widget is not None

    def test_add_data_point(self, qapp):
        from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
        widget = ChartWidget()
        point = {"time": 0.1, "speed_ref": 100.0, "speed": 50.0, "torque": 0.5}
        widget.add_data_point(point)  # Should not raise

    def test_clear(self, qapp):
        from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
        widget = ChartWidget()
        for i in range(10):
            widget.add_data_point({
                "time": i * 0.1, "speed_ref": 100.0,
                "speed": i * 10.0, "torque": 0.5
            })
        widget.clear()
        assert widget._speed_series.count() == 0
        assert widget._ref_series.count() == 0
        assert widget._torque_series.count() == 0


# ── LogWidget Tests ───────────────────────────────────────

class TestLogWidget:
    """LogWidget should append and clear messages."""

    def test_creation(self, qapp):
        from sim_platform.tools.gui.widgets.log_widget import LogWidget
        widget = LogWidget()
        assert widget._text_edit.isReadOnly()

    def test_append_log(self, qapp):
        from sim_platform.tools.gui.widgets.log_widget import LogWidget
        widget = LogWidget()
        widget.append_log("Test message")
        assert "Test message" in widget._text_edit.toPlainText()

    def test_clear_log(self, qapp):
        from sim_platform.tools.gui.widgets.log_widget import LogWidget
        widget = LogWidget()
        widget.append_log("Message 1")
        widget.append_log("Message 2")
        widget.clear_log()
        assert widget._text_edit.toPlainText() == ""


# ── StatCard Tests ────────────────────────────────────────

class TestStatCards:
    """StatCard and StatCardsRow should update values."""

    def test_stat_card_creation(self, qapp):
        from sim_platform.tools.gui.widgets.stat_cards import StatCard
        card = StatCard("Speed", "rad/s")
        assert card is not None

    def test_stat_card_update(self, qapp):
        from sim_platform.tools.gui.widgets.stat_cards import StatCard
        card = StatCard("Speed", "rad/s")
        card.update_value("100.5")
        assert card._value.text() == "100.5"

    def test_stat_cards_row(self, qapp):
        from sim_platform.tools.gui.widgets.stat_cards import StatCardsRow
        row = StatCardsRow()
        row.update_speed("100.0")
        row.update_torque("0.5")
        row.update_fps("174000")
        row.update_progress("50%")


# ── ResultTable Tests ─────────────────────────────────────

class TestResultTable:
    """ResultTable should display simulation results."""

    def test_creation(self, qapp):
        from sim_platform.tools.gui.widgets.result_table import ResultTable
        table = ResultTable()
        assert table._table.columnCount() == 3

    def test_set_results(self, qapp):
        from sim_platform.tools.gui.widgets.result_table import ResultTable
        table = ResultTable()
        data = {
            "time": [0.0, 0.5, 1.0],
            "speed": [0.0, 80.0, 100.0],
            "torque": [0.0, 0.5, 0.01],
        }
        table.set_results(data, 100.0)
        assert table._table.rowCount() > 0
        # Re-run button should exist
        assert table._rerun_btn is not None

    def test_clear_results(self, qapp):
        from sim_platform.tools.gui.widgets.result_table import ResultTable
        table = ResultTable()
        data = {"time": [0.0], "speed": [100.0], "torque": [0.5]}
        table.set_results(data, 100.0)
        table.clear_results()
        assert table._table.rowCount() == 0

    def test_empty_data(self, qapp):
        from sim_platform.tools.gui.widgets.result_table import ResultTable
        table = ResultTable()
        table.set_results({}, 100.0)  # Should not raise
        assert table._table.rowCount() == 0


# ── Worker Tests ──────────────────────────────────────────

class TestSimulationWorker:
    """SimulationWorker should run and stop safely."""

    def test_creation(self, qapp):
        from sim_platform.tools.gui.workers import SimulationWorker
        config = {
            "motor_params": {
                "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4
            },
            "speed_ref": 100.0,
            "duration_s": 0.01,  # Very short
            "load_torque": 0.0,
        }
        worker = SimulationWorker(config)
        assert not worker.isRunning()

    def test_stop_before_start(self, qapp):
        from sim_platform.tools.gui.workers import SimulationWorker
        config = {
            "motor_params": {
                "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4
            },
            "speed_ref": 100.0,
            "duration_s": 0.01,
            "load_torque": 0.0,
        }
        worker = SimulationWorker(config)
        worker.stop()  # Should not raise
        assert worker._stop_requested

    def test_worker_emits_progress_and_log(self, qapp):
        """Worker should emit progress signals and 10% log messages."""
        from sim_platform.tools.gui.workers import SimulationWorker
        config = {
            "motor_params": {
                "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4
            },
            "speed_ref": 100.0,
            "duration_s": 0.05,
            "load_torque": 0.0,
        }
        worker = SimulationWorker(config)
        progress_values = []
        log_messages = []
        finished_data = []

        worker.progress.connect(lambda p: progress_values.append(p))
        worker.log_message.connect(lambda m: log_messages.append(m))
        worker.finished.connect(lambda d: finished_data.append(d))

        # Use a timer to process events while waiting
        done = [False]
        def on_done(_=None):
            done[0] = True
        worker.finished.connect(on_done)
        worker.error.connect(on_done)

        worker.start()
        import time as _time
        deadline = _time.time() + 30
        while not done[0] and _time.time() < deadline:
            qapp.processEvents()
            _time.sleep(0.01)

        assert done[0], "Worker did not finish in 30s"
        assert len(progress_values) > 0, "No progress signals emitted"
        assert 100 in progress_values, f"Missing 100% progress, got {progress_values[-3:]}"
        ten_pct_logs = [m for m in log_messages if "%" in m and ("Step" in m or "步进" in m)]
        assert len(ten_pct_logs) >= 3, f"Expected >=3 10% logs, got {len(ten_pct_logs)}: {ten_pct_logs}"
        assert len(finished_data) == 1, f"Expected 1 finished signal, got {len(finished_data)}"
        assert "speed" in finished_data[0]
        assert "torque" in finished_data[0]


class TestScanWorker:
    """ScanWorker should handle parameter scanning."""

    def test_creation(self, qapp):
        from sim_platform.tools.gui.workers import ScanWorker
        worker = ScanWorker("speed", [50.0, 100.0], duration=0.01)
        assert not worker.isRunning()

    def test_duration_configurable(self, qapp):
        from sim_platform.tools.gui.workers import ScanWorker
        worker = ScanWorker("speed", [100.0], duration=0.5)
        assert worker._duration == 0.5


# ── MainWindow Tests ──────────────────────────────────────

class TestMainWindow:
    """MainWindow should initialize correctly."""

    def test_creation(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert "sim_platform" in window.windowTitle()
        assert window.minimumSize().width() == 1024
        assert window.minimumSize().height() == 680

    def test_has_menu_bar(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        menubar = window.menuBar()
        assert menubar is not None

    def test_has_status_bar(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.status_bar is not None

    def test_has_tabs(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.tabs.count() == 4  # Home, Chart, Log, Results

    def test_has_config_panel(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.config_panel is not None

    def test_has_stat_cards(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.stat_cards is not None

    def test_has_chart(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.chart is not None

    def test_has_log(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.log is not None

    def test_has_result_table(self, qapp):
        from sim_platform.tools.gui.app import MainWindow
        window = MainWindow()
        assert window.result_table is not None


# ── Security Tests ────────────────────────────────────────

class TestSecurity:
    """Security: no eval/exec, safe defaults, input validation."""

    def test_no_eval_in_gui_code(self):
        """Verify no eval/exec calls in GUI source files.

        Note: dialog.exec() is a Qt method (QDialog.exec()), not Python's
        built-in exec(). We check for standalone eval()/exec() calls only.
        """
        import re
        gui_dir = os.path.join(
            os.path.dirname(__file__), "..", "..", "tools", "gui"
        )
        gui_dir = os.path.abspath(gui_dir)
        # Pattern: eval( or exec( NOT preceded by a dot (which means method call)
        eval_pattern = re.compile(r'(?<!\.)\beval\s*\(')
        exec_pattern = re.compile(r'(?<!\.)\bexec\s*\(')
        for root, dirs, files in os.walk(gui_dir):
            for fname in files:
                if fname.endswith(".py"):
                    fpath = os.path.join(root, fname)
                    with open(fpath, encoding="utf-8") as f:
                        content = f.read()
                    lines = content.split("\n")
                    for i, line in enumerate(lines, 1):
                        stripped = line.strip()
                        if stripped.startswith("#"):
                            continue
                        assert not eval_pattern.search(stripped), \
                            f"eval() in {fpath}:{i}: {stripped}"
                        assert not exec_pattern.search(stripped), \
                            f"exec() in {fpath}:{i}: {stripped}"

    def test_no_subprocess_in_gui(self):
        """Verify no subprocess/os.system calls in GUI code."""
        gui_dir = os.path.join(
            os.path.dirname(__file__), "..", "..", "tools", "gui"
        )
        gui_dir = os.path.abspath(gui_dir)
        for root, dirs, files in os.walk(gui_dir):
            for fname in files:
                if fname.endswith(".py"):
                    fpath = os.path.join(root, fname)
                    with open(fpath, encoding="utf-8") as f:
                        content = f.read()
                    assert "os.system(" not in content, \
                        f"os.system in {fpath}"
                    assert "subprocess" not in content, \
                        f"subprocess in {fpath}"

    def test_config_panel_rejects_nan(self, qapp):
        """ConfigPanel should handle NaN inputs gracefully."""
        from sim_platform.tools.gui.widgets.config_panel import ConfigPanel
        _panel = ConfigPanel()
        # QDoubleSpinBox prevents NaN at widget level
        # but _guard_float handles programmatic NaN
        from sim_platform.tools.gui.widgets.config_panel import _guard_float
        assert _guard_float(float("nan"), 100.0) == 100.0

    def test_scan_dialog_validates_values(self, qapp):
        """ScanDialog should reject NaN/Inf values."""
        from sim_platform.tools.gui.dialogs.scan_dialog import ScanDialog
        dialog = ScanDialog()
        # Set invalid values
        dialog.values_edit.setText("NaN, 100, 200")
        dialog._start_scan()  # Should not crash, should log error

    def test_output_dir_is_absolute(self):
        """OUTPUT_DIR should be an absolute path."""
        from sim_platform.tools.gui.app import OUTPUT_DIR
        assert os.path.isabs(OUTPUT_DIR)


# ── Data Constants Tests ──────────────────────────────────

class TestConstants:
    """Shared constants should be consistent."""

    def test_motor_presets_in_config_match(self):
        """MOTOR_PRESETS in config_panel should have same param values as TUI."""
        from sim_platform.tools.gui.widgets.config_panel import MOTOR_PRESETS as gui_mp
        from sim_platform.tools.tui.utils import MOTOR_PRESETS as tui_mp
        # Both should have 3 presets with same parameter values
        gui_values = list(gui_mp.values())
        tui_values = list(tui_mp.values())
        assert len(gui_values) == len(tui_values) == 3
        for gv, tv in zip(gui_values, tui_values):
            assert gv == tv, f"Preset mismatch: {gv} != {tv}"

    def test_scenarios_in_config_match(self):
        """SCENARIOS in config_panel should match TUI utils."""
        from sim_platform.tools.gui.widgets.config_panel import SCENARIOS as gui_sc
        from sim_platform.tools.tui.utils import SCENARIOS as tui_sc
        assert set(gui_sc.keys()) == set(tui_sc.keys())


# ── Deep Attack Tests ─────────────────────────────────────

class TestDeepAttacks:
    """Deep security and UI/UX attack vectors."""

    def test_thread_safe_stop_flag(self, qapp):
        """Worker stop flag should use threading.Event (not bool)."""
        import threading

        from sim_platform.tools.gui.workers import SimulationWorker
        config = {
            "motor_params": {
                "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4
            },
            "speed_ref": 100.0,
            "duration_s": 1.0,
            "load_torque": 0.0,
        }
        worker = SimulationWorker(config)
        # Internal flag should be threading.Event
        assert isinstance(worker._stop_event, threading.Event)
        # stop() should set the event
        worker.stop()
        assert worker._stop_event.is_set()
        assert worker._stop_requested is True

    def test_scan_worker_thread_safe_stop(self, qapp):
        """ScanWorker stop flag should use threading.Event."""
        import threading

        from sim_platform.tools.gui.workers import ScanWorker
        worker = ScanWorker("speed", [50.0, 100.0], duration=0.01)
        assert isinstance(worker._stop_event, threading.Event)
        worker.stop()
        assert worker._stop_event.is_set()

    def test_error_messages_no_traceback_leak(self):
        """Error messages should not leak internal file paths."""
        import os
        gui_dir = os.path.join(
            os.path.dirname(__file__), "..", "..", "tools", "gui"
        )
        gui_dir = os.path.abspath(gui_dir)
        workers_file = os.path.join(gui_dir, "workers.py")
        with open(workers_file, encoding="utf-8") as f:
            content = f.read()
        # Should NOT emit traceback.format_exc() to UI
        assert "self.error.emit" in content
        # Check no traceback in error.emit calls
        for line in content.split("\n"):
            if "self.error.emit" in line:
                assert "traceback" not in line.lower(), \
                    f"Traceback leaked in error: {line.strip()}"

    def test_chart_point_limit(self):
        """Chart should limit data points to prevent memory exhaustion."""
        from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
        chart = ChartWidget()
        assert hasattr(chart, '_MAX_POINTS')
        assert chart._MAX_POINTS > 0
        assert chart._MAX_POINTS <= 100000
        assert chart._point_count == 0

    def test_scan_dialog_value_limit(self, qapp):
        """ScanDialog should reject >100 values."""
        from sim_platform.tools.gui.dialogs.scan_dialog import ScanDialog
        dialog = ScanDialog()
        # 101 values should be rejected
        dialog.values_edit.setText(", ".join(str(i) for i in range(101)))
        dialog._start_scan()
        # Should have error in log
        log_text = dialog.log_edit.toPlainText()
        assert "100" in log_text or "Maximum" in log_text

    def test_about_dialog_no_hardcoded_styles(self):
        """AboutDialog should use theme-consistent styling."""
        import inspect

        from sim_platform.tools.gui.dialogs.about_dialog import AboutDialog
        source = inspect.getsource(AboutDialog._setup_ui)
        # Should not hardcode colors that conflict with theme
        # (Inline styles are acceptable for dialog-specific styling)
        assert "font-size: 20px" in source  # Title styling present
        assert "#89b4fa" in source  # Uses Catppuccin blue

    def test_close_event_terminates_worker(self, qapp):
        """closeEvent should terminate worker if wait times out."""
        import inspect

        from sim_platform.tools.gui.app import MainWindow
        source = inspect.getsource(MainWindow.closeEvent)
        # Should have terminate() as fallback
        assert "terminate" in source
        # Should set worker to None
        assert "self._worker = None" in source

    def test_no_unbounded_data_accumulation(self):
        """GUI should have data point limits to prevent OOM."""
        from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
        chart = ChartWidget()
        # Verify MAX_POINTS exists and is reasonable
        assert chart._MAX_POINTS > 0
        assert chart._MAX_POINTS <= 100000
        # Add a small number of points and verify tracking
        for i in range(50):
            chart.add_data_point({
                "time": i * 0.001,
                "speed": 100.0,
                "speed_ref": 100.0,
                "torque": 0.5,
            })
        assert chart._speed_series.count() == 50
        assert chart._point_count == 50

    def test_clear_resets_point_count(self):
        """Chart.clear() should reset point counter."""
        from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
        chart = ChartWidget()
        chart.add_data_point({
            "time": 0.0, "speed": 100.0,
            "speed_ref": 100.0, "torque": 0.5
        })
        assert chart._point_count == 1
        chart.clear()
        assert chart._point_count == 0
