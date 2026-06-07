"""Result table widget for displaying simulation metrics.

QTableWidget showing comprehensive simulation performance metrics:
- Speed tracking (final, error, rise time, settling time, overshoot)
- Torque (final, peak)
- Currents (final id/iq, max)
- Timing (duration, data points)

Full bilingual support via i18n.
"""

from __future__ import annotations

import math

from PySide6.QtCore import Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QHBoxLayout,
    QHeaderView,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr


class ResultTable(QWidget):
    """Table displaying comprehensive simulation result metrics with re-run button."""

    # Signal emitted when user clicks "Re-run" button
    rerun_requested = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)

        # Create table
        self._table = QTableWidget(0, 3)
        self._table.setHorizontalHeaderLabels([
            tr("result.metric"), tr("result.value"), tr("result.unit")
        ])
        self._table.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch
        )
        self._table.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self._table.setSelectionBehavior(
            QTableWidget.SelectionBehavior.SelectRows
        )
        self._table.verticalHeader().setVisible(False)

        # Re-run button
        self._rerun_btn = QPushButton(tr("result.rerun"))
        self._rerun_btn.setObjectName("rerun_button")
        self._rerun_btn.setToolTip(tr("result.rerun.tooltip"))
        self._rerun_btn.clicked.connect(self.rerun_requested.emit)
        self._rerun_btn.setVisible(False)  # Hidden until results are shown

        # Layout
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self._table)

        btn_layout = QHBoxLayout()
        btn_layout.addStretch()
        btn_layout.addWidget(self._rerun_btn)
        layout.addLayout(btn_layout)

    def _compute_rise_time(self, time_data, speed_data, target):
        """Compute 10%-90% rise time."""
        if not time_data or not speed_data or target <= 0:
            return None
        ten_pct = target * 0.1
        ninety_pct = target * 0.9
        t_10 = t_90 = None
        for i, (t, s) in enumerate(zip(time_data, speed_data)):
            if t_10 is None and s >= ten_pct:
                t_10 = t
            if s >= ninety_pct:
                t_90 = t
                break
        if t_10 is not None and t_90 is not None:
            return t_90 - t_10
        return None

    def _compute_settling_time(self, time_data, speed_data, target, tolerance=0.02):
        """Compute settling time (last time outside tolerance band)."""
        if not time_data or not speed_data or target <= 0:
            return None
        band = target * tolerance
        last_outside = None
        for i, (t, s) in enumerate(zip(time_data, speed_data)):
            if abs(s - target) > band:
                last_outside = t
        return last_outside

    def _compute_overshoot(self, speed_data, target):
        """Compute overshoot percentage."""
        if not speed_data or target <= 0:
            return None
        peak = max(speed_data)
        if peak > target:
            return (peak - target) / target * 100
        return 0.0

    def set_results(self, data: dict, speed_ref: float):
        """Populate the table with comprehensive simulation results."""
        if not data.get("time"):
            return

        final_speed = data["speed"][-1] if data["speed"] else 0
        final_torque = data["torque"][-1] if data["torque"] else 0
        peak_torque = max(abs(t) for t in data["torque"]) if data["torque"] else 0
        duration = data["time"][-1] if data["time"] else 0
        error_pct = abs(final_speed - speed_ref) / max(speed_ref, 1) * 100

        speed_rpm = final_speed * 60 / (2 * math.pi)
        ref_rpm = speed_ref * 60 / (2 * math.pi)

        # Performance metrics
        rise_time = self._compute_rise_time(data["time"], data["speed"], speed_ref)
        settling_time = self._compute_settling_time(data["time"], data["speed"], speed_ref)
        overshoot = self._compute_overshoot(data["speed"], speed_ref)

        # Current metrics
        final_id = data["id"][-1] if data.get("id") else 0
        final_iq = data["iq"][-1] if data.get("iq") else 0
        max_ia = max(abs(a) for a in data["ia"]) if data.get("ia") else 0

        # Color coding (Apple semantic colors)
        green = "#30D158"
        yellow = "#FF9F0A"
        red = "#FF453A"

        rows = [
            (tr("result.speed_section"), "", "", None),
            (tr("result.target_speed"), f"{speed_ref:.1f}", "rad/s", None),
            (tr("result.target_speed"), f"{ref_rpm:.0f}", "rpm", None),
            (tr("result.final_speed"), f"{final_speed:.1f}", "rad/s", None),
            (tr("result.final_speed"), f"{speed_rpm:.0f}", "rpm", None),
            (tr("result.speed_error"), f"{error_pct:.2f}", "%",
             green if error_pct < 1 else yellow if error_pct < 5 else red),
            (tr("result.rise_time"), f"{rise_time:.4f}" if rise_time is not None else "N/A", "s", None),
            (tr("result.settling_time"), f"{settling_time:.4f}" if settling_time is not None else "N/A", "s", None),
            (tr("result.overshoot"), f"{overshoot:.2f}" if overshoot is not None else "N/A", "%",
             green if overshoot is not None and overshoot < 5 else yellow if overshoot is not None and overshoot < 15 else red),
            (tr("result.torque_section"), "", "", None),
            (tr("result.final_torque"), f"{final_torque:.4f}", "N*m", None),
            (tr("result.peak_torque"), f"{peak_torque:.4f}", "N*m", None),
            (tr("result.current_section"), "", "", None),
            (tr("result.final_id"), f"{final_id:.4f}", "A", None),
            (tr("result.final_iq"), f"{final_iq:.4f}", "A", None),
            (tr("result.peak_current"), f"{max_ia:.4f}", "A", None),
            (tr("result.timing_section"), "", "", None),
            (tr("result.duration"), f"{duration:.3f}", "s", None),
            (tr("result.data_points"), f"{len(data['time'])}", "", None),
        ]

        self._table.setRowCount(len(rows))
        for row_idx, (metric, value, unit, color) in enumerate(rows):
            metric_item = QTableWidgetItem(metric)
            if metric.startswith("──"):
                metric_item.setForeground(QColor("rgba(245,245,247,0.35)"))
                font = metric_item.font()
                font.setBold(True)
                metric_item.setFont(font)
            self._table.setItem(row_idx, 0, metric_item)
            val_item = QTableWidgetItem(value)
            if color:
                val_item.setForeground(QColor(color))
            self._table.setItem(row_idx, 1, val_item)
            self._table.setItem(row_idx, 2, QTableWidgetItem(unit))

        # Show re-run button after results are displayed
        self._rerun_btn.setVisible(True)

    def clear_results(self):
        """Clear the table."""
        self._table.setRowCount(0)
        self._rerun_btn.setVisible(False)

    def retranslate(self):
        """Update header labels for current language."""
        self._table.setHorizontalHeaderLabels([
            tr("result.metric"), tr("result.value"), tr("result.unit")
        ])
        self._rerun_btn.setText(tr("result.rerun"))
        self._rerun_btn.setToolTip(tr("result.rerun.tooltip"))
