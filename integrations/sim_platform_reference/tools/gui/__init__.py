"""sim_platform PySide6 GUI package.

Provides a desktop GUI for PMSM FOC simulation, replacing the Textual TUI.
"""

from __future__ import annotations


def main():
    """Entry point for the sim_platform GUI."""
    from sim_platform.tools.gui.app import run_app

    run_app()
