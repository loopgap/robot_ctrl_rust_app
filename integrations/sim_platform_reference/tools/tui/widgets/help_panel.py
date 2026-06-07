"""HelpPanel widget for the sim_platform TUI.

Provides a collapsible help panel that displays keyboard shortcuts and
context-sensitive help information using structured help content data.
"""


from textual.app import ComposeResult
from textual.containers import VerticalScroll
from textual.reactive import reactive
from textual.widgets import Static


class HelpPanel(VerticalScroll, can_focus=True):
    """A collapsible panel that displays context-sensitive help content.

    Renders help content organized by sections, with each section
    containing key-description pairs. The panel can be toggled
    visible/hidden using the toggle() method.

    Attributes:
        visible: Whether the panel is currently shown.
        help_content: The help data to display, organized by sections.
    """

    # Use hardcoded Catppuccin Mocha colors (self-contained, no variable dependencies)
    DEFAULT_CSS = """
    HelpPanel {
        background: #313244;
        border: round #89B4FA;
        padding: 1 2;
        margin: 0 0 1 0;
        height: auto;
        max-height: 16;
    }
    HelpPanel.hidden {
        display: none;
    }
    """

    visible: reactive[bool] = reactive(True)
    help_content: dict[str, list[tuple[str, str]]] = {}

    def __init__(
        self,
        help_content: dict[str, list[tuple[str, str]]] | None = None,
        id: str | None = None,
        classes: str | None = None,
    ):
        """Initialize HelpPanel.

        Args:
            help_content: Dictionary mapping section titles to lists of
                (key, description) tuples. If None, starts with empty content.
            id: Optional Textual widget id.
            classes: Optional CSS class names.
        """
        super().__init__(id=id, classes=classes)
        self.help_content = help_content or {}

    def compose(self) -> ComposeResult:
        yield Static("[bold cyan]Help & Shortcuts[/]", classes="help-title")
        for section_title, entries in self.help_content.items():
            yield Static(f"[bold]{section_title}[/]", classes="help-section")
            for key, desc in entries:
                yield Static(
                    f"  [bold cyan]{key}[/]  {desc}",
                    classes="help-item",
                )

    def toggle(self) -> None:
        """Toggle the help panel visibility."""
        self.visible = not self.visible
        self.set_class(not self.visible, "hidden")

    def update_content(self, content: dict[str, list[tuple[str, str]]]) -> None:
        """Replace the help content and re-render.

        Args:
            content: New help content dictionary.
        """
        self.help_content = content
        # Remove all children and re-compose
        self.remove_children()
        # Re-mount new content by calling compose
        for child in self._compose_content():
            self.mount(child)

    def _compose_content(self) -> list:
        """Build child widgets from current help content.

        Returns:
            List of Static widgets representing the help content.
        """
        children = []
        children.append(Static("[bold cyan]Help & Shortcuts[/]", classes="help-title"))
        for section_title, entries in self.help_content.items():
            children.append(Static(f"[bold]{section_title}[/]", classes="help-section"))
            for key, desc in entries:
                children.append(
                    Static(
                        f"  [bold cyan]{key}[/]  {desc}",
                        classes="help-item",
                    )
                )
        return children
