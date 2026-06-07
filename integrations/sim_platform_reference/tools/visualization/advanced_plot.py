"""Advanced visualization tools for sim_platform.

Features:
- Multi-axis time series plots
- FFT frequency analysis
- Phase portrait (id-iq plane)
- Combined dashboard

Usage:
    from sim_platform.tools.visualization.advanced_plot import AdvancedPlotter
"""

import math

import matplotlib
import numpy as np

matplotlib.use("Agg")  # Non-interactive backend
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec


class AdvancedPlotter:
    """Advanced plotting tools for simulation results."""

    def __init__(self, data: dict[str, list[float]], dt: float = 50e-6):
        """Initialize with simulation data.

        Args:
            data: Dictionary of time series data.
            dt: Simulation time step (seconds).
        """
        self.data = data
        self.dt = dt
        self.time = np.array(data.get("time", []))
        self.n = len(self.time)

    def plot_dashboard(self, save_path: str | None = None,
                       show: bool = False) -> str:
        """Generate comprehensive dashboard with 6 subplots.

        Returns:
            Path to saved figure.
        """
        fig = plt.figure(figsize=(16, 12))
        gs = GridSpec(3, 2, figure=fig, hspace=0.35, wspace=0.3)

        # 1. Speed tracking
        ax1 = fig.add_subplot(gs[0, 0])
        self._plot_speed(ax1)

        # 2. dq currents
        ax2 = fig.add_subplot(gs[0, 1])
        self._plot_dq_currents(ax2)

        # 3. Phase currents
        ax3 = fig.add_subplot(gs[1, 0])
        self._plot_phase_currents(ax3)

        # 4. Torque
        ax4 = fig.add_subplot(gs[1, 1])
        self._plot_torque(ax4)

        # 5. id-iq phase portrait
        ax5 = fig.add_subplot(gs[2, 0])
        self._plot_phase_portrait(ax5)

        # 6. Speed FFT
        ax6 = fig.add_subplot(gs[2, 1])
        self._plot_speed_fft(ax6)

        fig.suptitle("sim_platform Simulation Dashboard", fontsize=14, fontweight="bold")

        if save_path is None:
            import os
            import time as _time
            save_path = os.path.join("output", f"dashboard_{int(_time.time())}.png")

        fig.savefig(save_path, dpi=150, bbox_inches="tight")
        if show:
            plt.show()
        plt.close(fig)

        return save_path

    def _plot_speed(self, ax):
        """Plot speed tracking."""
        speed = np.array(self.data.get("speed", []))
        speed_ref = np.array(self.data.get("speed_ref", []))

        if len(speed) == 0:
            ax.text(0.5, 0.5, "No data", ha="center", va="center")
            return

        ax.plot(self.time * 1000, speed, "b-", linewidth=1, label="Actual")
        ax.plot(self.time * 1000, speed_ref, "r--", linewidth=1, label="Reference")
        ax.set_xlabel("Time [ms]")
        ax.set_ylabel("Speed [rad/s]")
        ax.set_title("Speed Tracking")
        ax.legend(loc="best")
        ax.grid(True, alpha=0.3)

        # Metrics
        if len(speed) > 0 and len(speed_ref) > 0:
            error = abs(speed[-1] - speed_ref[-1]) / max(abs(speed_ref[-1]), 1) * 100
            ax.text(0.02, 0.98, f"Error: {error:.2f}%",
                    transform=ax.transAxes, va="top", fontsize=9,
                    bbox=dict(boxstyle="round", facecolor="wheat", alpha=0.5))

    def _plot_dq_currents(self, ax):
        """Plot dq-axis currents."""
        id_arr = np.array(self.data.get("id", []))
        iq_arr = np.array(self.data.get("iq", []))

        if len(id_arr) == 0:
            ax.text(0.5, 0.5, "No data", ha="center", va="center")
            return

        ax.plot(self.time * 1000, id_arr, "b-", linewidth=1, label="id")
        ax.plot(self.time * 1000, iq_arr, "r-", linewidth=1, label="iq")
        ax.set_xlabel("Time [ms]")
        ax.set_ylabel("Current [A]")
        ax.set_title("dq-axis Currents")
        ax.legend(loc="best")
        ax.grid(True, alpha=0.3)

    def _plot_phase_currents(self, ax):
        """Plot three-phase currents."""
        ia = np.array(self.data.get("ia", []))
        ib = np.array(self.data.get("ib", []))
        ic = np.array(self.data.get("ic", []))

        if len(ia) == 0:
            ax.text(0.5, 0.5, "No data", ha="center", va="center")
            return

        ax.plot(self.time * 1000, ia, "r-", linewidth=0.8, label="ia")
        ax.plot(self.time * 1000, ib, "g-", linewidth=0.8, label="ib")
        ax.plot(self.time * 1000, ic, "b-", linewidth=0.8, label="ic")
        ax.set_xlabel("Time [ms]")
        ax.set_ylabel("Current [A]")
        ax.set_title("Three-Phase Currents")
        ax.legend(loc="best")
        ax.grid(True, alpha=0.3)

    def _plot_torque(self, ax):
        """Plot electromagnetic torque."""
        torque = np.array(self.data.get("torque", []))

        if len(torque) == 0:
            ax.text(0.5, 0.5, "No data", ha="center", va="center")
            return

        ax.plot(self.time * 1000, torque * 1000, "purple", linewidth=1)
        ax.set_xlabel("Time [ms]")
        ax.set_ylabel("Torque [mN*m]")
        ax.set_title("Electromagnetic Torque")
        ax.grid(True, alpha=0.3)

        # Stats
        peak = np.max(np.abs(torque)) * 1000
        mean = np.mean(torque) * 1000
        ax.text(0.02, 0.98, f"Peak: {peak:.1f} mN*m\nMean: {mean:.1f} mN*m",
                transform=ax.transAxes, va="top", fontsize=9,
                bbox=dict(boxstyle="round", facecolor="wheat", alpha=0.5))

    def _plot_phase_portrait(self, ax):
        """Plot id-iq phase portrait."""
        id_arr = np.array(self.data.get("id", []))
        iq_arr = np.array(self.data.get("iq", []))

        if len(id_arr) == 0:
            ax.text(0.5, 0.5, "No data", ha="center", va="center")
            return

        # Color by time
        colors = plt.cm.viridis(np.linspace(0, 1, len(id_arr)))
        ax.scatter(id_arr, iq_arr, c=colors, s=1, alpha=0.5)

        # Start and end markers
        ax.plot(id_arr[0], iq_arr[0], "go", markersize=8, label="Start")
        ax.plot(id_arr[-1], iq_arr[-1], "rs", markersize=8, label="End")

        ax.set_xlabel("id [A]")
        ax.set_ylabel("iq [A]")
        ax.set_title("id-iq Phase Portrait")
        ax.legend(loc="best")
        ax.grid(True, alpha=0.3)
        ax.set_aspect("equal")

    def _plot_speed_fft(self, ax):
        """Plot speed frequency spectrum."""
        speed = np.array(self.data.get("speed", []))

        if len(speed) < 100:
            ax.text(0.5, 0.5, "Insufficient data", ha="center", va="center")
            return

        # Remove DC component
        speed_ac = speed - np.mean(speed)

        # FFT
        n = len(speed_ac)
        freq = np.fft.rfftfreq(n, d=self.dt)
        fft_vals = np.abs(np.fft.rfft(speed_ac)) / n

        # Plot (skip DC)
        ax.semilogy(freq[1:], fft_vals[1:], "b-", linewidth=0.8)
        ax.set_xlabel("Frequency [Hz]")
        ax.set_ylabel("Magnitude")
        ax.set_title("Speed Frequency Spectrum")
        ax.grid(True, alpha=0.3)

        # Find dominant frequency
        if len(fft_vals) > 1:
            dominant_idx = np.argmax(fft_vals[1:]) + 1
            dominant_freq = freq[dominant_idx]
            ax.axvline(dominant_freq, color="r", linestyle="--", alpha=0.5)
            ax.text(0.02, 0.98, f"Dominant: {dominant_freq:.1f} Hz",
                    transform=ax.transAxes, va="top", fontsize=9,
                    bbox=dict(boxstyle="round", facecolor="wheat", alpha=0.5))


