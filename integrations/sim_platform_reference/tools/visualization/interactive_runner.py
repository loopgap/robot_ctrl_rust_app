#!/usr/bin/env python3
"""Guided Interactive Runner — 3-step simulation for everyone.

Usage:
    python -m sim_platform.tools.visualization.interactive_runner

This is the recommended entry point for new team members.
Walks through: scenario selection → parameter tuning → run → visualize.
"""

import math
import os
import shutil
import sys

# Add project root
_PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJECT_ROOT)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder
from sim_platform.tools.replay.hdf5_logger import HDF5Logger
from sim_platform.tools.visualization.plot_log import plot_foc_results

# FaultInjector imported lazily in step3_run to avoid tools→verification dependency

OUTPUT_DIR = os.path.join(_PROJECT_ROOT, "output")
os.makedirs(OUTPUT_DIR, exist_ok=True)

# ── UI helpers ──────────────────────────────────────────────

RESET = "\033[0m"
BOLD = "\033[1m"
DIM = "\033[2m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
RED = "\033[91m"
CYAN = "\033[96m"
MAGENTA = "\033[95m"


def clr():
    """Clear terminal using ANSI escape (avoids os.system CWE-78 risk)."""
    print("\033[2J\033[H", end="", flush=True)


def header():
    clr()
    w = shutil.get_terminal_size().columns
    print(f"{CYAN}{'='*w}{RESET}")
    print(f"{BOLD}{GREEN}  ⚡ sim_platform — PMSM FOC Interactive Runner{RESET}")
    print(f"{DIM}  3 steps to a running simulation | v1.0{RESET}")
    print(f"{CYAN}{'='*w}{RESET}\n")


def ask(question: str, default: str = "") -> str:
    """Ask user a question with optional default."""
    if default:
        resp = input(f"  {question} [{default}]: ").strip()
        return resp if resp else default
    return input(f"  {question}: ").strip()


def ask_float(question: str, default: float) -> float:
    while True:
        raw = ask(question, str(default))
        try:
            v = float(raw)
            if math.isnan(v) or math.isinf(v):
                print(f"  {RED}Invalid number{RESET}")
                continue
            return v
        except ValueError:
            print(f"  {RED}Please enter a number{RESET}")


def ask_choice(question: str, options: list) -> str:
    print(f"\n  {BOLD}{question}{RESET}")
    for i, opt in enumerate(options, 1):
        print(f"    {GREEN}{i}{RESET}. {opt}")
    while True:
        raw = input(f"  Pick [1-{len(options)}]: ").strip()
        try:
            idx = int(raw) - 1
            if 0 <= idx < len(options):
                return options[idx]
        except ValueError:
            pass
        print(f"  {RED}Invalid choice{RESET}")


def print_table(rows: list, headers: list, title: str = ""):
    """Print a formatted table."""
    if title:
        print(f"\n  {BOLD}{title}{RESET}")
    # Calculate column widths
    cols = len(headers)
    col_w = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            col_w[i] = max(col_w[i], len(str(cell)))
    # Header separator
    sep = "  " + "─" * (sum(col_w) + 3 * (cols - 1)) + "  "
    # Print header
    header_line = "  " + " │ ".join(h.ljust(col_w[i]) for i, h in enumerate(headers))
    print(header_line)
    print(sep)
    # Print rows
    for row in rows:
        line = "  " + " │ ".join(str(cell).ljust(col_w[i]) for i, cell in enumerate(row))
        print(line)
    print()


# ══════════════════════════════════════════════════════════════
#  STEP 1: Scenario Selection
# ══════════════════════════════════════════════════════════════

