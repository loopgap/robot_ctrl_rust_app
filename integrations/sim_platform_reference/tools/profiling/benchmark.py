"""Performance benchmarking and profiling tool.

Usage:
    python tools/profiling/benchmark.py
    python tools/profiling/benchmark.py --profile
"""

import cProfile
import io
import os
import pstats
import sys
import time

# Add project root to path
_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder


def run_simulation(steps: int = 10000) -> dict:
    """Run a standard PMSM+FOC simulation and return timing data."""
    dt_c = 50e-6
    dt_s = 1e-3
    speed_ratio = int(dt_s / dt_c)

    # Init models
    _battery = RintBattery(48.0, 0.05)
    inverter = AverageInverter(48.0)
    motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                        flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                        dt_ns=int(dt_c * 1e9))
    csensor = CurrentSensor(noise_std=0.1, bias=0.01)
    encoder = Encoder(noise_std=0.001)
    foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                        ts=dt_c, v_bus=48.0)
    spd = SpeedController(kp=0.05, ki=0.5, ts=dt_s)

    speed_ref = 100.0
    iq_ref = 0.0

    start = time.perf_counter()

    for step in range(steps):
        # Speed loop
        if step % speed_ratio == 0:
            sm = encoder.read_speed(motor.omega_m)
            iq_ref = spd.update(speed_ref, sm)

        # FOC
        ia_m, ib_m, ic_m = csensor.read_abc(motor.ia, motor.ib, motor.ic)
        th_m = encoder.read_angle(motor.theta_e)
        da, db, dc = foc.update(ia_m, ib_m, ic_m, th_m, 0.0, iq_ref)
        va, vb, vc = inverter.step(da, db, dc, 48.0, ia_m, ib_m, ic_m)
        motor.step_abc(va, vb, vc, tl=0.0, dt=dt_c)
        motor.update_abc_currents()

    elapsed = time.perf_counter() - start

    return {
        "steps": steps,
        "elapsed_s": elapsed,
        "steps_per_sec": steps / elapsed,
        "final_speed": motor.omega_m,
    }


def profile_simulation(steps: int = 10000) -> str:
    """Profile simulation and return formatted report."""
    pr = cProfile.Profile()
    pr.enable()

    result = run_simulation(steps)

    pr.disable()

    # Capture stats
    s = io.StringIO()
    ps = pstats.Stats(pr, stream=s).sort_stats("cumulative")
    ps.print_stats(20)  # Top 20 functions

    report = f"""
=== Performance Profile ===
Steps: {result['steps']}
Elapsed: {result['elapsed_s']:.3f}s
Speed: {result['steps_per_sec']:.0f} steps/sec
Final motor speed: {result['final_speed']:.1f} rad/s

=== Top 20 Functions (by cumulative time) ===
{s.getvalue()}
"""
    return report


def benchmark_optimizations():
    """Compare performance of different approaches."""
    print("=" * 60)
    print("sim_platform Performance Benchmark")
    print("=" * 60)

    # Warmup
    print("\n[Warmup]")
    run_simulation(1000)

    # Benchmark different sizes
    for steps in [1000, 5000, 10000, 20000]:
        result = run_simulation(steps)
        print(f"\n[{steps} steps]")
        print(f"  Elapsed: {result['elapsed_s']:.3f}s")
        print(f"  Throughput: {result['steps_per_sec']:.0f} steps/sec")
        print(f"  Final speed: {result['final_speed']:.1f} rad/s")


if __name__ == "__main__":
    import sys

    if "--profile" in sys.argv:
        print(profile_simulation(10000))
    else:
        benchmark_optimizations()
