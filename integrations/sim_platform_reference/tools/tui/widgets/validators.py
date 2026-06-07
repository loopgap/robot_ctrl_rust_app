"""Validated input widget for the sim_platform TUI.

Provides ValidatedInput — an Input widget that validates its value in
real-time, showing validation messages and applying visual feedback
via CSS classes (input-valid / input-invalid).
"""

import math
from collections.abc import Callable

from textual.app import ComposeResult
from textual.containers import Vertical
from textual.reactive import reactive
from textual.widgets import Input, Static


class ValidatedInput(Vertical):
    """An Input widget with real-time validation and visual feedback.

    Validates the current input value whenever it changes and displays
    a validation message below the input. Adds CSS classes
    'input-valid' or 'input-invalid' to the Input for visual styling.

    Attributes:
        label: The label text displayed above the input.
        input_id: The widget id for the inner Input element.
        value: The current input value (reactive).
        is_valid: Whether the current value passes validation.
        validation_message: The current validation feedback message.
        validation_color: CSS class for message color.
    """

    label: str
    input_id: str
    value: reactive[str] = reactive("")
    is_valid: reactive[bool] = reactive(True)
    validation_message: reactive[str] = reactive("")
    validation_color: reactive[str] = reactive("validation-msg-valid")

    def __init__(
        self,
        label: str,
        value: str = "",
        input_id: str = "validated-input",
        input_type: str = "text",
        placeholder: str = "",
        validator_fn: Callable[[str], tuple] | None = None,
        id: str | None = None,
        classes: str | None = None,
    ):
        """Initialize ValidatedInput.

        Args:
            label: Label text displayed above the input.
            value: Initial value for the input.
            input_id: Widget id for the inner Input.
            input_type: HTML input type (e.g., 'integer', 'number', 'text').
            placeholder: Placeholder text for the input.
            validator_fn: A callable that takes (value: str) and returns
                (is_valid: bool, message: str). If None, no validation is done.
            id: Optional Textual widget id for the container.
            classes: Optional CSS class names.
        """
        super().__init__(id=id, classes=classes)
        self.label = label
        self.input_id = input_id
        self.value = value
        self._input_type = input_type
        self._placeholder = placeholder
        self._validator_fn = validator_fn

    def compose(self) -> ComposeResult:
        yield Static(f"[bold]{self.label}[/]", classes="validated-label")
        yield Input(
            value=self.value,
            id=self.input_id,
            type=self._input_type,
            placeholder=self._placeholder,
        )
        yield Static(
            self.validation_message,
            classes=f"validation-msg {self.validation_color}",
            id=f"{self.input_id}-validation",
        )

    def on_mount(self) -> None:
        """Run initial validation after mount."""
        if self._validator_fn and self.value:
            self._run_validation(self.value)

    def on_input_changed(self, event: Input.Changed) -> None:
        """Handle input value changes — run validation."""
        if event.input.id == self.input_id:
            self.value = event.value
            self._run_validation(event.value)

    def _run_validation(self, value: str) -> None:
        """Execute the validator function and update visual state.

        Args:
            value: The current input value to validate.
        """
        if self._validator_fn is None:
            self.is_valid = True
            self.validation_message = ""
            self.validation_color = "validation-msg-valid"
            self._update_input_style(True)
            self._update_message("")
            return

        is_valid, message = self._validator_fn(value)
        self.is_valid = is_valid
        self.validation_message = message
        self.validation_color = "validation-msg-valid" if is_valid else "validation-msg"
        self._update_input_style(is_valid)
        self._update_message(message)

    def _update_input_style(self, is_valid: bool) -> None:
        """Apply valid/invalid CSS class to the inner Input.

        Args:
            is_valid: Whether the current value is valid.
        """
        try:
            input_widget = self.query_one(f"#{self.input_id}", Input)
            input_widget.set_class(is_valid, "input-valid")
            input_widget.set_class(not is_valid, "input-invalid")
        except Exception:
            pass

    def _update_message(self, message: str) -> None:
        """Update the validation message text and styling.

        Args:
            message: The validation feedback message.
        """
        try:
            msg_widget = self.query_one(f"#{self.input_id}-validation", Static)
            msg_widget.update(message)
            msg_widget.set_class(True, self.validation_color)
        except Exception:
            pass

    def get_value(self) -> str:
        """Return the current input value.

        Returns:
            The current value of the inner Input widget.
        """
        try:
            return self.query_one(f"#{self.input_id}", Input).value
        except Exception:
            return self.value


# ════════════════════════════════════════════════════════════
#  Built-in Validator Functions
# ════════════════════════════════════════════════════════════

def validate_float_range(
    value: str,
    min_val: float = 0.0,
    max_val: float = float("inf"),
    default: float = 0.0,
) -> tuple:
    """Validate a float string within a numeric range.

    Args:
        value: The string value to validate.
        min_val: Minimum allowed value (inclusive).
        max_val: Maximum allowed value (inclusive).
        default: Default value if parsing fails.

    Returns:
        A tuple of (is_valid: bool, message: str).
    """
    if not value.strip():
        return (False, "Value cannot be empty")
    try:
        v = float(value)
    except (ValueError, TypeError):
        return (False, "Must be a number")
    if math.isnan(v) or math.isinf(v):
        return (False, "Cannot be NaN or Inf")
    if v < min_val or v > max_val:
        return (False, f"Must be between {min_val} and {max_val}")
    return (True, "Valid")


def validate_comma_separated(value: str) -> tuple:
    """Validate a comma-separated list of numeric values.

    Args:
        value: The string value to validate (e.g. "50, 100, 150").

    Returns:
        A tuple of (is_valid: bool, message: str).
    """
    if not value.strip():
        return (False, "Value cannot be empty")
    try:
        raw = [v.strip() for v in value.split(",") if v.strip()]
        values = [float(v) for v in raw]
        if any(math.isnan(v) or math.isinf(v) for v in values):
            return (False, "Values cannot be NaN or Inf")
        if len(values) < 2:
            return (False, "Need at least 2 values")
        return (True, f"{len(values)} values parsed")
    except (ValueError, TypeError) as e:
        return (False, f"Invalid number: {e}")
