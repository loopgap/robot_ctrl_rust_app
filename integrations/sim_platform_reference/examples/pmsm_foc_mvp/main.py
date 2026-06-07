#!/usr/bin/env python3
"""PMSM+FOC MVP — Minimal Viable Prototype Simulation.

Demonstrates the complete closed-loop simulation chain:
  Battery → Inverter → PMSM → Sensor Noise → FOC → Speed PI → PWM

Usage:
    python examples/pmsm_foc_mvp/main.py
    python examples/pmsm_foc_mvp/main.py --config examples/pmsm_foc_mvp/config.yaml
    python examples/pmsm_foc_mvp/main.py --duration 2.0 --output results.h5
"""

import argparse
import math
import os
import sys

# Add project root to path (security: absolute path only, no user control)
_PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
_PARENT_ROOT = os.path.abspath(os.path.join(_PROJECT_ROOT, ".."))
sys.path.insert(0, _PARENT_ROOT)

# SECURITY: Output path sandbox (CWE-22)
_ALLOWED_OUTPUT_DIR = os.path.join(_PROJECT_ROOT, "output")
os.makedirs(_ALLOWED_OUTPUT_DIR, exist_ok=True)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder
from sim_platform.tools.replay.hdf5_logger import HDF5Logger
from sim_platform.tools.visualization.plot_log import plot_foc_results

# ── Default Parameters ──────────────────────────────────────

DEFAULT_CONFIG = {
    "simulation": {
        "duration_s": 1.0,
        "current_step_us": 50,    # 50us = 20kHz PWM
        "speed_step_us": 1000,     # 1ms speed loop
        "output_file": "foc_mvp_output.h5",
    },
    "battery": {"v_oc": 48.0, "r_int": 0.05},
    "motor": {
        "Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
        "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4,
    },
    "foc": {
        "kp_id": 5.0, "ki_id": 500.0,
        "kp_iq": 5.0, "ki_iq": 500.0,
    },
    "speed_pi": {"kp": 0.05, "ki": 0.5},
    "sensor": {
        "current_noise_std": 0.1, "current_bias": 0.01,
        "encoder_noise_std": 0.001, "encoder_quant_bits": 12,
    },
    "scenario": {
        "speed_ref_profile": "step",  # step | ramp | sine
        "speed_ref_value": 100.0,      # rad/s target
        "speed_ref_ramp_rate": 200.0,  # rad/s²
    },
}


def parse_args():
    p = argparse.ArgumentParser(description="PMSM FOC MVP Simulation")
    p.add_argument("--duration", type=float, default=None,
                   help="Sim duration [s] (must be > 0, max 1e6)")
    p.add_argument("--output", type=str, default=None,
                   help="Output filename (saved to sim_platform/output/)")
    p.add_argument("--speed", type=float, default=None,
                   help="Speed reference [rad/s] (must be > 0, max 1e6)")
    p.add_argument("--no-plot", action="store_true", help="Skip visualization")
    p.add_argument("--fault", action="store_true", help="Enable fault injection demo")
    args = p.parse_args()
    # SECURITY: Validate float params (CWE-20)
    if args.duration is not None and (math.isnan(args.duration) or
                                       math.isinf(args.duration) or
                                       args.duration <= 0 or args.duration > 1e6):
        p.error(f"Invalid duration: {args.duration}")
    if args.speed is not None and (math.isnan(args.speed) or
                                    math.isinf(args.speed) or
                                    args.speed <= 0 or args.speed > 1e6):
        p.error(f"Invalid speed: {args.speed}")
    if args.output is not None:
        # SECURITY: Reject path traversal (CWE-22)
        basename = os.path.basename(args.output)
        if basename != args.output:
            p.error(f"Invalid output path (path traversal not allowed): {args.output}")
        # Must end with .h5
        if not args.output.endswith(".h5"):
            args.output += ".h5"
    return args


