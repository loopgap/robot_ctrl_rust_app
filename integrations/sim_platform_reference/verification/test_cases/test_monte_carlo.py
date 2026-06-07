"""Monte Carlo parameter sensitivity analysis for sim_platform.

Tests system robustness under random parameter perturbations:
  - Motor parameters (Rs, Ld, Lq, flux_pm, J, B)
  - Controller gains (kp, ki)
  - Load torque variations
  - Initial condition variations

Usage:
    python -m pytest verification/test_cases/test_monte_carlo.py -v
"""

import math
import os
import random
import statistics
import sys
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.models.controller.foc import FOCController, SpeedController
from sim_platform.models.motor.pmsm_dq import PMSMdqModel
from sim_platform.models.power.power_models import AverageInverter


def run_closed_loop(motor_params, ctrl_params, speed_ref=100.0, steps=2000, dt=50e-6):
    """Run a closed-loop PMSM+FOC simulation and return results."""
    m = PMSMdqModel(**motor_params, dt_ns=int(dt * 1e9))
    foc = FOCController(**ctrl_params, ts=dt)
    sc = SpeedController(kp=ctrl_params.get("kp_iq", 5.0) * 0.01,
                         ki=ctrl_params.get("ki_iq", 500.0) * 0.001,
                         ts=1e-3)
    inv = AverageInverter(v_bus=48.0)

    results = {"speed": [], "id": [], "iq": [], "torque": []}

    for _ in range(steps):
        iq_ref = sc.update(speed_ref, m.omega_m)
        ia, ib, ic = m.update_abc_currents()
        da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
        va, vb, vc = inv.step(da, db, dc)
        m.step_abc(va, vb, vc)

        results["speed"].append(m.omega_m)
        results["id"].append(m.id)
        results["iq"].append(m.iq)
        results["torque"].append(m.torque)

    return results


class TestMonteCarloMotorParams(unittest.TestCase):
    """Monte Carlo: random motor parameter perturbations."""

    def test_random_rs_perturbation(self):
        """Rs ±50% should not cause instability."""
        base = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            params = base.copy()
            params["Rs"] = base["Rs"] * random.uniform(0.5, 1.5)
            r = run_closed_loop(params, ctrl, steps=1000)
            # All values must be finite
            for v in r["speed"]:
                self.assertTrue(math.isfinite(v), f"NaN speed with Rs={params['Rs']}")

    def test_random_inductance_perturbation(self):
        """Ld/Lq ±50% should not cause instability."""
        base = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            params = base.copy()
            params["Ld"] = base["Ld"] * random.uniform(0.5, 1.5)
            params["Lq"] = base["Lq"] * random.uniform(0.5, 1.5)
            r = run_closed_loop(params, ctrl, steps=1000)
            for v in r["speed"]:
                self.assertTrue(math.isfinite(v))

    def test_random_flux_perturbation(self):
        """flux_pm ±30% should not cause instability."""
        base = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            params = base.copy()
            params["flux_pm"] = base["flux_pm"] * random.uniform(0.7, 1.3)
            r = run_closed_loop(params, ctrl, steps=1000)
            for v in r["speed"]:
                self.assertTrue(math.isfinite(v))

    def test_random_inertia_perturbation(self):
        """J ±80% should not cause instability."""
        base = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            params = base.copy()
            params["J"] = base["J"] * random.uniform(0.2, 1.8)
            r = run_closed_loop(params, ctrl, steps=1000)
            for v in r["speed"]:
                self.assertTrue(math.isfinite(v))

    def test_all_params_random(self):
        """All parameters random simultaneously should not cause instability."""
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(30):
            params = {
                "Rs": 0.1 * random.uniform(0.3, 2.0),
                "Ld": 5e-4 * random.uniform(0.3, 2.0),
                "Lq": 1e-3 * random.uniform(0.3, 2.0),
                "flux_pm": 0.03 * random.uniform(0.5, 1.5),
                "J": 1e-3 * random.uniform(0.1, 3.0),
                "B": 0.001 * random.uniform(0.0, 2.0),
            }
            r = run_closed_loop(params, ctrl, steps=1000)
            for v in r["speed"]:
                self.assertTrue(math.isfinite(v), f"NaN with params={params}")


class TestMonteCarloControllerGains(unittest.TestCase):
    """Monte Carlo: random controller gain perturbations."""

    def test_random_foc_gains(self):
        """FOC gains ±50% should not cause instability."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}

        for _ in range(20):
            ctrl = {
                "kp_id": 5.0 * random.uniform(0.5, 1.5),
                "ki_id": 500.0 * random.uniform(0.5, 1.5),
                "kp_iq": 5.0 * random.uniform(0.5, 1.5),
                "ki_iq": 500.0 * random.uniform(0.5, 1.5),
            }
            r = run_closed_loop(motor, ctrl, steps=1000)
            for v in r["speed"]:
                self.assertTrue(math.isfinite(v))

    def test_random_speed_gains(self):
        """Speed controller gains ±80% should not cause instability."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            kp = 0.05 * random.uniform(0.2, 1.8)
            ki = 0.5 * random.uniform(0.2, 1.8)
            sc = SpeedController(kp=kp, ki=ki, ts=1e-3)
            m = PMSMdqModel(**motor, dt_ns=50000)
            foc = FOCController(**ctrl, ts=50e-6)
            inv = AverageInverter()

            for _ in range(1000):
                iq_ref = sc.update(100.0, m.omega_m)
                ia, ib, ic = m.update_abc_currents()
                da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
                va, vb, vc = inv.step(da, db, dc)
                m.step_abc(va, vb, vc)

            self.assertTrue(math.isfinite(m.omega_m), f"NaN with kp={kp}, ki={ki}")


