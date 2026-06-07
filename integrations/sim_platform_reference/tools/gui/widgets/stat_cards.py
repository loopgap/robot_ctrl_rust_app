"""Real-time statistic cards.

Four glassmorphism-inspired cards showing Speed, Torque, FPS, and Progress.
Updated via signal during simulation.

Design: M3 tonal surface with Apple-style semantic color indicators.
Each card has a colored accent bar at the top for quick visual scanning.
"""

from __future__ import annotations

from PySide6.QtCore import Qt, Slot
from PySide6.QtWidgets import (
    QFrame,
    QHBoxLayout,
    QLabel,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr


class StatCard(QFrame):
    """A glassmorphism-inspired statistic card with accent bar.

    Structure:
        ┌─ accent bar (2px, colored) ─────────────┐
        │  LABEL (uppercase, muted)                │
        │  VALUE (large, bold, colored)            │
        │  UNIT  (tiny, very muted)                │
        └──────────────────────────────────────────┘
    """

    def __init__(self, label_key: str, unit_key: str = "", parent=None):
        super().__init__(parent)
        self._label_key = label_key
        self._unit_key = unit_key
        self.setProperty("class", "stat-card")
        self.setFrameShape(QFrame.Shape.StyledPanel)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 8, 12, 8)
        layout.setSpacing(2)

        # Label (uppercase, muted)
        self._label = QLabel(tr(label_key))
        self._label.setProperty("class", "stat-label")
        self._label.setAlignment(Qt.AlignmentFlag.AlignCenter)

        # Value (large, bold)
        self._value = QLabel("--")
        self._value.setProperty("class", "stat-value")
        self._value.setAlignment(Qt.AlignmentFlag.AlignCenter)

        # Unit (tiny, very muted)
        self._unit = QLabel(unit_key)
        self._unit.setProperty("class", "stat-unit")
        self._unit.setAlignment(Qt.AlignmentFlag.AlignCenter)

        layout.addWidget(self._label)
        layout.addWidget(self._value)
        layout.addWidget(self._unit)

        # Default accent color
        self._accent_color = "#0A84FF"

    def set_accent(self, color: str):
        """Set the accent bar color via top border."""
        self._accent_color = color
        self.setStyleSheet(
            f"QFrame[class='stat-card'] {{ "
            f"border-top: 2px solid {color}; }}"
        )

    @Slot(str)
    def update_value(self, value: str, color: str = ""):
        """Update the displayed value with optional color indicator."""
        self._value.setText(value)
        if color:
            self._value.setStyleSheet(
                f"color: {color}; font-size: 20px; font-weight: 700; "
                f"letter-spacing: -0.5px; background: transparent;"
            )

    def retranslate(self):
        """Update label text for current language."""
        self._label.setText(tr(self._label_key))


class StatCardsRow(QWidget):
    """Row of four stat cards: Speed, Torque, FPS, Progress."""

    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(8)

        self.speed_card = StatCard("stat.speed", "rad/s")
        self.torque_card = StatCard("stat.torque", "N*m")
        self.fps_card = StatCard("stat.throughput", "steps/s")
        self.progress_card = StatCard("stat.progress", "%")

        # Set accent bar colors
        self.speed_card.set_accent("#64D2FF")    # Teal
        self.torque_card.set_accent("#FF9F0A")   # Orange
        self.fps_card.set_accent("#BF5AF2")      # Purple
        self.progress_card.set_accent("#30D158")  # Green

        layout.addWidget(self.speed_card)
        layout.addWidget(self.torque_card)
        layout.addWidget(self.fps_card)
        layout.addWidget(self.progress_card)

    @Slot(str)
    def update_speed(self, value: str, ref: float = 0.0):
        """Update speed with color: green if near ref, teal otherwise."""
        try:
            speed = float(value)
            if ref > 0 and abs(speed - ref) / ref < 0.05:
                color = "#30D158"   # Apple green: within 5% of reference
            else:
                color = "#64D2FF"   # Apple teal: normal
        except (ValueError, ZeroDivisionError):
            color = "#F5F5F7"       # Default
        self.speed_card.update_value(value, color)

    @Slot(str)
    def update_torque(self, value: str):
        """Update torque with color: red if high, orange if elevated."""
        try:
            tq = abs(float(value))
            if tq > 1.5:
                color = "#FF453A"   # Apple red: very high
            elif tq > 0.8:
                color = "#FF9F0A"   # Apple orange: elevated
            else:
                color = "#F5F5F7"   # Default
        except ValueError:
            color = "#F5F5F7"
        self.torque_card.update_value(value, color)

    @Slot(str)
    def update_fps(self, value: str):
        """Update throughput with color: purple if high, orange if low."""
        try:
            fps = float(value)
            if fps > 100000:
                color = "#BF5AF2"   # Apple purple: excellent
            elif fps > 50000:
                color = "#FF9F0A"   # Apple orange: moderate
            else:
                color = "#FF453A"   # Apple red: slow
        except ValueError:
            color = "#F5F5F7"
        self.fps_card.update_value(value, color)

    @Slot(str)
    def update_progress(self, value: str):
        """Update progress with color: green at 100%, accent otherwise."""
        try:
            pct = int(value.replace("%", ""))
            if pct >= 100:
                color = "#30D158"   # Apple green: complete
            else:
                color = "#0A84FF"   # Apple blue: in progress
        except ValueError:
            color = "#F5F5F7"
        self.progress_card.update_value(value, color)

    def retranslate(self):
        """Update all card labels for current language."""
        self.speed_card.retranslate()
        self.torque_card.retranslate()
        self.fps_card.retranslate()
        self.progress_card.retranslate()
