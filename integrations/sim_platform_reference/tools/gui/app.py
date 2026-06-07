"""Main application window for sim_platform PySide6 GUI.

Professional-grade QMainWindow with:
- Complete File menu (Open/Save/Export/Recent)
- Simulation control (Run/Pause/Resume/Stop)
- Multi-zone status bar
- Window state persistence (QSettings)
- Keyboard shortcuts
- Config panel lock during simulation
"""

from __future__ import annotations

import csv
import json
import os

# ── Workspace directory management ────────────────────────
# When frozen (PyInstaller), use the directory containing the executable
# Otherwise, use the project root (3 levels up from this file)
import sys as _sys
import time

from PySide6.QtCore import Qt, Slot
from PySide6.QtGui import QAction, QKeySequence
from PySide6.QtWidgets import (
    QApplication,
    QDockWidget,
    QFileDialog,
    QMainWindow,
    QMenu,
    QMessageBox,
    QProgressBar,
    QStatusBar,
    QTabWidget,
    QToolBar,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.dialogs.about_dialog import AboutDialog
from sim_platform.tools.gui.dialogs.conflict_dialog import ConflictDialog, show_conflict_dialog
from sim_platform.tools.gui.dialogs.onboarding_dialog import OnboardingDialog
from sim_platform.tools.gui.dialogs.scan_dialog import ScanDialog
from sim_platform.tools.gui.guided_tour import GuidedTourEngine
from sim_platform.tools.gui.icons import SimIcons, get_icon
from sim_platform.tools.gui.i18n import (
    get_language,
    get_supported_languages,
    load_language,
    set_language,
    tr,
)
from sim_platform.tools.gui.solver_presets import SolverPresetManager, get_preset_manager
from sim_platform.tools.gui.theme import get_stylesheet
from sim_platform.tools.gui.widgets.chart_widget import ChartWidget
from sim_platform.tools.gui.widgets.config_panel import ConfigPanel
from sim_platform.tools.gui.widgets.dashboard import DashboardWidget
from sim_platform.tools.gui.widgets.log_widget import LogWidget
from sim_platform.tools.gui.widgets.result_table import ResultTable
from sim_platform.tools.gui.widgets.stat_cards import StatCardsRow
from sim_platform.tools.gui.workers import SimulationWorker

if getattr(_sys, "frozen", False):
    # PyInstaller frozen: executable directory is the workspace
    _PROJ = os.path.dirname(_sys.executable)
else:
    _PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))

OUTPUT_DIR = os.path.join(_PROJ, "output")
CONFIGS_DIR = os.path.join(_PROJ, "configs")
LOGS_DIR = os.path.join(_PROJ, "logs")
os.makedirs(OUTPUT_DIR, exist_ok=True)
os.makedirs(CONFIGS_DIR, exist_ok=True)
os.makedirs(LOGS_DIR, exist_ok=True)


def _is_within_workspace(path: str) -> bool:
    """Check if a path is within the workspace directory (CWE-22 prevention)."""
    abs_path = os.path.abspath(path)
    return abs_path.startswith(_PROJ + os.sep) or abs_path == _PROJ


def _safe_path(path: str) -> str:
    """Validate and return absolute path within workspace. Raises ValueError if outside."""
    abs_path = os.path.abspath(path)
    if not _is_within_workspace(abs_path):
        raise ValueError(f"Path outside workspace not allowed: {path}")
    return abs_path

# ── Version ───────────────────────────────────────────────
try:
    import tomllib
    # Try multiple locations for pyproject.toml
    _candidates = [
        os.path.join(_PROJ, "pyproject.toml"),           # workspace root
        os.path.join(os.path.dirname(__file__), "..", "..", "..", "pyproject.toml"),  # source tree
    ]
    if getattr(_sys, "frozen", False):
        _candidates.insert(0, os.path.join(_sys._MEIPASS, "sim_platform", "pyproject.toml"))
    _VERSION = "0.0.1"
    for _p in _candidates:
        if os.path.exists(_p):
            with open(_p, "rb") as f:
                _VERSION = tomllib.load(f).get("project", {}).get("version", "0.0.1")
            break
except Exception:
    _VERSION = "0.0.1"

# ── QSettings keys ────────────────────────────────────────
_SETTINGS_ORG = "sim_platform"
_SETTINGS_APP = "sim_platform_gui"


