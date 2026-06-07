"""Real-time chart widget using QtCharts.

Displays speed and torque curves that update during simulation.
Supports zoom and pan via mouse wheel and drag.

Design: Clean dark chart with Apple-style subtle grid and
high-contrast data series. Empty state with guidance hint.
"""

from __future__ import annotations

from PySide6.QtCharts import (
    QChart,
    QChartView,
    QLineSeries,
    QValueAxis,
)
from PySide6.QtCore import Qt, Slot
from PySide6.QtGui import QColor, QPainter, QPen
from PySide6.QtWidgets import QLabel, QStackedWidget, QVBoxLayout, QWidget

from sim_platform.tools.gui.i18n import tr


class ChartWidget(QWidget):
    """QChartView displaying real-time speed and torque curves.

    Shows an empty state message when no data is available,
    and switches to the chart when simulation data arrives.
    """

    def __init__(self, parent=None):
        super().__init__(parent)

        # ── Stacked layout: empty state ↔ chart ──────────
        self._stack = QStackedWidget()
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.addWidget(self._stack)

        # ── Empty state widget ────────────────────────────
        empty_widget = QWidget()
        empty_layout = QVBoxLayout(empty_widget)
        empty_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self._empty_icon = QLabel("📈")
        self._empty_icon.setStyleSheet("font-size: 48px; background: transparent;")
        self._empty_icon.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self._empty_label = QLabel(tr("chart.empty"))
        self._empty_label.setStyleSheet(
            "font-size: 16px; font-weight: 600; color: rgba(245,245,247,0.35); "
            "background: transparent;"
        )
        self._empty_label.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self._hint_label = QLabel(tr("chart.hint.zoom"))
        self._hint_label.setStyleSheet(
            "font-size: 11px; color: rgba(245,245,247,0.20); background: transparent;"
        )
        self._hint_label.setAlignment(Qt.AlignmentFlag.AlignCenter)

        empty_layout.addStretch()
        empty_layout.addWidget(self._empty_icon)
        empty_layout.addWidget(self._empty_label)
        empty_layout.addSpacing(8)
        empty_layout.addWidget(self._hint_label)
        empty_layout.addStretch()

        self._stack.addWidget(empty_widget)  # index 0

        # ── Chart widget ──────────────────────────────────
        self._chart_view = QChartView()
        self._chart_view.setRenderHint(QPainter.RenderHint.Antialiasing)

        # Series (high-contrast dark theme palette)
        self._speed_series = QLineSeries()
        self._speed_series.setName(tr("chart.series.speed"))
        speed_pen = QPen(QColor("#64D2FF"), 2.0)  # Teal
        self._speed_series.setPen(speed_pen)

        self._ref_series = QLineSeries()
        self._ref_series.setName(tr("chart.series.ref"))
        ref_pen = QPen(QColor("#30D158"), 1.5, Qt.PenStyle.DashLine)  # Green
        self._ref_series.setPen(ref_pen)

        self._torque_series = QLineSeries()
        self._torque_series.setName(tr("chart.series.torque"))
        torque_pen = QPen(QColor("#FF9F0A"), 2.0)  # Orange
        self._torque_series.setPen(torque_pen)

        # Chart
        self._chart = QChart()
        self._chart.addSeries(self._speed_series)
        self._chart.addSeries(self._ref_series)
        self._chart.addSeries(self._torque_series)
        self._chart.setTitle(tr("chart.title"))
        self._chart.setAnimationOptions(QChart.AnimationOption.NoAnimation)

        # Dark theme chart styling
        self._chart.setBackgroundBrush(QColor("#0F1117"))
        self._chart.setBackgroundPen(QPen(QColor(0, 0, 0, 0)))  # No border
        self._chart.setTitleBrush(QColor("#F5F5F7"))
        self._chart.legend().setLabelColor("rgba(245, 245, 247, 0.55)")
        self._chart.legend().setAlignment(Qt.AlignmentFlag.AlignBottom)

        # Axes (subtle grid, clear labels)
        axis_label_color = QColor(245, 245, 247, 115)  # ~45% white
        axis_title_color = QColor(245, 245, 247, 140)  # ~55% white
        grid_color = QColor(255, 255, 255, 15)         # ~6% white

        self._axis_x = QValueAxis()
        self._axis_x.setTitleText(tr("chart.axis.time"))
        self._axis_x.setRange(0, 1.5)
        self._axis_x.setLabelsColor(axis_label_color)
        self._axis_x.setTitleBrush(axis_title_color)
        self._axis_x.setGridLineColor(grid_color)
        self._axis_x.setMinorGridLineColor(QColor(0, 0, 0, 0))

        self._axis_y_speed = QValueAxis()
        self._axis_y_speed.setTitleText(tr("chart.axis.speed"))
        self._axis_y_speed.setRange(0, 150)
        self._axis_y_speed.setLabelsColor(QColor("#64D2FF"))  # Teal to match series
        self._axis_y_speed.setTitleBrush(QColor("#64D2FF"))
        self._axis_y_speed.setGridLineColor(grid_color)

        self._axis_y_torque = QValueAxis()
        self._axis_y_torque.setTitleText(tr("chart.axis.torque"))
        self._axis_y_torque.setRange(-0.5, 2.0)
        self._axis_y_torque.setLabelsColor(QColor("#FF9F0A"))  # Orange to match series
        self._axis_y_torque.setTitleBrush(QColor("#FF9F0A"))
        self._axis_y_torque.setGridLineColor(grid_color)

        self._chart.addAxis(self._axis_x, Qt.AlignmentFlag.AlignBottom)
        self._chart.addAxis(self._axis_y_speed, Qt.AlignmentFlag.AlignLeft)
        self._chart.addAxis(self._axis_y_torque, Qt.AlignmentFlag.AlignRight)

        self._speed_series.attachAxis(self._axis_x)
        self._speed_series.attachAxis(self._axis_y_speed)
        self._ref_series.attachAxis(self._axis_x)
        self._ref_series.attachAxis(self._axis_y_speed)
        self._torque_series.attachAxis(self._axis_x)
        self._torque_series.attachAxis(self._axis_y_torque)

        self._chart_view.setChart(self._chart)

        # Enable zoom/pan
        self._chart_view.setDragMode(QChartView.DragMode.ScrollHandDrag)
        self._chart.legend().setVisible(True)

        self._stack.addWidget(self._chart_view)  # index 1

        # Show empty state initially
        self._stack.setCurrentIndex(0)

        # Data tracking for axis auto-range
        self._max_speed = 150.0
        self._max_torque = 2.0
        self._min_torque = -0.5
        self._max_time = 1.5
        self._point_count = 0
        self._MAX_POINTS = 50000  # Prevent memory exhaustion

    @Slot(dict)
    def add_data_point(self, point: dict):
        """Add a new data point to the chart.

        Args:
            point: dict with keys 'time', 'speed_ref', 'speed', 'torque'.
        """
        # Switch to chart view on first data point
        if self._point_count == 0:
            self._stack.setCurrentIndex(1)

        self._point_count += 1
        # Skip points after limit to prevent memory exhaustion (CWE-400)
        if self._point_count > self._MAX_POINTS:
            return

        t = point["time"]
        speed = point["speed"]
        ref = point["speed_ref"]
        torque = point["torque"]

        self._speed_series.append(t, speed)
        self._ref_series.append(t, ref)
        self._torque_series.append(t, torque)

        # Auto-range axes
        changed = False
        if t > self._max_time * 0.9:
            self._max_time = t * 1.3
            changed = True
        if speed > self._max_speed * 0.8:
            self._max_speed = max(speed * 1.3, ref * 1.3)
            changed = True
        if torque > self._max_torque * 0.8:
            self._max_torque = torque * 1.5
            changed = True
        if torque < self._min_torque * 0.8:
            self._min_torque = torque * 1.5
            changed = True

        if changed:
            self._axis_x.setRange(0, self._max_time)
            self._axis_y_speed.setRange(0, self._max_speed)
            self._axis_y_torque.setRange(self._min_torque, self._max_torque)

    def clear(self):
        """Clear all series data and show empty state."""
        self._speed_series.clear()
        self._ref_series.clear()
        self._torque_series.clear()
        self._point_count = 0
        self._max_speed = 150.0
        self._max_torque = 2.0
        self._min_torque = -0.5
        self._max_time = 1.5
        self._axis_x.setRange(0, self._max_time)
        self._axis_y_speed.setRange(0, self._max_speed)
        self._axis_y_torque.setRange(self._min_torque, self._max_torque)
        # Show empty state
        self._stack.setCurrentIndex(0)

    def wheelEvent(self, event):
        """Zoom with mouse wheel."""
        if event.angleDelta().y() > 0:
            self._chart.zoomIn()
        else:
            self._chart.zoomOut()
        super().wheelEvent(event)

    def retranslate(self):
        """Update all text for current language."""
        self._speed_series.setName(tr("chart.series.speed"))
        self._ref_series.setName(tr("chart.series.ref"))
        self._torque_series.setName(tr("chart.series.torque"))
        self._chart.setTitle(tr("chart.title"))
        self._axis_x.setTitleText(tr("chart.axis.time"))
        self._axis_y_speed.setTitleText(tr("chart.axis.speed"))
        self._axis_y_torque.setTitleText(tr("chart.axis.torque"))
        self._empty_label.setText(tr("chart.empty"))
        self._hint_label.setText(tr("chart.hint.zoom"))