SCENARIOS = {
    "1": {
        "name": "Speed Step Response (955 rpm)",
        "desc": "Step speed reference 0→100 rad/s, measure settling time",
        "params": {"duration_s": 1.5, "speed_ref_value": 100.0, "profile": "step"},
    },
    "2": {
        "name": "Ramp Acceleration Test",
        "desc": "Smooth ramp from 0→100 rad/s at 200 rad/s²",
        "params": {"duration_s": 1.5, "speed_ref_value": 100.0, "profile": "ramp"},
    },
    "3": {
        "name": "Load Torque Disturbance",
        "desc": "Constant speed with sudden load torque at t=0.5s",
        "params": {"duration_s": 2.0, "speed_ref_value": 100.0, "profile": "step",
                   "load_torque": 0.3},
    },
    "4": {
        "name": "Voltage Sag Fault Ride-Through",
        "desc": "Bus voltage drops 20V at t=0.6s for 100ms — test fault recovery",
        "params": {"duration_s": 2.0, "speed_ref_value": 100.0, "profile": "step",
                   "fault_sag": True},
    },
    "5": {
        "name": "Full Custom Setup",
        "desc": "You configure everything manually",
        "params": {},
    },
}


def step1_select_scenario() -> dict:
    header()
    print(f"  {BOLD}STEP 1/3: Choose a Scenario{RESET}\n")
    SCENARIOS_LIST = [
        (k, v["name"], v["desc"]) for k, v in SCENARIOS.items()
    ]
    print_table(SCENARIOS_LIST, ["#", "Scenario", "Description"], "Available Scenarios")

    while True:
        choice = input(f"  {GREEN}Select{RESET} [1-5]: ").strip()
        if choice in SCENARIOS:
            sel = SCENARIOS[choice]
            print(f"  → {BOLD}{sel['name']}{RESET}")
            return sel
        print(f"  {RED}Pick a number 1-5{RESET}")


# ══════════════════════════════════════════════════════════════
#  STEP 2: Parameter Tuning
# ══════════════════════════════════════════════════════════════

MOTOR_PRESETS = {
    "1": {"name": "Small PMSM (~200W drone motor)",
          "params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                     "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4}},
    "2": {"name": "Medium PMSM (~2kW e-bike motor)",
          "params": {"Rs": 0.05, "Ld": 0.2e-3, "Lq": 0.4e-3,
                     "flux_pm": 0.05, "J": 0.005, "B": 0.0005, "Pp": 4}},
    "3": {"name": "Large PMSM (~20kW EV traction)",
          "params": {"Rs": 0.01, "Ld": 0.1e-3, "Lq": 0.2e-3,
                     "flux_pm": 0.12, "J": 0.05, "B": 0.001, "Pp": 3}},
}