def load_config(args) -> dict:
    cfg = DEFAULT_CONFIG.copy()
    if args.duration:
        cfg["simulation"]["duration_s"] = args.duration
    if args.output:
        cfg["simulation"]["output_file"] = args.output
    if args.speed is not None:
        cfg["scenario"]["speed_ref_value"] = args.speed
    return cfg


def compute_speed_ref(t: float, cfg: dict) -> float:
    """Generate speed reference trajectory."""
    sc = cfg["scenario"]
    profile = sc["speed_ref_profile"]
    target = sc["speed_ref_value"]

    if profile == "step":
        return target
    elif profile == "ramp":
        rate = sc["speed_ref_ramp_rate"]
        return min(t * rate, target)
    elif profile == "sine":
        import math
        return target * math.sin(2 * math.pi * t)
    return target


def main():
    args = parse_args()
    cfg = load_config(args)
    sim_cfg = cfg["simulation"]

    # ── Time parameters ──────────────────────────────────
    dt_current_s = sim_cfg["current_step_us"] * 1e-6
    dt_speed_s = sim_cfg["speed_step_us"] * 1e-6
    duration_s = sim_cfg["duration_s"]
    speed_ratio = int(dt_speed_s / dt_current_s)

    # ── Initialize models ─────────────────────────────────
    battery = RintBattery(v_oc=cfg["battery"]["v_oc"],
                          r_int=cfg["battery"]["r_int"])
    inverter = AverageInverter(v_bus=battery.v_oc)
    motor = PMSMdqModel(**cfg["motor"], dt_ns=int(dt_current_s * 1e9))
    csensor = CurrentSensor(
        noise_std=cfg["sensor"]["current_noise_std"],
        bias=cfg["sensor"]["current_bias"])
    encoder = Encoder(
        noise_std=cfg["sensor"]["encoder_noise_std"],
        quantization=2 * 3.1415926535 / (2 ** cfg["sensor"]["encoder_quant_bits"]))
    foc = FOCController(
        kp_id=cfg["foc"]["kp_id"], ki_id=cfg["foc"]["ki_id"],
        kp_iq=cfg["foc"]["kp_iq"], ki_iq=cfg["foc"]["ki_iq"],
        ts=dt_current_s, v_bus=battery.v_oc)
    speed_ctrl = SpeedController(
        kp=cfg["speed_pi"]["kp"], ki=cfg["speed_pi"]["ki"],
        ts=dt_speed_s)

    # ── Fault injection (lazy import: avoids coupling verification into core) ──
    from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector
    injector = FaultInjector()
    if args.fault:
        injector.add_fault(FaultConfig(
            fault_id="bus_sag",
            fault_type="BIAS",
            target_path="power://v_bus",
            magnitude=-20.0,   # 20V sag
            start_time_s=duration_s * 0.4,
            duration_s=0.1,
        ))

    # ── Data collection ──────────────────────────────────
    log_data = {
        "time": [], "speed_ref": [], "speed": [],
        "id": [], "iq": [], "ia": [], "ib": [], "ic": [],
        "torque": [], "duty_a": [], "duty_b": [], "duty_c": [],
        "vd": [], "vq": [], "v_bus": [],
    }

    # ── Main simulation loop ─────────────────────────────
    total_steps = int(duration_s / dt_current_s)
    print(f"[PMSM FOC MVP] Simulating {duration_s:.1f}s at {1/dt_current_s:.0f}Hz...")
    print(f"   Motor: Rs={motor.Rs}Ω Ld={motor.Ld*1e3:.2f}mH Lq={motor.Lq*1e3:.2f}mH "
          f"Flux={motor.flux_pm*1e3:.1f}mWb J={motor.J*1e3:.1f}g·m² Pp={motor.Pp}")
    print(f"   Speed Ref: {cfg['scenario']['speed_ref_value']:.0f} rad/s "
          f"({cfg['scenario']['speed_ref_value']*60/(2*3.14159):.0f} rpm)")

    for step in range(total_steps):
        t = step * dt_current_s

        # ── Speed loop (every speed_ratio steps) ─────────
        if step % speed_ratio == 0:
            speed_meas = encoder.read_speed(motor.omega_m)
            speed_ref = compute_speed_ref(t, cfg)
            iq_ref = speed_ctrl.update(speed_ref, speed_meas)
        else:
            speed_meas = motor.omega_m  # use latest for logging
            speed_ref = compute_speed_ref(t, cfg)
            iq_ref = speed_ctrl.pi.prev_output  # hold last

        # ── Current measurement ──────────────────────────
        ia_meas, ib_meas, ic_meas = csensor.read_abc(
            motor.ia, motor.ib, motor.ic)
        theta_meas = encoder.read_angle(motor.theta_e)

        # ── FOC current control ──────────────────────────
        duty_a, duty_b, duty_c = foc.update(
            ia_meas, ib_meas, ic_meas, theta_meas,
            id_ref=0.0, iq_ref=iq_ref)

        # ── Fault injection ──────────────────────────────
        injector.activate_at(t)
        v_bus = injector.apply("power://v_bus", battery.v_oc, t)

        # ── Inverter + Motor ─────────────────────────────
        va, vb, vc = inverter.step(duty_a, duty_b, duty_c,
                                   v_bus, ia_meas, ib_meas, ic_meas)
        motor.step_abc(va, vb, vc, tl=0.0, dt=dt_current_s)
        motor.update_abc_currents()

        # ── Log data ─────────────────────────────────────
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

    # ── Save HDF5 ────────────────────────────────────────
    output_name = sim_cfg.get("output_file", "foc_mvp_output.h5")
    # SECURITY: Output path sandboxed to _ALLOWED_OUTPUT_DIR (CWE-22)
    output_file = os.path.join(_ALLOWED_OUTPUT_DIR, os.path.basename(output_name))
    print(f"[SAVE] Results -> {output_file}")
    with HDF5Logger(output_file) as log:
        for i, t in enumerate(log_data["time"]):
            log.record(t,
                       speed_ref=log_data["speed_ref"][i],
                       speed=log_data["speed"][i],
                       id=log_data["id"][i],
                       iq=log_data["iq"][i],
                       ia=log_data["ia"][i],
                       ib=log_data["ib"][i],
                       ic=log_data["ic"][i],
                       torque=log_data["torque"][i],
                       duty_a=log_data["duty_a"][i],
                       duty_b=log_data["duty_b"][i],
                       duty_c=log_data["duty_c"][i],
                       vd=log_data["vd"][i],
                       vq=log_data["vq"][i],
                       v_bus=log_data["v_bus"][i])

    # ── Report ───────────────────────────────────────────
    final_speed = log_data["speed"][-1]
    target_speed = cfg["scenario"]["speed_ref_value"]
    speed_error_pct = abs(final_speed - target_speed) / max(target_speed, 1) * 100
    max_torque = max(abs(t) for t in log_data["torque"])

    print("\n[RESULTS]")
    print(f"   Final Speed: {final_speed:.1f} rad/s ({final_speed*60/(2*3.14159):.0f} rpm)")
    print(f"   Target:      {target_speed:.1f} rad/s")
    print(f"   Error:       {speed_error_pct:.2f}%")
    print(f"   Peak Torque: {max_torque:.2f} N.m")
    print(f"   Steps:       {total_steps}")

    # ── Plot ─────────────────────────────────────────────
    if not args.no_plot:
        plot_path = output_file.replace(".h5", ".png")
        print(f"\n[PLOT] Generating: {plot_path}")
        plot_foc_results(log_data, plot_path,
                         title=f"PMSM FOC MVP - Speed Ref={target_speed:.0f} rad/s")

    print("\n[DONE] Simulation complete!")


if __name__ == "__main__":
    main()
