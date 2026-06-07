"""End-to-end integration tests — full simulation pipeline verification.

Tests the complete signal flow:
  Battery → Inverter → Motor → Sensor → Controller → Inverter
for each motor type (PMSM, BLDC, IM).

These tests verify that all modules work together correctly,
catching integration issues that unit tests miss.
"""

import math
import os
import sys

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

import pytest

from sim_platform.core.clock import ClockMode
from sim_platform.core.data_bus import DataBus, Signal
from sim_platform.core.orchestrator import Orchestrator, OrchestratorConfig
from sim_platform.models.controller.ekf import PMSMEKF
from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.bldc import BLDCController, BLDCModel
from sim_platform.models.motor.im_dq import IMdqModel, IMVectorController
from sim_platform.models.motor.pmsm_advanced import PMSMAdvanced
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter, RintBattery
from sim_platform.models.sensor.sensors import CurrentSensor, Encoder

NAN = float("nan")


# ══════════════════════════════════════════════════════════════
#  1. PMSM + FOC — Full Closed-Loop Integration
# ══════════════════════════════════════════════════════════════

class TestPMSM_FOC_Integration:
    """Full PMSM FOC closed-loop: Battery → Inverter → Motor → Sensor → FOC → Inverter."""

    def test_speed_step_response(self):
        """Motor should accelerate from 0 to target speed."""
        dt = 50e-6
        battery = RintBattery(48.0, 0.05)
        inverter = AverageInverter(48.0)
        motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                            flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                            dt_ns=int(dt * 1e9))
        csensor = CurrentSensor(noise_std=0.0, bias=0.0)
        encoder = Encoder(noise_std=0.0, quantization=0.0)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                            ts=dt, v_bus=48.0)
        spd = SpeedController(kp=0.05, ki=0.5, ts=1e-3)

        speed_ref = 100.0  # rad/s
        iq_ref = 0.0

        for step in range(20000):
            _t = step * dt

            # Speed loop (1kHz)
            if step % 20 == 0:
                speed_meas = encoder.read_speed(motor.omega_m)
                iq_ref = spd.update(speed_ref, speed_meas)

            # Current measurement + FOC (20kHz)
            ia, ib, ic = csensor.read_abc(motor.ia, motor.ib, motor.ic)
            theta = encoder.read_angle(motor.theta_e)
            da, db, dc = foc.update(ia, ib, ic, theta, id_ref=0.0, iq_ref=iq_ref)

            # Inverter + Motor
            v_bus = battery.step(0.0)
            va, vb, vc = inverter.step(da, db, dc, v_bus)
            motor.step_abc(va, vb, vc, dt=dt)
            motor.update_abc_currents()

        # Assertions: motor should have accelerated
        final_speed = motor.omega_m
        assert final_speed > 50.0, f"Motor speed too low: {final_speed:.1f} rad/s"
        assert math.isfinite(final_speed), "Speed is not finite"
        assert math.isfinite(motor.id), "id is not finite"
        assert math.isfinite(motor.iq), "iq is not finite"

    def test_full_pipeline_no_nan(self):
        """No NaN should appear in the full pipeline after 1000 steps."""
        dt = 50e-6
        motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                            flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                            dt_ns=int(dt * 1e9))
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                            ts=dt, v_bus=48.0)
        inverter = AverageInverter(48.0)
        csensor = CurrentSensor(noise_std=0.1, bias=0.01)
        encoder = Encoder(noise_std=0.001)

        for step in range(1000):
            ia, ib, ic = csensor.read_abc(motor.ia, motor.ib, motor.ic)
            theta = encoder.read_angle(motor.theta_e)
            da, db, dc = foc.update(ia, ib, ic, theta, 0.0, 10.0)
            va, vb, vc = inverter.step(da, db, dc)
            motor.step_abc(va, vb, vc, dt=dt)
            motor.update_abc_currents()

        state = motor.get_state()
        for key, val in state.items():
            assert math.isfinite(val), f"Motor state '{key}' is {val}"


# ══════════════════════════════════════════════════════════════
#  2. BLDC — Full Pipeline Integration
# ══════════════════════════════════════════════════════════════

