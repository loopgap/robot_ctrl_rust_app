"""Main dashboard screen.

Features a modern card-based layout with:
- Hero section with app title and version
- Quick action cards for primary workflows
- Preset scenario cards with icons and descriptions
- Keyboard shortcut reference
- F1 contextual help panel
"""

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal
from textual.screen import Screen
from textual.widgets import Button, Footer, Header, Static

from ..data.help_content import HELP_MAIN
from ..utils import SCENARIO_DETAILS, SCENARIOS
from ..widgets.cards import InfoCard
from ..widgets.dialogs import ConfirmDialog
from ..widgets.help_panel import HelpPanel


class MainScreen(Screen):
    """Modern dashboard with card-based layout and scenario presets."""

    BINDINGS = [
        Binding("r", "go_run", "Run"),
        Binding("c", "go_config", "Config"),
        Binding("s", "go_scan", "Scan"),
        Binding("f1", "toggle_help", "Help"),
        Binding("q", "quit_app", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Container(
            # ── Hero Section ──
            Static("", id="spacer-top"),
            Static("[bold blue]⚡ sim_platform[/]", id="app-title"),
            Static("[dim]Multi-Domain Co-Simulation Platform[/]", id="app-version"),
            Static("", id="spacer-hero"),

            # ── Quick Actions Section ──
            Static("[bold lavender]▸ Quick Actions[/]", classes="section-title"),
            Horizontal(
                InfoCard(
                    card_id="run",
                    icon="▶",
                    title="Run Simulation",
                    description="Configure and execute a simulation run",
                    id="card-run",
                ),
                InfoCard(
                    card_id="config",
                    icon="⚙",
                    title="Configure",
                    description="Adjust motor parameters and scenarios",
                    id="card-config",
                ),
                InfoCard(
                    card_id="scan",
                    icon="📊",
                    title="Parameter Scan",
                    description="Sweep parameters and compare results",
                    id="card-scan",
                ),
                classes="action-grid",
            ),
            Static("", id="spacer-actions"),

            # ── Preset Scenarios Section ──
            Static("[bold lavender]▸ Preset Scenarios[/]", classes="section-title"),
            Horizontal(
                *[
                    InfoCard(
                        card_id=name,
                        icon=SCENARIO_DETAILS.get(name, {}).get("icon", "●"),
                        title=name,
                        description=SCENARIO_DETAILS.get(name, {}).get("description", ""),
                    )
                    for name in SCENARIOS
                ],
                classes="preset-grid",
            ),
            Static("", id="spacer-presets"),

            # ── Keyboard Shortcuts ──
            Static("[bold lavender]▸ Keyboard Shortcuts[/]", classes="section-title"),
            Horizontal(
                Static("[blue]R[/] Run", classes="shortcut-item"),
                Static("[blue]C[/] Config", classes="shortcut-item"),
                Static("[blue]S[/] Scan", classes="shortcut-item"),
                Static("[blue]F1[/] Help", classes="shortcut-item"),
                Static("[blue]Q[/] Quit", classes="shortcut-item"),
                classes="shortcut-bar",
            ),

            # ── Help Panel (hidden by default) ──
            HelpPanel(HELP_MAIN, id="main-help", classes="hidden"),

            classes="main-container",
        )
        yield Footer()

    def on_info_card_selected(self, event: InfoCard.Selected) -> None:
        """Handle preset scenario card click."""
        card = event.sender
        self._select_scenario(card.card_id)

    def _select_scenario(self, scenario_name: str) -> None:
        """Navigate to config with a preset scenario selected."""
        from .config import ConfigScreen
        self.app.goto(ConfigScreen)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        bid = event.button.id
        if bid == "btn-run":
            self.action_go_run()
        elif bid == "btn-config":
            self.action_go_config()
        elif bid == "btn-scan":
            self.action_go_scan()

    def action_go_run(self) -> None:
        from .config import ConfigScreen
        self.app.goto(ConfigScreen)

    def action_go_config(self) -> None:
        from .config import ConfigScreen
        self.app.goto(ConfigScreen)

    def action_go_scan(self) -> None:
        from .scan import ScanScreen
        self.app.goto(ScanScreen)

    def action_toggle_help(self) -> None:
        """Toggle the help panel visibility."""
        help_panel = self.query_one("#main-help", HelpPanel)
        help_panel.toggle()

    def action_quit_app(self) -> None:
        self.app.push_screen(
            ConfirmDialog("Exit", "Quit sim_platform?", danger=True),
            lambda r: self.app.exit() if r else None,
        )
