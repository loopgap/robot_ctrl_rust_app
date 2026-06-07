"""Read-only log widget with filtering and search.

Features:
- Auto-timestamp on each message
- Color coding (error/warning/success/info)
- Level filtering via combo box
- Text search with highlight
- Log export
- Message limit for memory safety
- Full bilingual support
"""

from __future__ import annotations

import time

from PySide6.QtCore import Slot
from PySide6.QtGui import QFont, QTextCursor
from PySide6.QtWidgets import (
    QComboBox,
    QHBoxLayout,
    QLineEdit,
    QPushButton,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)

from sim_platform.tools.gui.i18n import tr


class LogWidget(QWidget):
    """Log widget with filtering, search, and export capabilities."""

    _MAX_MESSAGES = 10000
    _START_TIME = time.monotonic()

    def __init__(self, parent=None):
        super().__init__(parent)
        self._setup_ui()
        self._message_count = 0
        self._all_messages: list[str] = []  # Store raw HTML for filtering

    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        # ── Filter bar ────────────────────────────────────
        filter_bar = QHBoxLayout()
        filter_bar.setContentsMargins(4, 4, 4, 0)

        self._level_filter = QComboBox()
        self._level_filter.addItems([
            tr("log.filter.all"), tr("log.filter.info"),
            tr("log.filter.warning"), tr("log.filter.error"),
            tr("log.filter.success"),
        ])
        self._level_filter.setMaximumWidth(100)
        self._level_filter.setToolTip(tr("tooltip.filter_level"))
        self._level_filter.currentTextChanged.connect(self._apply_filter)
        filter_bar.addWidget(self._level_filter)

        self._search_box = QLineEdit()
        self._search_box.setPlaceholderText(tr("log.search"))
        self._search_box.setClearButtonEnabled(True)
        self._search_box.setToolTip(tr("tooltip.search_log"))
        self._search_box.textChanged.connect(self._apply_filter)
        filter_bar.addWidget(self._search_box)

        self._btn_export = QPushButton(tr("log.export"))
        self._btn_export.setMaximumWidth(60)
        self._btn_export.setToolTip(tr("tooltip.export_log"))
        self._btn_export.clicked.connect(self._export_log)
        filter_bar.addWidget(self._btn_export)

        self._btn_clear = QPushButton(tr("log.clear"))
        self._btn_clear.setMaximumWidth(60)
        self._btn_clear.setToolTip(tr("tooltip.clear_log"))
        self._btn_clear.clicked.connect(self.clear_log)
        filter_bar.addWidget(self._btn_clear)

        layout.addLayout(filter_bar)

        # ── Text area ─────────────────────────────────────
        self._text_edit = QTextEdit()
        self._text_edit.setReadOnly(True)
        font = QFont("JetBrains Mono", 10)
        font.setStyleHint(QFont.StyleHint.Monospace)
        self._text_edit.setFont(font)
        self._text_edit.setPlaceholderText(tr("log.placeholder"))
        layout.addWidget(self._text_edit)

    def _sanitize(self, text: str) -> str:
        """Sanitize message to prevent log injection (CWE-117)."""
        return (
            text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace('"', "&quot;")
        )

    def _get_level(self, message: str) -> str:
        """Detect log level from message content."""
        lower = message.lower()
        if "error" in lower or "fail" in lower:
            return "Error"
        elif "warn" in lower:
            return "Warning"
        elif "complete" in lower or "saved" in lower:
            return "Success"
        return "Info"

    @Slot(str)
    def append_log(self, message: str):
        """Append a timestamped, color-coded, sanitized log message."""
        self._message_count += 1
        if self._message_count > self._MAX_MESSAGES:
            return

        elapsed = time.monotonic() - self._START_TIME
        timestamp = f"[{elapsed:8.3f}s]"
        sanitized = self._sanitize(message)
        level = self._get_level(message)

        # Apple semantic colors
        color_map = {
            "Error": "#FF453A",
            "Warning": "#FF9F0A",
            "Success": "#30D158",
            "Info": "rgba(245, 245, 247, 0.55)",
        }
        color = color_map[level]

        html = (
            f'<span style="color: rgba(245, 245, 247, 0.25); '
            f'font-size: 10px;">{timestamp}</span> '
            f'<span style="color: {color};">{sanitized}</span>'
        )

        # Store for filtering
        self._all_messages.append((level, message.lower(), html))

        # Apply current filter
        if self._should_show(level, message.lower()):
            self._text_edit.append(html)
            cursor = self._text_edit.textCursor()
            cursor.movePosition(QTextCursor.MoveOperation.End)
            self._text_edit.setTextCursor(cursor)

    def _should_show(self, level: str, message_lower: str) -> bool:
        """Check if message passes current filters."""
        # Level filter — compare against translated filter text
        level_filter = self._level_filter.currentText()
        level_map = {
            tr("log.filter.all"): "All",
            tr("log.filter.info"): "Info",
            tr("log.filter.warning"): "Warning",
            tr("log.filter.error"): "Error",
            tr("log.filter.success"): "Success",
        }
        mapped = level_map.get(level_filter, "All")
        if mapped != "All" and level != mapped:
            return False
        # Search filter
        search = self._search_box.text().strip().lower()
        if search and search not in message_lower:
            return False
        return True

    def _apply_filter(self):
        """Re-apply filters to all stored messages."""
        self._text_edit.clear()
        for level, msg_lower, html in self._all_messages:
            if self._should_show(level, msg_lower):
                self._text_edit.append(html)

    def _export_log(self):
        """Export log to text file (restricted to workspace)."""
        import os

        from PySide6.QtWidgets import QFileDialog, QMessageBox

        # Workspace directory
        proj_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../.."))
        logs_dir = os.path.join(proj_dir, "logs")
        os.makedirs(logs_dir, exist_ok=True)

        path, _ = QFileDialog.getSaveFileName(
            self, tr("dialog.export_log"), os.path.join(logs_dir, "simulation_log.txt"),
            "Text Files (*.txt);;All Files (*)"
        )
        if not path:
            return

        # Validate path is within workspace
        abs_path = os.path.abspath(path)
        if not abs_path.startswith(proj_dir + os.sep) and abs_path != proj_dir:
            QMessageBox.warning(self, tr("dialog.access_denied"), tr("dialog.access_denied.msg"))
            return
        try:
            # Extract plain text from HTML
            with open(path, 'w', encoding='utf-8') as f:
                for _, _, html in self._all_messages:
                    # Simple HTML to text
                    import re
                    text = re.sub(r'<[^>]+>', '', html)
                    f.write(text + '\n')
        except Exception:
            pass

    def clear_log(self):
        """Clear all log content and reset counter."""
        self._text_edit.clear()
        self._message_count = 0
        self._all_messages.clear()

    def retranslate(self):
        """Update all text for current language."""
        # Update filter combo items
        _current_text = self._level_filter.currentText()
        self._level_filter.clear()
        self._level_filter.addItems([
            tr("log.filter.all"), tr("log.filter.info"),
            tr("log.filter.warning"), tr("log.filter.error"),
            tr("log.filter.success"),
        ])
        self._level_filter.setToolTip(tr("tooltip.filter_level"))
        self._search_box.setPlaceholderText(tr("log.search"))
        self._search_box.setToolTip(tr("tooltip.search_log"))
        self._btn_export.setText(tr("log.export"))
        self._btn_export.setToolTip(tr("tooltip.export_log"))
        self._btn_clear.setText(tr("log.clear"))
        self._btn_clear.setToolTip(tr("tooltip.clear_log"))
        self._text_edit.setPlaceholderText(tr("log.placeholder"))
