"""Onboarding dialog — comprehensive first-time welcome guide.

Shows a step-by-step introduction to the application with:
- Welcome message
- 4 guided steps with visual cards
- Don't-show-again option
- Full bilingual support (Chinese/English)

Design: Modern dark theme with Apple-style cards.
"""

from __future__ import annotations

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QCheckBox,
    QDialog,
    QFrame,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QScrollArea,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr


class StepCard(QFrame):
    """A card displaying an onboarding step."""

    def __init__(self, icon: str, title_key: str, desc_key: str, parent=None):
        super().__init__(parent)
        self.setProperty("class", "stat-card")
        self._title_key = title_key
        self._desc_key = desc_key
        self._icon = icon

        layout = QHBoxLayout(self)
        layout.setContentsMargins(16, 12, 16, 12)
        layout.setSpacing(12)

        # Icon
        icon_label = QLabel(icon)
        icon_label.setStyleSheet("font-size: 24px; background: transparent;")
        icon_label.setFixedWidth(36)
        icon_label.setAlignment(Qt.AlignmentFlag.AlignCenter)

        # Text
        text_layout = QVBoxLayout()
        text_layout.setSpacing(4)

        self._title_label = QLabel(tr(title_key))
        self._title_label.setStyleSheet(
            "font-size: 13px; font-weight: 600; color: #64D2FF; background: transparent;"
        )

        self._desc_label = QLabel(tr(desc_key))
        self._desc_label.setStyleSheet(
            "font-size: 11px; color: rgba(245,245,247,0.65); background: transparent;"
        )
        self._desc_label.setWordWrap(True)

        text_layout.addWidget(self._title_label)
        text_layout.addWidget(self._desc_label)

        layout.addWidget(icon_label)
        layout.addLayout(text_layout, 1)

    def retranslate(self):
        """Update text for current language."""
        self._title_label.setText(tr(self._title_key))
        self._desc_label.setText(tr(self._desc_key))


class OnboardingDialog(QDialog):
    """Welcome/onboarding dialog with step-by-step guide."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(tr("onboarding.title"))
        self.setMinimumWidth(560)
        self.setMaximumWidth(680)
        self.setMinimumHeight(520)
        self._steps: list[StepCard] = []
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(16)
        layout.setContentsMargins(24, 20, 24, 20)

        # ── Header ────────────────────────────────────────
        title = QLabel(tr("onboarding.title"))
        title.setStyleSheet(
            "font-size: 22px; font-weight: 700; color: #64D2FF; background: transparent;"
        )
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)

        welcome = QLabel(tr("onboarding.welcome"))
        welcome.setStyleSheet(
            "font-size: 13px; color: rgba(245,245,247,0.65); background: transparent;"
        )
        welcome.setAlignment(Qt.AlignmentFlag.AlignCenter)
        welcome.setWordWrap(True)

        layout.addWidget(title)
        layout.addWidget(welcome)

        # ── Separator ─────────────────────────────────────
        sep = QFrame()
        sep.setFrameShape(QFrame.Shape.HLine)
        sep.setStyleSheet("background: rgba(255,255,255,0.06);")
        layout.addWidget(sep)

        # ── Steps (scrollable) ────────────────────────────
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("QScrollArea { border: none; background: transparent; }")

        steps_widget = QWidget()
        steps_layout = QVBoxLayout(steps_widget)
        steps_layout.setSpacing(8)
        steps_layout.setContentsMargins(0, 0, 0, 0)

        step_data = [
            ("⚙️", "onboarding.step1.title", "onboarding.step1.desc"),
            ("▶️", "onboarding.step2.title", "onboarding.step2.desc"),
            ("📊", "onboarding.step3.title", "onboarding.step3.desc"),
            ("🔬", "onboarding.step4.title", "onboarding.step4.desc"),
        ]
        for icon, title_key, desc_key in step_data:
            card = StepCard(icon, title_key, desc_key)
            self._steps.append(card)
            steps_layout.addWidget(card)

        scroll.setWidget(steps_widget)
        layout.addWidget(scroll, 1)

        # ── Don't show again ──────────────────────────────
        self._dont_show = QCheckBox(tr("onboarding.dont_show"))
        self._dont_show.setStyleSheet(
            "color: rgba(245,245,247,0.45); font-size: 11px; background: transparent;"
        )
        layout.addWidget(self._dont_show)

        # ── Buttons ───────────────────────────────────────
        btn_layout = QHBoxLayout()
        btn_layout.addStretch()

        got_it_btn = QPushButton(tr("onboarding.got_it"))
        got_it_btn.setProperty("class", "primary")
        got_it_btn.setMinimumHeight(36)
        got_it_btn.setMinimumWidth(200)
        got_it_btn.setCursor(Qt.CursorShape.PointingHandCursor)
        got_it_btn.clicked.connect(self._on_got_it)
        btn_layout.addWidget(got_it_btn)

        btn_layout.addStretch()
        layout.addLayout(btn_layout)

    def _on_got_it(self):
        """Handle 'Got it' button click."""
        if self._dont_show.isChecked():
            try:
                from PySide6.QtCore import QSettings
                settings = QSettings("sim_platform", "sim_platform_gui")
                settings.setValue("onboarding_dismissed", True)
            except Exception:
                pass
        self.accept()

    def retranslate(self):
        """Update all text for current language."""
        self.setWindowTitle(tr("onboarding.title"))
        for step in self._steps:
            step.retranslate()
        self._dont_show.setText(tr("onboarding.dont_show"))

    @staticmethod
    def should_show() -> bool:
        """Check if onboarding should be shown (first time)."""
        try:
            from PySide6.QtCore import QSettings
            settings = QSettings("sim_platform", "sim_platform_gui")
            return not settings.value("onboarding_dismissed", False)
        except Exception:
            return True
