"""High-level QuickSim API for sim_platform.

Provides one-liner simulation methods that hide all the setup complexity:
- QuickSim.pmsm_foc(): PMSM with FOC control
- QuickSim.bldc(): BLDC with 6-step control
- QuickSim.compare(): side-by-side motor comparison
- QuickSim.sweep(): parameter sweep

Usage:
    from sim_platform.quicksim import QuickSim

    # One-liner PMSM FOC simulation
    result = QuickSim.pmsm_foc(speed_ref=100.0, duration=1.5)

    # Compare motor types
    comparison = QuickSim.compare(speed_ref=100.0)

    # Sweep speed values
    sweep_results = QuickSim.sweep(speed_values=[50, 100, 150])
"""

from __future__ import annotations

import time
from typing import Any

_DEFAULT_SPEED_REF = 100.0  # rad/s (~955 RPM)
_DEFAULT_DURATION = 1.5  # seconds
_DEFAULT_V_BUS = 48.0  # V
_DEFAULT_SPEED_RATIO = 20  # control:simulation step ratio
_DEFAULT_WINDING_R = 0.1  # Ohm
_DEFAULT_I_MAX = 200.0  # A
_DT_C = 50e-6  # Control time step [s]
_DT_SPEED = 1e-3  # Speed loop time step [s]


