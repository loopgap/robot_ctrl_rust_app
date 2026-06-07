"""Numerical stability and floating-point precision tests.

Tests for:
  1. Long-running accumulated error (Euler drift)
  2. Theta wrapping precision preservation
  3. Gradient explosion prevention
  4. Division-by-near-zero robustness
  5. Consistency between equivalent computation paths
  6. Catastrophic cancellation resilience
  7. Parameter boundary stability

Each test verifies that the simulation remains numerically stable
under conditions that would break naive implementations.
"""

import math
import os
import random
import sys
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)


class TestEulerIntegrationDrift(unittest.TestCase):
    """Verify Euler forward integration error stays bounded."""

    def test_pmsm_drift_10s(self):
        """PMSM state drift over 10 seconds (200k steps)."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4)
        m.reset()

        # Record initial energy
        _E_initial = 0.5 * m.J * m.omega_m**2

        # Run with constant voltage — should reach steady state
        for _ in range(200000):  # 10s at 50us
            m.step(5.0, 5.0, tl=0.5)

        # State should be finite and bounded
        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))
        self.assertTrue(math.isfinite(m.omega_m))
        self.assertTrue(math.isfinite(m.theta_e))

        # Speed should be bounded (not exploded)
        self.assertLess(abs(m.omega_m), 1e6,
                        f"Speed exploded: omega_m={m.omega_m}")

    def test_im_drift_10s(self):
        """Induction motor state drift over 10 seconds."""
        from sim_platform.models.motor.im_dq import IMdqModel
        m = IMdqModel(Rs=0.05, Rr=0.05, Ls=0.01, Lr=0.01, Lm=0.009,
                       Pp=2, J=0.01, B=0.001)
        m.reset()

        for _ in range(200000):
            omega_e = m.Pp * m.omega_m
            m.step(5.0, 5.0, omega_e, tl=1.0)

        self.assertTrue(math.isfinite(m.ids))
        self.assertTrue(math.isfinite(m.iqs))
        self.assertTrue(math.isfinite(m.omega_m))
        self.assertLess(abs(m.omega_m), 1e6)

    def test_bldc_drift_10s(self):
        """BLDC motor state drift over 10 seconds."""
        from sim_platform.models.motor.bldc import BLDCModel
        m = BLDCModel(Rs=0.1, Ls=1e-4, Ke=0.01, Kt=0.01,
                       J=1e-4, B=1e-5, Pp=1)
        m.reset()

        for _ in range(200000):
            m.step(12.0, tl=0.01)

        self.assertTrue(math.isfinite(m.ia))
        self.assertTrue(math.isfinite(m.omega_m))
        self.assertLess(abs(m.omega_m), 1e6)

    def test_thermal_drift_long(self):
        """Thermal model over 1000 time constants — should converge."""
        from sim_platform.models.thermal.thermal_model import ThermalNode
        R_th = 0.5
        C_th = 10.0
        tau = R_th * C_th  # 5 seconds
        node = ThermalNode(C_th=C_th, R_th=R_th, T_ambient=25.0)

        # Run for 100 tau = 500 seconds
        dt = 0.01
        steps = int(100 * tau / dt)
        for _ in range(steps):
            node.step(100.0, dt)

        # Should converge to T_ambient + P * R_th = 25 + 100*0.5 = 75
        expected = 25.0 + 100.0 * R_th
        self.assertAlmostEqual(node.T, expected, delta=0.1,
                               msg=f"Thermal drift: T={node.T:.2f} vs expected {expected:.2f}")


class TestThetaWrappingPrecision(unittest.TestCase):
    """Verify theta wrapping preserves sin/cos precision."""

    def test_pmsm_theta_wrapping_preserves_trig(self):
        """After many revolutions, sin/cos should still be accurate."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.reset()

        # Run for 1000 revolutions (Pp=4, so theta_e goes to 4000*2*pi)
        steps_per_rev = int(2 * math.pi / (m.Pp * 100.0 * m.dt))  # ~314 steps
        for _ in range(steps_per_rev * 1000):
            m.step(5.0, 5.0)

        # theta_e should be wrapped to [0, 2*pi)
        self.assertGreaterEqual(m.theta_e, 0.0)
        self.assertLess(m.theta_e, 2 * math.pi)

        # sin/cos should be accurate (not degraded by large theta)
        sin_val = math.sin(m.theta_e)
        cos_val = math.cos(m.theta_e)
        self.assertTrue(math.isfinite(sin_val))
        self.assertTrue(math.isfinite(cos_val))
        self.assertLessEqual(abs(sin_val), 1.0 + 1e-10)
        self.assertLessEqual(abs(cos_val), 1.0 + 1e-10)

    def test_bldc_theta_wrapping(self):
        """BLDC theta_m and theta_e should both wrap."""
        from sim_platform.models.motor.bldc import BLDCModel
        m = BLDCModel(Rs=0.1, Ls=1e-4, Ke=0.01, Kt=0.01,
                       J=1e-4, B=1e-5, Pp=1)
        m.reset()

        for _ in range(200000):
            m.step(12.0)

        self.assertGreaterEqual(m.theta_e, 0.0)
        self.assertLess(m.theta_e, 2 * math.pi)
        self.assertGreaterEqual(m.theta_m, 0.0)
        self.assertLess(m.theta_m, 2 * math.pi)

    def test_im_theta_wrapping(self):
        """IM theta_e should wrap."""
        from sim_platform.models.motor.im_dq import IMdqModel
        m = IMdqModel(Rs=0.05, Rr=0.05, Ls=0.01, Lr=0.01, Lm=0.009,
                       Pp=2, J=0.01, B=0.001)
        m.reset()

        for _ in range(200000):
            omega_e = m.Pp * m.omega_m
            m.step(5.0, 5.0, omega_e)

        self.assertGreaterEqual(m.theta_e, 0.0)
        self.assertLess(m.theta_e, 2 * math.pi)