class MainWindow(QMainWindow):
    """Main application window with professional features."""

    def __init__(self):
        super().__init__()
        # Load persisted language
        load_language()

        # Icon system
        self._icons = SimIcons()

        # Solver presets
        self._solver = get_preset_manager()

        # Guided tour
        self._tour = GuidedTourEngine(self)
        self._tour.tour_completed.connect(self._on_tour_completed)

        self.setWindowTitle(f"sim_platform v{_VERSION} — {tr('app.title')}")
        self.setMinimumSize(1024, 680)

        # Set application icon
        self.setWindowIcon(self._icons.app_icon(64))

        self._worker = None
        self._sim_data = None
        self._speed_ref = 100.0
        self._last_update_time = 0.0
        self._step_count = 0
        self._current_file = None
        self._recent_files: list[str] = []

        self._setup_ui()
        self._setup_menus()
        self._setup_toolbar()
        self._setup_statusbar()
        self._load_settings()

    def _setup_ui(self):
        """Create central widget, dock, and tabs."""
        # ── Central widget with tabs ──────────────────────
        central = QWidget()
        central_layout = QVBoxLayout(central)
        central_layout.setContentsMargins(4, 4, 4, 4)

        # Stat cards row
        self.stat_cards = StatCardsRow()
        central_layout.addWidget(self.stat_cards)

        # Tab widget
        self.tabs = QTabWidget()
        self.tabs.setTabPosition(QTabWidget.TabPosition.North)

        # Dashboard tab (unified home screen)
        self.dashboard = DashboardWidget()
        self.dashboard.action_triggered.connect(self._on_dashboard_action)
        self.dashboard.scenario_selected.connect(self._on_scenario_selected)
        self.tabs.addTab(self.dashboard, self._icons.get("home"), tr("tab.home"))

        # Chart tab
        self.chart = ChartWidget()
        self.tabs.addTab(self.chart, self._icons.get("chart"), tr("tab.chart"))

        # Log tab
        self.log = LogWidget()
        self.tabs.addTab(self.log, self._icons.get("log"), tr("tab.log"))

        # Results tab
        self.result_table = ResultTable()
        self.result_table.rerun_requested.connect(self._start_simulation)
        self.tabs.addTab(self.result_table, self._icons.get("results"), tr("tab.results"))

        central_layout.addWidget(self.tabs)

        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setValue(0)
        self.progress_bar.setMaximumHeight(6)
        central_layout.addWidget(self.progress_bar)

        self.setCentralWidget(central)

        # ── Config dock widget ────────────────────────────
        self.config_panel = ConfigPanel()
        dock = QDockWidget(tr("config.file"), self)
        dock.setObjectName("config_dock")
        dock.setWidget(self.config_panel)
        dock.setFeatures(
            QDockWidget.DockWidgetFeature.DockWidgetMovable
            | QDockWidget.DockWidgetFeature.DockWidgetFloatable
        )
        self.addDockWidget(Qt.DockWidgetArea.LeftDockWidgetArea, dock)
        self._dock = dock

    def _setup_menus(self):
        """Create menu bar with File, Simulation, View, Tools, Help."""
        menubar = self.menuBar()

        # ── File ──────────────────────────────────────────
        file_menu = menubar.addMenu(tr("menu.file"))

        new_action = QAction(tr("menu.file.new"), self)
        new_action.setShortcut(QKeySequence("Ctrl+N"))
        new_action.setToolTip(tr("tooltip.new"))
        new_action.triggered.connect(self._new_config)
        file_menu.addAction(new_action)

        open_action = QAction(tr("menu.file.open"), self)
        open_action.setShortcut(QKeySequence("Ctrl+O"))
        open_action.setToolTip(tr("tooltip.open"))
        open_action.triggered.connect(self._open_config)
        file_menu.addAction(open_action)

        save_action = QAction(tr("menu.file.save"), self)
        save_action.setShortcut(QKeySequence("Ctrl+S"))
        save_action.setToolTip(tr("tooltip.save"))
        save_action.triggered.connect(self._save_config)
        file_menu.addAction(save_action)

        save_as_action = QAction(tr("menu.file.save_as"), self)
        save_as_action.setShortcut(QKeySequence("Ctrl+Shift+S"))
        save_as_action.triggered.connect(self._save_config_as)
        file_menu.addAction(save_as_action)

        file_menu.addSeparator()

        export_csv_action = QAction(tr("menu.file.export_csv"), self)
        export_csv_action.setShortcut(QKeySequence("Ctrl+E"))
        export_csv_action.setToolTip(tr("tooltip.export_csv"))
        export_csv_action.triggered.connect(self._export_csv)
        file_menu.addAction(export_csv_action)

        export_json_action = QAction(tr("menu.file.export_json"), self)
        export_json_action.triggered.connect(self._export_json)
        file_menu.addAction(export_json_action)

        file_menu.addSeparator()

        # Recent files submenu
        self._recent_menu = QMenu(tr("menu.file.recent"), self)
        file_menu.addMenu(self._recent_menu)
        self._update_recent_menu()

        file_menu.addSeparator()

        exit_action = QAction(tr("menu.file.exit"), self)
        exit_action.setShortcut(QKeySequence("Ctrl+Q"))
        exit_action.triggered.connect(self.close)
        file_menu.addAction(exit_action)

        # ── Simulation ────────────────────────────────────
        sim_menu = menubar.addMenu(tr("menu.sim"))

        self._run_action = QAction(tr("menu.sim.run"), self)
        self._run_action.setShortcut(QKeySequence("F5"))
        self._run_action.triggered.connect(self._start_simulation)
        sim_menu.addAction(self._run_action)

        self._pause_action = QAction(tr("menu.sim.pause"), self)
        self._pause_action.setShortcut(QKeySequence("F6"))
        self._pause_action.setEnabled(False)
        self._pause_action.triggered.connect(self._toggle_pause)
        sim_menu.addAction(self._pause_action)

        self._stop_action = QAction(tr("menu.sim.stop"), self)
        self._stop_action.setShortcut(QKeySequence("Shift+F5"))
        self._stop_action.setEnabled(False)
        self._stop_action.triggered.connect(self._stop_simulation)
        sim_menu.addAction(self._stop_action)

        sim_menu.addSeparator()

        rerun_action = QAction(tr("result.rerun"), self)
        rerun_action.setShortcut(QKeySequence("Ctrl+R"))
        rerun_action.triggered.connect(self._start_simulation)
        sim_menu.addAction(rerun_action)

        # ── View ──────────────────────────────────────────
        view_menu = menubar.addMenu(tr("menu.view"))

        toggle_config = QAction(tr("menu.view.config"), self)
        toggle_config.setShortcut(QKeySequence("Ctrl+D"))
        toggle_config.triggered.connect(self._toggle_config_panel)
        view_menu.addAction(toggle_config)

        toggle_toolbar = QAction(tr("menu.view.toolbar"), self)
        toggle_toolbar.setShortcut(QKeySequence("Ctrl+T"))
        toggle_toolbar.triggered.connect(self._toggle_toolbar)
        view_menu.addAction(toggle_toolbar)

        view_menu.addSeparator()

        _tab_keys = ["tab.home", "tab.chart", "tab.log", "tab.results"]
        for i, key in enumerate(_tab_keys):
            name = tr(key)
            action = QAction(tr("menu.view.show", name), self)
            action.setShortcut(QKeySequence(f"Ctrl+{i+1}"))
            action.triggered.connect(lambda checked, idx=i: self.tabs.setCurrentIndex(idx))
            view_menu.addAction(action)

        view_menu.addSeparator()

        reset_layout = QAction(tr("menu.view.reset"), self)
        reset_layout.triggered.connect(self._reset_layout)
        view_menu.addAction(reset_layout)

        # ── Tools ─────────────────────────────────────────
        tools_menu = menubar.addMenu(tr("menu.tools"))

        scan_action = QAction(tr("menu.tools.scan"), self)
        scan_action.setShortcut(QKeySequence("Ctrl+Shift+P"))
        scan_action.triggered.connect(self._open_scan_dialog)
        tools_menu.addAction(scan_action)

        # ── Help ──────────────────────────────────────────
        help_menu = menubar.addMenu(tr("menu.help"))

        # Guided tour
        self._tour_action = QAction(tr("tour.start"), self)
        self._tour_action.setShortcut(QKeySequence("Ctrl+F1"))
        self._tour_action.triggered.connect(self._start_tour)
        help_menu.addAction(self._tour_action)

        help_menu.addSeparator()

        shortcuts_action = QAction(tr("menu.help.shortcuts"), self)
        shortcuts_action.setShortcut(QKeySequence("F1"))
        shortcuts_action.triggered.connect(self._show_shortcuts)
        help_menu.addAction(shortcuts_action)

        help_menu.addSeparator()

        # Language submenu
        lang_menu = help_menu.addMenu(tr("menu.help.language"))
        self._lang_actions: dict[str, QAction] = {}
        for code, display in get_supported_languages():
            action = QAction(display, self)
            action.setCheckable(True)
            action.setChecked(code == get_language())
            action.triggered.connect(lambda checked, c=code: self._switch_language(c))
            lang_menu.addAction(action)
            self._lang_actions[code] = action

        help_menu.addSeparator()

        # Show onboarding
        onboarding_action = QAction(tr("onboarding.title"), self)
        onboarding_action.triggered.connect(self._show_onboarding)
        help_menu.addAction(onboarding_action)

        help_menu.addSeparator()

        about_action = QAction(tr("menu.help.about"), self)
        about_action.triggered.connect(self._show_about)
        help_menu.addAction(about_action)

    def _setup_toolbar(self):
        """Create toolbar with simulation controls using SVG icons."""
        toolbar = QToolBar(tr("toolbar.title"))
        toolbar.setMovable(False)
        toolbar.setObjectName("main_toolbar")
        self.addToolBar(toolbar)

        tb_home = QAction(self._icons.get("home"), tr("toolbar.home"), self)
        tb_home.setToolTip(tr("tooltip.home"))
        tb_home.triggered.connect(lambda: self.tabs.setCurrentIndex(0))
        toolbar.addAction(tb_home)

        toolbar.addSeparator()

        self._tb_run = QAction(self._icons.get("run"), tr("toolbar.run"), self)
        self._tb_run.setToolTip(tr("tooltip.run"))
        self._tb_run.triggered.connect(self._start_simulation)
        toolbar.addAction(self._tb_run)

        self._tb_pause = QAction(self._icons.get("pause"), tr("toolbar.pause"), self)
        self._tb_pause.setToolTip(tr("tooltip.pause"))
        self._tb_pause.setEnabled(False)
        self._tb_pause.triggered.connect(self._toggle_pause)
        toolbar.addAction(self._tb_pause)

        self._tb_stop = QAction(self._icons.get("stop"), tr("toolbar.stop"), self)
        self._tb_stop.setToolTip(tr("tooltip.stop"))
        self._tb_stop.setEnabled(False)
        self._tb_stop.triggered.connect(self._stop_simulation)
        toolbar.addAction(self._tb_stop)

        toolbar.addSeparator()

        tb_results = QAction(self._icons.get("results"), tr("toolbar.results"), self)
        tb_results.setToolTip(tr("tooltip.results"))
        tb_results.triggered.connect(lambda: self.tabs.setCurrentIndex(2))
        toolbar.addAction(tb_results)

    def _setup_statusbar(self):
        """Create multi-zone status bar."""
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)

        # Zone 1: Main message (left)
        self._status_msg = self.status_bar

        # Zone 2: Simulation time
        self._status_time = self.status_bar

        # Zone 3: Config file
        self._status_file = self.status_bar

        self.status_bar.showMessage(tr("app.ready"))
        self.status_bar.addWidget(self._status_time, 0)
        self.status_bar.addPermanentWidget(self._status_file)

    def _load_settings(self):
        """Load window state and recent files from QSettings."""
        from PySide6.QtCore import QSettings
        settings = QSettings(_SETTINGS_ORG, _SETTINGS_APP)
        geometry = settings.value("geometry")
        if geometry:
            self.restoreGeometry(geometry)
        state = settings.value("windowState")
        if state:
            self.restoreState(state)
        recent = settings.value("recentFiles", [])
        if isinstance(recent, list):
            self._recent_files = recent
        self._update_recent_menu()

    def _save_settings(self):
        """Save window state and recent files to QSettings."""
        from PySide6.QtCore import QSettings
        settings = QSettings(_SETTINGS_ORG, _SETTINGS_APP)
        settings.setValue("geometry", self.saveGeometry())
        settings.setValue("windowState", self.saveState())
        settings.setValue("recentFiles", self._recent_files)

    def _add_recent_file(self, path: str):
        """Add a file to the recent files list."""
        if path in self._recent_files:
            self._recent_files.remove(path)
        self._recent_files.insert(0, path)
        self._recent_files = self._recent_files[:10]
        self._update_recent_menu()

    def _update_recent_menu(self):
        """Update the Recent Files submenu."""
        self._recent_menu.clear()
        for path in self._recent_files:
            action = QAction(os.path.basename(path), self)
            action.setToolTip(path)
            action.triggered.connect(lambda checked, p=path: self._load_config_file(p))
            self._recent_menu.addAction(action)
        if not self._recent_files:
            action = QAction("(empty)", self)
            action.setEnabled(False)
            self._recent_menu.addAction(action)

    # ── File operations ───────────────────────────────────

    def _new_config(self):
        """Reset configuration to defaults."""
        if self._worker and self._worker.isRunning():
            QMessageBox.warning(self, tr("dialog.warning"), tr("dialog.cannot_reset_running"))
            return
        self.config_panel._reset_defaults()
        self._current_file = None
        self._status_file.setText("")

    def _open_config(self):
        """Open configuration file dialog (restricted to workspace)."""
        path, _ = QFileDialog.getOpenFileName(
            self, "Open Configuration", CONFIGS_DIR,
            "YAML Files (*.yaml *.yml);;JSON Files (*.json);;All Files (*)"
        )
        if path:
            if not _is_within_workspace(path):
                QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
                return
            self._load_config_file(path)

    def _load_config_file(self, path: str):
        """Load a configuration file."""
        if not os.path.exists(path):
            QMessageBox.warning(self, tr("dialog.file_not_found").split("\n")[0], tr("dialog.file_not_found", path))
            self._recent_files = [f for f in self._recent_files if f != path]
            self._update_recent_menu()
            return
        try:
            import yaml
            with open(path, encoding='utf-8') as f:
                if path.endswith(('.yaml', '.yml')):
                    data = yaml.safe_load(f)
                else:
                    data = json.load(f)
            self.config_panel._apply_config_dict(data)
            self._current_file = path
            self._add_recent_file(path)
            self._status_file.setText(os.path.basename(path))
            self.log.append_log(f"Loaded config: {os.path.basename(path)}")
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.load_error"), tr("dialog.failed_load", str(e)))

    def _save_config(self):
        """Save current configuration."""
        if self._current_file:
            self._save_config_to(self._current_file)
        else:
            self._save_config_as()

    def _save_config_as(self):
        """Save configuration to a new file (restricted to workspace)."""
        path, _ = QFileDialog.getSaveFileName(
            self, "Save Configuration", os.path.join(CONFIGS_DIR, "config.yaml"),
            "YAML Files (*.yaml);;JSON Files (*.json)"
        )
        if path:
            if not _is_within_workspace(path):
                QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
                return
            self._save_config_to(path)

    def _save_config_to(self, path: str):
        """Save configuration to specified path."""
        try:
            cfg = self.config_panel.get_config()
            import yaml
            with open(path, 'w', encoding='utf-8') as f:
                yaml.dump(cfg, f, default_flow_style=False, sort_keys=False)
            self._current_file = path
            self._add_recent_file(path)
            self._status_file.setText(os.path.basename(path))
            self.log.append_log(f"Saved config: {os.path.basename(path)}")
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.save_error"), tr("dialog.failed_save", str(e)))

    def _export_csv(self):
        """Export simulation results to CSV (restricted to workspace)."""
        if not self._sim_data:
            QMessageBox.information(self, tr("dialog.no_data"), tr("dialog.no_data.msg"))
            return
        path, _ = QFileDialog.getSaveFileName(
            self, "Export CSV", os.path.join(OUTPUT_DIR, "results.csv"),
            "CSV Files (*.csv);;All Files (*)"
        )
        if not path:
            return
        if not _is_within_workspace(path):
            QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
            return
        try:
            data = self._sim_data
            headers = list(data.keys())
            with open(path, 'w', newline='', encoding='utf-8') as f:
                writer = csv.writer(f)
                writer.writerow(headers)
                for i in range(len(data["time"])):
                    writer.writerow([data[h][i] for h in headers])
            self.log.append_log(f"Exported CSV: {os.path.basename(path)}")
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.export_error"), tr("dialog.failed_export", str(e)))

    def _export_json(self):
        """Export simulation results to JSON (restricted to workspace)."""
        if not self._sim_data:
            QMessageBox.information(self, tr("dialog.no_data"), tr("dialog.failed_export", ""))
            return
        path, _ = QFileDialog.getSaveFileName(
            self, "Export JSON", os.path.join(OUTPUT_DIR, "results.json"),
            "JSON Files (*.json);;All Files (*)"
        )
        if not path:
            return
        if not _is_within_workspace(path):
            QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
            return
        try:
            with open(path, 'w', encoding='utf-8') as f:
                json.dump(self._sim_data, f, indent=2)
            self.log.append_log(f"Exported JSON: {os.path.basename(path)}")
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.export_error"), tr("dialog.failed_export", str(e)))

    # ── Simulation control ────────────────────────────────

    def _start_simulation(self):
        """Validate config and start simulation worker."""
        try:
            config = self.config_panel.get_config()
        except ValueError as e:
            QMessageBox.warning(self, tr("dialog.config_error"), str(e))
            return

        # Physics constraint validation with conflict resolver
        try:
            from sim_platform.models.physics_constraints import PhysicsValidator
            validator = PhysicsValidator()
            violations = validator.validate(config)
            errors = [v for v in violations if v.severity == "error"]
            warnings = [v for v in violations if v.severity == "warning"]

            if errors:
                # Use enhanced conflict dialog for errors
                modified_config, accepted = show_conflict_dialog(config, self)
                if not accepted:
                    return
                config = modified_config
                # Re-run validation on modified config
                violations = validator.validate(config)
                errors = [v for v in violations if v.severity == "error"]
                if errors:
                    QMessageBox.critical(
                        self, tr("dialog.config_error"),
                        "仍有未解决的参数错误，无法运行仿真。"
                    )
                    return
            elif warnings:
                # Use enhanced conflict dialog for warnings
                modified_config, accepted = show_conflict_dialog(config, self)
                if not accepted:
                    return
                config = modified_config
        except ImportError:
            self.log.append_log("Warning: PhysicsValidator not available, skipping validation")
        except Exception as e:
            self.log.append_log(f"Warning: Validation failed: {e}")

        # Apply solver presets
        solver_params = self._solver.current_parameters
        if solver_params:
            cfg_dt_c = config.get("dt_c", 50e-6)
            cfg_dt_s = config.get("dt_s", 1e-3)
            if cfg_dt_c != solver_params.dt_current or cfg_dt_s != solver_params.dt_speed:
                config["dt_c"] = solver_params.dt_current
                config["dt_s"] = solver_params.dt_speed

        # Clear previous state
        self.chart.clear()
        self.log.clear_log()
        self.result_table.clear_results()
        self.progress_bar.setValue(0)
        self.stat_cards.update_speed("--")
        self.stat_cards.update_torque("--")
        self.stat_cards.update_fps("--")
        self.stat_cards.update_progress("0%")

        self._speed_ref = config["speed_ref"]
        self._sim_data = None
        self._last_update_time = time.time()
        self._step_count = 0

        # Update UI state
        self._set_running(True)
        self.status_bar.showMessage(tr("status.running"))
        self.tabs.setCurrentIndex(0)  # Show chart

        # Start worker
        self._worker = SimulationWorker(config, self)
        self._worker.progress.connect(self._on_progress)
        self._worker.data_update.connect(self._on_data_update)
        self._worker.log_message.connect(self.log.append_log)
        self._worker.finished.connect(self._on_finished)
        self._worker.error.connect(self._on_error)
        self._worker.status.connect(self._on_status)
        self._worker.finished.connect(self._worker_cleanup)
        self._worker.error.connect(self._worker_cleanup)
        self._worker.start()

    def _stop_simulation(self):
        """Request simulation stop."""
        if self._worker:
            self._worker.stop()
            self.status_bar.showMessage(tr("status.stopping"))

    def _toggle_pause(self):
        """Toggle pause/resume."""
        if not self._worker:
            return
        if self._worker.is_paused:
            self._worker.resume()
            self._pause_action.setText(tr("menu.sim.pause"))
            self._tb_pause.setText(tr("toolbar.pause"))
            self.status_bar.showMessage(tr("status.resumed"))
        else:
            self._worker.pause()
            self._pause_action.setText(tr("menu.sim.resume"))
            self._tb_pause.setText(tr("toolbar.resume"))
            self.status_bar.showMessage(tr("status.paused"))

    def _set_running(self, running: bool):
        """Toggle UI elements for running/stopped state."""
        self._run_action.setEnabled(not running)
        self._tb_run.setEnabled(not running)
        self._stop_action.setEnabled(running)
        self._tb_stop.setEnabled(running)
        self._pause_action.setEnabled(running)
        self._tb_pause.setEnabled(running)
        if not running:
            self._pause_action.setText(tr("menu.sim.pause"))
            self._tb_pause.setText(tr("toolbar.pause"))
        # Lock config panel during simulation
        self.config_panel.setEnabled(not running)

    # ── Slots ─────────────────────────────────────────────

    @Slot(int)
    def _on_progress(self, pct: int):
        self.progress_bar.setValue(pct)
        self.stat_cards.update_progress(f"{pct}%")

    @Slot(dict)
    def _on_data_update(self, point: dict):
        self.chart.add_data_point(point)

        # Update stat cards
        self.stat_cards.update_speed(f"{point['speed']:.1f}", ref=self._speed_ref)
        self.stat_cards.update_torque(f"{point['torque']:.3f}")

        # FPS calculation
        self._step_count += 100
        now = time.time()
        elapsed = now - self._last_update_time
        if elapsed > 0:
            fps = self._step_count / elapsed
            self.stat_cards.update_fps(f"{fps:.0f}")

    @Slot(dict)
    def _on_finished(self, data: dict):
        self._sim_data = data
        self._set_running(False)
        self.progress_bar.setValue(100)
        self.status_bar.showMessage(tr("status.complete"))
        self.result_table.set_results(data, self._speed_ref)

        # Save HDF5 log
        try:
            from sim_platform.tools.replay.hdf5_logger import HDF5Logger
            safe_ref = int(max(0, min(500, self._speed_ref)))
            fname = f"gui_run_{safe_ref}rads.h5"
            fpath = os.path.join(OUTPUT_DIR, fname)
            with HDF5Logger(fpath) as hlog:
                for i in range(0, len(data["time"]), 10):
                    hlog.record(
                        data["time"][i],
                        speed_ref=data["speed_ref"][i],
                        speed=data["speed"][i],
                        torque=data["torque"][i],
                        id=data["id"][i],
                        iq=data["iq"][i],
                    )
            self.log.append_log(f"Saved -> {fname}")
        except Exception as e:
            self.log.append_log(f"Warning: Could not save HDF5: {e}")

    @Slot(str)
    def _on_error(self, msg: str):
        self._set_running(False)
        self.status_bar.showMessage(tr("status.failed"))
        self.log.append_log(f"Error: {msg}")
        QMessageBox.critical(self, tr("dialog.sim_error"), msg)

    @Slot(str)
    def _on_status(self, status: str):
        """Handle status changes from worker."""
        if status == "paused":
            self.status_bar.showMessage(tr("status.paused"))
        elif status == "running":
            self.status_bar.showMessage(tr("status.running"))

    @Slot()
    def _worker_cleanup(self):
        if self._worker:
            self._worker.wait()
            self._worker = None

    # ── Dashboard actions ─────────────────────────────────

    @Slot(str)
    def _on_dashboard_action(self, action: str):
        """Handle quick action clicks from dashboard."""
        if action == "new_sim":
            self._start_simulation()
        elif action == "open_config":
            self._open_config()
        elif action == "load_results":
            self._load_results()
        elif action == "scan":
            self._open_scan_dialog()
        elif action == "quick_start":
            # Set defaults and run
            self.config_panel._reset_defaults()
            self._start_simulation()

    @Slot(str)
    def _on_scenario_selected(self, name: str):
        """Handle scenario card click from dashboard."""
        from sim_platform.tools.gui.widgets.config_panel import SCENARIOS
        scenario = SCENARIOS.get(name)
        if scenario:
            self.config_panel._apply_config_dict({
                "speed_ref": scenario["speed_ref"],
                "duration_s": scenario["duration"],
                "load_torque": scenario["load"],
                "profile": scenario.get("profile", "step"),
            })
            self.config_panel.scenario_combo.setCurrentText(name)
            self.tabs.setCurrentIndex(0)  # Switch to Home
            self.status_bar.showMessage(f"Scenario loaded: {name}")

    def _load_results(self):
        """Load and display HDF5 results file."""
        from PySide6.QtWidgets import QFileDialog as FD
        path, _ = FD.getOpenFileName(
            self, "Load HDF5 Results", OUTPUT_DIR,
            "HDF5 Files (*.h5 *.hdf5);;All Files (*)"
        )
        if not path:
            return
        if not _is_within_workspace(path):
            QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
            return
        try:
            from sim_platform.tools.replay.hdf5_logger import HDF5Logger
            log = HDF5Logger(path, "r")
            log.open()
            data = {}
            for key in log.keys():
                data[key] = log.read(key).tolist() if hasattr(log.read(key), 'tolist') else list(log.read(key))
            log.close()

            if "speed" in data and "time" in data:
                self._sim_data = data
                self.chart.clear()
                for i in range(0, len(data["time"]), max(1, len(data["time"]) // 5000)):
                    point = {k: data[k][i] for k in data if i < len(data[k])}
                    self.chart.add_data_point(point)
                speed_ref = data.get("speed_ref", [100.0])[0] if data.get("speed_ref") else 100.0
                self.result_table.set_results(data, speed_ref)
                self.tabs.setCurrentIndex(2)  # Switch to Chart
                self.log.append_log(f"Loaded results: {os.path.basename(path)}")
            else:
                QMessageBox.information(self, tr("dialog.no_data"), tr("dialog.hdf5_no_data"))
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.load_error"), tr("dialog.hdf5_error", str(e)))

    # ── View operations ───────────────────────────────────

    def _toggle_config_panel(self):
        self._dock.setVisible(not self._dock.isVisible())

    def _toggle_toolbar(self):
        tb = self.findChild(QToolBar, "main_toolbar")
        if tb:
            tb.setVisible(not tb.isVisible())

    def _reset_layout(self):
        """Reset window layout to defaults."""
        self._dock.setVisible(True)
        self.addDockWidget(Qt.DockWidgetArea.LeftDockWidgetArea, self._dock)
        tb = self.findChild(QToolBar, "main_toolbar")
        if tb:
            tb.setVisible(True)

    # ── Dialogs ───────────────────────────────────────────

    def _open_scan_dialog(self):
        dialog = ScanDialog(self)
        dialog.exec()

    def _show_onboarding(self):
        """Show the onboarding welcome dialog."""
        dialog = OnboardingDialog(self)
        dialog.exec()

    def _start_tour(self):
        """Start the guided tour."""
        if self._tour._current_tour:
            self._tour.cancel_tour()
        self._tour.start_tour("main")

    def _on_tour_completed(self, tour_name: str):
        """Handle tour completion."""
        self.status_bar.showMessage("导览完成 — 祝您使用愉快！", 5000)

    def _show_context_help(self):
        """Show context-sensitive help for current UI context."""
        from sim_platform.tools.gui.guided_tour import ContextHelpProvider
        from PySide6.QtWidgets import QMessageBox

        # Determine current context based on active tab
        idx = self.tabs.currentIndex()
        context_map = {
            0: "dashboard",
            1: "chart",
            2: "log",
            3: "results",
        }
        context_id = context_map.get(idx, "dashboard")
        lang = get_language()
        help_content = ContextHelpProvider.get_help(context_id, lang)

        if help_content:
            QMessageBox.information(
                self, help_content["title"],
                help_content.get("body", "")
            )
        else:
            QMessageBox.information(
                self, tr("help.context"),
                tr("help.no_context")
            )

    def _switch_language(self, lang: str):
        """Switch UI language and update all text."""
        set_language(lang)
        # Update checkmarks
        for code, action in self._lang_actions.items():
            action.setChecked(code == lang)
        # Update window title
        self.setWindowTitle(f"sim_platform v{_VERSION} — {tr('app.title')}")
        # Update menu bar text (rebuild is simplest)
        self._retranslate_menus()
        # Update dashboard
        if hasattr(self, 'dashboard'):
            self.dashboard._setup_ui()  # Rebuild dashboard

    def _retranslate_menus(self):
        """Update menu bar text for current language."""
        menubar = self.menuBar()
        menubar.clear()
        self._setup_menus()

    def _show_about(self):
        dialog = AboutDialog(self)
        dialog.exec()

    def _show_shortcuts(self):
        """Show keyboard shortcuts dialog."""
        shortcuts = """
<h3>Keyboard Shortcuts</h3>
<table>
<tr><td><b>Ctrl+1</b></td><td>Home / Dashboard</td></tr>
<tr><td><b>Ctrl+2</b></td><td>Chart</td></tr>
<tr><td><b>Ctrl+3</b></td><td>Log</td></tr>
<tr><td><b>Ctrl+4</b></td><td>Results</td></tr>
<tr><td><b>F5</b></td><td>Run simulation</td></tr>
<tr><td><b>F6</b></td><td>Pause/Resume</td></tr>
<tr><td><b>Shift+F5</b></td><td>Stop simulation</td></tr>
<tr><td><b>Ctrl+N</b></td><td>New config (reset)</td></tr>
<tr><td><b>Ctrl+O</b></td><td>Open config file</td></tr>
<tr><td><b>Ctrl+S</b></td><td>Save config</td></tr>
<tr><td><b>Ctrl+Shift+S</b></td><td>Save config as...</td></tr>
<tr><td><b>Ctrl+E</b></td><td>Export results (CSV)</td></tr>
<tr><td><b>Ctrl+D</b></td><td>Toggle config panel</td></tr>
<tr><td><b>Ctrl+T</b></td><td>Toggle toolbar</td></tr>
<tr><td><b>Ctrl+Shift+P</b></td><td>Parameter scanner</td></tr>
<tr><td><b>F1</b></td><td>Keyboard shortcuts</td></tr>
<tr><td><b>Ctrl+Q</b></td><td>Exit</td></tr>
</table>
"""
        QMessageBox.information(self, tr("shortcuts.title"), shortcuts)

    # ── Window events ─────────────────────────────────────

    def closeEvent(self, event):
        """Clean up worker on window close."""
        if self._worker and self._worker.isRunning():
            reply = QMessageBox.question(
                self, "Confirm Exit",
                "Simulation is still running. Stop and exit?",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                QMessageBox.StandardButton.No,
            )
            if reply == QMessageBox.StandardButton.No:
                event.ignore()
                return
            self._worker.stop()
            if not self._worker.wait(3000):
                self._worker.terminate()
                self._worker.wait(1000)
        self._worker = None
        self._save_settings()
        event.accept()


def run_app():
    """Launch the sim_platform GUI application."""
    import sys

    app = QApplication(sys.argv)
    app.setApplicationName("sim_platform")
    app.setOrganizationName("sim_platform")
    app.setApplicationVersion(_VERSION)
    app.setStyleSheet(get_stylesheet())
    window = MainWindow()
    window.show()

    # Show onboarding on first launch
    if OnboardingDialog.should_show():
        from PySide6.QtCore import QTimer
        QTimer.singleShot(300, window._show_onboarding)

    sys.exit(app.exec())


if __name__ == "__main__":
    run_app()
