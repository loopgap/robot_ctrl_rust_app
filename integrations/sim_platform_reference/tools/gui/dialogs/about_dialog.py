"""About dialog for sim_platform GUI."""

from __future__ import annotations

import os

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QDialog,
    QLabel,
    QPushButton,
    QVBoxLayout,
)

from sim_platform.tools.gui.i18n import tr


def _get_version() -> str:
    """Read version from pyproject.toml or __init__.py."""
    import sys
    candidates = []
    if getattr(sys, "frozen", False):
        candidates.append(os.path.join(sys._MEIPASS, "sim_platform", "pyproject.toml"))
    candidates.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../..", "pyproject.toml")))
    for toml_path in candidates:
        try:
            if os.path.exists(toml_path):
                with open(toml_path, encoding="utf-8") as f:
                    for line in f:
                        if line.strip().startswith("version"):
                            return line.split("=")[1].strip().strip('"').strip("'")
        except Exception:
            continue
    # Fallback: try __init__.py
    try:
        from sim_platform import __version__
        return __version__
    except Exception:
        pass
    return "1.3.0"


class AboutDialog(QDialog):
    """Simple about dialog showing application info."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(tr("dialog.about.title"))
        self.setFixedSize(380, 220)
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(8)

        title = QLabel("sim_platform")
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        title.setStyleSheet("font-size: 20px; font-weight: bold; color: #89b4fa;")
        layout.addWidget(title)

        subtitle = QLabel("Multi-Domain Co-Simulation Platform")
        subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        subtitle.setStyleSheet("font-size: 13px; color: #a6adc8;")
        layout.addWidget(subtitle)

        version = QLabel(f"Version {_get_version()}")
        version.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(version)

        desc = QLabel(
            "PySide6 GUI for PMSM FOC simulation.\n"
            "Replaces Textual TUI with a modern desktop interface."
        )
        desc.setAlignment(Qt.AlignmentFlag.AlignCenter)
        desc.setWordWrap(True)
        desc.setStyleSheet("color: #bac2de;")
        layout.addWidget(desc)

        close_btn = QPushButton("Close")
        close_btn.clicked.connect(self.accept)
        layout.addWidget(close_btn, alignment=Qt.AlignmentFlag.AlignCenter)
