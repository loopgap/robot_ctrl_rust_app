"""Animation and transition system for sim_platform GUI.

Professional-grade animation framework providing:
- Page/screen transitions (slide, fade, crossfade)
- State transitions (running/paused/stopped/error)
- Progress animations (pulse, shimmer, progress fill)
- Status indicator animations (blink, pulse, rotate)
- Widget enter/leave animations
- Responsive vsync-aware timing

Uses Qt's internal animation framework (QPropertyAnimation, QVariantAnimation)
for zero-cost integration with the render loop.

Design principles:
- Subtle, not distracting — no bounce, no overshoot
- 200-400ms durations (Apple HIG motion guidelines)
- Ease-in-out for state transitions, ease-out for appear
- GPU-accelerated where possible (opacity, geometry)
"""

from __future__ import annotations

import math
from collections.abc import Callable

from PySide6.QtCore import (
    Property,
    QEasingCurve,
    QObject,
    QParallelAnimationGroup,
    QPoint,
    QPropertyAnimation,
    Qt,
    QTimer,
    QVariantAnimation,
    Signal,
)
from PySide6.QtGui import QColor, QPainter
from PySide6.QtWidgets import (
    QGraphicsOpacityEffect,
    QLabel,
    QProgressBar,
    QWidget,
)

# ── Duration Constants (Apple HIG motion) ─────────────────

class Duration:
    """Animation duration presets (milliseconds)."""
    INSTANT = 100       # Immediate feedback
    FAST = 200          # Quick transitions
    DEFAULT = 300       # Standard UI transitions
    SMOOTH = 400        # Smoother state changes
    SLOW = 600          # Emphasis transitions
    GLACIAL = 1000      # Guided tour emphasis


# ── Easing Presets ─────────────────────────────────────────

class Easing:
    """Pre-configured easing curves."""
    EASE_OUT = QEasingCurve(QEasingCurve.Type.OutCubic)
    EASE_IN = QEasingCurve(QEasingCurve.Type.InCubic)
    EASE_IN_OUT = QEasingCurve(QEasingCurve.Type.InOutCubic)
    EASE_OUT_EXPO = QEasingCurve(QEasingCurve.Type.OutExpo)
    LINEAR = QEasingCurve(QEasingCurve.Type.Linear)


# ── Animation Engine ───────────────────────────────────────

