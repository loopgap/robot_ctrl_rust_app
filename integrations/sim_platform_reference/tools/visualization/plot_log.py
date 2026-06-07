"""Visualization tools — plot simulation results.

Supports CSV data and direct data structure plotting via matplotlib.

Security:
  - CWE-22: output_path sandboxed to prevent path traversal
  - CWE-400: input data size validation
"""

import os

import matplotlib

matplotlib.use("Agg")  # non-interactive backend

import matplotlib.pyplot as plt
import numpy as np

# Security: allowed output directories and extensions
_ALLOWED_EXT = (".png", ".svg", ".pdf")
_MAX_DATA_POINTS = 500_000


def _sanitize_path(path: str, default: str = "results.png") -> str:
    """Strip directory and enforce allowed extension."""
    basename = os.path.basename(path)
    _, ext = os.path.splitext(basename)
    if ext.lower() not in _ALLOWED_EXT:
        basename = os.path.splitext(basename)[0] + ".png"
    return basename


def plot_foc_results(data: dict[str, list[float]], output_path: str = "foc_results.png",
                     title: str = "PMSM FOC Simulation Results") -> str:
    """Generate multi-panel plot. Path is sandboxed to filename only."""
    # SECURITY (CWE-22): strip any path components
    safe_path = _sanitize_path(output_path)

    t = np.array(data.get("time", []))
    if len(t) == 0:
        raise ValueError("No time data")
    # SECURITY (CWE-400): reject oversized data
    if len(t) > _MAX_DATA_POINTS:
        raise ValueError(f"Data too large: {len(t)} points, max {_MAX_DATA_POINTS}")

    fig, axes = plt.subplots(3, 2, figsize=(14, 12))
    # Title: sanitize to prevent log injection
    safe_title = title.replace("\n", " ").replace("\r", "")[:200]
    fig.suptitle(safe_title, fontsize=14, fontweight="bold")

    # Speed tracking
    ax = axes[0, 0]
    has_labels = False
    if "speed_ref" in data:
        ax.plot(t, data["speed_ref"], "k--", label="Speed Ref", linewidth=1)
        has_labels = True
    if "speed" in data:
        ax.plot(t, data["speed"], "b-", label="Speed Meas", linewidth=1)
        has_labels = True
    ax.set_ylabel("Speed [rad/s]")
    if has_labels:
        ax.legend()
    ax.grid(True); ax.set_title("Speed Tracking")

    # dq currents
    ax = axes[0, 1]
    has_labels = False
    if "id" in data:
        ax.plot(t, data["id"], "r-", label="id", linewidth=1)
        has_labels = True
    if "iq" in data:
        ax.plot(t, data["iq"], "b-", label="iq", linewidth=1)
        has_labels = True
    ax.set_ylabel("Current [A]")
    if has_labels:
        ax.legend()
    ax.grid(True); ax.set_title("d-q Axis Currents")

    # Phase currents
    ax = axes[1, 0]
    has_labels = False
    for ph, color in [("ia", "r"), ("ib", "g"), ("ic", "b")]:
        if ph in data:
            ax.plot(t, data[ph], color=color, label=ph, linewidth=0.8)
            has_labels = True
    ax.set_ylabel("Current [A]")
    if has_labels:
        ax.legend()
    ax.grid(True); ax.set_title("Phase Currents")

    # Torque
    ax = axes[1, 1]
    if "torque" in data:
        ax.plot(t, data["torque"], "m-", label="Torque", linewidth=1)
        ax.legend()
    ax.set_ylabel("Torque [N·m]")
    ax.grid(True); ax.set_title("Electromagnetic Torque")

    # Duty cycles
    ax = axes[2, 0]
    has_labels = False
    for ph, color in [("duty_a", "r"), ("duty_b", "g"), ("duty_c", "b")]:
        if ph in data:
            ax.plot(t, data[ph], color=color, label=ph, linewidth=0.8)
            has_labels = True
    ax.set_ylabel("Duty Cycle"); ax.set_xlabel("Time [s]")
    if has_labels:
        ax.legend()
    ax.grid(True); ax.set_title("PWM Duty Cycles")

    # dq voltages
    ax = axes[2, 1]
    has_labels = False
    if "vd" in data:
        ax.plot(t, data["vd"], "r-", label="vd_ref", linewidth=1)
        has_labels = True
    if "vq" in data:
        ax.plot(t, data["vq"], "b-", label="vq_ref", linewidth=1)
        has_labels = True
    ax.set_ylabel("Voltage [V]"); ax.set_xlabel("Time [s]")
    if has_labels:
        ax.legend()
    ax.grid(True); ax.set_title("d-q Voltage References")

    plt.tight_layout()
    fig.savefig(safe_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    return safe_path


def plot_quick(x: list[float], y: list[float],
               xlabel: str = "Time [s]", ylabel: str = "",
               title: str = "", output_path: str = "quick.png") -> str:
    """Quick single-panel plot with path sanitization."""
    safe_path = _sanitize_path(output_path)
    safe_title = title.replace("\n", " ").replace("\r", "")[:200]
    # CWE-400: size guard
    if len(x) > _MAX_DATA_POINTS or len(y) > _MAX_DATA_POINTS:
        raise ValueError(f"Data too large: {max(len(x), len(y))} points")
    fig, ax = plt.subplots(figsize=(8, 4))
    ax.plot(x, y, linewidth=1)
    ax.set_xlabel(xlabel)
    ax.set_ylabel(ylabel)
    ax.set_title(safe_title)
    ax.grid(True)
    fig.savefig(safe_path, dpi=100, bbox_inches="tight")
    plt.close(fig)
    return safe_path