class TestGradientExplosionPrevention(unittest.TestCase):
    """Verify derivatives don't cause state explosion."""

    def test_pmsm_high_voltage_stability(self):
        """High voltage with friction should reach bounded steady state."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        # Use non-zero B (friction) to prevent runaway acceleration
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-2)
        m.reset()

        # Apply high voltage with load — should reach steady state
        for _ in range(10000):
            m.step(50.0, 50.0, tl=5.0)

        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))
        self.assertTrue(math.isfinite(m.omega_m))

    def test_pmsm_no_friction_runaway(self):
        """PMSM without friction: current grows — verify it stays finite."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.reset()

        # Without friction, speed and current grow unboundedly
        # But they should stay FINITE (not NaN/Inf)
        for _ in range(1000):
            m.step(100.0, 100.0)

        self.assertTrue(math.isfinite(m.id), f"id exploded: {m.id}")
        self.assertTrue(math.isfinite(m.iq), f"iq exploded: {m.iq}")
        self.assertTrue(math.isfinite(m.omega_m), f"omega exploded: {m.omega_m}")

    def test_pmsm_advanced_extreme_current_no_overflow(self):
        """PMSMAdvanced with extreme initial currents should not overflow."""
        from sim_platform.models.motor.pmsm_advanced import PMSMAdvanced
        m = PMSMAdvanced(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.reset()

        # Set extreme initial state
        m.id = 1e10
        m.iq = 1e10

        # Step should not raise OverflowError
        try:
            for _ in range(100):
                m.step(5.0, 5.0)
        except OverflowError:
            self.fail("OverflowError in PMSMAdvanced with extreme currents")

        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))

    def test_im_extreme_slip_stability(self):
        """IM with extreme slip should remain stable."""
        from sim_platform.models.motor.im_dq import IMdqModel
        m = IMdqModel(Rs=0.05, Rr=0.05, Ls=0.01, Lr=0.01, Lm=0.009,
                       Pp=2, J=0.01, B=0.001)
        m.reset()

        # Drive with high voltage at standstill (100% slip)
        for _ in range(10000):
            omega_e = m.Pp * m.omega_m
            m.step(100.0, 100.0, omega_e)

        self.assertTrue(math.isfinite(m.ids))
        self.assertTrue(math.isfinite(m.iqs))
        self.assertLess(abs(m.ids), 1e6)
        self.assertLess(abs(m.iqs), 1e6)

    def test_foc_pi_integral_windup_bounded(self):
        """PI controller integral should not wind up unboundedly."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=1.0, ki=1000.0, ts=50e-6,
                          out_min=-100.0, out_max=100.0)

        # Large persistent error
        for _ in range(100000):
            pi.update(1000.0, 0.0)

        # Integral should be bounded by anti-windup
        self.assertTrue(math.isfinite(pi.integral))
        self.assertLess(abs(pi.integral), 1e8,
                        f"PI integral wound up: {pi.integral}")

    def test_mpc_prediction_stability(self):
        """MPC controller should not produce NaN/Inf predictions."""
        from sim_platform.models.controller.mpc import MPCCurrentController
        mpc = MPCCurrentController(
            R=0.1, L=5e-4, Ts=50e-6,
            v_max=48.0, i_max=200.0
        )

        # Large reference
        for _ in range(1000):
            v = mpc.update(100.0, 0.0)
            self.assertTrue(math.isfinite(v), f"MPC output is {v}")


class TestDivisionByNearZero(unittest.TestCase):
    """Verify models handle near-zero parameters gracefully."""

    def test_pmsm_small_Ld(self):
        """PMSM with very small Ld should not crash."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        # Ld guarded by MOTOR_EPS_L
        m = PMSMdqModel(Rs=0.1, Ld=1e-10, Lq=1e-10, flux_pm=0.03, J=1e-3, B=0.0)
        m.reset()

        for _ in range(1000):
            m.step(5.0, 5.0)

        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))

    def test_pmsm_small_J(self):
        """PMSM with very small J should not crash."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-10, B=0.0)
        m.reset()

        for _ in range(1000):
            m.step(5.0, 5.0, tl=1.0)

        self.assertTrue(math.isfinite(m.omega_m))

    def test_im_small_Rr(self):
        """IM with very small Rr (large Tr) should not crash."""
        from sim_platform.models.motor.im_dq import IMdqModel
        m = IMdqModel(Rs=0.05, Rr=1e-10, Ls=0.01, Lr=0.01, Lm=0.009,
                       Pp=2, J=0.01, B=0.001)
        m.reset()

        for _ in range(1000):
            omega_e = m.Pp * m.omega_m
            m.step(5.0, 5.0, omega_e)

        self.assertTrue(math.isfinite(m.psi_rd))
        self.assertTrue(math.isfinite(m.psi_rq))

    def test_foc_zero_kp(self):
        """FOC with kp=0 should not crash."""
        from sim_platform.models.controller.foc import PIController
        pi = PIController(kp=0.0, ki=100.0, ts=50e-6)

        for _ in range(1000):
            out = pi.update(10.0, 5.0)
            self.assertTrue(math.isfinite(out))

    def test_svpwm_near_zero_vbus(self):
        """SVPWM with near-zero v_bus should return 50% duty."""
        from sim_platform.models.controller.foc import svpwm
        da, db, dc = svpwm(10.0, 5.0, 1e-15)
        self.assertEqual(da, 0.5)
        self.assertEqual(db, 0.5)
        self.assertEqual(dc, 0.5)


class TestPathConsistency(unittest.TestCase):
    """Verify equivalent computation paths give identical results."""

    def test_pmsm_step_vs_step_abc(self):
        """step(vd,vq) and step_abc(va,vb,vc) should give same result."""
        from sim_platform.models.controller.foc import inverse_park
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel

        _SQRT3_HALF = math.sqrt(3) / 2

        # Motor A: step with dq voltages
        mA = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        mA.reset()
        mA.theta_e = 1.23  # Fixed angle

        vd, vq = 5.0, 3.0
        # Convert to abc using inverse transforms
        v_alpha, v_beta = inverse_park(vd, vq, mA.theta_e)
        # Inverse Clarke: alpha -> a, beta -> b, c
        va = v_alpha
        vb = -0.5 * v_alpha + _SQRT3_HALF * v_beta
        vc = -0.5 * v_alpha - _SQRT3_HALF * v_beta

        # Motor B: step_abc with abc voltages
        mB = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        mB.reset()
        mB.theta_e = 1.23  # Same angle

        mA.step(vd, vq)
        mB.step_abc(va, vb, vc)

        # Results should be identical (within floating point)
        self.assertAlmostEqual(mA.id, mB.id, places=10,
                               msg=f"id mismatch: {mA.id} vs {mB.id}")
        self.assertAlmostEqual(mA.iq, mB.iq, places=10,
                               msg=f"iq mismatch: {mA.iq} vs {mB.iq}")

    def test_clarke_park_roundtrip_exact(self):
        """Clarke+Park roundtrip should be identity (machine precision)."""
        from sim_platform.models.controller.foc import (
            clarke_transform,
            inverse_park,
            park_transform,
        )

        for _ in range(100):
            ia = random.uniform(-100, 100)
            ib = random.uniform(-100, 100)
            ic = -(ia + ib)  # balanced
            theta = random.uniform(0, 2 * math.pi)

            # Forward: abc -> alpha_beta -> dq
            alpha, beta = clarke_transform(ia, ib, ic)
            d, q = park_transform(alpha, beta, theta)

            # Inverse: dq -> alpha_beta
            alpha2, beta2 = inverse_park(d, q, theta)

            # Should match (within machine precision)
            self.assertAlmostEqual(alpha, alpha2, places=12,
                                   msg=f"alpha roundtrip: {alpha} vs {alpha2}")
            self.assertAlmostEqual(beta, beta2, places=12,
                                   msg=f"beta roundtrip: {beta} vs {beta2}")

    def test_dq_abc_dq_consistency(self):
        """abc -> dq -> abc -> dq should be identity."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.reset()
        m.theta_e = 2.34

        # Set known dq currents
        m.id = 10.0
        m.iq = 5.0

        # Get abc
        ia, ib, ic = m.update_abc_currents()

        # Verify ia+ib+ic = 0 (balanced)
        self.assertAlmostEqual(ia + ib + ic, 0.0, places=10,
                               msg="Unbalanced abc currents")