class AnimationEngine(QObject):
    """Central animation manager with batched transitions.

    Handles:
    - Opacity-based fade transitions
    - Geometry-based slide transitions
    - Combined crossfade transitions
    - State indicator animations
    - Progress fill animations
    """

    animation_completed = Signal(str)
    animation_started = Signal(str)

    def __init__(self, parent: QObject | None = None):
        super().__init__(parent)
        self._running: dict[str, QObject] = {}

    def fade_in(self, widget: QWidget, duration: int = Duration.DEFAULT,
                on_finished: Callable | None = None):
        """Fade a widget from transparent to opaque."""
        self._ensure_opacity_effect(widget)
        effect = widget.graphicsEffect()

        anim = QPropertyAnimation(effect, b"opacity", self)
        anim.setDuration(duration)
        anim.setStartValue(0.0)
        anim.setEndValue(1.0)
        anim.setEasingCurve(Easing.EASE_OUT)

        widget.setVisible(True)
        if on_finished:
            anim.finished.connect(on_finished)
        anim.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    def fade_out(self, widget: QWidget, duration: int = Duration.DEFAULT,
                 on_finished: Callable | None = None):
        """Fade a widget from opaque to transparent, then hide."""
        self._ensure_opacity_effect(widget)
        effect = widget.graphicsEffect()

        anim = QPropertyAnimation(effect, b"opacity", self)
        anim.setDuration(duration)
        anim.setStartValue(1.0)
        anim.setEndValue(0.0)
        anim.setEasingCurve(Easing.EASE_IN)

        def _on_done():
            widget.setVisible(False)
            if on_finished:
                on_finished()

        anim.finished.connect(_on_done)
        anim.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    def crossfade(self, old_widget: QWidget, new_widget: QWidget,
                  duration: int = Duration.SMOOTH):
        """Simultaneously fade out old widget and fade in new widget."""
        group = QParallelAnimationGroup(self)

        self._ensure_opacity_effect(old_widget)
        self._ensure_opacity_effect(new_widget)

        fade_out = QPropertyAnimation(old_widget.graphicsEffect(), b"opacity")
        fade_out.setDuration(duration)
        fade_out.setStartValue(1.0)
        fade_out.setEndValue(0.0)
        fade_out.setEasingCurve(Easing.EASE_IN)

        fade_in = QPropertyAnimation(new_widget.graphicsEffect(), b"opacity")
        fade_in.setDuration(duration)
        fade_in.setStartValue(0.0)
        fade_in.setEndValue(1.0)
        fade_in.setEasingCurve(Easing.EASE_OUT)

        new_widget.setVisible(True)

        def _on_done():
            old_widget.setVisible(False)

        group.addAnimation(fade_out)
        group.addAnimation(fade_in)
        group.finished.connect(_on_done)
        group.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    def slide_in(self, widget: QWidget, direction: str = "right",
                 duration: int = Duration.DEFAULT,
                 on_finished: Callable | None = None):
        """Slide a widget in from a direction with simultaneous fade.

        Args:
            widget: Target widget.
            direction: 'left', 'right', 'top', 'bottom'.
            duration: Animation duration.
            on_finished: Callback on completion.
        """
        self._ensure_opacity_effect(widget)
        effect = widget.graphicsEffect()

        width = widget.width() or 200
        height = widget.height() or 100

        offsets = {
            "right": QPoint(width, 0),
            "left": QPoint(-width, 0),
            "top": QPoint(0, -height),
            "bottom": QPoint(0, height),
        }
        offset = offsets.get(direction, QPoint(width, 0))
        original_pos = widget.pos()

        # Start position
        widget.move(original_pos + offset)

        pos_anim = QPropertyAnimation(widget, b"pos", self)
        pos_anim.setDuration(duration)
        pos_anim.setStartValue(original_pos + offset)
        pos_anim.setEndValue(original_pos)
        pos_anim.setEasingCurve(Easing.EASE_OUT)

        opacity_anim = QPropertyAnimation(effect, b"opacity")
        opacity_anim.setDuration(duration)
        opacity_anim.setStartValue(0.0)
        opacity_anim.setEndValue(1.0)
        opacity_anim.setEasingCurve(Easing.EASE_OUT)

        group = QParallelAnimationGroup(self)
        group.addAnimation(pos_anim)
        group.addAnimation(opacity_anim)

        widget.setVisible(True)
        if on_finished:
            group.finished.connect(on_finished)
        group.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    def pulse(self, widget: QWidget, color_from: QColor = None,
              color_to: QColor = None, duration: int = Duration.FAST,
              cycles: int = 3):
        """Pulse a widget's background color (for status alerts).

        Args:
            widget: Target widget.
            color_from: Starting color (default: transparent).
            color_to: Target color (default: accent blue surface).
            cycles: Number of pulse cycles.
        """
        if color_from is None:
            color_from = QColor(0, 0, 0, 0)
        if color_to is None:
            color_to = QColor(10, 132, 255, 30)

        anim = QVariantAnimation(self)
        anim.setDuration(duration)
        anim.setStartValue(color_from)
        anim.setEndValue(color_to)
        anim.setEasingCurve(Easing.EASE_IN_OUT)
        anim.setLoopCount(cycles)

        def _update_style(c: QVariantAnimation, value, widget=widget):
            style = f"background-color: rgba({value.red()},{value.green()},{value.blue()},{value.alpha()});"
            widget.setStyleSheet(style)

        anim.valueChanged.connect(
            lambda v: widget.setStyleSheet(
                f"background-color: rgba({v.red()},{v.green()},{v.blue()},{v.alpha()});"
            )
        )
        anim.finished.connect(lambda: widget.setStyleSheet(""))
        anim.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    @staticmethod
    def spinner_label(label: QLabel, icon_name: str = "pending",
                      interval: int = 50) -> QTimer:
        """Create a rotating spinner on a QLabel using opacity pulsing.

        Args:
            label: Target label to animate.
            icon_name: Icon to use for spinner.
            interval: Update interval in ms.

        Returns:
            QTimer instance (caller must keep reference to prevent GC).
        """
        from sim_platform.tools.gui.icons import get_pixmap
        pix = get_pixmap(icon_name, size=16)
        label.setPixmap(pix)

        timer = QTimer()
        opacity = [1.0]

        def _pulse():
            opacity[0] = 0.3 + 0.7 * abs(math.sin(opacity[0] * 10))
            label.setWindowOpacity(opacity[0])

        timer.timeout.connect(_pulse)
        timer.start(interval)
        return timer

    @staticmethod
    def _ensure_opacity_effect(widget: QWidget):
        """Ensure the widget has a QGraphicsOpacityEffect."""
        if not widget.graphicsEffect():
            effect = QGraphicsOpacityEffect(widget)
            effect.setOpacity(1.0)
            widget.setGraphicsEffect(effect)


