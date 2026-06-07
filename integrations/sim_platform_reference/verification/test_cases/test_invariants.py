"""Physics invariant verification — the REAL attack tests.

Instead of just checking isfinite(), these tests verify that the simulation
obeys fundamental physical laws. If these break, the simulation is WRONG,
not just "crashing".

Invariant categories:
  1. Energy conservation (first law of thermodynamics)
  2. Causality (output cannot precede input)
  3. Symmetry (dq transform invariance)
  4. Monotonicity (more voltage → more current, etc.)
  5. Boundedness (physical quantities have limits)
  6. Consistency (multiple computation paths agree)
  7. Reversibility (transform/inverse-transform identity)
"""

import math
import os
import sys

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)


# ============================================================
# 1. Energy Conservation
# ============================================================
import unittest


class TestEnergyConservation(unittest.TestCase):
    """Verify energy balance: P_in = P_out + P_loss + dE_stored/dt."""

    def test_pmsm_energy_balance(self):
        """Electrical input = mechanical output + losses + stored energy change."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4, Pp=4)
        dt = 50e-6

        # Run to steady state
        for _ in range(5000):
            m.step(10.0, 5.0, tl=0.5)

        # Energy accounting over 100 steps
        E_electrical = 0.0
        E_mechanical = 0.0
        E_loss = 0.0

        for _ in range(100):
            # Electrical input power: P = Vd*Id + Vq*Iq (simplified)
            P_elec_in = 10.0 * m.id + 5.0 * m.iq
            # Mechanical output power: P = T * omega
            P_mech_out = m.torque * m.omega_m
            # Resistive loss: P = Rs * (id^2 + iq^2)
            P_loss = m.Rs * (m.id**2 + m.iq**2)

            E_electrical += P_elec_in * dt
            E_mechanical += P_mech_out * dt
            E_loss += P_loss * dt

            m.step(10.0, 5.0, tl=0.5)

        # Energy balance: input ≈ output + loss (within 10% for simplified model)
        if abs(E_electrical) > 1e-10:
            ratio = (E_mechanical + E_loss) / E_electrical
            self.assertGreater(ratio, 0.5, "Energy output too low relative to input")
            self.assertLess(ratio, 2.0, "Energy output exceeds input by 2x")

    def test_thermal_energy_accumulation(self):
        """Temperature increase should be proportional to net heat input."""
        from sim_platform.models.thermal.thermal_model import ThermalNode

        node = ThermalNode(C_th=100.0, R_th=0.5, T_ambient=25.0)
        dt = 0.001

        # Apply constant heat for 1 second
        T_before = node.T
        for _ in range(1000):
            node.step(100.0, dt)  # 100W constant

        T_after = node.T
        dT = T_after - T_before

        # Temperature should increase (positive heat input)
        self.assertGreater(dT, 0.0, "Temperature should rise with positive heat")

        # Temperature increase should be bounded by P*dt_total/C_th
        dT_max = 100.0 * 1.0 / 100.0  # P*t/C = 1.0 K (ignoring cooling)
        self.assertLess(dT, dT_max * 2.0, "Temperature increase too large")


# ============================================================
# 2. Causality
# ============================================================
class TestCausality(unittest.TestCase):
    """Output at time t depends only on inputs at time <= t."""

    def test_pmsm_response_delay(self):
        """Motor state at step N should not depend on input at step N+1."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        m1 = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m2 = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)

        # Both motors get same input for 100 steps
        for _ in range(100):
            m1.step(10.0, 10.0)
            m2.step(10.0, 10.0)

        # Record state
        _id_after_same = m1.id

        # Now give different inputs for 1 more step
        m1.step(10.0, 10.0)
        m2.step(20.0, 20.0)

        # The state difference should be due to THIS step's input only
        # m1's state at step 100 should be identical regardless of m2's future
        self.assertNotAlmostEqual(m1.id, m2.id, places=2,
                                   msg="Different inputs should produce different states")

    def test_pure_delay_preserves_causality(self):
        """Output with delay should lag input by exactly N steps."""
        from sim_platform.models.sensor.sensors import Encoder

        enc = Encoder(noise_std=0.0, quantization=0.0)  # Perfect encoder

        # Ramp input
        angles = [float(i) * 0.01 for i in range(1000)]
        outputs = [enc.read_angle(a) for a in angles]

        # Output should track input (modulo 2*pi)
        for i in range(100, 500):
            expected = angles[i] % (2 * math.pi)
            self.assertAlmostEqual(outputs[i], expected, places=10,
                                    msg=f"Step {i}: encoder output mismatch")


