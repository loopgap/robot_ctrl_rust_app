"""sim_platform Textual TUI -- Terminal User Interface.

Modular TUI with:
- Main dashboard with quick actions and system status
- Configurable simulation parameter forms with validation
- Simulation runner with real-time progress and RichLog
- Results viewer with sortable DataTables
- Parameter scanner with multi-run comparison

Security:
- Input bounds validation on all numeric fields
- NaN/Inf rejection at every entry point
- Safe path defaults (no path traversal)
- No eval/exec/unsafe deserialization
"""


from textual.app import App
from textual.binding import Binding
from textual.screen import Screen

from .screens import ConfigScreen, MainScreen, ResultsScreen, RunScreen, ScanScreen
from .theme import TUI_CSS

# ════════════════════════════════════════════════════════════
#  APP
# ════════════════════════════════════════════════════════════

class SimPlatformTUI(App):
    """sim_platform Textual TUI Application."""

    TITLE = "sim_platform"
    SUB_TITLE = "Multi-Domain Co-Simulation"
    CSS = TUI_CSS

    SCREENS = {}

    # Screen instance store (set in on_mount)
    main_screen: Screen | None = None
    config_screen: Screen | None = None
    run_screen: Screen | None = None
    results_screen: Screen | None = None
    scan_screen: Screen | None = None

    BINDINGS = [
        Binding("ctrl+q", "quit", "Quit", priority=True),
        Binding("ctrl+h", "home", "Home", priority=True),
        Binding("ctrl+l", "back", "Back", priority=True),
    ]

    def on_mount(self) -> None:
        self.main_screen = MainScreen()
        self.config_screen = ConfigScreen()
        self.run_screen = RunScreen()
        self.results_screen = ResultsScreen()
        self.scan_screen = ScanScreen()
        self.push_screen(self.main_screen)

    def goto(self, screen_class) -> None:
        """Navigate to a screen by class."""
        instance = getattr(self, screen_class.__name__.lower() + "_screen", None)
        if instance:
            self.switch_screen(instance)

    def action_back(self) -> None:
        self.pop_screen()

    def action_quit(self) -> None:
        self.exit()

    def action_home(self) -> None:
        self.switch_screen(self.main_screen)


# ════════════════════════════════════════════════════════════
#  Entry point
# ════════════════════════════════════════════════════════════

def main():
    app = SimPlatformTUI()
    app.run()


if __name__ == "__main__":
    main()
