"""QThread-based simulation worker for PySide6 GUI.

Runs the PMSM FOC simulation loop in a background thread,
emitting signals for progress, data updates, log messages,
and completion. Supports pause/resume and configurable parameters.
"""

from __future__ import annotations

import math
import threading
import time

from PySide6.QtCore import QThread, Signal

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder
from sim_platform.tools.gui.i18n import tr


class SimulationWorker(QThread):
    """Background thread running the PMSM FOC simulation.

    Signals:
        progress(int): 0-100 percentage
        data_update(dict): latest data point for real-time chart
        log_message(str): log text to append
        finished(dict): full simulation data on completion
        error(str): error message on failure
        status(str): status change (running/paused/stopped)
    """

    progress = Signal(int)
    data_update = Signal(dict)
    log_message = Signal(str)
    finished = Signal(dict)
    error = Signal(str)
    status = Signal(str)

    def __init__(self, config: dict, parent=None):
        super().__init__(parent)
        self._config = config
        self._stop_event = threading.Event()
        self._pause_event = threading.Event()  # Set = running, Clear = paused

    def stop(self):
        """Request graceful stop (thread-safe)."""
        self._stop_event.set()
        self._pause_event.set()  # Unpause to allow stop

    def pause(self):
        """Pause simulation (thread-safe)."""
        self._pause_event.clear()
        self.status.emit("paused")

    def resume(self):
        """Resume simulation (thread-safe)."""
        self._pause_event.set()
        self.status.emit("running")

    @property
    def is_paused(self) -> bool:
        return not self._pause_event.is_set()

    @property
    def _stop_requested(self) -> bool:
        """Backward-compatible property for tests."""
        return self._stop_event.is_set()

    def run(self):
        """Execute the simulation loop."""
        cfg = self._config

        # Read time parameters from config (was hardcoded)
        dt_c = cfg.get("dt_c", 50e-6)
        dt_s = cfg.get("dt_s", 1e-3)
        duration = cfg["duration_s"]
        speed_ref = cfg["speed_ref"]
        speed_ratio = max(1, int(dt_s / dt_c))
        total_steps = int(duration / dt_c)

        # Read battery parameters from config (was hardcoded)
        bat_cfg = cfg.get("battery", {})
        v_bus = bat_cfg.get("voltage", 48.0)
        r_bat = bat_cfg.get("resistance", 0.05)

        # Read FOC gains from config (was hardcoded)
        foc_cfg = cfg.get("foc", {})
        kp_id = foc_cfg.get("kp_id", 5.0)
        ki_id = foc_cfg.get("ki_id", 500.0)
        kp_iq = foc_cfg.get("kp_iq", 5.0)
        ki_iq = foc_cfg.get("ki_iq", 500.0)

        # Read speed loop gains from config (was hardcoded)
        spd_cfg = cfg.get("speed_pi", {})
        spd_kp = spd_cfg.get("kp", 0.05)
        spd_ki = spd_cfg.get("ki", 0.5)

        # Read sensor parameters from config (was hardcoded)
        sen_cfg = cfg.get("sensors", {})
        cs_noise = sen_cfg.get("current_noise", 0.1)
        cs_bias = sen_cfg.get("current_bias", 0.01)
        enc_noise = sen_cfg.get("encoder_noise", 0.001)

        self._pause_event.set()  # Start in running state
        self.status.emit("running")

        self.log_message.emit(tr("worker.starting"))
        self.log_message.emit(tr("worker.duration_info", str(duration), str(int(dt_c*1e6)), str(round(dt_s*1e3, 1))))
        self.log_message.emit(tr("worker.motor", cfg.get("scenario_name", "Custom")))
        self.log_message.emit(tr("worker.battery", str(v_bus), str(r_bat)))
        self.log_message.emit(tr("worker.foc_params", str(kp_id), str(ki_id), str(kp_iq), str(ki_iq)))
        self.log_message.emit(tr("worker.speed_pi_params", str(spd_kp), str(spd_ki)))
        self.log_message.emit(tr("worker.target", str(speed_ref), str(round(speed_ref * 60 / (2 * math.pi)))))
        self.log_message.emit("")

        try:
            # ── Init models ───────────────────────────────
            _battery = RintBattery(v_bus, r_bat)
            inverter = AverageInverter(v_bus)
            motor = PMSMdqModel(
                **cfg["motor_params"], dt_ns=int(dt_c * 1e9)
            )
            csensor = CurrentSensor(noise_std=cs_noise, bias=cs_bias)
            encoder = Encoder(noise_std=enc_noise)
            foc = FOCController(
                kp_id=kp_id, ki_id=ki_id,
                kp_iq=kp_iq, ki_iq=ki_iq,
                ts=dt_c, v_bus=v_bus,
            )
            spd = SpeedController(kp=spd_kp, ki=spd_ki, ts=dt_s)

            # ── Data buffers ──────────────────────────────
            data = {
                k: []
                for k in [
                    "time", "speed_ref", "speed",
                    "id", "iq", "ia", "ib", "ic",
                    "torque", "duty_a", "duty_b", "duty_c",
                    "vd", "vq", "v_bus",
                ]
            }

            iq_ref = 0.0
            sm = 0.0
            prev_pct = -1
            last_update = time.time()

            for step in range(total_steps):
                # Check stop
                if self._stop_event.is_set():
                    self.log_message.emit(tr("worker.stopped"))
                    return

                # Check pause (blocks until resumed)
                self._pause_event.wait()

                t = step * dt_c

                # Speed loop (every speed_ratio steps)
                if step % speed_ratio == 0:
                    sm = encoder.read_speed(motor.omega_m)
                    iq_ref = spd.update(speed_ref, sm)

                # Load torque (applied at t >= 0.5s)
                load_tl = cfg.get("load_torque", 0) if t >= 0.5 else 0

                # FOC control
                ia_m, ib_m, ic_m = csensor.read_abc(
                    motor.ia, motor.ib, motor.ic
                )
                th_m = encoder.read_angle(motor.theta_e)
                da, db, dc = foc.update(
                    ia_m, ib_m, ic_m, th_m, 0.0, iq_ref
                )
                va, vb, vc = inverter.step(
                    da, db, dc, v_bus, ia_m, ib_m, ic_m
                )
                motor.step_abc(va, vb, vc, tl=load_tl, dt=dt_c)
                motor.update_abc_currents()

                # Record data
                for k, v in [
                    ("time", t), ("speed_ref", speed_ref), ("speed", sm),
                    ("id", motor.id), ("iq", motor.iq),
                    ("ia", motor.ia), ("ib", motor.ib), ("ic", motor.ic),
                    ("torque", motor.torque),
                    ("duty_a", da), ("duty_b", db), ("duty_c", dc),
                    ("vd", foc.vd_ref), ("vq", foc.vq_ref), ("v_bus", v_bus),
                ]:
                    data[k].append(v)

                # Progress update (every 2%)
                pct = int((step + 1) / total_steps * 100)
                if pct != prev_pct:
                    prev_pct = pct
                    self.progress.emit(pct)
                    if pct % 10 == 0:
                        fps = (step + 1) / max(1, time.time() - last_update)
                        eta = (total_steps - step - 1) / max(1, fps)
                        self.log_message.emit(tr("worker.progress", str(step+1), str(total_steps), str(pct), str(round(fps)), f"{eta:.1f}"))

                # Data update every 100 steps for chart
                if step % 100 == 0:
                    self.data_update.emit({
                        "time": t, "speed_ref": speed_ref, "speed": sm,
                        "torque": motor.torque,
                        "id": motor.id, "iq": motor.iq,
                        "v_bus": v_bus,
                    })

            # ── Complete ──────────────────────────────────
            self.progress.emit(100)
            self.status.emit("completed")
            self.log_message.emit("")
            self.log_message.emit(tr("worker.results"))
            self.log_message.emit(tr("worker.final_speed", f"{motor.omega_m:.1f}", str(round(motor.omega_m * 60 / (2 * math.pi)))))
            self.log_message.emit(tr("worker.error_pct", f"{abs(motor.omega_m - speed_ref) / max(speed_ref, 1) * 100:.2f}"))
            self.log_message.emit(tr("worker.peak_torque", f"{max(abs(t) for t in data['torque']):.3f}"))
            self.finished.emit(data)

        except Exception as e:
            self.status.emit("error")
            self.error.emit(str(e))