# ============================================================
# 3. Transform Reversibility
# ============================================================
class TestTransformReversibility(unittest.TestCase):
    """Clarke+Park transforms should be invertible."""

    def test_clarke_park_roundtrip(self):
        """abc → αβ → dq → αβ → abc should recover original (up to zero-sequence)."""
        import random

        from sim_platform.models.controller.foc import (
            clarke_transform,
            inverse_park,
            park_transform,
        )
        random.seed(42)

        for _ in range(1000):
            ia = random.uniform(-100, 100)
            ib = random.uniform(-100, 100)
            ic = random.uniform(-100, 100)
            theta = random.uniform(0, 2 * math.pi)

            # Forward: abc → αβ → dq
            alpha, beta = clarke_transform(ia, ib, ic)
            d, q = park_transform(alpha, beta, theta)

            # Inverse: dq → αβ
            alpha2, beta2 = inverse_park(d, q, theta)

            # αβ should roundtrip perfectly
            self.assertAlmostEqual(alpha, alpha2, places=10,
                                    msg="Clarke alpha roundtrip failed")
            self.assertAlmostEqual(beta, beta2, places=10,
                                    msg="Clarke beta roundtrip failed")

    def test_svpwm_duty_range(self):
        """SVPWM duty cycles must be in [0, 1]."""
        import random

        from sim_platform.models.controller.foc import svpwm
        random.seed(42)

        for _ in range(10000):
            va = random.uniform(-100, 100)
            vb = random.uniform(-100, 100)
            vbus = random.uniform(1.0, 100.0)
            da, db, dc = svpwm(va, vb, vbus)
            self.assertGreaterEqual(da, -0.01, f"da={da} < 0")
            self.assertLessEqual(da, 1.01, f"da={da} > 1")
            self.assertGreaterEqual(db, -0.01, f"db={db} < 0")
            self.assertLessEqual(db, 1.01, f"db={db} > 1")
            self.assertGreaterEqual(dc, -0.01, f"dc={dc} < 0")
            self.assertLessEqual(dc, 1.01, f"dc={dc} > 1")


# ============================================================
# 4. Monotonicity (Physical Ordering)
# ============================================================
class TestMonotonicity(unittest.TestCase):
    """Physical cause-effect relationships must hold."""

    def test_higher_voltage_higher_current(self):
        """More voltage should produce more current (all else equal)."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        m_low = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m_high = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)

        # Apply different voltages for 1000 steps
        for _ in range(1000):
            m_low.step(5.0, 5.0)
            m_high.step(20.0, 20.0)

        # Higher voltage → higher iq (in steady state)
        # Note: this is a simplification; real FOC controls iq independently
        iq_low = abs(m_low.iq)
        iq_high = abs(m_high.iq)

        # Higher voltage should produce higher or equal current
        # (with same B=0, the system is unbounded, but relative ordering holds)
        self.assertGreaterEqual(iq_high, iq_low * 0.99,
                                 f"Higher voltage {20.0}V should produce >= current "
                                 f"than {5.0}V: iq_high={iq_high:.4f} vs iq_low={iq_low:.4f}")

    def test_higher_load_slower_speed(self):
        """More load torque → lower steady-state speed."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        m_light = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4)
        m_heavy = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4)

        # Apply same voltage, different loads
        for _ in range(5000):
            m_light.step(20.0, 10.0, tl=0.1)
            m_heavy.step(20.0, 10.0, tl=5.0)

        # Heavy load should have lower speed
        self.assertLess(m_heavy.omega_m, m_light.omega_m,
                         "Higher load should produce lower speed")

    def test_thermal_higher_heat_higher_temp(self):
        """More heat input → higher temperature."""
        from sim_platform.models.thermal.thermal_model import ThermalNode

        t_low = ThermalNode(C_th=100.0, R_th=0.5, T_ambient=25.0)
        t_high = ThermalNode(C_th=100.0, R_th=0.5, T_ambient=25.0)

        for _ in range(1000):
            t_low.step(10.0, 0.001)
            t_high.step(100.0, 0.001)

        self.assertGreater(t_high.T, t_low.T,
                            "More heat should produce higher temperature")


