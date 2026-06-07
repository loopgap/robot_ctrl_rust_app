"""Results viewer screen.

Features KPI stat cards for key metrics and SparkLine trend charts
for speed and torque visualization.
"""

import os
import time

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal
from textual.screen import Screen
from textual.widgets import Button, DataTable, Footer, Header, Static

from sim_platform.tools.visualization.plot_log import plot_foc_results

from ..utils import OUTPUT_DIR
from ..widgets.cards import StatCard
from ..widgets.dialogs import ErrorDialog
from ..widgets.sparkline import SparkLine


class ResultsScreen(Screen):
    """Results viewer with KPI cards, SparkLine charts, and DataTable."""

    BINDINGS = [
        Binding("escape", "back", "Back"),
        Binding("r", "re_run", "Run Again"),
        Binding("p", "plot", "Generate Plot"),
    ]

    results: dict | None = None

    def set_results(self, data: dict):
        self.results = data

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Container(
            Static("[bold green]📊 Simulation Results[/]", id="results-title"),
            Static("", id="spacer-1"),

            # ── KPI Cards Row ──
            Static("[bold lavender]▸ Key Metrics[/]", classes="section-title"),
            Horizontal(
                StatCard("Final Speed", "--", "rad/s", id="kpi-speed"),
                StatCard("Speed Error", "--", "%", id="kpi-error"),
                StatCard("Peak Torque", "--", "N*m", id="kpi-torque"),
                StatCard("Peak Current", "--", "A", id="kpi-current"),
                classes="kpi-row",
            ),
            Static("", id="spacer-2"),

            # ── SparkLine Trend Charts ──
            Static("[bold lavender]▸ Trends[/]", classes="section-title"),
            Horizontal(
                SparkLine(title="Speed Trend", id="spark-speed", width=60),
                SparkLine(title="Torque Trend", id="spark-torque", width=60),
                classes="chart-row",
            ),
            Static("", id="spacer-3"),

            # ── Data Table ──
            Static("[bold lavender]▸ Detailed Data[/]", classes="section-title"),
            DataTable(id="metrics-table"),
            Static("", id="results-stats"),

            # ── Action Buttons ──
            Horizontal(
                Button("🔄 Run Again", variant="primary", id="rerun"),
                Button("📈 Generate Plot", variant="success", id="plot"),
                Button("← Back", variant="default", id="back"),
                classes="button-row",
            ),
            classes="results-container",
        )
        yield Footer()

    def on_mount(self) -> None:
        if self.results:
            self._show_results()

    def _show_results(self) -> None:
        data = self.results
        t = data.get("time", [])
        if not t:
            return
        speed = data.get("speed", [0])[-1]
        speed_ref = data.get("speed_ref", [0])[-1]
        torque = max(abs(v) for v in data.get("torque", [0]))
        id_val = data.get("id", [0])[-1]
        iq_val = data.get("iq", [0])[-1]

        error_pct = abs(speed - speed_ref) / max(speed_ref, 1) * 100
        peak_current = max(
            max(abs(v) for v in data.get("ia", [0])),
            max(abs(v) for v in data.get("ib", [0])),
            max(abs(v) for v in data.get("ic", [0])),
        )

        # Update KPI stat cards
        error_class = "good" if error_pct < 2 else ("warn" if error_pct < 5 else "bad")
        try:
            self.query_one("#kpi-speed", StatCard).update_value(
                f"{speed:.1f}", "good" if error_pct < 5 else "bad"
            )
            self.query_one("#kpi-error", StatCard).update_value(
                f"{error_pct:.2f}", error_class
            )
            self.query_one("#kpi-torque", StatCard).update_value(
                f"{torque:.3f}"
            )
            self.query_one("#kpi-current", StatCard).update_value(
                f"{peak_current:.2f}", "warn" if peak_current > 10 else ""
            )
        except Exception:
            pass

        # Update SparkLine charts
        try:
            speed_data = data.get("speed", [])
            self.query_one("#spark-speed", SparkLine).update_data(
                speed_data, title="Speed Trend (rad/s)"
            )
            torque_data = [abs(v) for v in data.get("torque", [])]
            self.query_one("#spark-torque", SparkLine).update_data(
                torque_data, title="Torque Trend (N*m)"
            )
        except Exception:
            pass

        # Populate DataTable
        table = self.query_one("#metrics-table", DataTable)
        table.columns.clear()
        table.add_columns("Metric", "Value", "Unit")
        table.add_rows([
            ("Final Speed", f"{speed:.1f}", "rad/s"),
            ("Speed Error", f"{error_pct:.2f}", "%"),
            ("Peak Torque", f"{torque:.3f}", "N*m"),
            ("Peak Phase Current", f"{peak_current:.2f}", "A"),
            ("d-axis Current (id)", f"{id_val:.3f}", "A"),
            ("q-axis Current (iq)", f"{iq_val:.3f}", "A"),
            ("Simulation Steps", str(len(t)), "steps"),
            ("Status", "✓ OK" if error_pct < 5 else "✗ HIGH ERROR", ""),
        ])

        self.query_one("#results-stats", Static).update(
            f"[bold]Tracking Accuracy: {error_pct:.2f}%[/]"
        )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        bid = event.button.id
        if bid == "rerun":
            from .config import ConfigScreen
            self.app.goto(ConfigScreen)
        elif bid == "plot" and self.results:
            self._generate_plot()
        elif bid == "back":
            from .main import MainScreen
            self.app.goto(MainScreen)

    def action_back(self) -> None:
        from .main import MainScreen
        self.app.goto(MainScreen)

    def action_re_run(self) -> None:
        from .config import ConfigScreen
        self.app.goto(ConfigScreen)

    def action_plot(self) -> None:
        self._generate_plot()

    def _generate_plot(self) -> None:
        if not self.results:
            return
        try:
            fname = f"tui_plot_{int(time.time())}.png"
            fpath = plot_foc_results(self.results, os.path.join(OUTPUT_DIR, fname))
            self.app.push_screen(ErrorDialog(
                "Plot Generated", f"Saved: {os.path.basename(fpath)}"
            ))
        except Exception as e:
            self.app.push_screen(ErrorDialog("Plot Error", str(e)))
