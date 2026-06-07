"""Unified dashboard widget — the central hub of the application.

Provides a single-entry-point experience with:
- Quick action cards for common tasks
- Scenario presets as clickable cards
- Getting started guide
- Recent activity
- System status overview
- Workspace info
- Full bilingual support (Chinese/English) via i18n
"""

from __future__ import annotations

import os

from PySide6.QtCore import Qt, Signal
from PySide6.QtWidgets import (
    QFrame,
    QGridLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QScrollArea,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr


class ActionCard(QFrame):
    """A clickable card for quick actions."""

    clicked = Signal()

    def __init__(self, icon: str, title: str, subtitle: str, parent=None):
        super().__init__(parent)
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setProperty("class", "stat-card")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 12, 16, 12)
        layout.setSpacing(4)

        self._icon_label = QLabel(icon)
        self._icon_label.setStyleSheet("font-size: 28px; background: transparent;")
        self._icon_label.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self._title_label = QLabel(title)
        self._title_label.setStyleSheet(
            "font-size: 13px; font-weight: 600; color: #F5F5F7; background: transparent;"
        )
        self._title_label.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self._sub_label = QLabel(subtitle)
        self._sub_label.setStyleSheet(
            "font-size: 10px; color: rgba(245,245,247,0.45); background: transparent;"
        )
        self._sub_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._sub_label.setWordWrap(True)

        layout.addWidget(self._icon_label)
        layout.addWidget(self._title_label)
        layout.addWidget(self._sub_label)

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit()
        super().mousePressEvent(event)


class ScenarioCard(QFrame):
    """A clickable scenario preset card."""

    clicked = Signal(str)

    def __init__(self, name_key: str, desc_key: str, params: dict, parent=None):
        super().__init__(parent)
        self._name_key = name_key
        self._desc_key = desc_key
        self.setCursor(Qt.CursorShape.PointingHandCursor)
        self.setProperty("class", "stat-card")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 10, 12, 10)
        layout.setSpacing(3)

        self._title = QLabel(tr(name_key))
        self._title.setStyleSheet(
            "font-size: 12px; font-weight: 600; color: #64D2FF; background: transparent;"
        )

        self._desc = QLabel(tr(desc_key))
        self._desc.setStyleSheet(
            "font-size: 10px; color: rgba(245,245,247,0.55); background: transparent;"
        )
        self._desc.setWordWrap(True)

        # Parameter chips
        chips_layout = QHBoxLayout()
        chips_layout.setSpacing(4)
        for key in ["speed", "duration", "load"]:
            val = params.get(key, "")
            if val != "":
                chip = QLabel(f"{key}: {val}")
                chip.setStyleSheet(
                    "font-size: 9px; color: rgba(245,245,247,0.35); "
                    "background: rgba(255,255,255,0.05); "
                    "padding: 2px 6px; border-radius: 4px;"
                )
                chips_layout.addWidget(chip)
        chips_layout.addStretch()

        layout.addWidget(self._title)
        layout.addWidget(self._desc)
        layout.addLayout(chips_layout)

    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            self.clicked.emit(self._name_key)
        super().mousePressEvent(event)

    def retranslate(self):
        """Update text for current language."""
        self._title.setText(tr(self._name_key))
        self._desc.setText(tr(self._desc_key))


