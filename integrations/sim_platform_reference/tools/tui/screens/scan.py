"""Parameter scanner screen."""

import math

from textual import work
from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal
from textual.screen import Screen
from textual.widgets import (
    Button,
    Footer,
    Header,
    Input,
    Label,
    ProgressBar,
    RichLog,
    Select,
    Static,
)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder

from ..data.help_content import HELP_SCAN
from ..utils import SCAN_PARAMS
from ..widgets.help_panel import HelpPanel


class ScanScreen(Screen):
    """Parameter scanner with multi-run comparison."""

    BINDINGS = [
        Binding("escape", "back", "Back"),
        Binding("s", "start_scan", "Start Scan"),
        Binding("f1", "show_help", "Help"),
    ]

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Container(
            Static("[bold magenta]📊 Parameter Scanner[/]", id="scan-title"),
            Static("", id="spacer-1"),

            # ── Parameter Selection ──
            Static("[bold lavender]▸ Parameter[/]", classes="section-title"),
            Select(
                [(n, n) for n in SCAN_PARAMS],
                id="scan-param",
                value="Speed Reference",
            ),
            Static("", id="spacer-2"),

            # ── Values Input ──
            Static("[bold lavender]▸ Values[/]", classes="section-title"),
            Label("Comma-separated values:"),
            Input(value="50, 100, 150, 200", id="scan-values",
                  placeholder="e.g. 50, 100, 150"),
            Static("", id="spacer-3"),

            # ── Progress ──
            ProgressBar(total=100, id="scan-progress"),
            RichLog(max_lines=100, highlight=True, markup=True, id="scan-log"),
            Static("", id="spacer-4"),

            # ── Action Buttons ──
            Horizontal(
                Button("▶ Start Scan", variant="primary", id="start-scan"),
                Button("← Back", variant="default", id="back"),
                classes="button-row",
            ),

            # ── Help Panel ──
            HelpPanel(HELP_SCAN, id="scan-help", classes="hidden"),

            classes="scan-container",
        )
        yield Footer()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "start-scan":
            self.action_start_scan()
        elif event.button.id == "back":
            from .main import MainScreen
            self.app.goto(MainScreen)

    def action_back(self) -> None:
        from .main import MainScreen
        self.app.goto(MainScreen)

    def action_show_help(self) -> None:
        """Toggle the help panel."""
        help_panel = self.query_one("#scan-help", HelpPanel)
        help_panel.toggle()

    @work
    async def action_start_scan(self) -> None:
        """Run parameter scan with progress."""
        log = self.query_one("#scan-log", RichLog)
        progress = self.query_one("#scan-progress", ProgressBar)
        log.clear()

        param_name = str(self.query_one("#scan-param", Select).value)
        values_str = self.query_one("#scan-values", Input).value

        # Parse values
        try:
            raw = [v.strip() for v in values_str.split(",") if v.strip()]
            values = [float(v) for v in raw]
            # Validate
            if any(math.isnan(v) or math.isinf(v) for v in values):
                raise ValueError("Values cannot be NaN or Inf")
            if len(values) < 2:
                raise ValueError("Need at least 2 values")
        except Exception as e:
            log.write(f"[red]Error parsing values: {e}[/]")
            return

        param_key, _ = SCAN_PARAMS[param_name]
        log.write(f"[bold blue]⚡ Scanning:[/] {param_name} ({param_key})")
        log.write(f"[bold blue]Values:[/] {values}")
        log.write("")

        results = []
        for i, val in enumerate(values):
            pct = int((i + 1) / len(values) * 100)
            progress.update(progress=pct)
            log.write(f"  [{i+1}/{len(values)}] Running {param_key}={val}...")

            try:
                # Quick run
                motor = PMSMdqModel(
                    Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                    flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                    dt_ns=50000,
                )
                inverter = AverageInverter(48.0)
                cs = CurrentSensor(noise_std=0.05, bias=0.01)
                enc = Encoder(noise_std=0.001)
                foc = FOCController(
                    kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                    ts=50e-6, v_bus=48.0,
                )
                spd = SpeedController(kp=0.05, ki=0.5, ts=1e-3)

                speed_ref = val if param_key == "speed" else 100.0
                load = val if param_key == "load" else 0.0
                fkp = val if param_key == "kp_id" else 5.0
                fki = val if param_key == "ki_id" else 500.0

                # Use parameter overrides for FOC gains
                if param_key in ("kp_id", "ki_id", "kp_iq", "ki_iq"):
                    foc = FOCController(
                        kp_id=fkp if param_key == "kp_id" else 5.0,
                        ki_id=fki if param_key == "ki_id" else 500.0,
                        kp_iq=fkp if param_key == "kp_iq" else 5.0,
                        ki_iq=fki if param_key == "ki_iq" else 500.0,
                        ts=50e-6, v_bus=48.0,
                    )

                if param_key == "spd_kp":
                    spd = SpeedController(kp=val, ki=0.5, ts=1e-3)
                elif param_key == "spd_ki":
                    spd = SpeedController(kp=0.05, ki=val, ts=1e-3)

                iq_ref = 0.0
                for step in range(20000):
                    if step % 20 == 0:
                        iq_ref = spd.update(speed_ref, enc.read_speed(motor.omega_m))
                    ia_m, ib_m, ic_m = cs.read_abc(motor.ia, motor.ib, motor.ic)
                    th_m = enc.read_angle(motor.theta_e)
                    da, db, dc = foc.update(ia_m, ib_m, ic_m, th_m, 0.0, iq_ref)
                    va, vb, vc = inverter.step(da, db, dc, 48.0, ia_m, ib_m, ic_m)
                    motor.step_abc(va, vb, vc, tl=load, dt=50e-6)
                    motor.update_abc_currents()

                err = abs(motor.omega_m - speed_ref) / max(speed_ref, 1) * 100
                results.append((val, motor.omega_m, err))
                log.write(f"  → Speed: {motor.omega_m:.1f} rad/s, Error: {err:.2f}%")

            except Exception as e:
                log.write(f"  [red]Failed: {e}[/]")

        # Summary
        log.write("\n[bold green]✓ Scan Complete![/]")
        log.write(f"  {'Value':>12} | {'Speed':>10} | {'Error':>8} | {'Status':>8}")
        log.write(f"  {'-'*12}-+-{'-'*10}-+-{'-'*8}-+-{'-'*8}")
        for val, spd_val, err in results:
            ok = "[green]OK[/]" if err < 5 else "[red]HIGH[/]"
            log.write(f"  {val:>12.1f} | {spd_val:>8.1f} rad/s | {err:>5.2f}% | {ok}")
        if results:
            best_val, best_spd, best_err = min(results, key=lambda r: r[2])
            log.write(f"\n[bold]Best:[/] value={best_val}, error={best_err:.2f}%")