def step2_tune_parameters(scenario: dict) -> dict:
    header()
    print(f"  {BOLD}STEP 2/3: Configure Simulation{RESET}\n")

    params = dict(scenario.get("params", {}))
    is_custom = scenario["name"] == "Full Custom Setup"

    # ── Motor preset or custom ──
    print(f"  {BOLD}Motor Preset:{RESET}")
    preset_list = [(k, v["name"]) for k, v in MOTOR_PRESETS.items()]
    print_table(preset_list, ["#", "Motor Type"])
    motor_choice = ask("Select motor preset", "1")
    if motor_choice in MOTOR_PRESETS:
        motor_p = MOTOR_PRESETS[motor_choice]["params"]
    else:
        motor_p = MOTOR_PRESETS["1"]["params"]

    if is_custom:
        print(f"\n  {DIM}Custom Parameters (press Enter to keep default){RESET}")
        motor_p = {}
        for key, unit, default in [
            ("Rs", "ohm", 0.1), ("Ld", "mH", 0.5), ("Lq", "mH", 1.0),
            ("flux_pm", "Wb", 0.03), ("J", "kg·m²", 0.001),
            ("B", "N·m·s/rad", 0.0001), ("Pp", "", 4)
        ]:
            if key in ("Ld", "Lq"):
                val = ask_float(f"  {key} [{unit}]", default) * 1e-3
            else:
                val = ask_float(f"  {key} [{unit}]", default)
            motor_p[key] = val

    # ── Controller gains ──
    print(f"\n  {BOLD}Controller Gains:{RESET}")
    if is_custom:
        foc_kp_id = ask_float("  FOC kp_id", 5.0)
        foc_ki_id = ask_float("  FOC ki_id", 500.0)
        foc_kp_iq = ask_float("  FOC kp_iq", 5.0)
        foc_ki_iq = ask_float("  FOC ki_iq", 500.0)
        spd_kp = ask_float("  Speed kp", 0.05)
        spd_ki = ask_float("  Speed ki", 0.5)
    else:
        foc_kp_id, foc_ki_id, foc_kp_iq, foc_ki_iq = 5.0, 500.0, 5.0, 500.0
        spd_kp, spd_ki = 0.05, 0.5
        print(f"  FOC: kp={foc_kp_id}, ki={foc_ki_id} | Speed: kp={spd_kp}, ki={spd_ki}")
        if ask("  Tune gains?", "n") in ("y", "yes"):
            foc_kp_id = ask_float("    FOC kp_id", 5.0)
            foc_ki_id = ask_float("    FOC ki_id", 500.0)
            spd_kp = ask_float("    Speed kp", 0.05)
            spd_ki = ask_float("    Speed ki", 0.5)

    # ── Speed reference ──
    if "speed_ref_value" not in params:
        params["speed_ref_value"] = ask_float("Speed reference [rad/s]", 100.0)
    if "duration_s" not in params:
        params["duration_s"] = ask_float("Simulation duration [s]", 2.0)

    return {
        "motor_params": motor_p,
        "foc": {"kp_id": foc_kp_id, "ki_id": foc_ki_id,
                "kp_iq": foc_kp_iq, "ki_iq": foc_ki_iq},
        "speed_pi": {"kp": spd_kp, "ki": spd_ki},
        "duration_s": params.get("duration_s", 1.5),
        "speed_ref_value": params.get("speed_ref_value", 100.0),
        "profile": params.get("profile", "step"),
        "load_torque": params.get("load_torque", 0.0),
        "fault_sag": params.get("fault_sag", False),
    }


# ══════════════════════════════════════════════════════════════
#  STEP 3: Run & Visualize
# ══════════════════════════════════════════════════════════════

