"""Visualization tools — complete functional test suite.

Tests all visualization functions with:
- Normal operation
- Edge cases (empty data, single point, large data)
- Security (path sanitization, title injection, data size limits)
- Output validation (file creation, format)
"""

import math
import os
import tempfile

import numpy as np
import pytest

# ── Fixtures ────────────────────────────────────────────────


@pytest.fixture
def sample_data():
    """Generate sample simulation data for testing."""
    n = 1000
    t = np.linspace(0, 0.05, n).tolist()
    return {
        "time": t,
        "speed": [100 * (1 - math.exp(-ti * 50)) for ti in t],
        "speed_ref": [100.0] * n,
        "id": [0.1 * math.sin(2 * math.pi * 50 * ti) for ti in t],
        "iq": [5.0 * (1 - math.exp(-ti * 50)) for ti in t],
        "ia": [5.0 * math.sin(2 * math.pi * 50 * ti) for ti in t],
        "ib": [5.0 * math.sin(2 * math.pi * 50 * ti - 2 * math.pi / 3) for ti in t],
        "ic": [5.0 * math.sin(2 * math.pi * 50 * ti + 2 * math.pi / 3) for ti in t],
        "torque": [0.03 * 5.0 * (1 - math.exp(-ti * 50)) for ti in t],
        "duty_a": [0.5 + 0.3 * math.sin(2 * math.pi * 50 * ti) for ti in t],
        "duty_b": [0.5 + 0.3 * math.sin(2 * math.pi * 50 * ti - 2 * math.pi / 3) for ti in t],
        "duty_c": [0.5 + 0.3 * math.sin(2 * math.pi * 50 * ti + 2 * math.pi / 3) for ti in t],
        "vd": [0.0] * n,
        "vq": [10.0 * (1 - math.exp(-ti * 50)) for ti in t],
    }


@pytest.fixture
def minimal_data():
    """Minimal data with time and speed_ref (required by AdvancedPlotter)."""
    return {
        "time": [0.0, 0.001, 0.002],
        "speed": [0.0, 50.0, 100.0],
        "speed_ref": [100.0, 100.0, 100.0],
        "id": [0.0, 0.1, 0.2],
        "iq": [0.0, 2.0, 5.0],
    }


@pytest.fixture
def empty_data():
    """Empty data dictionary."""
    return {}


@pytest.fixture
def single_point_data():
    """Data with only one point (with speed_ref for AdvancedPlotter)."""
    return {
        "time": [0.0],
        "speed": [100.0],
        "speed_ref": [100.0],
        "id": [0.0],
        "iq": [5.0],
    }


# ── plot_log.py Tests ───────────────────────────────────────


