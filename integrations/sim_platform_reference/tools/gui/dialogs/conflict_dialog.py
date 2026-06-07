"""Conflict resolution dialog for interactive parameter conflict handling.

Replaces the simple Yes/No QMessageBox with a comprehensive dialog that:
- Lists all detected conflicts with severity indicators
- Shows impact analysis per conflict (which subsystems are affected)
- Provides multiple resolution strategies per conflict
- Displays heatmap visualization of overall impact
- Supports batch resolution (apply to all similar conflicts)
"""

from __future__ import annotations

from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QCheckBox,
    QComboBox,
    QDialog,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QScrollArea,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)

from sim_platform.models.physics_constraints import ConstraintViolation
from sim_platform.tools.gui.conflict_resolver import (
    ConflictImpact,
    ConflictResolutionEngine,
    ConflictRule,
    ResolutionStrategy,
    generate_impact_heatmap,
)
from sim_platform.tools.gui.icons import get_pixmap


class ConflictRow(QWidget):
    """A single conflict row showing violation details and resolution options."""

    strategy_changed = Signal(str, str)  # parameter_name, strategy

    def __init__(self, violation: ConstraintViolation,
                 impact: ConflictImpact,
                 parent=None):
        super().__init__(parent)
        self._violation = violation
        self._impact = impact

        layout = QHBoxLayout(self)
        layout.setContentsMargins(8, 6, 8, 6)
        layout.setSpacing(10)

        # Severity icon
        icon_name = {
            "error": "error",
            "warning": "warning",
            "info": "info",
        }.get(violation.severity, "info")

        icon_color = {
            "error": "#FF453A",
            "warning": "#FFD60A",
            "info": "#64D2FF",
        }.get(violation.severity, "#B0B8C8")

        icon_label = QLabel()
        icon_label.setPixmap(get_pixmap(icon_name, color=icon_color, size=18))
        icon_label.setFixedSize(22, 22)
        layout.addWidget(icon_label)

        # Details
        details_layout = QVBoxLayout()
        details_layout.setSpacing(2)

        # Message
        msg = QLabel(violation.message)
        msg.setStyleSheet("color: #F5F5F7; font-size: 13px; font-weight: 500;")
        msg.setWordWrap(True)
        details_layout.addWidget(msg)

        # Fix suggestion
        if violation.fix_suggestion:
            fix = QLabel(f"→ {violation.fix_suggestion}")
            fix.setStyleSheet("color: rgba(245, 245, 247, 0.45); font-size: 11px;")
            fix.setWordWrap(True)
            details_layout.addWidget(fix)

        # Impact info
        impacts = []
        if impact.result_accuracy_impact > 0.3:
            impacts.append("精度")
        if impact.stability_impact > 0.5:
            impacts.append("稳定性")
        if impact.convergence_impact > 0.5:
            impacts.append("收敛性")
        if impacts:
            impact_label = QLabel(f"影响: {', '.join(impacts)}")
            impact_label.setStyleSheet("color: rgba(245, 245, 247, 0.35); font-size: 10px;")
            details_layout.addWidget(impact_label)

        layout.addLayout(details_layout, stretch=1)

        # Strategy selector
        self._strategy_combo = QComboBox()
        strategies = [
            (ResolutionStrategy.ASK_EACH_TIME, "每次询问"),
            (ResolutionStrategy.AUTO_FIX, "自动修复"),
            (ResolutionStrategy.MANUAL_OVERRIDE, "手动调整"),
            (ResolutionStrategy.IGNORE_THIS_RUN, "本次忽略"),
            (ResolutionStrategy.IGNORE_ALWAYS, "始终忽略"),
        ]
        for strategy, label in strategies:
            self._strategy_combo.addItem(label, strategy)

        # Select appropriate default
        if violation.severity == "error":
            idx = self._strategy_combo.findData(ResolutionStrategy.AUTO_FIX)
        else:
            idx = self._strategy_combo.findData(ResolutionStrategy.ASK_EACH_TIME)
        if idx >= 0:
            self._strategy_combo.setCurrentIndex(idx)

        self._strategy_combo.setFixedWidth(100)
        self._strategy_combo.currentIndexChanged.connect(self._on_strategy_changed)
        layout.addWidget(self._strategy_combo)

        self.setStyleSheet("""
            ConflictRow {
                background-color: rgba(255, 255, 255, 0.03);
                border-radius: 6px;
            }
            ConflictRow:hover {
                background-color: rgba(255, 255, 255, 0.05);
            }
        """)

    def _on_strategy_changed(self, index: int):
        strategy = self._strategy_combo.itemData(index)
        self.strategy_changed.emit(self._violation.parameter, strategy.value if strategy else "ask")

    @property
    def selected_strategy(self) -> ResolutionStrategy:
        return self._strategy_combo.currentData()