class TestBLDC_Integration:
    """BLDC motor + controller integration test."""

    def test_bldc_speed_control(self):
        """BLDC motor should spin up under speed control."""
        dt = 50e-6
        motor = BLDCModel(Rs=0.5, Ls=1e-3, Ke=0.01, Kt=0.01,
                          J=1e-4, B=1e-5, Pp=1, dt_ns=int(dt * 1e9))
        ctrl = BLDCController(kp_speed=0.1, ki_speed=1.0, dt=dt)

        for step in range(20000):
            speed_meas = motor.omega_m
            duty = ctrl.update(100.0, speed_meas)
            v_bus = 24.0 * abs(duty)
            motor.step(v_bus, tl=0.0, dt=dt)

        assert motor.omega_m > 10.0, f"BLDC speed too low: {motor.omega_m:.1f}"
        assert math.isfinite(motor.omega_m)


# ══════════════════════════════════════════════════════════════
#  3. Induction Motor — Full Pipeline Integration
# ══════════════════════════════════════════════════════════════

class TestIM_Integration:
    """Induction motor + vector controller integration test."""

    def test_im_speed_control(self):
        """IM should accelerate under vector control."""
        dt = 50e-6
        motor = IMdqModel(Rs=0.5, Rr=0.5, Ls=0.01, Lr=0.01,
                          Lm=0.009, J=0.01, B=0.001, Pp=2,
                          dt_ns=int(dt * 1e9))
        ctrl = IMVectorController(motor=motor, ts=dt)

        speed_ref = 100.0
        flux_ref = 0.1

        vsd, vsq, omega_e = 0.0, 0.0, 0.0
        for step in range(10000):
            if step % 20 == 0:
                vsd, vsq, omega_e = ctrl.update_speed(
                    speed_ref, motor.omega_m, flux_ref)
            motor.step(vsd, vsq, omega_e, tl=0.0, dt=dt)

        # IM with simplified vector control should at least start moving
        assert motor.omega_m > 0.0, f"IM not moving: {motor.omega_m:.1f}"
        assert math.isfinite(motor.omega_m)


# ══════════════════════════════════════════════════════════════
#  4. Advanced PMSM — Integration
# ══════════════════════════════════════════════════════════════

class TestPMSMAdvanced_Integration:
    """Advanced PMSM with saturation + temperature integration."""

    def test_advanced_pmsm_full_loop(self):
        """Advanced PMSM should run without NaN in closed loop."""
        dt = 50e-6
        motor = PMSMAdvanced(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                             flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                             dt_ns=int(dt * 1e9))
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                            ts=dt, v_bus=48.0)
        inverter = AverageInverter(48.0)
        csensor = CurrentSensor(noise_std=0.0, bias=0.0)
        encoder = Encoder(noise_std=0.0, quantization=0.0)

        for step in range(2000):
            ia, ib, ic = csensor.read_abc(motor.ia, motor.ib, motor.ic)
            theta = encoder.read_angle(motor.theta_e)
            da, db, dc = foc.update(ia, ib, ic, theta, 0.0, 10.0)
            va, vb, vc = inverter.step(da, db, dc)
            motor.step_abc(va, vb, vc, dt=dt)
            motor.update_abc_currents()

        state = motor.get_state()
        for key, val in state.items():
            if isinstance(val, (int, float)):
                assert math.isfinite(val), f"State '{key}' is {val}"


# ══════════════════════════════════════════════════════════════
#  5. Orchestrator — Full System Integration
# ══════════════════════════════════════════════════════════════