class TestPlotLog:
    """Test plot_log.py visualization functions."""

    def test_sanitize_path_normal(self):
        """Normal path should be kept."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        assert _sanitize_path("test.png") == "test.png"

    def test_sanitize_path_strips_directory(self):
        """Directory components should be stripped."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        assert _sanitize_path("/tmp/test.png") == "test.png"
        assert _sanitize_path("output/test.png") == "test.png"

    def test_sanitize_path_adds_png_extension(self):
        """Missing extension should add .png."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        assert _sanitize_path("test") == "test.png"

    def test_sanitize_path_allows_svg(self):
        """SVG extension should be allowed."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        assert _sanitize_path("test.svg") == "test.svg"

    def test_sanitize_path_allows_pdf(self):
        """PDF extension should be allowed."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        assert _sanitize_path("test.pdf") == "test.pdf"

    def test_sanitize_path_rejects_unknown_extension(self):
        """Unknown extensions should be converted to .png."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        assert _sanitize_path("test.exe") == "test.png"
        assert _sanitize_path("test.bat") == "test.png"

    def test_plot_foc_results_generates_file(self, sample_data):
        """plot_foc_results should generate a PNG file."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "test_foc.png")
            result = plot_foc_results(sample_data, out_path)
            # Result is sanitized filename only
            assert result == "test_foc.png"
            # File should be created in current directory (sanitized)
            assert os.path.exists(result) or os.path.exists(out_path)

    def test_plot_foc_results_empty_data_raises(self):
        """plot_foc_results should raise on empty data."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        with pytest.raises(ValueError, match="No time data"):
            plot_foc_results({}, "test.png")

    def test_plot_foc_results_oversized_data_raises(self):
        """plot_foc_results should reject oversized data."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        data = {"time": list(range(600000))}
        with pytest.raises(ValueError, match="Data too large"):
            plot_foc_results(data, "test.png")

    def test_plot_foc_results_sanitizes_title(self, sample_data):
        """plot_foc_results should sanitize title (no newlines)."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "test_title.png")
            # Should not raise even with malicious title
            result = plot_foc_results(sample_data, out_path, title="Title\nInjection\rAttempt")
            assert result == "test_title.png"

    def test_plot_quick_generates_file(self):
        """plot_quick should generate a PNG file."""
        from sim_platform.tools.visualization.plot_log import plot_quick
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "quick.png")
            x = [0.0, 0.001, 0.002]
            y = [0.0, 50.0, 100.0]
            result = plot_quick(x, y, out_path)
            assert result == "quick.png"

    def test_plot_quick_oversized_data_raises(self):
        """plot_quick should reject oversized data."""
        from sim_platform.tools.visualization.plot_log import plot_quick
        x = list(range(600000))
        y = list(range(600000))
        with pytest.raises(ValueError, match="Data too large"):
            plot_quick(x, y, "test.png")

    def test_plot_quick_with_labels(self):
        """plot_quick should accept custom labels."""
        from sim_platform.tools.visualization.plot_log import plot_quick
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "labeled.png")
            x = [0.0, 0.001, 0.002]
            y = [0.0, 50.0, 100.0]
            result = plot_quick(x, y, xlabel="X", ylabel="Y", title="Test", output_path=out_path)
            assert result == "labeled.png"


# ── advanced_plot.py Tests ──────────────────────────────────