class QuickSim:
    """High-level simulation API."""

    @staticmethod
    def pmsm_foc(
        speed_ref: float = _DEFAULT_SPEED_REF,
        duration: float = _DEFAULT_DURATION,
        **kwargs: Any,
    ) -> dict:
        """Run a PMSM FOC simulation and return results.

        Args:
            speed_ref: Target speed in rad/s (default 100 rad/s ~ 955 RPM).
            duration: Simulation duration in seconds.
            **kwargs: Override any parameter (Rs, Ld, Lq, flux_pm, J, B, Pp,
                      kp_id, ki_id, kp_iq, ki_iq, kp_speed, ki_speed, v_bus, dt_c, dt_s).

        Returns:
            dict with keys: speed, speed_error, steps, elapsed, torque_ripple.
        """
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter, RintBattery
        from sim_platform.models.sensor.sensors import CurrentSensor

        # Parameters with defaults and kwargs override
        Rs = kwargs.get("Rs", _DEFAULT_WINDING_R)
        Ld = kwargs.get("Ld", 5e-4)
        Lq = kwargs.get("Lq", 1e-3)
        flux_pm = kwargs.get("flux_pm", 0.03)
        J = kwargs.get("J", 1e-3)
        B = kwargs.get("B", 0.0001)
        _Pp = kwargs.get("Pp", 4)
        v_bus = kwargs.get("v_bus", _DEFAULT_V_BUS)
        dt_c = kwargs.get("dt_c", _DT_C)
        dt_s = kwargs.get("dt_s", _DT_SPEED)

        kp_id = kwargs.get("kp_id", 5.0)
        ki_id = kwargs.get("ki_id", 500.0)
        kp_iq = kwargs.get("kp_iq", 5.0)
        ki_iq = kwargs.get("ki_iq", 500.0)
        kp_speed = kwargs.get("kp_speed", 0.05)
        ki_speed = kwargs.get("ki_speed", 0.5)

        # Create components
        motor = PMSMdqModel(Rs=Rs, Ld=Ld, Lq=Lq, flux_pm=flux_pm, J=J, B=B)
        foc = FOCController(kp_id=kp_id, ki_id=ki_id, kp_iq=kp_iq, ki_iq=ki_iq,
                            ts=dt_c, v_bus=v_bus)
        speed_ctrl = SpeedController(kp=kp_speed, ki=ki_speed, ts=dt_s,
                                     iq_min=-_DEFAULT_I_MAX, iq_max=_DEFAULT_I_MAX)
        battery = RintBattery(v_oc=v_bus)
        inverter = AverageInverter(v_bus=v_bus)
        current_sensor = CurrentSensor()

        # Run simulation
        total_steps = int(duration / dt_c)
        speed_ratio = int(dt_s / dt_c) if dt_s > dt_c else _DEFAULT_SPEED_RATIO
        if speed_ratio < 1:
            speed_ratio = 1

        speed_meas = 0.0
        torque_values = []

        start = time.perf_counter()
        for step_idx in range(total_steps):
            # Measure currents (with noise)
            ia_m, ib_m, ic_m = current_sensor.read_abc(motor.ia, motor.ib, motor.ic)

            # Speed loop (runs at reduced rate)
            if step_idx % speed_ratio == 0:
                speed_meas = motor.omega_m
                iq_ref = speed_ctrl.update(speed_ref, speed_meas)
            else:
                iq_ref = speed_ctrl.pi.prev_output  # hold last value

            # Current loop — use actual theta_e from motor
            da, db, dc = foc.update(ia_m, ib_m, ic_m, motor.theta_e, 0.0, iq_ref)

            # Power stage
            v_batt = battery.step()
            va, vb, vc = inverter.step(da, db, dc, v_bus=v_batt)

            # Motor step
            motor.step_abc(va, vb, vc, tl=kwargs.get("tl", 0.0), dt=dt_c)
            motor.update_abc_currents()  # CRITICAL: update abc currents for next iteration

            torque_values.append(motor.torque)

        elapsed = time.perf_counter() - start
        speed_error = abs(motor.omega_m - speed_ref) / speed_ref * 100 if speed_ref > 0 else 0
        torque_ripple = (max(torque_values) - min(torque_values)) if torque_values else 0.0

        return {
            "speed": motor.omega_m,
            "speed_ref": speed_ref,
            "speed_error": speed_error,
            "steps": total_steps,
            "elapsed": elapsed,
            "torque_ripple": torque_ripple,
            "motor_type": "PMSM",
            "controller": "FOC",
        }

    @staticmethod
    def bldc(
        speed_ref: float = _DEFAULT_SPEED_REF,
        duration: float = _DEFAULT_DURATION,
        **kwargs: Any,
    ) -> dict:
        """Run a BLDC simulation and return results.

        Args:
            speed_ref: Target speed in rad/s.
            duration: Simulation duration in seconds.
            **kwargs: Override any motor/controller parameter.

        Returns:
            dict with keys: speed, speed_error, steps, elapsed, torque_ripple.
        """
        from sim_platform.models.motor.bldc import BLDCController, BLDCModel

        Rs = kwargs.get("Rs", _DEFAULT_WINDING_R)
        Ls = kwargs.get("Ls", 5e-4)
        Ke = kwargs.get("Ke", 0.05)
        J = kwargs.get("J", 1e-3)
        B = kwargs.get("B", 0.0001)
        Pp = kwargs.get("Pp", 4)
        v_bus = kwargs.get("v_bus", _DEFAULT_V_BUS)
        dt_c = kwargs.get("dt_c", _DT_C)

        kp = kwargs.get("kp", 2.0)
        ki = kwargs.get("ki", 100.0)

        motor = BLDCModel(Rs=Rs, Ls=Ls, Ke=Ke, Kt=Ke, J=J, B=B, Pp=Pp)
        controller = BLDCController(kp_speed=kp, ki_speed=ki, dt=dt_c, v_bus=v_bus)

        total_steps = int(duration / dt_c)
        torque_values = []

        start = time.perf_counter()
        for _ in range(total_steps):
            speed_meas = motor.omega_m
            v_out = controller.update(speed_ref, speed_meas)
            motor.step(min(v_out, v_bus), tl=kwargs.get("tl", 0.0), dt=dt_c)
            torque_values.append(motor.torque)

        elapsed = time.perf_counter() - start
        speed_error = abs(motor.omega_m - speed_ref) / speed_ref * 100 if speed_ref > 0 else 0
        torque_ripple = (max(torque_values) - min(torque_values)) if torque_values else 0.0

        return {
            "speed": motor.omega_m,
            "speed_ref": speed_ref,
            "speed_error": speed_error,
            "steps": total_steps,
            "elapsed": elapsed,
            "torque_ripple": torque_ripple,
            "motor_type": "BLDC",
            "controller": "6-Step",
        }

    @staticmethod
    def compare(
        speed_ref: float = _DEFAULT_SPEED_REF,
        duration: float = _DEFAULT_DURATION,
        **kwargs: Any,
    ) -> dict:
        """Compare PMSM vs BLDC side-by-side.

        Args:
            speed_ref: Target speed in rad/s.
            duration: Simulation duration in seconds.

        Returns:
            dict with keys: pmsm, bldc (each containing the result dict).
        """
        print(f"=== Motor Comparison @ {speed_ref:.0f} rad/s ===\n")
        results = {}

        print("  PMSM + FOC...")
        results["pmsm"] = QuickSim.pmsm_foc(speed_ref=speed_ref, duration=duration, **kwargs)
        p = results["pmsm"]
        print(f"    Speed: {p['speed']:.1f} rad/s  Error: {p['speed_error']:.2f}%  "
              f"Time: {p['elapsed']:.2f}s")

        print("  BLDC + 6-Step...")
        results["bldc"] = QuickSim.bldc(speed_ref=speed_ref, duration=duration, **kwargs)
        b = results["bldc"]
        print(f"    Speed: {b['speed']:.1f} rad/s  Error: {b['speed_error']:.2f}%  "
              f"Time: {b['elapsed']:.2f}s")

        print(f"\n  Winner (speed accuracy): {'PMSM' if p['speed_error'] < b['speed_error'] else 'BLDC'}")
        return results

    @staticmethod
    def sweep(
        speed_values: list[float] = None,
        duration: float = 1.0,
        motor_type: str = "pmsm",
        **kwargs: Any,
    ) -> dict:
        """Sweep through multiple speed reference values.

        Args:
            speed_values: List of speed reference values to test.
            duration: Simulation duration per speed value.
            motor_type: "pmsm" or "bldc".

        Returns:
            dict mapping speed value → result dict.
        """
        if speed_values is None:
            speed_values = [50, 100, 150]

        results = {}
        print(f"=== Speed Sweep ({motor_type.upper()}) ===\n")

        for sr in speed_values:
            if motor_type == "pmsm":
                r = QuickSim.pmsm_foc(speed_ref=sr, duration=duration, **kwargs)
            else:
                r = QuickSim.bldc(speed_ref=sr, duration=duration, **kwargs)
            results[sr] = r
            print(f"  {sr:.0f} rad/s → speed={r['speed']:.1f} rad/s  "
                  f"error={r['speed_error']:.2f}%  {r['elapsed']:.2f}s")

        return results


# ── Convenience function ────────────────────────────────────

def quick(motor: str = "pmsm", speed_ref: float = 100.0, duration: float = 1.5) -> dict:
    """Ultra-quick simulation — shortest possible one-liner.

    Usage:
        >>> from sim_platform.quicksim import quick
        >>> result = quick("pmsm", 150.0)
        >>> print(f"Speed: {result['speed']:.1f} rad/s")

    Args:
        motor: "pmsm" or "bldc".
        speed_ref: Target speed in rad/s.
        duration: Simulation duration in seconds.

    Returns:
        Result dict from QuickSim.pmsm_foc() or QuickSim.bldc().
    """
    if motor == "bldc":
        return QuickSim.bldc(speed_ref=speed_ref, duration=duration)
    return QuickSim.pmsm_foc(speed_ref=speed_ref, duration=duration)