# ── State Transition Animations ────────────────────────────

class StateAnimation:
    """Pre-built animation sequences for simulation state changes."""

    @staticmethod
    def run_to_pause(button_widget: QWidget, indicator_widget: QWidget,
                     engine: AnimationEngine | None = None):
        """Animate transition from running to paused state."""
        if engine is None:
            engine = AnimationEngine()
        engine.pulse(indicator_widget, cycles=1)
        engine.pulse(button_widget, cycles=2)

    @staticmethod
    def pause_to_run(button_widget: QWidget, indicator_widget: QWidget,
                     engine: AnimationEngine | None = None):
        """Animate transition from paused to running state."""
        if engine is None:
            engine = AnimationEngine()
        engine.pulse(indicator_widget,
                     color_to=QColor(48, 209, 88, 40), cycles=1)

    @staticmethod
    def complete_flash(status_widget: QWidget,
                       engine: AnimationEngine | None = None):
        """Flash green on simulation completion."""
        if engine is None:
            engine = AnimationEngine()
        engine.pulse(status_widget,
                     color_to=QColor(48, 209, 88, 40), cycles=2)

    @staticmethod
    def error_flash(status_widget: QWidget,
                    engine: AnimationEngine | None = None):
        """Flash red on simulation error."""
        if engine is None:
            engine = AnimationEngine()
        engine.pulse(status_widget,
                     color_to=QColor(255, 69, 58, 40), cycles=3)


# ── Progress Animation Enhancer ────────────────────────────

class AnimatedProgressBar(QProgressBar):
    """QProgressBar subclass with smooth, animated value transitions.

    Interpolates between discrete progress values to avoid jarring
    jumps in the progress indicator.
    """

    def __init__(self, parent=None):
        super().__init__(parent)
        self._target_value = 0
        self._animation: QPropertyAnimation | None = None

    def setValue(self, value: int):
        """Animate to target value smoothly."""
        if value == self._target_value:
            return

        self._target_value = value

        if self._animation and self._animation.state() == QPropertyAnimation.State.Running:
            self._animation.stop()

        self._animation = QPropertyAnimation(self, b"animValue", self)
        self._animation.setDuration(200)
        self._animation.setStartValue(self.value())
        self._animation.setEndValue(value)
        self._animation.setEasingCurve(Easing.EASE_OUT)
        self._animation.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    def _get_anim_value(self) -> int:
        return super().value()

    def _set_anim_value(self, value: float):
        super().setValue(int(value))

    animValue = Property(int, _get_anim_value, _set_anim_value)

    def setValueInstant(self, value: int):
        """Set value without animation."""
        if self._animation and self._animation.state() == QPropertyAnimation.State.Running:
            self._animation.stop()
        self._target_value = value
        super().setValue(value)


# ── Widget Transition Helper ───────────────────────────────

