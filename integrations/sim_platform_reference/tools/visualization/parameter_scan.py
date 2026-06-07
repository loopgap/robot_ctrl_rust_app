#!/usr/bin/env python3
"""Parameter Scanner — sweep parameters, batch-run simulations, compare results.

Usage:
    python tools/parameter_scan.py                    # interactive
    python tools/parameter_scan.py --param speed --values 50,100,150  # CLI
    python tools/parameter_scan.py --param kp_id --values 1,5,10
    python tools/parameter_scan.py --list              # available parameters
"""

import os
import sys

_PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJECT_ROOT)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder

OUTPUT_DIR = os.path.join(_PROJECT_ROOT, "output", "scans")
os.makedirs(OUTPUT_DIR, exist_ok=True)

# ── Scanable parameters ────────────────────────────────────

SCANABLE_PARAMS = {
    "speed": {
        "name": "Speed Reference",
        "unit": "rad/s",
        "default_values": [50, 100, 150, 200],
        "path": "speed_ref_value",
    },
    "kp_id": {
        "name": "FOC d-axis Kp",
        "unit": "V/A",
        "default_values": [1.0, 5.0, 20.0],
        "path": "foc_kp_id",
    },
    "ki_id": {
        "name": "FOC d-axis Ki",
        "unit": "V/(A·s)",
        "default_values": [100, 500, 2000],
        "path": "foc_ki_id",
    },
    "kp_iq": {
        "name": "FOC q-axis Kp",
        "unit": "V/A",
        "default_values": [1.0, 5.0, 20.0],
        "path": "foc_kp_iq",
    },
    "ki_iq": {
        "name": "FOC q-axis Ki",
        "unit": "V/(A·s)",
        "default_values": [100, 500, 2000],
        "path": "foc_ki_iq",
    },
    "spd_kp": {
        "name": "Speed Loop Kp",
        "unit": "A·s/rad",
        "default_values": [0.01, 0.05, 0.2],
        "path": "spd_kp",
    },
    "spd_ki": {
        "name": "Speed Loop Ki",
        "unit": "A/rad",
        "default_values": [0.1, 0.5, 2.0],
        "path": "spd_ki",
    },
    "load": {
        "name": "Load Torque",
        "unit": "N·m",
        "default_values": [0, 0.2, 0.5, 1.0],
        "path": "load_torque",
    },
}


def run_single(param_name: str, param_value: float, base_cfg: dict) -> dict:
    """Run a single simulation and return metrics."""
    cfg = dict(base_cfg)
    dt_c = 50e-6
    dt_s = 1e-3
    duration = cfg.get("duration_s", 1.5)
    speed_ratio = int(dt_s / dt_c)

    # Apply parameter override
    if param_name == "speed":
        speed_ref = param_value
        foc_kp_id, foc_ki_id, foc_kp_iq, foc_ki_iq = 5.0, 500.0, 5.0, 500.0
        spd_kp, spd_ki = 0.05, 0.5
        load_tl = 0.0
    elif param_name in ("kp_id", "ki_id", "kp_iq", "ki_iq"):
        speed_ref = 100.0
        foc_kp_id = param_value if param_name == "kp_id" else 5.0
        foc_ki_id = param_value if param_name == "ki_id" else 500.0
        foc_kp_iq = param_value if param_name == "kp_iq" else 5.0
        foc_ki_iq = param_value if param_name == "ki_iq" else 500.0
        spd_kp, spd_ki = 0.05, 0.5
        load_tl = 0.0
    elif param_name == "spd_kp":
        speed_ref = 100.0; spd_kp = param_value; spd_ki = 0.5
        foc_kp_id, foc_ki_id, foc_kp_iq, foc_ki_iq = 5.0, 500.0, 5.0, 500.0
        load_tl = 0.0
    elif param_name == "spd_ki":
        speed_ref = 100.0; spd_kp = 0.05; spd_ki = param_value
        foc_kp_id, foc_ki_id, foc_kp_iq, foc_ki_iq = 5.0, 500.0, 5.0, 500.0
        load_tl = 0.0
    elif param_name == "load":
        speed_ref = 100.0; load_tl = param_value
        foc_kp_id, foc_ki_id, foc_kp_iq, foc_ki_iq = 5.0, 500.0, 5.0, 500.0
        spd_kp, spd_ki = 0.05, 0.5

    # Init models
    _battery = RintBattery(48.0, 0.05)
    inverter = AverageInverter(48.0)
    motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                        flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                        dt_ns=int(dt_c * 1e9))
    cs = CurrentSensor(noise_std=0.05, bias=0.01)
    enc = Encoder(noise_std=0.001)
    foc = FOCController(kp_id=foc_kp_id, ki_id=foc_ki_id,
                        kp_iq=foc_kp_iq, ki_iq=foc_ki_iq,
                        ts=dt_c, v_bus=48.0)
    spd = SpeedController(kp=spd_kp, ki=spd_ki, ts=dt_s)

    total_steps = int(duration / dt_c)
    iq_ref = 0.0

    for step in range(total_steps):
        _t = step * dt_c
        if step % speed_ratio == 0:
            sm = enc.read_speed(motor.omega_m)
            iq_ref = spd.update(speed_ref, sm)

        ia_m, ib_m, ic_m = cs.read_abc(motor.ia, motor.ib, motor.ic)
        th_m = enc.read_angle(motor.theta_e)
        da, db, dc = foc.update(ia_m, ib_m, ic_m, th_m, 0.0, iq_ref)
        va, vb, vc = inverter.step(da, db, dc, 48.0, ia_m, ib_m, ic_m)
        motor.step_abc(va, vb, vc, tl=load_tl, dt=dt_c)
        motor.update_abc_currents()

    # Compute metrics
    final_speed = motor.omega_m
    error_pct = abs(final_speed - speed_ref) / max(speed_ref, 1) * 100
    return {
        "param_name": param_name,
        "param_value": param_value,
        "final_speed": round(final_speed, 1),
        "speed_error_pct": round(error_pct, 2),
        "peak_torque": round(abs(motor.torque), 3),
        "final_iq": round(motor.iq, 3),
        "settled": error_pct < 5.0,
    }