class ConflictDialog(QDialog):
    """Interactive dialog for resolving parameter conflicts.

    Features:
    - Lists all conflicts with severity, impact, and suggestions
    - Per-conflict resolution strategy selection
    - Impact heatmap visualization
    - Batch strategy application
    - Remember choice option for future conflicts
    """

    def __init__(self, config: dict, parent=None):
        super().__init__(parent)
        self._config = config
        self._engine = ConflictResolutionEngine()
        self._results: dict[str, ResolutionStrategy] = {}

        self.setWindowTitle("参数冲突解析器 — 多策略冲突处理")
        self.setMinimumSize(700, 520)
        self.setStyleSheet("""
            QDialog {
                background-color: #161A22;
                color: #F5F5F7;
            }
        """)

        self._setup_ui()
        self._load_conflicts()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(12)
        layout.setContentsMargins(20, 16, 20, 16)

        # Header
        header = QLabel("参数冲突解析")
        header.setStyleSheet("""
            font-size: 16px;
            font-weight: 700;
            color: #F5F5F7;
        """)
        layout.addWidget(header)

        subtitle = QLabel("检测到参数配置存在潜在问题。请为每个冲突选择处理策略。")
        subtitle.setStyleSheet("color: rgba(245, 245, 247, 0.55); font-size: 12px;")
        subtitle.setWordWrap(True)
        layout.addWidget(subtitle)

        # Scrollable conflict list
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setStyleSheet("""
            QScrollArea {
                border: 1px solid rgba(255, 255, 255, 0.06);
                border-radius: 8px;
                background-color: #1A1E26;
            }
        """)

        self._conflict_container = QWidget()
        self._conflict_layout = QVBoxLayout(self._conflict_container)
        self._conflict_layout.setSpacing(4)
        self._conflict_layout.setContentsMargins(4, 4, 4, 4)
        self._conflict_layout.addStretch()

        scroll.setWidget(self._conflict_container)
        layout.addWidget(scroll, stretch=1)

        # Heatmap (compact)
        self._heatmap = QTextEdit()
        self._heatmap.setReadOnly(True)
        self._heatmap.setMaximumHeight(140)
        self._heatmap.setStyleSheet("""
            QTextEdit {
                background-color: #1A1E26;
                border: 1px solid rgba(255, 255, 255, 0.06);
                border-radius: 6px;
                color: #B0B8C8;
                font-family: "JetBrains Mono", "Consolas", monospace;
                font-size: 11px;
                padding: 8px;
            }
        """)
        layout.addWidget(self._heatmap)

        # Batch controls
        batch_layout = QHBoxLayout()

        batch_label = QLabel("批量处理:")
        batch_label.setStyleSheet("color: rgba(245, 245, 247, 0.55); font-size: 12px;")
        batch_layout.addWidget(batch_label)

        batch_auto = QPushButton("全部自动修复")
        batch_auto.setStyleSheet("""
            QPushButton {
                background-color: rgba(10, 132, 255, 0.15);
                color: #0A84FF;
                border: 1px solid rgba(10, 132, 255, 0.3);
                border-radius: 6px;
                padding: 6px 14px;
                font-size: 12px;
            }
            QPushButton:hover {
                background-color: rgba(10, 132, 255, 0.25);
            }
        """)
        batch_auto.clicked.connect(self._batch_auto_fix)
        batch_layout.addWidget(batch_auto)

        batch_ignore = QPushButton("全部忽略")
        batch_ignore.setStyleSheet("""
            QPushButton {
                background-color: rgba(255, 255, 255, 0.05);
                color: #B0B8C8;
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 6px;
                padding: 6px 14px;
                font-size: 12px;
            }
            QPushButton:hover {
                background-color: rgba(255, 255, 255, 0.1);
            }
        """)
        batch_ignore.clicked.connect(self._batch_ignore)
        batch_layout.addWidget(batch_ignore)

        batch_layout.addStretch()
        layout.addLayout(batch_layout)

        # Remember
        self._remember_check = QCheckBox("记住我的选择，下次不再询问此类冲突")
        self._remember_check.setStyleSheet("""
            color: rgba(245, 245, 247, 0.55);
            font-size: 12px;
        """)
        layout.addWidget(self._remember_check)

        # Bottom buttons
        btn_layout = QHBoxLayout()
        btn_layout.addStretch()

        cancel_btn = QPushButton("取消")
        cancel_btn.setStyleSheet("""
            QPushButton {
                background-color: rgba(255, 255, 255, 0.05);
                color: #B0B8C8;
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 6px;
                padding: 8px 20px;
                font-size: 13px;
            }
            QPushButton:hover {
                background-color: rgba(255, 255, 255, 0.1);
            }
        """)
        cancel_btn.clicked.connect(self.reject)
        btn_layout.addWidget(cancel_btn)

        apply_btn = QPushButton("应用并继续")
        apply_btn.setStyleSheet("""
            QPushButton {
                background-color: #0A84FF;
                color: white;
                border: none;
                border-radius: 6px;
                padding: 8px 20px;
                font-size: 13px;
                font-weight: 600;
            }
            QPushButton:hover {
                background-color: #409CFF;
            }
        """)
        apply_btn.clicked.connect(self._on_apply)
        btn_layout.addWidget(apply_btn)

        layout.addLayout(btn_layout)

    def _load_conflicts(self):
        """Detect conflicts and populate the UI."""
        conflicts = self._engine._detector.detect(self._config)

        if not conflicts:
            label = QLabel("✓ 未检测到参数冲突，配置有效。")
            label.setStyleSheet("color: #30D158; font-size: 14px; padding: 20px;")
            self._conflict_layout.insertWidget(0, label)
            return

        for violation, impact in conflicts:
            row = ConflictRow(violation, impact, self)
            row.strategy_changed.connect(self._on_strategy_changed)
            self._conflict_layout.insertWidget(
                self._conflict_layout.count() - 1, row
            )

        # Update heatmap
        heatmap = generate_impact_heatmap(
            [c[0] for c in conflicts],
            [c[1] for c in conflicts],
        )
        self._heatmap.setPlainText(heatmap)

    def _on_strategy_changed(self, param: str, strategy: str):
        """Track strategy changes."""
        try:
            self._results[param] = ResolutionStrategy(strategy)
        except ValueError:
            pass

    def _batch_auto_fix(self):
        """Set all conflicts to auto-fix."""
        for i in range(self._conflict_layout.count()):
            widget = self._conflict_layout.itemAt(i).widget()
            if isinstance(widget, ConflictRow):
                idx = widget._strategy_combo.findData(ResolutionStrategy.AUTO_FIX)
                if idx >= 0:
                    widget._strategy_combo.setCurrentIndex(idx)

    def _batch_ignore(self):
        """Set all conflicts to ignore."""
        for i in range(self._conflict_layout.count()):
            widget = self._conflict_layout.itemAt(i).widget()
            if isinstance(widget, ConflictRow):
                idx = widget._strategy_combo.findData(ResolutionStrategy.IGNORE_THIS_RUN)
                if idx >= 0:
                    widget._strategy_combo.setCurrentIndex(idx)

    def _on_apply(self):
        """Apply resolution strategies to config."""
        resolutions = self._engine.resolve(self._config, auto_apply=False)

        for i in range(self._conflict_layout.count()):
            widget = self._conflict_layout.itemAt(i).widget()
            if isinstance(widget, ConflictRow) and i < len(resolutions):
                resolutions[i].strategy = widget.selected_strategy

        # Apply auto-fix resolutions
        modified_config = dict(self._config)
        for resolution in resolutions:
            if resolution.strategy == ResolutionStrategy.AUTO_FIX:
                modified_config = self._engine.apply_resolution(
                    modified_config, resolution
                )

        # Save rules if remember is checked
        if self._remember_check.isChecked():
            for resolution in resolutions:
                rule = ConflictRule(
                    rule_id=f"user.{resolution.violation.parameter}",
                    name=f"User rule for {resolution.violation.parameter}",
                    description="User-defined resolution rule from ConflictDialog",
                    parameter_pattern=resolution.violation.parameter,
                    default_strategy=resolution.strategy,
                    priority=20,
                )
                self._engine.add_rule(rule)

        self._modified_config = modified_config
        self._resolutions = resolutions
        self.accept()

    def get_modified_config(self) -> dict:
        """Get the modified configuration after resolution."""
        return getattr(self, '_modified_config', self._config)

    def get_resolutions(self) -> list:
        """Get all resolution decisions."""
        return getattr(self, '_resolutions', [])


def show_conflict_dialog(config: dict, parent=None) -> tuple[dict, bool]:
    """Convenience function to show the conflict dialog.

    Args:
        config: Simulation configuration dictionary.
        parent: Parent widget.

    Returns:
        Tuple of (modified_config, was_accepted).
    """
    dialog = ConflictDialog(config, parent)
    result = dialog.exec()

    if result == QDialog.DialogCode.Accepted:
        return dialog.get_modified_config(), True
    return config, False