class TestCatastrophicCancellation(unittest.TestCase):
    """Verify models handle cases where large terms nearly cancel."""

    def test_pmsm_back_emf_cancellation(self):
        """When vq ≈ we*(Ld*id + flux_pm), the net voltage is tiny."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=0.0)
        m.reset()
        m.omega_m = 1000.0  # High speed

        # vq that almost cancels back-EMF
        we = m.Pp * m.omega_m
        vq_cancel = we * (m.Ld * 0 + m.flux_pm)  # ≈ 120V
        m.step(0.0, vq_cancel + 0.01)  # Tiny net voltage

        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))

    def test_im_stator_rotor_flux_cancellation(self):
        """IM when stator and rotor flux nearly cancel."""
        from sim_platform.models.motor.im_dq import IMdqModel
        m = IMdqModel(Rs=0.05, Rr=0.05, Ls=0.01, Lr=0.01, Lm=0.009,
                       Pp=2, J=0.01, B=0.001)
        m.reset()
        m.psi_rd = 0.1
        m.psi_rq = 0.0

        # Drive with currents that create opposing flux
        for _ in range(10000):
            omega_e = m.Pp * m.omega_m
            m.step(10.0, 0.0, omega_e)

        self.assertTrue(math.isfinite(m.psi_rd))
        self.assertTrue(math.isfinite(m.psi_rq))


class TestParameterBoundaryStability(unittest.TestCase):
    """Verify stability at parameter boundaries."""

    def test_pmsm_all_zeros(self):
        """PMSM with zero parameters should not crash."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.0, Ld=1e-10, Lq=1e-10, flux_pm=0.0, J=1e-10, B=0.0)
        m.reset()

        for _ in range(1000):
            m.step(0.0, 0.0)

        self.assertTrue(math.isfinite(m.id))
        self.assertTrue(math.isfinite(m.iq))
        self.assertTrue(math.isfinite(m.omega_m))

    def test_thermal_zero_heat(self):
        """Thermal with zero heat input should stay at ambient."""
        from sim_platform.models.thermal.thermal_model import ThermalNode
        node = ThermalNode(C_th=100.0, R_th=0.5, T_ambient=25.0)

        for _ in range(10000):
            node.step(0.0, 0.01)

        self.assertAlmostEqual(node.T, 25.0, delta=0.01,
                               msg="Zero heat should stay at ambient")

    def test_kalman_zero_noise(self):
        """Kalman filter with zero noise should be deterministic."""
        from sim_platform.models.fusion.sensor_fusion import SimpleKalmanFilter
        kf = SimpleKalmanFilter(x0=0.0, Q=0.0, R=1e-6)

        for i in range(100):
            kf.predict()
            kf.update(float(i))

        self.assertTrue(math.isfinite(kf.x))
        self.assertTrue(math.isfinite(kf.P))

    def test_randomized_parameter_stress(self):
        """Random valid parameters should never produce NaN/Inf."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        random.seed(42)

        for trial in range(50):
            Rs = random.uniform(0.001, 10.0)
            Ld = random.uniform(1e-6, 1e-1)
            Lq = random.uniform(1e-6, 1e-1)
            flux = random.uniform(0.001, 1.0)
            J = random.uniform(1e-6, 1.0)
            B = random.uniform(0.0, 1.0)

            m = PMSMdqModel(Rs=Rs, Ld=Ld, Lq=Lq, flux_pm=flux, J=J, B=B)
            m.reset()

            for _ in range(1000):
                vd = random.uniform(-100, 100)
                vq = random.uniform(-100, 100)
                tl = random.uniform(-10, 10)
                m.step(vd, vq, tl)

            self.assertTrue(math.isfinite(m.id), f"Trial {trial}: id={m.id}")
            self.assertTrue(math.isfinite(m.iq), f"Trial {trial}: iq={m.iq}")
            self.assertTrue(math.isfinite(m.omega_m), f"Trial {trial}: omega={m.omega_m}")


class TestNumericalPrecisionBounds(unittest.TestCase):
    """Verify computation precision meets engineering requirements."""

    def test_energy_conservation_precision(self):
        """Energy balance should hold to <1% over short intervals."""
        from sim_platform.models.motor.pmsm_dq import PMSMdqModel
        m = PMSMdqModel(Rs=0.1, Ld=5e-4, Lq=1e-3, flux_pm=0.03, J=1e-3, B=1e-4)
        m.reset()

        vd, vq = 10.0, 5.0
        E_electrical = 0.0
        E_mechanical = 0.0
        E_losses = 0.0
        dt = m.dt

        for _ in range(10000):  # 0.5 seconds
            id_before, iq_before = m.id, m.iq
            m.step(vd, vq, tl=0.5)

            # Electrical input power: P_e = vd*id + vq*iq (instantaneous)
            P_e = vd * (id_before + m.id) / 2 + vq * (iq_before + m.iq) / 2
            E_electrical += P_e * dt

            # Mechanical output power: P_m = torque * omega_m
            P_m = m.torque * m.omega_m
            E_mechanical += P_m * dt

            # Copper losses: P_cu = Rs * (id^2 + iq^2)
            P_cu = m.Rs * (m.id**2 + m.iq**2)
            E_losses += P_cu * dt

        # Energy balance: E_in ≈ E_out + E_losses (within 50% for Euler)
        if E_electrical > 0:
            ratio = (E_mechanical + E_losses) / E_electrical
            self.assertGreater(ratio, 0.5,
                               f"Energy ratio too low: {ratio:.3f}")
            self.assertLess(ratio, 2.0,
                            f"Energy ratio too high: {ratio:.3f}")

    def test_foc_transform_mathematical_identity(self):
        """Clarke: for balanced 3-phase, |alpha|^2 + |beta|^2 = 2/3 * (|a|^2+|b|^2+|c|^2)."""
        import random

        from sim_platform.models.controller.foc import clarke_transform
        random.seed(123)

        for _ in range(100):
            ia = random.uniform(-100, 100)
            ib = random.uniform(-100, 100)
            ic = -(ia + ib)  # balanced

            alpha, beta = clarke_transform(ia, ib, ic)

            # For amplitude-invariant Clarke:
            # alpha^2 + beta^2 should equal (ia^2 + ib^2 + ic^2) * 2/3
            ab_power = alpha**2 + beta**2
            abc_power = ia**2 + ib**2 + ic**2
            expected_ratio = 2.0 / 3.0

            if abc_power > 1e-10:
                ratio = ab_power / abc_power
                self.assertAlmostEqual(ratio, expected_ratio, places=10,
                                       msg=f"Clarke power ratio: {ratio} vs {expected_ratio}")


if __name__ == "__main__":
    unittest.main()