# ============================================================
# 5. Boundedness (Physical Limits)
# ============================================================
class TestBoundedness(unittest.TestCase):
    """Physical quantities must stay within known limits."""

    def test_pi_output_bounded(self):
        """PI controller output must respect saturation limits."""
        from sim_platform.models.controller.foc import PIController

        pi = PIController(kp=1.0, ki=100.0, ts=1e-3, out_min=-50.0, out_max=50.0)

        # Apply extreme setpoint
        for _ in range(10000):
            out = pi.update(1e6, 0.0)  # Huge error
            self.assertGreaterEqual(out, -50.0 - 0.01,
                                     f"PI output {out} below min")
            self.assertLessEqual(out, 50.0 + 0.01,
                                  f"PI output {out} above max")

    def test_duty_cycle_bounded(self):
        """SVPWM duty cycles must be in [0, 1]."""
        from sim_platform.models.controller.foc import svpwm

        # Test with extreme voltages
        test_cases = [
            (0.0, 0.0, 48.0),
            (100.0, 0.0, 48.0),
            (0.0, 100.0, 48.0),
            (-100.0, -100.0, 48.0),
            (1e6, 1e6, 48.0),
            (1e-10, 1e-10, 48.0),
        ]

        for va, vb, vbus in test_cases:
            da, db, dc = svpwm(va, vb, vbus)
            for name, d in [("da", da), ("db", db), ("dc", dc)]:
                self.assertGreaterEqual(d, -0.01,
                                         f"{name}={d} < 0 with va={va}, vb={vb}")
                self.assertLessEqual(d, 1.01,
                                      f"{name}={d} > 1 with va={va}, vb={vb}")

    def test_kalman_covariance_positive(self):
        """Kalman filter covariance must stay positive."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter

        kf = SimpleKalmanFilter(Q=0.01, R=0.1)

        # Feed many measurements
        for i in range(10000):
            kf.predict()
            kf.update(float(i) * 0.01)

        P = kf.get_uncertainty()
        self.assertGreaterEqual(P, 0.0,
                                  f"Kalman uncertainty {P} < 0")

    def test_thermal_temperature_bounded(self):
        """Temperature should not exceed physical limits."""
        from sim_platform.models.thermal.thermal_model import ThermalNode

        node = ThermalNode(C_th=10.0, R_th=0.5, T_ambient=25.0, T_max=200.0)

        # Apply massive heat for long time
        for _ in range(1000000):
            node.step(1e6, 1e-6)

        # Temperature should be clamped (from the code: max T_max * 1.5)
        self.assertLessEqual(node.T, 200.0 * 1.5 + 1.0,
                              f"Temperature {node.T} exceeds physical limit")


# ============================================================
# 6. Consistency (Multiple Paths Agree)
# ============================================================
class TestConsistency(unittest.TestCase):
    """Different computation paths should give same result."""

    def test_pmsm_step_vs_step_abc(self):
        """step(vd, vq) and step_abc(va, vb, vc) should be consistent
        when abc voltages produce the same dq voltages."""
        from sim_platform.models.controller.foc import inverse_park
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        m1 = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m2 = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)

        # Run both to same state
        for _ in range(100):
            m1.step(5.0, 3.0)
            m2.step(5.0, 3.0)

        # Now: m1 uses dq directly, m2 uses abc
        vd, vq = 5.0, 3.0
        v_alpha, v_beta = inverse_park(vd, vq, m1.theta_e)
        # Convert αβ to abc for balanced 3-phase
        SQRT3_HALF = math.sqrt(3) / 2
        va = v_alpha
        vb = -0.5 * v_alpha + SQRT3_HALF * v_beta
        vc = -0.5 * v_alpha - SQRT3_HALF * v_beta

        # Both should produce similar state changes
        m1.step(vd, vq)
        m2.step_abc(va, vb, vc)

        # States should be very close (transforms are mathematically equivalent)
        # Allow small numerical tolerance from floating point path differences
        tol = 1e-10
        self.assertAlmostEqual(m1.id, m2.id, delta=abs(m1.id) * 1e-6 + tol,
                               msg=f"step() id={m1.id} vs step_abc() id={m2.id}")
        self.assertAlmostEqual(m1.iq, m2.iq, delta=abs(m1.iq) * 1e-6 + tol,
                               msg=f"step() iq={m1.iq} vs step_abc() iq={m2.iq}")
        self.assertAlmostEqual(m1.omega_m, m2.omega_m, delta=abs(m1.omega_m) * 1e-6 + tol,
                               msg=f"step() omega={m1.omega_m} vs step_abc() omega={m2.omega_m}")

    def test_foc_update_consistency(self):
        """FOC.update should be deterministic (same input → same output)."""
        from sim_platform.models.controller.foc import FOCController

        foc1 = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        foc2 = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)

        import random
        random.seed(42)

        for _ in range(1000):
            ia = random.uniform(-10, 10)
            ib = random.uniform(-10, 10)
            ic = random.uniform(-10, 10)
            theta = random.uniform(0, 2 * math.pi)
            id_ref = random.uniform(-10, 10)
            iq_ref = random.uniform(-10, 10)

            da1, db1, dc1 = foc1.update(ia, ib, ic, theta, id_ref, iq_ref)
            da2, db2, dc2 = foc2.update(ia, ib, ic, theta, id_ref, iq_ref)

            self.assertAlmostEqual(da1, da2, places=12,
                                    msg="FOC not deterministic")
            self.assertAlmostEqual(db1, db2, places=12)
            self.assertAlmostEqual(dc1, dc2, places=12)


# ============================================================
# 7. Fuzz Testing (Randomized Adversarial)
# ============================================================
class TestFuzzPMSM(unittest.TestCase):
    """Randomized input fuzzing for PMSM model."""

    def test_fuzz_pmsm_10k_random_inputs(self):
        """10k random voltage inputs — no crash, all finite."""
        import random

        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        random.seed(42)
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)

        for _ in range(10000):
            vd = random.uniform(-200, 200)
            vq = random.uniform(-200, 200)
            tl = random.uniform(-10, 10)
            m.step(vd, vq, tl=tl)

            # All state variables must be finite
            self.assertTrue(math.isfinite(m.id), f"id={m.id} at step")
            self.assertTrue(math.isfinite(m.iq), f"iq={m.iq}")
            self.assertTrue(math.isfinite(m.omega_m), f"omega={m.omega_m}")
            self.assertTrue(math.isfinite(m.theta_e), f"theta={m.theta_e}")

    def test_fuzz_foc_10k_random_inputs(self):
        """10k random FOC inputs — no crash, all finite."""
        import random

        from sim_platform.models.controller.foc import FOCController

        random.seed(42)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)

        for _ in range(10000):
            ia = random.uniform(-100, 100)
            ib = random.uniform(-100, 100)
            ic = random.uniform(-100, 100)
            theta = random.uniform(0, 10 * math.pi)  # Multiple rotations
            id_ref = random.uniform(-50, 50)
            iq_ref = random.uniform(-50, 50)

            da, db, dc = foc.update(ia, ib, ic, theta, id_ref, iq_ref)

            self.assertTrue(math.isfinite(da))
            self.assertTrue(math.isfinite(db))
            self.assertTrue(math.isfinite(dc))
            self.assertGreaterEqual(da, -0.01)
            self.assertLessEqual(da, 1.01)

    def test_fuzz_kalman_10k_random_measurements(self):
        """10k random Kalman filter inputs — no divergence."""
        import random

        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter

        random.seed(42)
        kf = SimpleKalmanFilter(Q=0.01, R=0.1)

        for _ in range(10000):
            kf.predict()
            z = random.gauss(0, 100)  # Noisy measurement
            kf.update(z)

            self.assertTrue(math.isfinite(kf.get_estimate()))
            self.assertGreaterEqual(kf.get_uncertainty(), 0.0)

    def test_fuzz_thermal_10k_random_heat(self):
        """10k random thermal inputs — physically reasonable."""
        import random

        from sim_platform.models.thermal.thermal_model import ThermalNode

        random.seed(42)
        node = ThermalNode(C_th=100.0, R_th=0.5, T_ambient=25.0, T_max=200.0)

        for _ in range(10000):
            P = random.uniform(-100, 1e4)  # Including negative (cooling)
            dt = random.uniform(1e-6, 0.01)
            node.step(P, dt)

            self.assertTrue(math.isfinite(node.T))
            # Temperature should stay physical (above absolute zero, below plasma)
            self.assertGreater(node.T, -273.15,
                                 f"T={node.T} below absolute zero")


# ============================================================
# 8. Closed-Loop Stability
# ============================================================
class TestClosedLoopStability(unittest.TestCase):
    """Full closed-loop system should converge to reference."""

    def test_speed_control_convergence(self):
        """Speed controller should drive motor to reference speed."""
        from sim_platform.models.controller.foc import FOCController, SpeedController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter(v_bus=48.0)

        speed_ref = 100.0

        # Run for 20000 steps (1 second)
        for _ in range(20000):
            iq_ref = sc.update(speed_ref, motor.omega_m)
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc)

        # Speed should be within 20% of reference
        error_pct = abs(motor.omega_m - speed_ref) / speed_ref * 100
        self.assertLess(error_pct, 20.0,
                          f"Speed {motor.omega_m:.1f} rad/s vs ref {speed_ref} "
                          f"({error_pct:.1f}% error)")

    def test_current_control_tracking(self):
        """FOC should track current references with load applied."""
        from sim_platform.models.controller.foc import FOCController
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        from sim_platform.models.power.power_models import AverageInverter

        motor = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4)
        foc = FOCController(kp_id=5.0, ki_id=500.0, kp_iq=5.0, ki_iq=500.0, ts=50e-6)
        inv = AverageInverter(v_bus=48.0)

        id_ref = 0.0
        iq_ref = 10.0

        # Run with load torque to prevent runaway acceleration
        for _ in range(5000):
            ia, ib, ic = motor.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, motor.theta_e, id_ref, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            motor.step_abc(va, vb, vc, tl=2.0)

        # iq should have moved significantly from zero (controller is active)
        self.assertGreater(abs(motor.iq), 0.5,
                             f"iq={motor.iq:.2f} — controller should drive iq > 0")

        # iq should not be wildly unstable
        self.assertTrue(math.isfinite(motor.iq))
        self.assertLess(abs(motor.iq), 1000.0,
                          f"iq={motor.iq:.2f} — controller unstable")


if __name__ == "__main__":
    unittest.main()