def quick_dashboard(data: dict[str, list[float]], save_path: str | None = None) -> str:
    """Quick helper to generate dashboard from simulation data.

    Args:
        data: Simulation data dictionary.
        save_path: Optional path to save figure.

    Returns:
        Path to saved figure.
    """
    plotter = AdvancedPlotter(data)
    return plotter.plot_dashboard(save_path)


if __name__ == "__main__":
    # Demo with synthetic data

    dt = 50e-6
    steps = 20000
    t = [i * dt for i in range(steps)]
    speed = [100 * (1 - math.exp(-ti * 2)) + 2 * math.sin(2 * math.pi * 50 * ti) for ti in t]
    speed_ref = [100.0] * steps
    id_arr = [0.1 * math.sin(2 * math.pi * 100 * ti) for ti in t]
    iq_arr = [5.0 * (1 - math.exp(-ti * 3)) for ti in t]
    torque = [0.05 * iq_i + 0.01 * math.sin(2 * math.pi * 200 * ti) for iq_i, ti in zip(iq_arr, t)]

    data = {
        "time": t,
        "speed": speed,
        "speed_ref": speed_ref,
        "id": id_arr,
        "iq": iq_arr,
        "ia": [id_i * 0.8 + iq_i * 0.6 for id_i, iq_i in zip(id_arr, iq_arr)],
        "ib": [id_i * (-0.4) + iq_i * 0.866 for id_i, iq_i in zip(id_arr, iq_arr)],
        "ic": [-(ia + ib) for ia, ib in zip(
            [id_i * 0.8 + iq_i * 0.6 for id_i, iq_i in zip(id_arr, iq_arr)],
            [id_i * (-0.4) + iq_i * 0.866 for id_i, iq_i in zip(id_arr, iq_arr)]
        )],
        "torque": torque,
    }

    plotter = AdvancedPlotter(data, dt)
    path = plotter.plot_dashboard("demo_dashboard.png", show=False)
    print(f"Dashboard saved: {path}")
