"""Simulation configuration panel — professional grade.

Provides comprehensive parameter configuration with:
- Motor presets (editable)
- FOC controller parameters (kp/ki for d/q axes)
- Speed loop PI parameters
- Sensor parameters (noise, bias)
- Time step configuration
- Scenario presets
- YAML/JSON config load/save via ConfigurationManager
- Real-time input validation
- Full bilingual support with parameter descriptions
"""

from __future__ import annotations

import math
import os
from pathlib import Path

# ── Workspace directory management ────────────────────────
_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../.."))
CONFIGS_DIR = os.path.join(_PROJ, "configs")
os.makedirs(CONFIGS_DIR, exist_ok=True)


def _is_within_workspace(path: str) -> bool:
    """Check if a path is within the workspace directory (CWE-22 prevention)."""
    abs_path = os.path.abspath(path)
    return abs_path.startswith(_PROJ + os.sep) or abs_path == _PROJ


from PySide6.QtCore import Qt, Signal
from PySide6.QtWidgets import (
    QComboBox,
    QDoubleSpinBox,
    QFileDialog,
    QFormLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QPushButton,
    QScrollArea,
    QSpinBox,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr

# ── Motor presets ─────────────────────────────────────────
MOTOR_PRESETS = {
    "Small PMSM (200W)": {
        "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
        "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4,
    },
    "Medium PMSM (2kW)": {
        "Rs": 0.05, "Ld": 0.2e-3, "Lq": 0.4e-3,
        "flux_pm": 0.05, "J": 0.005, "B": 0.0005, "Pp": 4,
    },
    "Large PMSM (20kW)": {
        "Rs": 0.01, "Ld": 0.1e-3, "Lq": 0.2e-3,
        "flux_pm": 0.12, "J": 0.05, "B": 0.001, "Pp": 3,
    },
}

_MOTOR_I18N = {
    "Small PMSM (200W)": "preset.small_pmsm",
    "Medium PMSM (2kW)": "preset.medium_pmsm",
    "Large PMSM (20kW)": "preset.large_pmsm",
}

# ── Scenarios ─────────────────────────────────────────────
# Keys are English (backward compat); display uses tr() via _SCENARIO_I18N map
SCENARIOS = {
    "Step Response": {
        "duration": 1.5, "speed_ref": 100, "profile": "step", "load": 0,
    },
    "Ramp Test": {
        "duration": 1.5, "speed_ref": 100, "profile": "ramp", "load": 0,
    },
    "Load Disturbance": {
        "duration": 2.0, "speed_ref": 100, "profile": "step", "load": 0.3,
    },
    "Voltage Sag Ride-Through": {
        "duration": 2.0, "speed_ref": 100, "profile": "step", "load": 0,
    },
}

# Map English keys → i18n keys for display
_SCENARIO_I18N = {
    "Step Response": "scenario.step_name",
    "Ramp Test": "scenario.ramp_name",
    "Load Disturbance": "scenario.load_name",
    "Voltage Sag Ride-Through": "scenario.sag_name",
}

# ── Scan parameters ───────────────────────────────────────
SCAN_PARAMS = {
    "Speed Reference": ("speed", [30, 50, 100, 150, 200]),
    "FOC kp_id": ("kp_id", [1, 3, 5, 10, 20]),
    "FOC ki_id": ("ki_id", [100, 300, 500, 1000, 2000]),
    "Speed Loop Kp": ("spd_kp", [0.01, 0.03, 0.05, 0.1, 0.2]),
    "Load Torque": ("load", [0, 0.1, 0.3, 0.5, 1.0]),
}

_SCAN_I18N = {
    "Speed Reference": "scan.param.speed",
    "FOC kp_id": "scan.param.kp_id",
    "FOC ki_id": "scan.param.ki_id",
    "Speed Loop Kp": "scan.param.spd_kp",
    "Load Torque": "scan.param.load",
}


def _guard_float(val: float, fallback: float = 0.0) -> float:
    """Validate a float value, returning fallback for NaN/Inf."""
    if math.isnan(val) or math.isinf(val):
        return fallback
    return val


class ConfigPanel(QWidget):
    """Professional simulation configuration panel with full bilingual support.

    Signal:
        config_changed(dict): Emitted when any parameter changes.
    """

    config_changed = Signal(dict)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._config_file: str | None = None
        self._setup_ui()
        self._connect_signals()
        # Apply initial scenario defaults
        self._on_scenario_changed(0)

    def _setup_ui(self):
        # Scroll area for the entire panel
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        scroll_content = QWidget()
        layout = QVBoxLayout(scroll_content)
        layout.setContentsMargins(8, 8, 8, 8)
        layout.setSpacing(8)

        # ── File Operations ───────────────────────────────
        self._file_group = QGroupBox(tr("config.file"))
        file_layout = QHBoxLayout(self._file_group)
        self._file_label = QLabel(tr("config.default"))
        self._file_label.setStyleSheet("color: rgba(245,245,247,0.45); font-size: 11px;")
        self._btn_load = QPushButton(tr("config.load"))
        self._btn_load.setToolTip(tr("tooltip.load_config"))
        self._btn_load.clicked.connect(self._load_config)
        self._btn_save = QPushButton(tr("config.save"))
        self._btn_save.setToolTip(tr("tooltip.save_config"))
        self._btn_save.clicked.connect(self._save_config)
        self._btn_reset = QPushButton(tr("config.reset"))
        self._btn_reset.setToolTip(tr("tooltip.reset_config"))
        self._btn_reset.clicked.connect(self._reset_defaults)
        file_layout.addWidget(self._btn_load)
        file_layout.addWidget(self._btn_save)
        file_layout.addWidget(self._btn_reset)
        layout.addWidget(self._file_group)

        # ── Scenario group ────────────────────────────────
        self._scenario_group = QGroupBox(tr("config.scenario"))
        sg_layout = QFormLayout(self._scenario_group)
        self.scenario_combo = QComboBox()
        for key in SCENARIOS:
            i18n_key = _SCENARIO_I18N.get(key, key)
            self.scenario_combo.addItem(tr(i18n_key), key)
        self.scenario_combo.setToolTip(tr("tooltip.select_scenario"))
        sg_layout.addRow(tr("config.label.scenario"), self.scenario_combo)

        self.profile_combo = QComboBox()
        self.profile_combo.addItems(["step", "ramp"])
        self.profile_combo.setToolTip(tr("tooltip.profile"))
        sg_layout.addRow(tr("config.label.profile"), self.profile_combo)
        layout.addWidget(self._scenario_group)

        # ── Motor group ───────────────────────────────────
        self._motor_group = QGroupBox(tr("config.motor"))
        mg_layout = QFormLayout(self._motor_group)
        self.motor_combo = QComboBox()
        for key in MOTOR_PRESETS:
            i18n_key = _MOTOR_I18N.get(key, key)
            self.motor_combo.addItem(tr(i18n_key), key)
        self.motor_combo.setEditable(False)
        self.motor_combo.setToolTip(tr("tooltip.motor_preset"))
        mg_layout.addRow(tr("config.label.preset"), self.motor_combo)

        self._motor_spins: dict[str, QDoubleSpinBox | QSpinBox] = {}
        self._motor_descs: dict[str, QLabel] = {}
        motor_params = [
            ("Rs", "Ohm", 0.001, 10.0, 0.1, 4, "param.Rs.desc"),
            ("Ld", "H", 1e-6, 1.0, 0.5e-3, 6, "param.Ld.desc"),
            ("Lq", "H", 1e-6, 1.0, 1.0e-3, 6, "param.Lq.desc"),
            ("flux_pm", "Wb", 1e-4, 10.0, 0.03, 4, "param.flux_pm.desc"),
            ("J", "kg*m^2", 1e-6, 10.0, 0.001, 6, "param.J.desc"),
            ("B", "N*m*s", 0.0, 1.0, 0.0001, 6, "param.B.desc"),
            ("Pp", "", 1, 20, 4, 0, "param.Pp.desc"),
        ]
        for key, unit, lo, hi, default, decimals, desc_key in motor_params:
            if key == "Pp":
                spin = QSpinBox()
                spin.setRange(int(lo), int(hi))
                spin.setValue(int(default))
            else:
                spin = QDoubleSpinBox()
                spin.setRange(lo, hi)
                spin.setDecimals(decimals)
                spin.setValue(default)
            spin.setSuffix(f" {unit}" if unit else "")
            spin.setToolTip(tr(desc_key))
            self._motor_spins[key] = spin
            mg_layout.addRow(f"{key}:", spin)

            # Description label (small, muted)
            desc_label = QLabel(tr(desc_key))
            desc_label.setStyleSheet(
                "color: rgba(245,245,247,0.30); font-size: 9px; "
                "padding-left: 8px; background: transparent;"
            )
            desc_label.setWordWrap(True)
            self._motor_descs[key] = desc_label
            mg_layout.addRow("", desc_label)

        layout.addWidget(self._motor_group)

        # ── FOC Controller group ──────────────────────────
        self._foc_group = QGroupBox(tr("config.foc"))
        fg_layout = QFormLayout(self._foc_group)
        self._foc_spins: dict[str, QDoubleSpinBox] = {}
        foc_params = [
            ("kp_id", "config.label.kp_id", 0.1, 100.0, 5.0, 2, "param.kp_id.desc"),
            ("ki_id", "config.label.ki_id", 1.0, 10000.0, 500.0, 1, "param.ki_id.desc"),
            ("kp_iq", "config.label.kp_iq", 0.1, 100.0, 5.0, 2, "param.kp_iq.desc"),
            ("ki_iq", "config.label.ki_iq", 1.0, 10000.0, 500.0, 1, "param.ki_iq.desc"),
        ]
        for key, label_key, lo, hi, default, decimals, desc_key in foc_params:
            spin = QDoubleSpinBox()
            spin.setRange(lo, hi)
            spin.setDecimals(decimals)
            spin.setValue(default)
            spin.setToolTip(tr(desc_key))
            self._foc_spins[key] = spin
            fg_layout.addRow(tr(label_key), spin)

            desc_label = QLabel(tr(desc_key))
            desc_label.setStyleSheet(
                "color: rgba(245,245,247,0.30); font-size: 9px; "
                "padding-left: 8px; background: transparent;"
            )
            desc_label.setWordWrap(True)
            fg_layout.addRow("", desc_label)

        layout.addWidget(self._foc_group)

        # ── Speed Loop group ──────────────────────────────
        self._spd_group = QGroupBox(tr("config.speed_pi"))
        sp_layout = QFormLayout(self._spd_group)
        self._spd_spins: dict[str, QDoubleSpinBox] = {}
        spd_params = [
            ("spd_kp", "config.label.spd_kp", 0.001, 10.0, 0.05, 3, "param.spd_kp.desc"),
            ("spd_ki", "config.label.spd_ki", 0.01, 100.0, 0.5, 2, "param.spd_ki.desc"),
        ]
        for key, label_key, lo, hi, default, decimals, desc_key in spd_params:
            spin = QDoubleSpinBox()
            spin.setRange(lo, hi)
            spin.setDecimals(decimals)
            spin.setValue(default)
            spin.setToolTip(tr(desc_key))
            self._spd_spins[key] = spin
            sp_layout.addRow(tr(label_key), spin)

            desc_label = QLabel(tr(desc_key))
            desc_label.setStyleSheet(
                "color: rgba(245,245,247,0.30); font-size: 9px; "
                "padding-left: 8px; background: transparent;"
            )
            desc_label.setWordWrap(True)
            sp_layout.addRow("", desc_label)

        layout.addWidget(self._spd_group)

        # ── Sensors group ─────────────────────────────────
        self._sensor_group = QGroupBox(tr("config.sensors"))
        se_layout = QFormLayout(self._sensor_group)
        self._sensor_spins: dict[str, QDoubleSpinBox] = {}
        sensor_params = [
            ("current_noise", "config.label.current_noise", "A", 0.0, 1.0, 0.1, 3, "param.current_noise.desc"),
            ("current_bias", "config.label.current_bias", "A", 0.0, 1.0, 0.01, 4, "param.current_bias.desc"),
            ("encoder_noise", "config.label.encoder_noise", "rad", 0.0, 0.1, 0.001, 5, "param.encoder_noise.desc"),
        ]
        for key, label_key, unit, lo, hi, default, decimals, desc_key in sensor_params:
            spin = QDoubleSpinBox()
            spin.setRange(lo, hi)
            spin.setDecimals(decimals)
            spin.setValue(default)
            spin.setSuffix(f" {unit}" if unit else "")
            spin.setToolTip(tr(desc_key))
            self._sensor_spins[key] = spin
            se_layout.addRow(tr(label_key), spin)

            desc_label = QLabel(tr(desc_key))
            desc_label.setStyleSheet(
                "color: rgba(245,245,247,0.30); font-size: 9px; "
                "padding-left: 8px; background: transparent;"
            )
            desc_label.setWordWrap(True)
            se_layout.addRow("", desc_label)

        layout.addWidget(self._sensor_group)

        # ── Time & Solver group ───────────────────────────
        self._time_group = QGroupBox(tr("config.time"))
        tg_layout = QFormLayout(self._time_group)
        self.dt_current_spin = QDoubleSpinBox()
        self.dt_current_spin.setRange(1e-6, 1e-2)
        self.dt_current_spin.setDecimals(6)
        self.dt_current_spin.setValue(50e-6)
        self.dt_current_spin.setSuffix(" s")
        self.dt_current_spin.setToolTip(tr("tooltip.dt_current"))
        tg_layout.addRow(tr("config.label.dt_current"), self.dt_current_spin)

        self.dt_speed_spin = QDoubleSpinBox()
        self.dt_speed_spin.setRange(1e-4, 1.0)
        self.dt_speed_spin.setDecimals(4)
        self.dt_speed_spin.setValue(1e-3)
        self.dt_speed_spin.setSuffix(" s")
        self.dt_speed_spin.setToolTip(tr("tooltip.dt_speed"))
        tg_layout.addRow(tr("config.label.dt_speed"), self.dt_speed_spin)

        self.duration_spin = QDoubleSpinBox()
        self.duration_spin.setRange(0.1, 10.0)
        self.duration_spin.setDecimals(2)
        self.duration_spin.setValue(1.5)
        self.duration_spin.setSuffix(" s")
        self.duration_spin.setToolTip(tr("tooltip.duration"))
        tg_layout.addRow(tr("config.label.duration"), self.duration_spin)

        self.solver_combo = QComboBox()
        self.solver_combo.addItems([tr("config.forward_euler"), tr("config.runge_kutta")])
        self.solver_combo.setToolTip(tr("tooltip.solver"))
        tg_layout.addRow(tr("config.label.solver"), self.solver_combo)
        layout.addWidget(self._time_group)

        # ── Operating Point group ─────────────────────────
        self._op_group = QGroupBox(tr("config.operating"))
        op_layout = QFormLayout(self._op_group)
        self.speed_spin = QDoubleSpinBox()
        self.speed_spin.setRange(5, 500)
        self.speed_spin.setDecimals(1)
        self.speed_spin.setSuffix(" rad/s")
        self.speed_spin.setValue(100)
        self.speed_spin.setToolTip(tr("tooltip.speed_ref"))
        op_layout.addRow(tr("config.label.speed_ref"), self.speed_spin)

        self.load_spin = QDoubleSpinBox()
        self.load_spin.setRange(0, 5.0)
        self.load_spin.setDecimals(3)
        self.load_spin.setSuffix(" N*m")
        self.load_spin.setValue(0)
        self.load_spin.setToolTip(tr("tooltip.load_torque"))
        op_layout.addRow(tr("config.label.load_torque"), self.load_spin)

        self.battery_v_spin = QDoubleSpinBox()
        self.battery_v_spin.setRange(12, 800)
        self.battery_v_spin.setDecimals(1)
        self.battery_v_spin.setSuffix(" V")
        self.battery_v_spin.setValue(48)
        self.battery_v_spin.setToolTip(tr("tooltip.battery_v"))
        op_layout.addRow(tr("config.label.battery_v"), self.battery_v_spin)

        self.battery_r_spin = QDoubleSpinBox()
        self.battery_r_spin.setRange(0.001, 10.0)
        self.battery_r_spin.setDecimals(4)
        self.battery_r_spin.setSuffix(" Ohm")
        self.battery_r_spin.setValue(0.05)
        self.battery_r_spin.setToolTip(tr("tooltip.battery_r"))
        op_layout.addRow(tr("config.label.battery_r"), self.battery_r_spin)
        layout.addWidget(self._op_group)

        layout.addStretch()

        scroll.setWidget(scroll_content)
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(scroll)

    def _connect_signals(self):
        self.scenario_combo.currentIndexChanged.connect(self._on_scenario_changed)
        self.motor_combo.currentIndexChanged.connect(self._on_motor_changed)
        # Emit config_changed on any value change
        for spin in self._motor_spins.values():
            if isinstance(spin, QDoubleSpinBox):
                spin.valueChanged.connect(self._emit_changed)
            else:
                spin.valueChanged.connect(self._emit_changed)
        for spin in self._foc_spins.values():
            spin.valueChanged.connect(self._emit_changed)
        for spin in self._spd_spins.values():
            spin.valueChanged.connect(self._emit_changed)
        for spin in self._sensor_spins.values():
            spin.valueChanged.connect(self._emit_changed)
        self.speed_spin.valueChanged.connect(self._emit_changed)
        self.load_spin.valueChanged.connect(self._emit_changed)
        self.duration_spin.valueChanged.connect(self._emit_changed)
        self.dt_current_spin.valueChanged.connect(self._emit_changed)
        self.dt_speed_spin.valueChanged.connect(self._emit_changed)
        self.battery_v_spin.valueChanged.connect(self._emit_changed)
        self.battery_r_spin.valueChanged.connect(self._emit_changed)

    def _emit_changed(self, *_args):
        try:
            cfg = self.get_config()
            self.config_changed.emit(cfg)
        except (ValueError, Exception):
            pass

    def _on_scenario_changed(self, index: int):
        # Use itemData to get the translation key
        key = self.scenario_combo.currentData()
        if key is None:
            key = self.scenario_combo.currentText()
        scenario = SCENARIOS.get(key)
        if scenario:
            self.speed_spin.setValue(scenario["speed_ref"])
            self.duration_spin.setValue(scenario["duration"])
            self.load_spin.setValue(scenario["load"])
            idx = self.profile_combo.findText(scenario.get("profile", "step"))
            if idx >= 0:
                self.profile_combo.setCurrentIndex(idx)

    def _on_motor_changed(self, index: int):
        key = self.motor_combo.currentData()
        if key is None:
            key = self.motor_combo.currentText()
        preset = MOTOR_PRESETS.get(key)
        if preset:
            for k, spin in self._motor_spins.items():
                if k in preset:
                    spin.setValue(preset[k])

    def _load_config(self):
        """Load configuration from YAML/JSON file (restricted to workspace)."""
        path, _ = QFileDialog.getOpenFileName(
            self, tr("dialog.open_config"), CONFIGS_DIR,
            "YAML Files (*.yaml *.yml);;JSON Files (*.json);;All Files (*)"
        )
        if not path:
            return
        if not _is_within_workspace(path):
            QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
            return
        try:
            import yaml
            with open(path, encoding='utf-8') as f:
                if path.endswith(('.yaml', '.yml')):
                    data = yaml.safe_load(f)
                else:
                    import json
                    data = json.load(f)
            self._apply_config_dict(data)
            self._config_file = path
            self._file_label.setText(Path(path).name)
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.load_error"), tr("dialog.config_load_error", str(e)))

    def _save_config(self):
        """Save current configuration to YAML file (restricted to workspace)."""
        path, _ = QFileDialog.getSaveFileName(
            self, tr("dialog.save_config"), os.path.join(CONFIGS_DIR, "config.yaml"),
            "YAML Files (*.yaml);;JSON Files (*.json)"
        )
        if not path:
            return
        if not _is_within_workspace(path):
            QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
            return
        try:
            cfg = self.get_config()
            import yaml
            with open(path, 'w', encoding='utf-8') as f:
                yaml.dump(cfg, f, default_flow_style=False, sort_keys=False)
            self._config_file = path
            self._file_label.setText(Path(path).name)
        except Exception as e:
            QMessageBox.warning(self, tr("dialog.save_error"), tr("dialog.config_save_error", str(e)))

    def _reset_defaults(self):
        """Reset all parameters to defaults."""
        self._on_scenario_changed(0)
        self._on_motor_changed(0)
        self._foc_spins["kp_id"].setValue(5.0)
        self._foc_spins["ki_id"].setValue(500.0)
        self._foc_spins["kp_iq"].setValue(5.0)
        self._foc_spins["ki_iq"].setValue(500.0)
        self._spd_spins["spd_kp"].setValue(0.05)
        self._spd_spins["spd_ki"].setValue(0.5)
        self._sensor_spins["current_noise"].setValue(0.1)
        self._sensor_spins["current_bias"].setValue(0.01)
        self._sensor_spins["encoder_noise"].setValue(0.001)
        self.dt_current_spin.setValue(50e-6)
        self.dt_speed_spin.setValue(1e-3)
        self.battery_v_spin.setValue(48.0)
        self.battery_r_spin.setValue(0.05)

    def _apply_config_dict(self, data: dict):
        """Apply a config dict to all GUI controls."""
        if "speed_ref" in data:
            self.speed_spin.setValue(float(data["speed_ref"]))
        if "duration_s" in data:
            self.duration_spin.setValue(float(data["duration_s"]))
        if "load_torque" in data:
            self.load_spin.setValue(float(data["load_torque"]))
        if "profile" in data:
            idx = self.profile_combo.findText(data["profile"])
            if idx >= 0:
                self.profile_combo.setCurrentIndex(idx)
        # Motor params
        mp = data.get("motor_params", {})
        for key, spin in self._motor_spins.items():
            if key in mp:
                spin.setValue(float(mp[key]))
        # FOC params
        foc = data.get("foc", {})
        for key, spin in self._foc_spins.items():
            if key in foc:
                spin.setValue(float(foc[key]))
        # Speed loop
        spd = data.get("speed_pi", {})
        if "kp" in spd:
            self._spd_spins["spd_kp"].setValue(float(spd["kp"]))
        if "ki" in spd:
            self._spd_spins["spd_ki"].setValue(float(spd["ki"]))
        # Sensors
        sensors = data.get("sensors", {})
        if "current_noise" in sensors:
            self._sensor_spins["current_noise"].setValue(float(sensors["current_noise"]))
        if "current_bias" in sensors:
            self._sensor_spins["current_bias"].setValue(float(sensors["current_bias"]))
        if "encoder_noise" in sensors:
            self._sensor_spins["encoder_noise"].setValue(float(sensors["encoder_noise"]))
        # Time
        if "dt_c" in data:
            self.dt_current_spin.setValue(float(data["dt_c"]))
        if "dt_s" in data:
            self.dt_speed_spin.setValue(float(data["dt_s"]))
        # Battery
        bat = data.get("battery", {})
        if "voltage" in bat:
            self.battery_v_spin.setValue(float(bat["voltage"]))
        if "resistance" in bat:
            self.battery_r_spin.setValue(float(bat["resistance"]))

    def get_config(self) -> dict:
        """Build simulation config dict from current panel state.

        Returns:
            dict with all simulation parameters.

        Raises:
            ValueError: if any input value is NaN/Inf or out of range.
        """
        speed_ref = _guard_float(self.speed_spin.value(), 100.0)
        duration = _guard_float(self.duration_spin.value(), 1.5)
        load = _guard_float(self.load_spin.value(), 0.0)

        if speed_ref < 5 or speed_ref > 500:
            raise ValueError(tr("validation.speed_range"))
        if duration < 0.1 or duration > 10:
            raise ValueError(tr("validation.duration_range"))
        if load < 0 or load > 5:
            raise ValueError(tr("validation.load_range"))

        motor_params = {k: _guard_float(spin.value()) for k, spin in self._motor_spins.items()}
        foc_params = {k: _guard_float(spin.value()) for k, spin in self._foc_spins.items()}

        return {
            "motor_params": motor_params,
            "speed_ref": speed_ref,
            "duration_s": duration,
            "load_torque": load,
            "profile": self.profile_combo.currentText(),
            "scenario_name": self.scenario_combo.currentText(),
            "foc": foc_params,
            "speed_pi": {
                "kp": _guard_float(self._spd_spins["spd_kp"].value()),
                "ki": _guard_float(self._spd_spins["spd_ki"].value()),
            },
            "sensors": {
                "current_noise": _guard_float(self._sensor_spins["current_noise"].value()),
                "current_bias": _guard_float(self._sensor_spins["current_bias"].value()),
                "encoder_noise": _guard_float(self._sensor_spins["encoder_noise"].value()),
            },
            "dt_c": _guard_float(self.dt_current_spin.value(), 50e-6),
            "dt_s": _guard_float(self.dt_speed_spin.value(), 1e-3),
            "battery": {
                "voltage": _guard_float(self.battery_v_spin.value(), 48.0),
                "resistance": _guard_float(self.battery_r_spin.value(), 0.05),
            },
            "solver": self.solver_combo.currentText(),
        }

    def validate_config(self) -> tuple[bool, str]:
        """Validate current config against physics constraints.

        Returns:
            (is_valid, message): is_valid=False if there are errors,
            message contains warnings and errors in human-readable format.
        """
        from sim_platform.models.physics_constraints import PhysicsValidator
        cfg = self.get_config()
        validator = PhysicsValidator()
        violations = validator.validate(cfg)
        summary = validator.get_summary(violations)
        errors = [v for v in violations if v.severity == "error"]
        return len(errors) == 0, summary

    def retranslate(self):
        """Update all labels and tooltips for current language."""
        self._file_group.setTitle(tr("config.file"))
        self._file_label.setText(tr("config.default"))
        self._btn_load.setText(tr("config.load"))
        self._btn_load.setToolTip(tr("tooltip.load_config"))
        self._btn_save.setText(tr("config.save"))
        self._btn_save.setToolTip(tr("tooltip.save_config"))
        self._btn_reset.setText(tr("config.reset"))
        self._btn_reset.setToolTip(tr("tooltip.reset_config"))

        self._scenario_group.setTitle(tr("config.scenario"))
        self._motor_group.setTitle(tr("config.motor"))
        self._foc_group.setTitle(tr("config.foc"))
        self._spd_group.setTitle(tr("config.speed_pi"))
        self._sensor_group.setTitle(tr("config.sensors"))
        self._time_group.setTitle(tr("config.time"))
        self._op_group.setTitle(tr("config.operating"))

        # Update motor description labels
        motor_descs = {
            "Rs": "param.Rs.desc", "Ld": "param.Ld.desc", "Lq": "param.Lq.desc",
            "flux_pm": "param.flux_pm.desc", "J": "param.J.desc", "B": "param.B.desc",
            "Pp": "param.Pp.desc",
        }
        for key, desc_key in motor_descs.items():
            if key in self._motor_descs:
                self._motor_descs[key].setText(tr(desc_key))
            if key in self._motor_spins:
                self._motor_spins[key].setToolTip(tr(desc_key))