def step3_run(cfg: dict):
    header()
    print(f"  {BOLD}STEP 3/3: Running Simulation{RESET}\n")

    # Print config summary
    print(f"  {BOLD}Configuration:{RESET}")
    print(f"    Motor: {cfg['motor_params']}")
    print(f"    FOC:   kp_id={cfg['foc']['kp_id']}, ki_id={cfg['foc']['ki_id']}")
    print(f"    Speed: {cfg['speed_ref_value']} rad/s, {cfg['duration_s']}s")
    print()

    # ── Time parameters ──────────────────────────────────
    dt_current_s = 50e-6  # 20kHz
    dt_speed_s = 1e-3     # 1kHz
    duration_s = cfg["duration_s"]
    speed_ratio = int(dt_speed_s / dt_current_s)

    # ── Initialize models ─────────────────────────────────
    battery = RintBattery(v_oc=48.0, r_int=0.05)
    inverter = AverageInverter(v_bus=48.0)
    motor = PMSMdqModel(**cfg["motor_params"], dt_ns=int(dt_current_s * 1e9))
    csensor = CurrentSensor(noise_std=0.1, bias=0.01)
    encoder = Encoder(noise_std=0.001,
                      quantization=2 * math.pi / 4096)
    foc = FOCController(
        kp_id=cfg["foc"]["kp_id"], ki_id=cfg["foc"]["ki_id"],
        kp_iq=cfg["foc"]["kp_iq"], ki_iq=cfg["foc"]["ki_iq"],
        ts=dt_current_s, v_bus=48.0)
    speed_ctrl = SpeedController(
        kp=cfg["speed_pi"]["kp"], ki=cfg["speed_pi"]["ki"],
        ts=dt_speed_s)

    # ── Fault injection (lazy import to avoid tools→verification dependency) ──
    from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector
    injector = FaultInjector()
    if cfg["fault_sag"]:
        injector.add_fault(FaultConfig(
            fault_id="bus_sag", fault_type="BIAS",
            target_path="power://v_bus", magnitude=-20.0,
            start_time_s=duration_s * 0.3, duration_s=0.1))

    # ── Data collection ──────────────────────────────────
    log_data = {
        "time": [], "speed_ref": [], "speed": [],
        "id": [], "iq": [], "ia": [], "ib": [], "ic": [],
        "torque": [], "duty_a": [], "duty_b": [], "duty_c": [],
        "vd": [], "vq": [], "v_bus": [],
    }
    total_steps = int(duration_s / dt_current_s)

    # ── Speed reference function ─────────────────────────
    def compute_speed_ref(t):
        target = cfg["speed_ref_value"]
        profile = cfg["profile"]
        if profile == "ramp":
            return min(t * 200, target)
        return target

    # ── Progress bar ─────────────────────────────────────
    print(f"  Simulating {total_steps} steps ({dt_current_s*1e6:.0f}us each)...")

    iq_ref = 0.0
    for step in range(total_steps):
        t = step * dt_current_s

        # Progress: show bar every 5%
        if step % max(1, total_steps // 20) == 0:
            pct = int(step / total_steps * 100)
            bar = "█" * (pct // 5) + "░" * (20 - pct // 5)
            print(f"\r  [{bar}] {pct}%", end="", flush=True)

        # Speed loop
        if step % speed_ratio == 0:
            speed_meas = encoder.read_speed(motor.omega_m)
            speed_ref = compute_speed_ref(t)
            iq_ref = speed_ctrl.update(speed_ref, speed_meas)
        else:
            speed_ref = compute_speed_ref(t)

        # Inject load torque if configured
        load_tl = cfg["load_torque"] if t >= 0.5 and cfg["load_torque"] > 0 else 0.0

        # Current measurement + FOC
        ia_m, ib_m, ic_m = csensor.read_abc(motor.ia, motor.ib, motor.ic)
        th_m = encoder.read_angle(motor.theta_e)
        duty_a, duty_b, duty_c = foc.update(
            ia_m, ib_m, ic_m, th_m, id_ref=0.0, iq_ref=iq_ref)

        # Fault injection
        injector.activate_at(t)
        v_bus = injector.apply("power://v_bus", battery.v_oc, t)

        # Inverter + Motor
        va, vb, vc = inverter.step(duty_a, duty_b, duty_c, v_bus,
                                   ia_m, ib_m, ic_m)
        motor.step_abc(va, vb, vc, tl=load_tl, dt=dt_current_s)
        motor.update_abc_currents()

        # Log
        log_data["time"].append(t)
        log_data["speed_ref"].append(speed_ref)
        log_data["speed"].append(speed_meas)
        log_data["id"].append(motor.id)
        log_data["iq"].append(motor.iq)
        log_data["ia"].append(motor.ia)
        log_data["ib"].append(motor.ib)
        log_data["ic"].append(motor.ic)
        log_data["torque"].append(motor.torque)
        log_data["duty_a"].append(duty_a)
        log_data["duty_b"].append(duty_b)
        log_data["duty_c"].append(duty_c)
        log_data["vd"].append(foc.vd_ref)
        log_data["vq"].append(foc.vq_ref)
        log_data["v_bus"].append(v_bus)

    print(f"\r  [{'█'*20}] 100%")

    # ── Results ──────────────────────────────────────────
    print(f"\n  {BOLD}Results:{RESET}")
    final_speed = log_data["speed"][-1]
    target_speed = cfg["speed_ref_value"]
    error_pct = abs(final_speed - target_speed) / max(target_speed, 1) * 100
    max_torque = max(abs(t) for t in log_data["torque"])
    settling_time = 0.0
    for i, s in enumerate(log_data["speed"]):
        if abs(s - target_speed) < target_speed * 0.02:
            settling_time = log_data["time"][i]
            break

    result_rows = [
        ("Final Speed", f"{final_speed:.1f} rad/s ({final_speed*60/(2*math.pi):.0f} rpm)"),
        ("Speed Error", f"{error_pct:.2f}%"),
        ("Peak Torque", f"{max_torque:.2f} N·m"),
        ("Settling Time", f"{settling_time*1000:.0f} ms ({'did not settle' if settling_time==0 else 'settled'})"),
        ("Total Steps", str(total_steps)),
    ]
    print_table(result_rows, ["Metric", "Value"], "Performance")

    # ── Save & Plot ──────────────────────────────────────
    output_name = f"run_{int(math.floor(cfg['speed_ref_value']))}rads.h5"
    output_path = os.path.join(OUTPUT_DIR, output_name)

    print(f"  Saving → {output_name}")
    with HDF5Logger(output_path) as log:
        for i, t in enumerate(log_data["time"]):
            log.record(t,
                       speed_ref=log_data["speed_ref"][i],
                       speed=log_data["speed"][i],
                       id=log_data["id"][i], iq=log_data["iq"][i],
                       ia=log_data["ia"][i], ib=log_data["ib"][i], ic=log_data["ic"][i],
                       torque=log_data["torque"][i],
                       duty_a=log_data["duty_a"][i], duty_b=log_data["duty_b"][i],
                       duty_c=log_data["duty_c"][i],
                       vd=log_data["vd"][i], vq=log_data["vq"][i],
                       v_bus=log_data["v_bus"][i])

    plot_path = os.path.join(OUTPUT_DIR, output_name.replace(".h5", ".png"))
    print(f"  Plot   → {os.path.basename(plot_path)}")
    try:
        plot_foc_results(log_data, plot_path,
                         title=f"PMSM FOC — Speed={cfg['speed_ref_value']:.0f} rad/s")
        print(f"  {GREEN}✅ Plot generated{RESET}")
    except Exception as e:
        print(f"  {YELLOW}⚠ Plot error: {e} (install matplotlib){RESET}")

    # ── Summary ──────────────────────────────────────────
    print()
    print(f"  {BOLD}{GREEN}═══ Simulation Complete ═══{RESET}")
    print(f"  Speed:    {final_speed:.1f} rad/s (error {error_pct:.1f}%)")
    print(f"  Torque:   {max_torque:.1f} N·m peak")
    print(f"  Output:   {os.path.basename(output_path)}")
    print()


# ══════════════════════════════════════════════════════════════
#  MAIN
# ══════════════════════════════════════════════════════════════

def welcome():
    """Print welcome banner with quick start info."""
    header()
    print(f"  {BOLD}Quick Start:{RESET}")
    print("  1. Select a pre-built scenario")
    print("  2. Configure motor/controller parameters")
    print("  3. Run and review results")
    print(f"\n  {DIM}Also available:{RESET}")
    print("    python examples/pmsm_foc_mvp/main.py --help    # CLI mode")
    print("    python tools/parameter_scan.py                  # parameter sweeps")
    print()


def main():
    try:
        welcome()
        input(f"  Press {GREEN}Enter{RESET} to begin...")

        # Step 1
        scenario = step1_select_scenario()
        input(f"\n  Press {GREEN}Enter{RESET} to configure...")

        # Step 2
        cfg = step2_tune_parameters(scenario)
        input(f"\n  Press {GREEN}Enter{RESET} to run...")

        # Step 3
        step3_run(cfg)

        # Next steps
        print(f"  {BOLD}What next?{RESET}")
        print(f"    {GREEN}1{RESET}. Run again with different parameters")
        print(f"    {GREEN}2{RESET}. Try parameter scanning (python tools/parameter_scan.py)")
        print(f"    {GREEN}3{RESET}. Check architecture docs (.workbuddy/artifacts/)")
        print(f"    {GREEN}4{RESET}. Exit")
        again = input(f"\n  [{GREEN}1{RESET}|2|3|4]: ").strip()
        if again in ("2", "3", "4"):
            print(f"\n  {GREEN}Thanks for using sim_platform!{RESET}")
        else:
            main()
    except KeyboardInterrupt:
        print(f"\n\n  {YELLOW}Exited.{RESET}")
    except Exception as e:
        print(f"\n  {RED}Error: {e}{RESET}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
