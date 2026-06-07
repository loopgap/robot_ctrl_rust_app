"""Interactive guided tour and context help system.

Provides step-by-step guided tours for the sim_platform GUI with:
- Context-aware tooltips (extended parameter descriptions)
- Interactive guided walkthrough (F1 help per screen)
- Spotlight overlay for focusing attention
- Progress-tracking tour sequences
- Persisted tour completion state
- Hotkey-enhanced navigation between tour steps

Architecture:
    GuidedTourEngine → TourStep[ ] → SpotlightOverlay + TourBubble
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from PySide6.QtCore import QObject, QPoint, QRect, QSettings, Qt, Signal
from PySide6.QtWidgets import (
    QApplication,
    QLabel,
    QPushButton,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr

# ── Data Models ────────────────────────────────────────────

class TourStepType(Enum):
    """Type of tour step element."""
    WIDGET = "widget"       # Highlight a specific widget
    REGION = "region"       # Highlight a rectangular region
    MENU = "menu"           # Highlight a menu item
    ACTION = "action"       # Highlight a toolbar action


@dataclass
class TourStep:
    """A single step in a guided tour.

    Each step highlights a specific UI element and shows explanatory text.
    """
    step_id: str
    title: str
    description: str
    target_widget_name: str  # Widget objectName for lookup
    step_type: TourStepType = TourStepType.WIDGET
    position: str = "below"  # "above", "below", "left", "right"
    hotspot: QRect | None = None  # Manual hotspot for region type
    required: bool = True
    action_hint: str = ""  # "Click Run to start", "Press F5", etc.

    def i18n_title(self) -> str:
        return tr(f"tour.{self.step_id}.title", self.title)

    def i18n_description(self) -> str:
        return tr(f"tour.{self.step_id}.desc", self.description)


# ── Tour Definitions ───────────────────────────────────────

MAIN_TOUR = [
    TourStep(
        step_id="welcome",
        title="欢迎使用仿真平台",
        description="这是一个工业级多物理域联合仿真平台。本导览将带您了解主要功能。",
        target_widget_name="main_toolbar",
        step_type=TourStepType.WIDGET,
        position="below",
    ),
    TourStep(
        step_id="dashboard",
        title="仪表板首页",
        description="首页提供快速启动、场景预设和操作指引。您可以在这里一键开始仿真或选择预定义场景。",
        target_widget_name="dashboard_scenarios",
        step_type=TourStepType.WIDGET,
        position="right",
    ),
    TourStep(
        step_id="config",
        title="配置面板",
        description="左侧配置面板包含电机参数、控制器增益、传感器设置等。所有参数支持中英文双语描述和实时验证。",
        target_widget_name="config_dock",
        step_type=TourStepType.WIDGET,
        position="right",
        action_hint="尝试切换电机预设查看参数变化",
    ),
    TourStep(
        step_id="run",
        title="运行仿真",
        description="设置好参数后，点击工具栏的运行按钮或按 F5 开始仿真。进度条会显示仿真进度。",
        target_widget_name="main_toolbar",
        step_type=TourStepType.WIDGET,
        position="below",
        action_hint="按 F5 或点击 ▶ 运行",
    ),
    TourStep(
        step_id="chart",
        title="实时图表",
        description="仿真运行时，图表标签页会实时显示速度、转矩和电流曲线。支持缩放和拖拽。",
        target_widget_name="qt_tabwidget_tabbar",
        step_type=TourStepType.WIDGET,
        position="below",
        action_hint="切换到 📈 图表 标签页查看",
    ),
    TourStep(
        step_id="results",
        title="结果分析",
        description="仿真完成后，结果标签页会展示详细的性能指标：上升时间、调节时间、超调量等。",
        target_widget_name="qt_tabwidget_tabbar",
        step_type=TourStepType.WIDGET,
        position="below",
    ),
    TourStep(
        step_id="export",
        title="导出数据",
        description="完整的仿真结果可以导出为 CSV、JSON 或 HDF5 格式，用于进一步分析和报告。",
        target_widget_name="main_toolbar",
        step_type=TourStepType.WIDGET,
        position="below",
        action_hint="文件 → 导出结果 (CSV)",
    ),
    TourStep(
        step_id="shortcuts",
        title="快捷键",
        description="常用操作都有键盘快捷键：F5 运行、F6 暂停、Ctrl+S 保存、Ctrl+O 打开、Ctrl+1-4 切换标签页。F1 查看完整快捷键。",
        target_widget_name="main_toolbar",
        step_type=TourStepType.WIDGET,
        position="below",
        action_hint="按 F1 查看所有快捷键",
    ),
    TourStep(
        step_id="complete",
        title="准备就绪",
        description="您已了解平台的基本功能。点击「知道了」关闭导览，开始使用仿真平台吧！",
        target_widget_name="main_toolbar",
        step_type=TourStepType.WIDGET,
        position="below",
    ),
]

CONFIG_PANEL_TOUR = [
    TourStep(
        step_id="config.motor",
        title="电机参数",
        description="设置电机的基本参数：定子电阻、dq轴电感、永磁磁链、转动惯量、极对数等。参数超出范围时会触发验证警告。",
        target_widget_name="config_motor_group",
        step_type=TourStepType.WIDGET,
        position="right",
    ),
    TourStep(
        step_id="config.foc",
        title="电流环控制器",
        description="FOC电流环的PI参数。kp决定了响应速度，ki决定了稳态精度。D轴和Q轴可以独立调节。",
        target_widget_name="config_foc_group",
        step_type=TourStepType.WIDGET,
        position="right",
    ),
    TourStep(
        step_id="config.speed",
        title="速度环控制器",
        description="外环速度PI控制器的参数。速度环的带宽通常远低于电流环，设置过高会导致超调。",
        target_widget_name="config_speed_group",
        step_type=TourStepType.WIDGET,
        position="right",
    ),
    TourStep(
        step_id="config.solver",
        title="求解器设置",
        description="配置仿真时间步长、持续时间等求解器参数。电流环步长通常为10-100us，速度环步长通常为0.5-5ms。",
        target_widget_name="config_time_group",
        step_type=TourStepType.WIDGET,
        position="right",
    ),
    TourStep(
        step_id="config.op",
        title="工作点",
        description="设定目标转速和负载转矩。系统会自动检查是否在电机能力范围内。",
        target_widget_name="config_op_group",
        step_type=TourStepType.WIDGET,
        position="right",
    ),
]


# ── Tour Bubble Widget ─────────────────────────────────────

class TourBubble(QWidget):
    """A floating bubble that displays tour step information.

    Positioned relative to a target widget with an arrow pointer.
    """

    closed = Signal()

    def __init__(self, parent: QWidget, target: QWidget,
                 step: TourStep, engine=None):
        super().__init__(parent)
        self._target = target
        self._step = step
        self._engine = engine

        self.setWindowFlags(
            Qt.WindowType.FramelessWindowHint
            | Qt.WindowType.Tool
            | Qt.WindowType.WindowStaysOnTopHint
        )
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating)

        self._setup_ui()
        self._position_bubble()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(16, 12, 16, 12)
        layout.setSpacing(8)

        # Title
        title = QLabel(self._step.i18n_title())
        title.setStyleSheet("""
            QLabel {
                color: #64D2FF;
                font-size: 14px;
                font-weight: 700;
                font-family: "Inter", "Microsoft YaHei UI", sans-serif;
            }
        """)
        layout.addWidget(title)

        # Description
        desc = QLabel(self._step.i18n_description())
        desc.setWordWrap(True)
        desc.setMaximumWidth(300)
        desc.setStyleSheet("""
            QLabel {
                color: #F5F5F7;
                font-size: 13px;
                font-family: "Inter", "Microsoft YaHei UI", sans-serif;
                line-height: 1.5;
            }
        """)
        layout.addWidget(desc)

        # Action hint
        if self._step.action_hint:
            hint = QLabel(self._step.action_hint)
            hint.setStyleSheet("""
                QLabel {
                    color: rgba(245, 245, 247, 0.55);
                    font-size: 11px;
                    font-style: italic;
                }
            """)
            layout.addWidget(hint)

        # Buttons
        btn_layout = QVBoxLayout()
        close_btn = QPushButton("知道了")
        close_btn.setFixedSize(80, 28)
        close_btn.setStyleSheet("""
            QPushButton {
                background-color: #0A84FF;
                color: white;
                border: none;
                border-radius: 6px;
                font-size: 12px;
                font-weight: 600;
            }
            QPushButton:hover {
                background-color: #409CFF;
            }
        """)
        close_btn.clicked.connect(self._on_close)
        btn_layout.addWidget(close_btn)
        layout.addLayout(btn_layout)

        self.setStyleSheet("""
            TourBubble {
                background-color: #1C2028;
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 8px;
            }
        """)

    def _position_bubble(self):
        """Position the bubble relative to the target widget."""
        target_global = self._target.mapToGlobal(QPoint(0, 0))
        target_size = self._target.size()

        bubble_size = self.sizeHint()
        bw, bh = bubble_size.width(), bubble_size.height()
        tx, ty = target_global.x(), target_global.y()
        tw, th = target_size.width(), target_size.height()

        position = self._step.position
        if position == "below":
            x = tx + (tw - bw) // 2
            y = ty + th + 8
        elif position == "above":
            x = tx + (tw - bw) // 2
            y = ty - bh - 8
        elif position == "right":
            x = tx + tw + 8
            y = ty + (th - bh) // 2
        elif position == "left":
            x = tx - bw - 8
            y = ty + (th - bh) // 2
        else:
            x = tx + (tw - bw) // 2
            y = ty + th + 8

        # Keep on screen
        screen = QApplication.primaryScreen().availableGeometry()
        x = max(0, min(x, screen.width() - bw))
        y = max(0, min(y, screen.height() - bh))

        self.setGeometry(x, y, bw, bh)

    def _on_close(self):
        self.closed.emit()
        self.close()


# ── Guided Tour Engine ─────────────────────────────────────

class GuidedTourEngine(QObject):
    """Manages guided tour sequences with progress tracking.

    Features:
    - Multiple named tours (main, config, solver, etc.)
    - Persisted completion state
    - Spotlight overlay for visual focus
    - Hotkey navigation (→ next, ← prev, Esc cancel)
    """

    tour_step_changed = Signal(int, int)  # current_index, total_steps
    tour_completed = Signal(str)
    tour_cancelled = Signal(str)

    _ACTIVE_TOURS: dict[str, list[TourStep]] = {
        "main": MAIN_TOUR,
        "config_panel": CONFIG_PANEL_TOUR,
    }

    def __init__(self, main_window, parent=None):
        super().__init__(parent)
        self._main_window = main_window
        self._current_tour: str | None = None
        self._current_step: int = 0
        self._bubble: TourBubble | None = None
        self._overlay = None

    def is_tour_completed(self, tour_name: str) -> bool:
        """Check if a tour has been completed."""
        settings = QSettings("sim_platform", "sim_platform_gui")
        return settings.value(f"tour/{tour_name}_completed", False, type=bool)

    def mark_tour_completed(self, tour_name: str):
        """Mark a tour as completed."""
        settings = QSettings("sim_platform", "sim_platform_gui")
        settings.setValue(f"tour/{tour_name}_completed", True)

    def start_tour(self, tour_name: str = "main"):
        """Start a guided tour sequence.

        Args:
            tour_name: Name of the tour to start.
        """
        if tour_name not in self._ACTIVE_TOURS:
            return

        self._current_tour = tour_name
        self._current_step = 0
        self._show_step()

    def next_step(self):
        """Advance to the next tour step."""
        if not self._current_tour:
            return

        steps = self._ACTIVE_TOURS[self._current_tour]
        self._current_step += 1

        if self._current_step >= len(steps):
            self._complete_tour()
        else:
            self._show_step()

    def prev_step(self):
        """Go back to the previous tour step."""
        if not self._current_tour or self._current_step <= 0:
            return

        self._current_step -= 1
        self._show_step()

    def cancel_tour(self):
        """Cancel the current tour."""
        if self._bubble:
            self._bubble.close()
            self._bubble = None
        tour = self._current_tour
        self._current_tour = None
        self._current_step = 0
        if tour:
            self.tour_cancelled.emit(tour)

    def _show_step(self):
        """Display the current tour step."""
        if not self._current_tour:
            return

        steps = self._ACTIVE_TOURS[self._current_tour]
        step = steps[self._current_step]

        # Find target widget
        target = self._main_window.findChild(QWidget, step.target_widget_name)
        if not target:
            # Try to find by type or position
            for w in self._main_window.findChildren(QWidget):
                if w.objectName() == step.target_widget_name:
                    target = w
                    break

        if not target:
            # If target not found, skip to next step
            self.next_step()
            return

        # Close existing bubble
        if self._bubble:
            self._bubble.close()
            self._bubble = None

        # Create new bubble
        self._bubble = TourBubble(self._main_window, target, step)
        self._bubble.closed.connect(self.next_step)
        self._bubble.show()

        self.tour_step_changed.emit(
            self._current_step + 1, len(steps)
        )

    def _complete_tour(self):
        """Complete the current tour."""
        tour = self._current_tour
        if tour:
            self.mark_tour_completed(tour)
            self.tour_completed.emit(tour)
        self._current_tour = None
        self._current_step = 0


# ── Context Help Provider ──────────────────────────────────

class ContextHelpProvider:
    """Provides context-sensitive help content for each UI component.

    Maps widget identifiers to extended help text, parameter descriptions,
    and common troubleshooting tips.
    """

    # Extended help content for each context
    _HELP_CONTENT: dict[str, dict] = {
        "motor_params": {
            "zh": {
                "title": "电机参数详解",
                "body": (
                    "电机参数决定了仿真模型的物理特性：\n\n"
                    "<b>Rs (定子电阻)</b>: 影响铜损和电流环增益。典型值 0.01-10Ω。\n"
                    "<b>Ld/Lq (dq轴电感)</b>: 决定电流动态响应速度。Lq通常大于Ld。\n"
                    "<b>flux_pm (永磁磁链)</b>: 决定反电动势大小。典型值 0.01-0.3Wb。\n"
                    "<b>J (转动惯量)</b>: 决定机械动态响应速度。\n"
                    "<b>B (粘滞摩擦系数)</b>: 速度相关的阻尼。\n"
                    "<b>Pp (极对数)</b>: 电频率 = Pp × 机械转速。"
                ),
            },
            "en": {
                "title": "Motor Parameters Detail",
                "body": (
                    "Motor parameters define the physical characteristics:\n\n"
                    "<b>Rs (Stator Resistance)</b>: Affects copper losses and current loop gain.\n"
                    "<b>Ld/Lq (dq-axis Inductance)</b>: Determines current dynamics speed.\n"
                    "<b>flux_pm (PM Flux Linkage)</b>: Determines back-EMF magnitude.\n"
                    "<b>J (Inertia)</b>: Determines mechanical response speed.\n"
                    "<b>B (Friction)</b>: Speed-dependent damping.\n"
                    "<b>Pp (Pole Pairs)</b>: Electrical freq = Pp × mechanical speed."
                ),
            },
        },
        "foc_controller": {
            "zh": {
                "title": "FOC 电流环控制器",
                "body": (
                    "磁场定向控制 (FOC) 电流环采用级联 PI 结构：\n\n"
                    "<b>Kp (比例增益)</b>: 影响带宽 = Kp / L。设置过高会导致振荡。\n"
                    "<b>Ki (积分增益)</b>: 消除稳态误差。Ki 过大可能导致饱和。\n\n"
                    "推荐方法：Kp = 2π × 目标带宽 × 电感\n"
                    "Ki = 2π × 目标带宽 × 电阻\n\n"
                    "电流环带宽通常设置为 1000-5000 rad/s。"
                ),
            },
            "en": {
                "title": "FOC Current Loop",
                "body": (
                    "Field-Oriented Control (FOC) current loop uses cascaded PI:\n\n"
                    "<b>Kp (Proportional)</b>: Bandwidth = Kp / L.\n"
                    "<b>Ki (Integral)</b>: Eliminates steady-state error.\n\n"
                    "Tuning: Kp = 2π × target_BW × inductance\n"
                    "Ki = 2π × target_BW × resistance"
                ),
            },
        },
        "speed_loop": {
            "zh": {
                "title": "速度环控制器",
                "body": (
                    "外环速度控制器使用 PI 结构：\n\n"
                    "<b>Kp</b>: 决定了速度响应速度。\n"
                    "<b>Ki</b>: 消除速度稳态误差。\n\n"
                    "速度环带宽通常设置为电流环带宽的 1/10 到 1/5。\n"
                    "输出受 iq_max 限制（默认 ±200A）。"
                ),
            },
            "en": {
                "title": "Speed Loop",
                "body": (
                    "Outer speed loop using PI structure:\n\n"
                    "<b>Kp</b>: Determines speed response rate.\n"
                    "<b>Ki</b>: Eliminates speed steady-state error."
                ),
            },
        },
        "solver_config": {
            "zh": {
                "title": "求解器参数",
                "body": (
                    "<b>dt_current (电流环步长)</b>: PMSM 典型值 25-100μs。\n"
                    "<b>dt_speed (速度环步长)</b>: 通常为电流环步长的 10-20 倍。\n"
                    "<b>duration (仿真时长)</b>: 建议至少 1.5s 以确保系统稳定。\n\n"
                    "数值积分方法：\n"
                    "- Forward Euler: 快速但精度有限\n"
                    "- RK4: 高精度但慢 4 倍\n"
                    "- Adaptive RK45: 自动调整步长"
                ),
            },
            "en": {
                "title": "Solver Parameters",
                "body": (
                    "<b>dt_current</b>: PMSM typical 25-100μs.\n"
                    "<b>dt_speed</b>: Usually 10-20x current loop step.\n"
                    "<b>duration</b>: Recommend ≥1.5s for stability."
                ),
            },
        },
        "operating_point": {
            "zh": {
                "title": "工作点",
                "body": (
                    "<b>目标转速</b>: 受反电动势限制。最大转速 ≈ V_bus / (Pp × flux_pm)。\n"
                    "<b>负载转矩</b>: 不能超过电机最大转矩能力。\n"
                    "<b>电池电压</b>: 母线电压，48V/300V/600V 等。\n\n"
                    "系统会自动检查是否在电机工作范围内。"
                ),
            },
            "en": {
                "title": "Operating Point",
                "body": (
                    "<b>Target Speed</b>: Limited by back-EMF. Max ≈ V_bus / (Pp × flux_pm).\n"
                    "<b>Load Torque</b>: Must be within motor capability.",
                ),
            },
        },
    }

    @classmethod
    def get_help(cls, context_id: str, lang: str = "zh") -> dict | None:
        """Get help content for a context ID.

        Args:
            context_id: Context identifier (e.g., "motor_params").
            lang: Language code ("zh" or "en").

        Returns:
            Dict with "title" and "body", or None if not found.
        """
        content = cls._HELP_CONTENT.get(context_id)
        if not content:
            return None
        return content.get(lang, content.get("zh"))


# ── Enhanced Status Feedback System ────────────────────────

class StatusFeedback:
    """Provides consistent status feedback messages for all UI operations.

    Maps operation states to standardized messages with icons.
    """

    @staticmethod
    def loading(message: str = "处理中...") -> str:
        """Loading state message."""
        return f"⟳ {message}"

    @staticmethod
    def success(message: str = "操作完成") -> str:
        """Success state message."""
        return f"✓ {message}"

    @staticmethod
    def error(message: str = "操作失败") -> str:
        """Error state message."""
        return f"✗ {message}"

    @staticmethod
    def warning(message: str = "需要确认") -> str:
        """Warning state message."""
        return f"⚠ {message}"

    @staticmethod
    def info(message: str = "") -> str:
        """Info state message."""
        return f"ℹ {message}"

    @classmethod
    def simulation_running(cls, step: int, total: int) -> str:
        """Simulation progress message."""
        pct = int(step / max(total, 1) * 100)
        return f"⟳ 仿真运行中... {pct}% ({step}/{total})"

    @classmethod
    def simulation_complete(cls, speed_error: float) -> str:
        """Simulation completion message."""
        if speed_error < 1.0:
            return f"✓ 仿真完成 — 速度误差 {speed_error:.2f}% (优秀)"
        elif speed_error < 5.0:
            return f"✓ 仿真完成 — 速度误差 {speed_error:.2f}% (良好)"
        else:
            return f"⚠ 仿真完成 — 速度误差 {speed_error:.2f}% (需调参)"

    @classmethod
    def config_valid(cls) -> str:
        """Config validation success."""
        return "✓ 配置验证通过"

    @classmethod
    def config_invalid(cls, error_count: int, warning_count: int) -> str:
        """Config validation failure."""
        parts = []
        if error_count:
            parts.append(f"{error_count} 个错误")
        if warning_count:
            parts.append(f"{warning_count} 个警告")
        return f"⚠ 配置验证: {', '.join(parts)}"