class TestMonteCarloLoadDisturbance(unittest.TestCase):
    """Monte Carlo: random load torque disturbances."""

    def test_random_load_torque(self):
        """Random load torque should not cause instability."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}
        m = PMSMdqModel(**motor, dt_ns=50000)
        foc = FOCController(**ctrl, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter()

        for step in range(5000):
            # Random load torque between -0.5 and 1.0 Nm
            tl = random.uniform(-0.5, 1.0)
            iq_ref = sc.update(100.0, m.omega_m)
            ia, ib, ic = m.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            m.step_abc(va, vb, vc, tl=tl)

            self.assertTrue(math.isfinite(m.omega_m), f"NaN at step {step} with tl={tl}")

    def test_step_load_disturbance(self):
        """Sudden load step should recover."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}
        m = PMSMdqModel(**motor, dt_ns=50000)
        foc = FOCController(**ctrl, ts=50e-6)
        sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
        inv = AverageInverter()

        for step in range(5000):
            # Step load at step 2000
            tl = 0.5 if step > 2000 else 0.0
            iq_ref = sc.update(100.0, m.omega_m)
            ia, ib, ic = m.update_abc_currents()
            da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
            va, vb, vc = inv.step(da, db, dc)
            m.step_abc(va, vb, vc, tl=tl)

            self.assertTrue(math.isfinite(m.omega_m))

        # After settling, speed should be near reference
        self.assertGreater(m.omega_m, 50.0, "Speed dropped too low after load step")


class TestMonteCarloInitialConditions(unittest.TestCase):
    """Monte Carlo: random initial conditions."""

    def test_random_initial_speed(self):
        """Random initial speed should not cause instability."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            m = PMSMdqModel(**motor, dt_ns=50000)
            m.omega_m = random.uniform(-500, 500)
            foc = FOCController(**ctrl, ts=50e-6)
            sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
            inv = AverageInverter()

            for _ in range(2000):
                iq_ref = sc.update(100.0, m.omega_m)
                ia, ib, ic = m.update_abc_currents()
                da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
                va, vb, vc = inv.step(da, db, dc)
                m.step_abc(va, vb, vc)

            self.assertTrue(math.isfinite(m.omega_m))

    def test_random_initial_currents(self):
        """Random initial currents should not cause instability."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}

        for _ in range(20):
            m = PMSMdqModel(**motor, dt_ns=50000)
            m.id = random.uniform(-100, 100)
            m.iq = random.uniform(-100, 100)
            foc = FOCController(**ctrl, ts=50e-6)
            sc = SpeedController(kp=0.05, ki=0.5, ts=1e-3)
            inv = AverageInverter()

            for _ in range(2000):
                iq_ref = sc.update(100.0, m.omega_m)
                ia, ib, ic = m.update_abc_currents()
                da, db, dc = foc.update(ia, ib, ic, m.theta_e, 0.0, iq_ref)
                va, vb, vc = inv.step(da, db, dc)
                m.step_abc(va, vb, vc)

            self.assertTrue(math.isfinite(m.omega_m))
            self.assertTrue(math.isfinite(m.id))


class TestMonteCarloStatistics(unittest.TestCase):
    """Monte Carlo: statistical analysis of simulation outcomes."""

    def test_speed_convergence_statistics(self):
        """Speed should converge to reference in most cases."""
        motor = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}
        speed_ref = 100.0
        final_speeds = []

        for _ in range(50):
            params = {k: v * random.uniform(0.8, 1.2) for k, v in motor.items()}
            r = run_closed_loop(params, ctrl, speed_ref=speed_ref, steps=3000)
            final_speeds.append(r["speed"][-1])

        # Statistics
        mean_speed = statistics.mean(final_speeds)
        std_speed = statistics.stdev(final_speeds) if len(final_speeds) > 1 else 0

        # All finite
        self.assertTrue(all(math.isfinite(s) for s in final_speeds))
        # Mean should be positive and moving toward reference
        self.assertGreater(mean_speed, 0, "Mean speed should be positive")
        # Standard deviation should be reasonable
        self.assertLess(std_speed, speed_ref * 1.0,
                        f"Speed variation too high: std={std_speed:.1f}")

    def test_no_nan_in_monte_carlo(self):
        """Zero NaN across 100 random simulations."""
        motor_base = {"Rs": 0.1, "Ld": 5e-4, "Lq": 1e-3, "flux_pm": 0.03, "J": 1e-3, "B": 0.0}
        ctrl = {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0}
        nan_count = 0

        for _ in range(100):
            params = {k: v * random.uniform(0.5, 2.0) for k, v in motor_base.items()}
            try:
                r = run_closed_loop(params, ctrl, steps=500)
                if any(not math.isfinite(v) for v in r["speed"]):
                    nan_count += 1
            except Exception:
                nan_count += 1

        self.assertEqual(nan_count, 0, f"NaN in {nan_count}/100 simulations")


if __name__ == "__main__":
    unittest.main()
