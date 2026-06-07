"""Parameter configuration screen.

Uses ValidatedInput widgets for real-time validation of numeric fields,
and HelpPanel for contextual F1 help.
"""

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal
from textual.screen import Screen
from textual.widgets import (
    Button,
    Footer,
    Header,
    Select,
    Static,
)

from ..data.help_content import HELP_CONFIG
from ..utils import MOTOR_PRESETS, SCENARIOS, guard_float
from ..widgets.dialogs import ErrorDialog
from ..widgets.help_panel import HelpPanel
from ..widgets.validators import (
    ValidatedInput,
    validate_float_range,
)


class ConfigScreen(Screen):
    """Parameter configuration form with real-time validation."""

    BINDINGS = [
        Binding("escape", "back", "Back"),
        Binding("r", "run", "Run"),
        Binding("f1", "show_help", "Help"),
    ]

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Container(
            Static("[bold yellow]⚙ Configuration[/]", id="config-title"),
            Static("", id="spacer-1"),

            # ── Scenario Selection ──
            Static("[bold lavender]▸ Scenario[/]", classes="section-title"),
            Select([(n, n) for n in SCENARIOS], id="scenario", value="Step Response"),
            Static("", id="spacer-2"),

            # ── Motor Selection ──
            Static("[bold lavender]▸ Motor Preset[/]", classes="section-title"),
            Select([(n, n) for n in MOTOR_PRESETS], id="motor_preset",
                   value="Small PMSM (200W drone)"),
            Static("", id="spacer-3"),

            # ── Parameters ──
            Static("[bold lavender]▸ Parameters[/]", classes="section-title"),
            ValidatedInput(
                label="Speed Reference [rad/s]",
                value="100",
                input_id="speed_ref",
                input_type="integer",
                placeholder="5-500",
                validator_fn=lambda v: validate_float_range(v, 5.0, 500.0, 100.0),
            ),
            ValidatedInput(
                label="Simulation Duration [s]",
                value="1.5",
                input_id="duration",
                input_type="integer",
                placeholder="0.1-60",
                validator_fn=lambda v: validate_float_range(v, 0.1, 60.0, 1.5),
            ),
            ValidatedInput(
                label="Load Torque [N*m]",
                value="0.0",
                input_id="load_torque",
                input_type="integer",
                placeholder="0-10",
                validator_fn=lambda v: validate_float_range(v, 0.0, 10.0, 0.0),
            ),
            Static("", id="spacer-4"),

            # ── Action Buttons ──
            Horizontal(
                Button("▶ Run Now", variant="primary", id="run-now"),
                Button("📊 Run + Plot", variant="success", id="run-plot"),
                Button("← Back", variant="default", id="back"),
                classes="button-row",
            ),

            # ── Help Panel ──
            HelpPanel(HELP_CONFIG, id="config-help", classes="hidden"),

            classes="config-container",
        )
        yield Footer()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "back":
            self.action_back()
        elif event.button.id in ("run-now", "run-plot"):
            self._validate_and_run(event.button.id == "run-plot")

    def action_back(self) -> None:
        from .main import MainScreen
        self.app.goto(MainScreen)

    def action_run(self) -> None:
        self._validate_and_run(True)

    def action_show_help(self) -> None:
        """Toggle the help panel."""
        help_panel = self.query_one("#config-help", HelpPanel)
        help_panel.toggle()

    def _validate_and_run(self, do_plot: bool) -> None:
        """Validate inputs, build config, start run."""
        try:
            cfg = self._build_config()
            from .run import RunScreen
            self.app.run_screen.set_config(cfg, do_plot)
            self.app.goto(RunScreen)
        except Exception as e:
            self.app.push_screen(ErrorDialog("Configuration Error", str(e)))

    def _build_config(self) -> dict:
        """Build simulation config from form with validation."""
        scenario_name = self.query_one("#scenario", Select).value
        scenario = SCENARIOS.get(str(scenario_name), SCENARIOS["Step Response"])

        motor_name = str(self.query_one("#motor_preset", Select).value)
        motor_params = MOTOR_PRESETS.get(motor_name, MOTOR_PRESETS["Small PMSM (200W drone)"])

        # Read from ValidatedInput widgets
        speed_val = guard_float(self.query_one("#speed_ref", ValidatedInput).get_value(), 100)
        dur_val = guard_float(self.query_one("#duration", ValidatedInput).get_value(), 1.5)
        load_val = guard_float(self.query_one("#load_torque", ValidatedInput).get_value(), 0)

        if speed_val < 5 or speed_val > 500:
            raise ValueError("Speed reference must be 5-500 rad/s")
        if dur_val < 0.1 or dur_val > 60:
            raise ValueError("Duration must be 0.1-60s")
        if load_val < 0 or load_val > 10:
            raise ValueError("Load torque must be 0-10 N*m")

        return {
            "motor_params": motor_params,
            "speed_ref": speed_val,
            "duration_s": dur_val,
            "load_torque": load_val,
            "profile": scenario["profile"],
            "scenario_name": scenario_name,
            "do_plot": False,
        }