class ScanWorker(QThread):
    """Background thread for parameter scanning.

    Signals:
        progress(int): 0-100 percentage
        log_message(str): log text
        result_ready(dict): scan results on completion
        error(str): error message on failure
    """

    progress = Signal(int)
    log_message = Signal(str)
    result_ready = Signal(dict)
    error = Signal(str)

    def __init__(self, param_key: str, values: list[float],
                 duration: float = 1.0, parent=None):
        super().__init__(parent)
        self._param_key = param_key
        self._values = values
        self._duration = duration
        self._stop_event = threading.Event()

    def stop(self):
        """Request graceful stop (thread-safe)."""
        self._stop_event.set()

    @property
    def _stop_requested(self) -> bool:
        """Backward-compatible property for tests."""
        return self._stop_event.is_set()

    def run(self):
        param_key = self._param_key
        values = self._values
        results = []

        self.log_message.emit(tr("scan.scanning", param_key))
        self.log_message.emit(tr("scan.values", str(values)))
        self.log_message.emit("")

        try:
            for i, val in enumerate(values):
                if self._stop_event.is_set():
                    self.log_message.emit(tr("scan.stopped"))
                    return

                pct = int((i + 1) / len(values) * 100)
                self.progress.emit(pct)
                self.log_message.emit(
                    f"  [{i+1}/{len(values)}] Running {param_key}={val}..."
                )

                motor = PMSMdqModel(
                    Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                    flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                    dt_ns=50000,
                )
                inverter = AverageInverter(48.0)
                cs = CurrentSensor(noise_std=0.05, bias=0.01)
                enc = Encoder(noise_std=0.001)

                speed_ref = val if param_key == "speed" else 100.0
                load = val if param_key == "load" else 0.0
                fkp = val if param_key == "kp_id" else 5.0
                fki = val if param_key == "ki_id" else 500.0

                foc = FOCController(
                    kp_id=fkp if param_key == "kp_id" else 5.0,
                    ki_id=fki if param_key == "ki_id" else 500.0,
                    kp_iq=fkp if param_key == "kp_iq" else 5.0,
                    ki_iq=fki if param_key == "ki_iq" else 500.0,
                    ts=50e-6, v_bus=48.0,
                )
                spd = SpeedController(kp=0.05, ki=0.5, ts=1e-3)

                if param_key == "spd_kp":
                    spd = SpeedController(kp=val, ki=0.5, ts=1e-3)
                elif param_key == "spd_ki":
                    spd = SpeedController(kp=0.05, ki=val, ts=1e-3)

                iq_ref = 0.0
                total_steps = int(self._duration / 50e-6)
                for step in range(total_steps):
                    if self._stop_event.is_set():
                        self.log_message.emit(tr("scan.stopped"))
                        return
                    if step % 20 == 0:
                        iq_ref = spd.update(
                            speed_ref, enc.read_speed(motor.omega_m)
                        )
                    ia_m, ib_m, ic_m = cs.read_abc(
                        motor.ia, motor.ib, motor.ic
                    )
                    th_m = enc.read_angle(motor.theta_e)
                    da, db, dc = foc.update(
                        ia_m, ib_m, ic_m, th_m, 0.0, iq_ref
                    )
                    va, vb, vc = inverter.step(
                        da, db, dc, 48.0, ia_m, ib_m, ic_m
                    )
                    motor.step_abc(va, vb, vc, tl=load, dt=50e-6)
                    motor.update_abc_currents()

                err = abs(motor.omega_m - speed_ref) / max(speed_ref, 1) * 100
                results.append({
                    "value": val,
                    "speed": motor.omega_m,
                    "error": err,
                })
                self.log_message.emit(
                    f"  -> Speed: {motor.omega_m:.1f} rad/s, Error: {err:.2f}%"
                )

            self.progress.emit(100)
            self.result_ready.emit({"param_key": param_key, "results": results})

        except Exception as e:
            self.error.emit(str(e))
