"""Parameter scan dialog.

Allows selecting a parameter and custom values for batch simulation runs
with result comparison in a table.

Full bilingual support via i18n.
"""

from __future__ import annotations

import math

from PySide6.QtWidgets import (
    QComboBox,
    QDialog,
    QGroupBox,
    QHBoxLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QProgressBar,
    QPushButton,
    QTableWidget,
    QTableWidgetItem,
    QTextEdit,
    QVBoxLayout,
)

from sim_platform.tools.gui.i18n import tr
from sim_platform.tools.gui.widgets.config_panel import _SCAN_I18N, SCAN_PARAMS
from sim_platform.tools.gui.workers import ScanWorker


class ScanDialog(QDialog):
    """Parameter scan dialog with batch simulation and result comparison."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle(tr("scan.title"))
        self.setMinimumSize(650, 550)
        self._worker = None
        self._setup_ui()

    def _setup_ui(self):
        layout = QVBoxLayout(self)

        # ── Parameter selection ────────────────────────────
        self._param_group = QGroupBox(tr("scan.group.parameter"))
        pg_layout = QVBoxLayout(self._param_group)

        self._select_label = QLabel(tr("scan.select_param"))
        pg_layout.addWidget(self._select_label)

        self.param_combo = QComboBox()
        for key in SCAN_PARAMS:
            i18n_key = _SCAN_I18N.get(key, key)
            self.param_combo.addItem(tr(i18n_key), key)
        pg_layout.addWidget(self.param_combo)

        self._values_label = QLabel(tr("scan.values_label"))
        self._values_label.setWordWrap(True)
        pg_layout.addWidget(self._values_label)

        self.values_edit = QLineEdit()
        self.values_edit.setPlaceholderText(tr("scan.placeholder"))
        pg_layout.addWidget(self.values_edit)

        layout.addWidget(self._param_group)

        # ── Progress ──────────────────────────────────────
        self.progress_bar = QProgressBar()
        layout.addWidget(self.progress_bar)

        # ── Log ───────────────────────────────────────────
        self.log_edit = QTextEdit()
        self.log_edit.setReadOnly(True)
        self.log_edit.setMaximumHeight(200)
        layout.addWidget(self.log_edit)

        # ── Results table ─────────────────────────────────
        self.result_table = QTableWidget(0, 3)
        self.result_table.setHorizontalHeaderLabels([
            tr("scan.col.value"), tr("scan.col.speed"), tr("scan.col.error")
        ])
        self.result_table.horizontalHeader().setSectionResizeMode(
            QHeaderView.ResizeMode.Stretch
        )
        self.result_table.setEditTriggers(
            QTableWidget.EditTrigger.NoEditTriggers
        )
        layout.addWidget(self.result_table)

        # ── Buttons ───────────────────────────────────────
        btn_layout = QHBoxLayout()
        self.run_btn = QPushButton(tr("scan.start"))
        self.run_btn.setProperty("class", "primary")
        self.run_btn.clicked.connect(self._start_scan)

        self.stop_btn = QPushButton(tr("scan.stop"))
        self.stop_btn.setProperty("class", "danger")
        self.stop_btn.setEnabled(False)
        self.stop_btn.clicked.connect(self._stop_scan)

        close_btn = QPushButton(tr("dialog.close"))
        close_btn.clicked.connect(self.close)

        btn_layout.addWidget(self.run_btn)
        btn_layout.addWidget(self.stop_btn)
        btn_layout.addWidget(close_btn)
        layout.addLayout(btn_layout)

    def _start_scan(self):
        """Parse inputs and start scan worker."""
        # Use itemData to get the translation key
        param_key_data = self.param_combo.currentData()
        if param_key_data is None:
            param_key_data = self.param_combo.currentText()
        _, param_values_default = SCAN_PARAMS.get(param_key_data, ("speed", [50, 100]))
        param_key = SCAN_PARAMS.get(param_key_data, ("speed", [50, 100]))[0]

        values_str = self.values_edit.text()
        try:
            raw = [v.strip() for v in values_str.split(",") if v.strip()]
            values = [float(v) for v in raw]
            if any(math.isnan(v) or math.isinf(v) for v in values):
                raise ValueError(tr("scan.error.nan"))
            if len(values) < 2:
                raise ValueError(tr("scan.error.need2"))
            if len(values) > 100:
                raise ValueError(tr("scan.error.max100"))
        except Exception as e:
            self.log_edit.append(tr("scan.error.parse", str(e)))
            return

        self.log_edit.clear()
        self.result_table.setRowCount(0)
        self.progress_bar.setValue(0)
        self.run_btn.setEnabled(False)
        self.stop_btn.setEnabled(True)

        self._worker = ScanWorker(param_key, values, duration=1.0, parent=self)
        self._worker.progress.connect(self.progress_bar.setValue)
        self._worker.log_message.connect(self.log_edit.append)
        self._worker.result_ready.connect(self._on_scan_done)
        self._worker.error.connect(self._on_scan_error)
        self._worker.finished.connect(self._on_scan_finished)
        self._worker.start()

    def _stop_scan(self):
        if self._worker:
            from PySide6.QtWidgets import QMessageBox
            reply = QMessageBox.question(
                self,
                tr("scan.cancel_confirm.title"),
                tr("scan.cancel_confirm"),
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
                QMessageBox.StandardButton.No,
            )
            if reply == QMessageBox.StandardButton.Yes:
                self._worker.stop()

    def _on_scan_done(self, result: dict):
        """Populate results table."""
        results = result["results"]
        self.result_table.setRowCount(len(results))
        for i, r in enumerate(results):
            self.result_table.setItem(
                i, 0, QTableWidgetItem(f"{r['value']:.2f}")
            )
            self.result_table.setItem(
                i, 1, QTableWidgetItem(f"{r['speed']:.1f}")
            )
            self.result_table.setItem(
                i, 2, QTableWidgetItem(f"{r['error']:.2f}")
            )

    def _on_scan_error(self, msg: str):
        self.log_edit.append(tr("scan.error.parse", msg))

    def _on_scan_finished(self):
        self.run_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)

    def closeEvent(self, event):
        """Clean up worker on dialog close (CWE-404: proper resource cleanup)."""
        if self._worker and self._worker.isRunning():
            self._worker.stop()
            if not self._worker.wait(2000):
                self._worker.terminate()
                self._worker.wait(1000)
        self._worker = None
        event.accept()

    def retranslate(self):
        """Update all text for current language."""
        self.setWindowTitle(tr("scan.title"))
        self._param_group.setTitle(tr("scan.group.parameter"))
        self._select_label.setText(tr("scan.select_param"))
        self._values_label.setText(tr("scan.values_label"))
        self.values_edit.setPlaceholderText(tr("scan.placeholder"))
        self.run_btn.setText(tr("scan.start"))
        self.stop_btn.setText(tr("scan.stop"))
        self.result_table.setHorizontalHeaderLabels([
            tr("scan.col.value"), tr("scan.col.speed"), tr("scan.col.error")
        ])