class TestOrchestrator_Integration:
    """Orchestrator running a complete simulation."""

    def test_orchestrator_with_steppers(self):
        """Orchestrator should run multiple steppers correctly."""
        cfg = OrchestratorConfig(mode=ClockMode.OFFLINE, enable_energy_audit=True)
        o = Orchestrator(cfg)

        results = []

        def stepper_a(dt_ns):
            results.append(("a", o.clock.sim_time_ns))
            return StepResult(solver_id="a", converged=True)

        def stepper_b(dt_ns):
            results.append(("b", o.clock.sim_time_ns))
            return StepResult(solver_id="b", converged=True)

        o.register_stepper("motor", stepper_a)
        o.register_stepper("controller", stepper_b)

        _audits = o.run(step_ns=100000, duration_s=0.01)

        assert len(results) > 0
        # Both steppers should have been called
        a_count = sum(1 for r in results if r[0] == "a")
        b_count = sum(1 for r in results if r[0] == "b")
        assert a_count > 0
        assert b_count > 0
        assert a_count == b_count

    def test_orchestrator_with_fault_injection(self):
        """Orchestrator should handle fault injection gracefully."""
        o = Orchestrator(OrchestratorConfig(mode=ClockMode.OFFLINE))

        fault_triggered = [False]

        def my_fault():
            fault_triggered[0] = True

        o.schedule_fault(0.005, my_fault)

        def step_fn(dt_ns):
            pass

        o.register_stepper("sim", lambda dt_ns: StepResult("sim"))
        o.run(step_ns=100000, duration_s=0.01)

        assert fault_triggered[0]


# ══════════════════════════════════════════════════════════════
#  6. EKF + Motor — State Estimation Integration
# ══════════════════════════════════════════════════════════════

class TestEKF_Integration:
    """EKF state estimation with PMSM motor integration."""

    def test_ekf_tracks_motor_state(self):
        """EKF should track motor state within tolerance."""
        dt = 50e-6
        motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                            flux_pm=0.03, J=0.001, B=0.0001, Pp=4,
                            dt_ns=int(dt * 1e9))
        ekf = PMSMEKF(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                       flux_pm=0.03, Pp=4, dt=dt)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0,
                            ts=dt, v_bus=48.0)
        inverter = AverageInverter(48.0)
        csensor = CurrentSensor(noise_std=0.0, bias=0.0)
        encoder = Encoder(noise_std=0.0, quantization=0.0)

        iq_ref = 10.0
        for step in range(2000):
            ia, ib, ic = csensor.read_abc(motor.ia, motor.ib, motor.ic)
            theta = encoder.read_angle(motor.theta_e)
            da, db, dc = foc.update(ia, ib, ic, theta, 0.0, iq_ref)
            va, vb, vc = inverter.step(da, db, dc)
            motor.step_abc(va, vb, vc, dt=dt)
            motor.update_abc_currents()

            # EKF estimation
            omega_enc = encoder.read_speed(motor.omega_m)
            id_est, iq_est, omega_est, theta_est = ekf.estimate(
                foc.vd_ref, foc.vq_ref,
                motor.ia, motor.ib, motor.ic, omega_enc)

        # EKF should track motor state (within tolerance for simplified model)
        if abs(motor.iq) > 1.0:
            iq_error = abs(iq_est - motor.iq) / abs(motor.iq)
            assert iq_error < 1.0, f"EKF iq error too large: {iq_error:.2%}"
        # EKF estimates should be finite
        assert math.isfinite(id_est)
        assert math.isfinite(iq_est)
        assert math.isfinite(omega_est)


# ══════════════════════════════════════════════════════════════
#  7. DataBus + Full System — Data Flow Integration
# ══════════════════════════════════════════════════════════════

class TestDataBus_Integration:
    """DataBus integration with full simulation data flow."""

    def test_data_bus_full_flow(self):
        """DataBus should handle motor data flow correctly."""
        bus = DataBus()
        bus.register_module("motor")
        bus.register_module("sensor")
        bus.register_module("controller")

        received = []

        def on_speed(sig):
            received.append(sig.value)

        bus.subscribe("motor/speed", on_speed, module_id="module://controller")

        motor = PMSMdqModel(Rs=0.1, Ld=0.5e-3, Lq=1.0e-3,
                            flux_pm=0.03, J=0.001, B=0.0001, Pp=4)

        for _ in range(100):
            motor.step(10.0, 10.0, 0.0)
            sig = Signal(source="motor://pmsm", signal_type="speed",
                         value=motor.omega_m, unit="rad/s")
            bus.publish("motor/speed", sig, module_id="module://motor")

        assert len(received) == 100
        assert all(math.isfinite(v) for v in received)


# Need StepResult for orchestrator tests
from sim_platform.core.orchestrator import StepResult

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