class PageTransition:
    """Manages animated transitions between stacked widgets/pages.

    Usage:
        transition = PageTransition(stack)
        transition.switch_to(new_page, direction="right")
    """

    def __init__(self, widget_container: QWidget, engine: AnimationEngine | None = None):
        self._container = widget_container
        self._engine = engine or AnimationEngine()
        self._current: QWidget | None = None

    def switch_to(self, new_widget: QWidget, direction: str = "right",
                  duration: int = Duration.DEFAULT):
        """Animated switch to a new widget within the container.

        Args:
            new_widget: Target widget (should be a child of container).
            direction: Slide direction ('left', 'right', 'top', 'bottom').
            duration: Animation duration in ms.
        """
        old = self._current

        if old is new_widget:
            return

        if old and old != new_widget:
            self._engine.fade_out(old, duration // 2)

        self._engine.slide_in(new_widget, direction, duration)
        self._current = new_widget


# ── Guided Tour Spotlight Effect ───────────────────────────

class SpotlightOverlay(QWidget):
    """A semi-transparent overlay that highlights a target widget.

    Used for guided tours to draw attention to specific UI elements.
    The overlay darkens everything except the target area.
    """

    def __init__(self, parent: QWidget, target: QWidget,
                 message: str = ""):
        super().__init__(parent)
        self._target = target
        self._message = message
        self.setAttribute(Qt.WidgetAttribute.WA_TransparentForMouseEvents, False)

        # Fill entire parent
        self.setGeometry(parent.rect())

        # Make semi-transparent
        self._ensure_opacity()

    def _ensure_opacity(self):
        effect = QGraphicsOpacityEffect(self)
        effect.setOpacity(0.0)
        self.setGraphicsEffect(effect)

        anim = QPropertyAnimation(effect, b"opacity", self)
        anim.setDuration(400)
        anim.setStartValue(0.0)
        anim.setEndValue(0.92)
        anim.setEasingCurve(Easing.EASE_OUT)
        anim.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)

    def paintEvent(self, event):
        """Paint the overlay with a cutout for the target widget."""
        from PySide6.QtGui import QPainterPath

        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        # Fill entire area with dark color
        painter.setBrush(QColor(0, 0, 0, 200))
        painter.setPen(Qt.PenStyle.NoPen)

        # Create cutout for target widget
        path = QPainterPath()
        path.addRect(self.rect())

        target_rect = self._target.geometry()
        target_rect.translate(self._target.parentWidget().mapTo(self.parentWidget(), QPoint(0, 0)))
        # Add padding
        target_rect = target_rect.adjusted(-8, -8, 8, 8)

        path.addRoundedRect(target_rect, 6, 6)
        painter.drawPath(path)

        # Draw border around target
        pen = painter.pen()
        pen = painter.pen()
        pen.setColor(QColor(10, 132, 255, 220))
        pen.setWidth(2)
        painter.setPen(pen)
        painter.setBrush(Qt.BrushStyle.NoBrush)
        painter.drawRoundedRect(target_rect.adjusted(-1, -1, 1, 1), 8, 8)

        painter.end()

    def dismiss(self):
        """Fade out and close the overlay."""
        effect = self.graphicsEffect()
        if effect:
            anim = QPropertyAnimation(effect, b"opacity", self)
            anim.setDuration(300)
            anim.setStartValue(effect.opacity())
            anim.setEndValue(0.0)
            anim.setEasingCurve(Easing.EASE_IN)
            anim.finished.connect(self._on_dismissed)
            anim.start(QPropertyAnimation.DeletionPolicy.DeleteWhenStopped)
        else:
            self._on_dismissed()

    def _on_dismissed(self):
        self.setParent(None)
        self.deleteLater()


# ── Module-level convenience ───────────────────────────────

_default_engine = AnimationEngine()


def fade_in(widget: QWidget, duration: int = Duration.DEFAULT):
    """Convenience: fade in a widget."""
    _default_engine.fade_in(widget, duration)


def fade_out(widget: QWidget, duration: int = Duration.DEFAULT):
    """Convenience: fade out a widget."""
    _default_engine.fade_out(widget, duration)


def crossfade(old: QWidget, new: QWidget, duration: int = Duration.SMOOTH):
    """Convenience: crossfade between two widgets."""
    _default_engine.crossfade(old, new, duration)


def pulse(widget: QWidget, color_from=None, color_to=None,
          duration: int = Duration.FAST, cycles: int = 3):
    """Convenience: pulse a widget."""
    _default_engine.pulse(widget, color_from, color_to, duration, cycles)