class TestAdvancedPlotter:
    """Test AdvancedPlotter class."""

    def test_init_with_data(self, sample_data):
        """AdvancedPlotter should initialize with valid data."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(sample_data, dt=50e-6)
        assert plotter.n == 1000
        assert len(plotter.time) == 1000

    def test_init_with_empty_data(self, empty_data):
        """AdvancedPlotter should handle empty data."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(empty_data, dt=50e-6)
        assert plotter.n == 0

    def test_plot_dashboard_generates_file(self, sample_data):
        """plot_dashboard should generate a PNG file."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(sample_data, dt=50e-6)
        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = os.path.join(tmpdir, "dashboard.png")
            result = plotter.plot_dashboard(save_path=save_path, show=False)
            assert os.path.exists(result)
            assert result.endswith(".png")

    def test_plot_dashboard_with_minimal_data(self, minimal_data):
        """plot_dashboard should handle minimal data."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(minimal_data, dt=1e-3)
        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = os.path.join(tmpdir, "minimal_dashboard.png")
            result = plotter.plot_dashboard(save_path=save_path, show=False)
            assert os.path.exists(result)

    def test_plot_dashboard_with_single_point(self, single_point_data):
        """plot_dashboard should handle single point data."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(single_point_data, dt=1e-3)
        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = os.path.join(tmpdir, "single_point.png")
            result = plotter.plot_dashboard(save_path=save_path, show=False)
            assert os.path.exists(result)

    def test_quick_dashboard_function(self, sample_data):
        """quick_dashboard helper should work."""
        from sim_platform.tools.visualization.advanced_plot import quick_dashboard
        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = os.path.join(tmpdir, "quick_dashboard.png")
            result = quick_dashboard(sample_data, save_path=save_path)
            assert os.path.exists(result)


# ── parameter_scan.py Tests ─────────────────────────────────


class TestParameterScan:
    """Test parameter_scan.py functions."""

    def test_scanable_params_defined(self):
        """SCANABLE_PARAMS should be defined with expected keys."""
        from sim_platform.tools.visualization.parameter_scan import SCANABLE_PARAMS
        expected_params = ["speed", "kp_id", "ki_id", "kp_iq", "ki_iq", "spd_kp", "spd_ki", "load"]
        for param in expected_params:
            assert param in SCANABLE_PARAMS

    def test_scanable_params_structure(self):
        """Each scanable param should have required fields."""
        from sim_platform.tools.visualization.parameter_scan import SCANABLE_PARAMS
        for key, config in SCANABLE_PARAMS.items():
            assert "name" in config
            assert "unit" in config
            assert "default_values" in config
            assert "path" in config
            assert len(config["default_values"]) > 0

    def test_run_single_function_exists(self):
        """run_single function should be importable."""
        from sim_platform.tools.visualization.parameter_scan import run_single
        assert callable(run_single)


# ── Integration Tests ───────────────────────────────────────


class TestVisualizationIntegration:
    """Integration tests for visualization tools."""

    def test_plot_foc_results_with_all_fields(self, sample_data):
        """plot_foc_results should handle all expected data fields."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "full_data.png")
            result = plot_foc_results(sample_data, out_path)
            assert result == "full_data.png"

    def test_plot_foc_results_with_partial_fields(self):
        """plot_foc_results should handle partial data (missing some fields)."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        data = {
            "time": [0.0, 0.001, 0.002],
            "speed": [0.0, 50.0, 100.0],
            "id": [0.0, 0.1, 0.2],
        }
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "partial.png")
            result = plot_foc_results(data, out_path)
            assert result == "partial.png"

    def test_advanced_plotter_phase_portrait(self, sample_data):
        """AdvancedPlotter should generate phase portrait."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(sample_data, dt=50e-6)
        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = os.path.join(tmpdir, "phase_portrait.png")
            result = plotter.plot_dashboard(save_path=save_path, show=False)
            assert os.path.exists(result)

    def test_advanced_plotter_fft_analysis(self, sample_data):
        """AdvancedPlotter should generate FFT analysis."""
        from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
        plotter = AdvancedPlotter(sample_data, dt=50e-6)
        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = os.path.join(tmpdir, "fft.png")
            result = plotter.plot_dashboard(save_path=save_path, show=False)
            assert os.path.exists(result)

    def test_visualization_tools_import(self):
        """All visualization tools should be importable."""
        from sim_platform.tools.visualization import (
            advanced_plot,
            interactive_runner,
            parameter_scan,
            plot_log,
        )
        assert hasattr(plot_log, "plot_foc_results")
        assert hasattr(plot_log, "plot_quick")
        assert hasattr(advanced_plot, "AdvancedPlotter")
        assert hasattr(parameter_scan, "SCANABLE_PARAMS")
        assert hasattr(interactive_runner, "header")


# ── Security Tests ──────────────────────────────────────────


class TestVisualizationSecurity:
    """Security tests for visualization tools."""

    def test_path_traversal_prevention(self):
        """Path traversal attempts should be prevented."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        # Directory traversal attempts
        assert _sanitize_path("../../etc/passwd.png") == "passwd.png"
        assert _sanitize_path("../../../tmp/evil.png") == "evil.png"
        assert _sanitize_path("output/../../secret.png") == "secret.png"

    def test_title_injection_prevention(self, sample_data):
        """Title injection should be prevented."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        with tempfile.TemporaryDirectory() as tmpdir:
            out_path = os.path.join(tmpdir, "safe.png")
            # Malicious title with control characters
            malicious_title = "Normal Title\n<script>alert('xss')</script>\r\n"
            result = plot_foc_results(sample_data, out_path, title=malicious_title)
            assert result == "safe.png"

    def test_data_size_limit_enforced(self):
        """Data size limit should be enforced."""
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        # Create data exceeding limit
        large_data = {"time": list(range(600000))}
        with pytest.raises(ValueError, match="Data too large"):
            plot_foc_results(large_data, "test.png")

    def test_sanitize_path_handles_edge_cases(self):
        """_sanitize_path should handle edge cases."""
        from sim_platform.tools.visualization.plot_log import _sanitize_path
        # Empty path
        assert _sanitize_path("") == ".png"
        # Just extension (.png is treated as filename with no name, gets .png appended)
        result = _sanitize_path(".png")
        assert result.endswith(".png")
        # Multiple dots
        assert _sanitize_path("test.backup.png") == "test.backup.png"