def run_scan(param: str, values: list, base_cfg: dict = None) -> list:
    """Run a parameter scan and return sorted results."""
    base_cfg = base_cfg or {}
    results = []
    total = len(values)
    for i, val in enumerate(values):
        print(f"\r  [{i+1}/{total}] {param}={val}...", end="", flush=True)
        try:
            r = run_single(param, val, base_cfg)
            results.append(r)
        except Exception as e:
            print(f"\n  ⚠ {param}={val} failed: {e}")
    print()
    return sorted(results, key=lambda r: r["param_value"])


def format_scan_report(results: list) -> str:
    """Generate human-readable scan report as a string."""
    if not results:
        return "(no results)"
    param_name = results[0]["param_name"]
    param_info = SCANABLE_PARAMS.get(param_name, {"name": param_name, "unit": ""})
    lines = []

    pheader = f"Parameter Scan: {param_info['name']} ({param_info['unit']})"
    lines.append(f"\n{'=' * (len(pheader) + 4)}")
    lines.append(f"  {pheader}")
    lines.append(f"{'=' * (len(pheader) + 4)}")
    lines.append(f"  {'Value':>10} | {'Final Speed':>12} | {'Error %':>8} | {'Peak Torque':>12} | {'Settled':>8}")
    lines.append(f"  {'─'*10}─┼─{'─'*12}─┼─{'─'*8}─┼─{'─'*12}─┼─{'─'*8}")

    for r in results:
        settled = "(OK)" if r["settled"] else "(--)"
        lines.append(
            f"  {str(r['param_value']):>10} | {r['final_speed']:>8.1f} rad/s | "
            f"{r['speed_error_pct']:>6.2f}% | {r['peak_torque']:>8.3f} N·m | {settled:>6}"
        )

    # Find best
    best = min(results, key=lambda r: r["speed_error_pct"])
    lines.append(f"\n  Best: {param_name}={best['param_value']} "
                 f"(error={best['speed_error_pct']}%, settled={best['settled']})")

    return "\n".join(lines)


def run_scan_and_report(param: str, values: list) -> str:
    """Full workflow: scan → report → return report string."""
    print(f"\n  Scanning {param} with values: {values}")
    results = run_scan(param, values)
    report = format_scan_report(results)
    print(report)

    # Save report
    safe_name = f"scan_{param}_{min(values):g}_{max(values):g}.txt"
    report_path = os.path.join(OUTPUT_DIR, safe_name)
    with open(report_path, "w") as f:
        f.write(report)
    print(f"\n  Report saved: {report_path}")
    return report


def list_params():
    """Print available parameters for scanning."""
    print("\n  Available scan parameters:\n")
    print(f"  {'Name':<12} {'Description':<30} {'Unit':<12} {'Example Values'}")
    print(f"  {'─'*12}─┼─{'─'*30}─┼─{'─'*12}─┼─{'─'*30}")
    for key, info in SCANABLE_PARAMS.items():
        vals = ", ".join(str(v) for v in info["default_values"][:3])
        print(f"  {key:<12} | {info['name']:<28} | {info['unit']:<10} | {vals}...")
    print()


def main():
    import argparse
    ap = argparse.ArgumentParser(description="Parameter Scanner for sim_platform")
    ap.add_argument("--list", action="store_true", help="List available parameters")
    ap.add_argument("--param", type=str, default=None, help="Parameter to scan")
    ap.add_argument("--values", type=str, default=None,
                    help="Comma-separated values (e.g. 50,100,150)")
    args = ap.parse_args()

    if args.list:
        list_params()
        return

    if args.param and args.values:
        param = args.param.lower()
        if param not in SCANABLE_PARAMS:
            print(f"Unknown param '{param}'. Use --list to see available.")
            return
        values = [float(v.strip()) for v in args.values.split(",")]
        run_scan_and_report(param, values)
        return

    # Interactive mode
    list_params()
    param = input("  Parameter to scan: ").strip().lower()
    if param not in SCANABLE_PARAMS:
        print(f"Unknown: {param}")
        return
    info = SCANABLE_PARAMS[param]
    vals_input = input(f"  Values (default: {','.join(str(v) for v in info['default_values'])}): ").strip()
    values = [float(v.strip()) for v in vals_input.split(",")] if vals_input else info["default_values"]
    run_scan_and_report(param, values)


if __name__ == "__main__":
    main()