class DashboardWidget(QWidget):
    """Unified dashboard — the central hub of the application.

    Signals:
        action_triggered(str): Emitted when a quick action is clicked.
        scenario_selected(str): Emitted when a scenario card is clicked.
    """

    action_triggered = Signal(str)
    scenario_selected = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._scenario_cards: list[ScenarioCard] = []
        self._setup_ui()

    def _setup_ui(self):
        # Clear existing layout
        if self.layout():
            while self.layout().count():
                item = self.layout().takeAt(0)
                if item.widget():
                    item.widget().deleteLater()

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        scroll.setFrameShape(QFrame.Shape.NoFrame)

        content = QWidget()
        layout = QVBoxLayout(content)
        layout.setContentsMargins(24, 20, 24, 20)
        layout.setSpacing(20)

        # ── Welcome banner ────────────────────────────────
        banner = QFrame()
        banner.setProperty("class", "stat-card")
        banner_layout = QHBoxLayout(banner)
        banner_layout.setContentsMargins(20, 16, 20, 16)

        banner_text = QVBoxLayout()
        welcome = QLabel(tr("dashboard.welcome"))
        welcome.setStyleSheet(
            "font-size: 24px; font-weight: 700; color: #64D2FF; background: transparent;"
        )
        subtitle = QLabel(tr("dashboard.subtitle"))
        subtitle.setStyleSheet(
            "font-size: 13px; color: rgba(245,245,247,0.55); background: transparent;"
        )
        banner_text.addWidget(welcome)
        banner_text.addWidget(subtitle)

        banner_layout.addLayout(banner_text)
        banner_layout.addStretch()

        # Quick start button
        quick_btn = QPushButton(tr("dashboard.quick_start"))
        quick_btn.setProperty("class", "primary")
        quick_btn.setMinimumHeight(36)
        quick_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        quick_btn.clicked.connect(lambda: self.action_triggered.emit("quick_start"))
        banner_layout.addWidget(quick_btn)

        layout.addWidget(banner)

        # ── Getting Started Guide ─────────────────────────
        gs_group = QGroupBox(tr("dashboard.getting_started"))
        gs_layout = QVBoxLayout(gs_group)
        gs_layout.setSpacing(6)

        for step_key in ["dashboard.step1", "dashboard.step2", "dashboard.step3", "dashboard.step4"]:
            step_label = QLabel(tr(step_key))
            step_label.setStyleSheet(
                "font-size: 12px; color: rgba(245,245,247,0.65); "
                "padding: 4px 0; background: transparent;"
            )
            step_label.setWordWrap(True)
            gs_layout.addWidget(step_label)

        layout.addWidget(gs_group)

        # ── Quick Actions ─────────────────────────────────
        actions_label = QLabel(tr("dashboard.quick_actions"))
        actions_label.setStyleSheet(
            "font-size: 14px; font-weight: 600; color: #F5F5F7; "
            "letter-spacing: 0.5px; margin-bottom: 4px;"
        )
        layout.addWidget(actions_label)

        actions_grid = QGridLayout()
        actions_grid.setSpacing(12)

        actions = [
            ("▶", "dashboard.new_sim", "dashboard.new_sim.desc", "new_sim"),
            ("📂", "dashboard.open_config", "dashboard.open_config.desc", "open_config"),
            ("📊", "dashboard.load_results", "dashboard.load_results.desc", "load_results"),
            ("🔍", "dashboard.scan", "dashboard.scan.desc", "scan"),
        ]
        for i, (icon, title_key, subtitle_key, action_id) in enumerate(actions):
            card = ActionCard(icon, tr(title_key), tr(subtitle_key))
            card.clicked.connect(lambda aid=action_id: self.action_triggered.emit(aid))
            actions_grid.addWidget(card, i // 2, i % 2)

        layout.addLayout(actions_grid)

        # ── Scenario Presets ──────────────────────────────
        scenarios_label = QLabel(tr("dashboard.scenarios"))
        scenarios_label.setStyleSheet(
            "font-size: 14px; font-weight: 600; color: #F5F5F7; "
            "letter-spacing: 0.5px; margin-bottom: 4px;"
        )
        layout.addWidget(scenarios_label)

        scenarios_grid = QGridLayout()
        scenarios_grid.setSpacing(12)

        scenarios = [
            ("scenario.step", "scenario.step.desc",
             {"speed": "100 rad/s", "duration": "1.5s", "load": "0"}),
            ("scenario.ramp", "scenario.ramp.desc",
             {"speed": "100 rad/s", "duration": "1.5s", "load": "0"}),
            ("scenario.load", "scenario.load.desc",
             {"speed": "100 rad/s", "duration": "2.0s", "load": "0.3 N*m"}),
            ("scenario.sag", "scenario.sag.desc",
             {"speed": "100 rad/s", "duration": "2.0s", "load": "0"}),
        ]
        for i, (name_key, desc_key, params) in enumerate(scenarios):
            card = ScenarioCard(name_key, desc_key, params)
            card.clicked.connect(self.scenario_selected)
            self._scenario_cards.append(card)
            scenarios_grid.addWidget(card, i // 2, i % 2)

        layout.addLayout(scenarios_grid)

        # ── System Info ───────────────────────────────────
        info_group = QGroupBox(tr("dashboard.workspace"))
        info_layout = QVBoxLayout(info_group)

        proj_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../.."))
        info_items = [
            (tr("workspace.dir"), proj_dir),
            (tr("workspace.configs"), os.path.join(proj_dir, "configs")),
            (tr("workspace.output"), os.path.join(proj_dir, "output")),
            (tr("workspace.logs"), os.path.join(proj_dir, "logs")),
        ]
        for label, path in info_items:
            row = QHBoxLayout()
            lbl = QLabel(f"<b>{label}:</b>")
            lbl.setStyleSheet("color: rgba(245,245,247,0.55); font-size: 11px; background: transparent;")
            lbl.setFixedWidth(80)
            val = QLabel(path)
            val.setStyleSheet(
                "color: rgba(245,245,247,0.35); font-size: 10px; "
                "font-family: 'JetBrains Mono', monospace; background: transparent;"
            )
            val.setTextInteractionFlags(Qt.TextInteractionFlag.TextSelectableByMouse)
            row.addWidget(lbl)
            row.addWidget(val)
            info_layout.addLayout(row)

        layout.addWidget(info_group)
        layout.addStretch()

        scroll.setWidget(content)
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(scroll)

    def retranslate(self):
        """Rebuild UI with current language."""
        self._scenario_cards.clear()
        self._setup_ui()
