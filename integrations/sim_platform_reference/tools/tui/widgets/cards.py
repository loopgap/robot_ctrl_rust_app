"""Card widgets for the sim_platform TUI.

Provides InfoCard for clickable scenario presets with icon, title,
and description; and StatCard for displaying key-value metrics with
conditional color styling.
"""


from textual.app import ComposeResult
from textual.containers import Horizontal, Vertical
from textual.message import Message
from textual.reactive import reactive
from textual.widgets import Static


class InfoCard(Static, can_focus=True, can_focus_children=False):
    """A clickable card displaying an icon, title, and description.

    Used on the main dashboard to present preset simulation scenarios
    as interactive cards that respond to clicks and Enter/Space.

    Attributes:
        card_id: Unique identifier for this card.
        icon: Short icon string (emoji or ASCII art).
        title: Card title text.
        description: Multi-line description or details.
    """

    # Use hardcoded Catppuccin Mocha colors (self-contained, no variable dependencies)
    DEFAULT_CSS = """
    InfoCard {
        background: #313244;
        border: round #45475A;
        padding: 1 2;
        margin: 0 0 1 0;
        height: auto;
        min-width: 30;
    }
    InfoCard:hover {
        background: #45475A;
        border: round #89B4FA;
    }
    InfoCard:focus {
        background: #45475A;
        border: tall #89B4FA;
    }
    .card-icon {
        width: 3;
        text-align: center;
        color: #CBA6F7;
        text-style: bold;
    }
    .card-title {
        text-style: bold;
        color: #89B4FA;
        padding: 0 0 0 1;
    }
    .card-description {
        color: #A6ADC8;
        padding: 0 0 0 1;
    }
    """

    card_id: str
    icon: str
    title: str
    description: str

    def __init__(
        self,
        card_id: str,
        icon: str,
        title: str,
        description: str,
        id: str | None = None,
        classes: str | None = None,
    ):
        """Initialize InfoCard.

        Args:
            card_id: Internal identifier used in event handlers.
            icon: Short icon/emoji string displayed on the left.
            title: Bold title text displayed at the top.
            description: Detailed description text below the title.
            id: Optional Textual widget id.
            classes: Optional CSS class names.
        """
        super().__init__(id=id, classes=classes)
        self.card_id = card_id
        self.icon = icon
        self.title = title
        self.description = description

    def compose(self) -> ComposeResult:
        with Horizontal():
            yield Static(f"[bold]{self.icon}[/]", classes="card-icon")
            with Vertical():
                yield Static(f"[bold blue]{self.title}[/]", classes="card-title")
                yield Static(self.description, classes="card-description")

    def on_click(self) -> None:
        """Handle mouse click — activate this card."""
        self.post_message(self.Selected(self))

    def action_select(self) -> None:
        """Handle keyboard activation (Enter/Space)."""
        self.post_message(self.Selected(self))

    class Selected(Message):
        """Posted when the InfoCard is clicked or activated via keyboard."""

        def __init__(self, sender: "InfoCard") -> None:
            super().__init__()
            self.sender = sender


class StatCard(Static):
    """A card displaying a labeled metric value with optional unit and color.

    Supports conditional color coding based on value thresholds:
    - 'good' / 'warn' / 'bad' CSS classes for metric coloring.

    Attributes:
        label: The metric name (e.g., "Final Speed").
        value: The formatted metric value string.
        unit: Optional unit string (e.g., "rad/s").
        color_class: One of 'good', 'warn', 'bad', or '' (default).
    """

    # Use hardcoded Catppuccin Mocha colors (self-contained, no variable dependencies)
    DEFAULT_CSS = """
    StatCard {
        background: #313244;
        border: round #45475A;
        padding: 1 2;
        margin: 0 1 0 0;
        width: 1fr;
        height: auto;
    }
    .stat-label {
        color: #A6ADC8;
        text-style: bold;
        text-align: center;
        padding: 0 0 0 0;
    }
    .stat-value {
        text-style: bold;
        color: #CDD6F4;
        text-align: center;
        padding: 0 0 0 0;
    }
    .stat-unit {
        color: #7F849C;
        text-align: center;
        padding: 0 0 0 0;
    }
    .stat-good {
        color: #A6E3A1;
    }
    .stat-warn {
        color: #F9E2AF;
    }
    .stat-bad {
        color: #F38BA8;
    }
    """

    label: str
    value: str
    unit: str
    color_class: reactive[str] = reactive("")

    def __init__(
        self,
        label: str,
        value: str,
        unit: str = "",
        color_class: str = "",
        id: str | None = None,
        classes: str | None = None,
    ):
        """Initialize StatCard.

        Args:
            label: The metric name displayed above the value.
            value: The formatted metric value.
            unit: Optional unit string displayed below the value.
            color_class: CSS class for value coloring ('good', 'warn', 'bad').
            id: Optional Textual widget id.
            classes: Optional CSS class names.
        """
        super().__init__(id=id, classes=classes)
        self.label = label
        self.value = value
        self.unit = unit
        self.color_class = color_class

    def compose(self) -> ComposeResult:
        yield Static(self.label, classes="stat-label")
        value_classes = f"stat-value stat-{self.color_class}" if self.color_class else "stat-value"
        yield Static(self.value, classes=value_classes)
        if self.unit:
            yield Static(self.unit, classes="stat-unit")

    def update_value(self, value: str, color_class: str = "") -> None:
        """Update the displayed value and color class.

        Args:
            value: New formatted value string.
            color_class: New color class ('good', 'warn', 'bad', or '').
        """
        self.value = value
        self.color_class = color_class
        try:
            value_widget = self.query_one(".stat-value", Static)
            value_widget.update(value)
            # Update color class
            value_widget.remove_class("stat-good")
            value_widget.remove_class("stat-warn")
            value_widget.remove_class("stat-bad")
            if color_class:
                value_widget.add_class(f"stat-{color_class}")
        except Exception:
            pass
