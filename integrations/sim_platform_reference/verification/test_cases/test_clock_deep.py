"""Deep Clock Tests — Cover all execution paths.

Targets uncovered lines in clock.py (72% -> 85%+).
"""

import os
import sys
import time
import unittest

_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
sys.path.insert(0, _PROJ)

from sim_platform.core.clock import ClockMode, ClockState, GlobalClock, s_to_ns


class TestClockAdvanced(unittest.TestCase):
    """Test clock advanced features."""

    def test_s_to_ns_valid(self):
        """s_to_ns should convert seconds to nanoseconds."""
        self.assertEqual(s_to_ns(1.0), 1_000_000_000)
        self.assertEqual(s_to_ns(0.0), 0)
        self.assertEqual(s_to_ns(0.5), 500_000_000)

    def test_s_to_ns_nan(self):
        """s_to_ns should return 0 for NaN."""
        self.assertEqual(s_to_ns(float('nan')), 0)

    def test_s_to_ns_inf(self):
        """s_to_ns should return 0 for Inf."""
        self.assertEqual(s_to_ns(float('inf')), 0)

    def test_s_to_ns_negative(self):
        """s_to_ns should handle negative values."""
        self.assertEqual(s_to_ns(-1.0), -1_000_000_000)

    def test_advance_valid(self):
        """advance should accept valid positive int."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(1000)
        self.assertEqual(c.sim_time_ns, 1000)

    def test_advance_zero(self):
        """advance should accept zero."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(0)
        self.assertEqual(c.sim_time_ns, 0)

    def test_advance_negative_raises(self):
        """advance should reject negative values."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        with self.assertRaises(ValueError):
            c.advance(-1000)

    def test_advance_float_raises(self):
        """advance should reject float values."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        with self.assertRaises(TypeError):
            c.advance(1.5)

    def test_advance_increments_step_count(self):
        """advance should increment step count."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(1000)
        c.advance(1000)
        self.assertEqual(c.step_count, 2)

    def test_advance_when_paused(self):
        """advance should be no-op when paused."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.pause()
        c.advance(1000)
        self.assertEqual(c.sim_time_ns, 0)

    def test_pause_resume(self):
        """pause and resume should toggle state."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.pause()
        self.assertTrue(c._paused)
        c.resume()
        self.assertFalse(c._paused)

    def test_resume_when_not_paused(self):
        """resume when not paused should be no-op."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.resume()
        self.assertFalse(c._paused)

    def test_resume_recalculates_wall_start(self):
        """resume should recalculate wall start for realtime mode."""
        c = GlobalClock(mode=ClockMode.REALTIME)
        c.advance(1000)
        c.pause()
        time.sleep(0.01)
        c.resume()
        # Should have recalculated _wall_start_ns
        self.assertIsNotNone(c._wall_start_ns)

    def test_snapshot_restore(self):
        """snapshot and restore should preserve state."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(5000)
        state = c.snapshot()
        c.advance(1000)
        c.restore(state)
        self.assertEqual(c.sim_time_ns, 5000)

    def test_restore_invalid_negative_time(self):
        """restore should reject negative sim_time_ns."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        invalid_state = ClockState(
            sim_time_ns=-1,
            wall_time_base_ns=0,
            step_count=0,
            mode=ClockMode.OFFLINE,
        )
        with self.assertRaises(ValueError):
            c.restore(invalid_state)

    def test_restore_invalid_negative_step_count(self):
        """restore should reject negative step_count."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        invalid_state = ClockState(
            sim_time_ns=0,
            wall_time_base_ns=0,
            step_count=-1,
            mode=ClockMode.OFFLINE,
        )
        with self.assertRaises(ValueError):
            c.restore(invalid_state)

    def test_sim_time_s_property(self):
        """sim_time_s should return seconds."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(1_000_000_000)
        self.assertAlmostEqual(c.sim_time_s, 1.0)

    def test_diverged_property(self):
        """diverged property should reflect state."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        self.assertFalse(c.diverged)
        c.mark_diverged()
        self.assertTrue(c.diverged)

    def test_mode_change(self):
        """mode should be changeable."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.mode = ClockMode.REALTIME
        self.assertEqual(c.mode, ClockMode.REALTIME)

    def test_realtime_factor_default(self):
        """realtime_factor should default to 0.0 (no wall time elapsed)."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        self.assertEqual(c.realtime_factor, 0.0)


class TestClockRealtime(unittest.TestCase):
    """Test clock realtime mode paths."""

    def test_sync_wallclock_offline_noop(self):
        """_sync_wallclock should be no-op in offline mode."""
        c = GlobalClock(mode=ClockMode.OFFLINE)
        c.advance(1000)
        # Should not raise or block
        self.assertEqual(c.sim_time_ns, 1000)

    def test_sync_wallclock_realtime_basic(self):
        """_sync_wallclock should work in realtime mode."""
        c = GlobalClock(mode=ClockMode.REALTIME)
        c.advance(1000)
        # Should complete without blocking too long
        self.assertEqual(c.sim_time_ns, 1000)


if __name__ == "__main__":
    unittest.main()
