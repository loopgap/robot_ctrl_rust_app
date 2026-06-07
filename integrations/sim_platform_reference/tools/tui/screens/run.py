"""Simulation runner screen.

Features multi-stage progress indicators and real-time stat cards
for speed, torque, and throughput monitoring.
"""

import asyncio
import math
import os
import time
import traceback

from textual import work
from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal
from textual.screen import Screen
from textual.widgets import (
    Button,
    Footer,
    Header,
    ProgressBar,
    RichLog,
    Static,
)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel

# ── Simulation models ─────────────────────────────────────
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder
from sim_platform.tools.replay.hdf5_logger import HDF5Logger
from sim_platform.tools.visualization.plot_log import plot_foc_results

from ..utils import OUTPUT_DIR
from ..widgets.cards import StatCard

# ── Simulation stages ─────────────────────────────────────
STAGES = ["Init", "Simulate", "Log", "Plot"]


class RunScreen(Screen):
    """Simulation runner with progress bar, stage indicators, and stat cards."""

    BINDINGS = [
        Binding("escape", "back", "Back to Results"),
        Binding("q", "quit", "Quit"),
    ]

    config: dict = {}
    do_plot: bool = False
    results: dict | None = None

    def set_config(self, cfg: dict, plot: bool = False):
        self.config = cfg
        self.do_plot = plot
        self.results = None

    def _render_stage_indicator(self, current_stage: int) -> str:
        """Render the multi-stage progress indicator."""
        parts = []
        for i, stage in enumerate(STAGES):
            if i < current_stage:
                parts.append(f"[green]✓ {stage}[/]")
            elif i == current_stage:
                parts.append(f"[bold blue]▶ {stage}[/]")
            else:
                parts.append(f"[dim]○ {stage}[/]")
        return "  →  ".join(parts)

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Container(
            Static("[bold green]▶ Running Simulation[/]", id="run-title"),
            # Stage indicator
            Static(self._render_stage_indicator(-1), id="stage-indicator",
                   classes="stage-indicator"),
            Static("", id="run-status"),
            ProgressBar(total=100, show_eta=True, id="progress"),
            # Real-time stat cards
            Horizontal(
                StatCard("Speed", "--", "rad/s", id="stat-speed"),
                StatCard("Torque", "--", "N*m", id="stat-torque"),
                StatCard("FPS", "--", "steps/s", id="stat-fps"),
                StatCard("Progress", "0%", "", id="stat-progress"),
            ),
            Static("", id="run-stats"),
            RichLog(max_lines=200, highlight=True, markup=True, id="run-log"),
            Horizontal(
                Button("📊 Results", variant="primary", id="view-results", disabled=True),
                Button("🔄 Run Again", variant="default", id="run-again"),
                Button("← Back", variant="default", id="back"),
                classes="button-row",
            ),
            classes="run-container",
        )
        yield Footer()

    def on_mount(self) -> None:
        if self.config:
            self.run_simulation()

    @work
    async def run_simulation(self) -> None:
        """Async simulation with progress updates."""
        log = self.query_one("#run-log", RichLog)
        progress = self.query_one("#progress", ProgressBar)
        status = self.query_one("#run-status", Static)
        stats = self.query_one("#run-stats", Static)
        stage_widget = self.query_one("#stage-indicator", Static)

        cfg = self.config
        dt_c = 50e-6
        dt_s = 1e-3
        duration = cfg["duration_s"]
        speed_ref = cfg["speed_ref"]
        speed_ratio = int(dt_s / dt_c)
        total_steps = int(duration / dt_c)

        log.clear()
        log.write("[bold blue]⚡ Starting simulation...[/]")
        log.write(f"  Duration: {duration}s | Step: {dt_c*1e6:.0f}us")
        log.write(f"  Motor: {cfg.get('scenario_name', 'Custom')}")
        log.write(f"  Target Speed: {speed_ref} rad/s ({speed_ref*60/(2*math.pi):.0f} rpm)")
        log.write("")

        try:
            # ── Stage 0: Init ────────────────────────────
            stage_widget.update(self._render_stage_indicator(0))
            await asyncio.sleep(0)

            _battery = RintBattery(48.0, 0.05)
            inverter = AverageInverter(48.0)
            motor = PMSMdqModel(**cfg["motor_params"], dt_ns=int(dt_c * 1e9))
            csensor = CurrentSensor(noise_std=0.1, bias=0.01)
            encoder = Encoder(noise_std=0.001)
            foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                                ts=dt_c, v_bus=48.0)
            spd = SpeedController(kp=0.05, ki=0.5, ts=dt_s)

            # ── Data buffers ─────────────────────────────
            data = {k: [] for k in [
                "time", "speed_ref", "speed", "id", "iq", "ia", "ib", "ic",
                "torque", "duty_a", "duty_b", "duty_c", "vd", "vq", "v_bus",
            ]}

            # ── Stage 1: Simulate ────────────────────────
            stage_widget.update(self._render_stage_indicator(1))
            await asyncio.sleep(0)

            iq_ref = 0.0
            prev_pct = -1
            last_update = time.time()

            for step in range(total_steps):
                t = step * dt_c

                # Speed loop
                if step % speed_ratio == 0:
                    sm = encoder.read_speed(motor.omega_m)
                    iq_ref = spd.update(speed_ref, sm)

                # Load torque
                load_tl = cfg.get("load_torque", 0) if t >= 0.5 else 0

                # FOC
                ia_m, ib_m, ic_m = csensor.read_abc(motor.ia, motor.ib, motor.ic)
                th_m = encoder.read_angle(motor.theta_e)
                da, db, dc = foc.update(ia_m, ib_m, ic_m, th_m, 0.0, iq_ref)
                va, vb, vc = inverter.step(da, db, dc, 48.0, ia_m, ib_m, ic_m)
                motor.step_abc(va, vb, vc, tl=load_tl, dt=dt_c)
                motor.update_abc_currents()

                # Log
                for k, v in [
                    ("time", t), ("speed_ref", speed_ref), ("speed", sm),
                    ("id", motor.id), ("iq", motor.iq),
                    ("ia", motor.ia), ("ib", motor.ib), ("ic", motor.ic),
                    ("torque", motor.torque),
                    ("duty_a", da), ("duty_b", db), ("duty_c", dc),
                    ("vd", foc.vd_ref), ("vq", foc.vq_ref), ("v_bus", 48.0),
                ]:
                    data[k].append(v)

                # Progress update (throttled to every 2%)
                pct = int((step + 1) / total_steps * 100)
                if pct != prev_pct:
                    prev_pct = pct
                    progress.update(progress=pct)

                    # Update stat cards and status (every 10%)
                    if pct % 10 == 0:
                        fps = (step + 1) / max(1, time.time() - last_update)
                        status.update(
                            f"Step {step+1}/{total_steps} ({pct}%) -- {fps:.0f} steps/s"
                        )
                        stats.update(
                            f"Speed: {motor.omega_m:.1f} | Torque: {motor.torque:.3f} N*m"
                        )
                        # Update stat cards
                        try:
                            self.query_one("#stat-speed", StatCard).update_value(
                                f"{motor.omega_m:.1f}", "good"
                            )
                            self.query_one("#stat-torque", StatCard).update_value(
                                f"{motor.torque:.3f}"
                            )
                            self.query_one("#stat-fps", StatCard).update_value(
                                f"{fps:.0f}"
                            )
                            self.query_one("#stat-progress", StatCard).update_value(
                                f"{pct}%"
                            )
                        except Exception:
                            pass
                        await asyncio.sleep(0)  # yield to event loop

            # ── Stage 2: Log ─────────────────────────────
            stage_widget.update(self._render_stage_indicator(2))
            progress.update(progress=100)
            status.update("[bold green]✓ Simulation complete![/]")
            log.write("\n[bold green]Results:[/]")
            log.write(f"  Final Speed: {motor.omega_m:.1f} rad/s "
                      f"({motor.omega_m*60/(2*math.pi):.0f} rpm)")
            log.write(f"  Error: {abs(motor.omega_m - speed_ref)/max(speed_ref,1)*100:.2f}%")
            log.write(f"  Peak Torque: {max(abs(t) for t in data['torque']):.3f} N*m")
            log.write("")

            self.results = data
            self.query_one("#view-results", Button).disabled = False

            # Save HDF5
            fname = f"tui_run_{int(speed_ref)}rads.h5"
            fpath = os.path.join(OUTPUT_DIR, fname)
            log.write(f"Saving -> {fname}")
            with HDF5Logger(fpath) as hlog:
                for i in range(0, len(data["time"]), 10):
                    hlog.record(
                        data["time"][i],
                        speed_ref=data["speed_ref"][i],
                        speed=data["speed"][i],
                        torque=data["torque"][i],
                        id=data["id"][i], iq=data["iq"][i],
                    )

            # ── Stage 3: Plot ────────────────────────────
            if self.do_plot:
                stage_widget.update(self._render_stage_indicator(3))
                plot_path = os.path.join(OUTPUT_DIR, fname.replace(".h5", ".png"))
                log.write(f"Plot -> {os.path.basename(plot_path)}")
                plot_foc_results(data, plot_path)

            # Mark all stages done
            stage_widget.update(self._render_stage_indicator(4))

        except Exception as e:
            progress.update(progress=0)
            status.update(f"[bold red]Error: {e}[/]")
            log.write(f"\n[red]Simulation failed: {e}[/]")
            log.write(traceback.format_exc())

    def on_button_pressed(self, event: Button.Pressed) -> None:
        bid = event.button.id
        if bid == "view-results" and self.results:
            from .results import ResultsScreen
            self.app.results_screen.set_results(self.results)
            self.app.goto(ResultsScreen)
        elif bid == "run-again":
            self.run_simulation()
        elif bid == "back":
            from .config import ConfigScreen
            self.app.goto(ConfigScreen)

    def action_back(self) -> None:
        if self.results:
            from .results import ResultsScreen
            self.app.results_screen.set_results(self.results)
            self.app.goto(ResultsScreen)
        else:
            from .config import ConfigScreen
            self.app.goto(ConfigScreen)

    def action_quit(self) -> None:
        self.app.exit()
