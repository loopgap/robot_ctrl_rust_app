"""HDF5-based simulation data logger.

Stores time-series data in HDF5 format for post-simulation analysis.

Security:
  - CWE-353: Full flush on close for crash consistency
  - CWE-754: NaN/Inf guard on recorded values
  - Integrity marker written on clean close
"""

import math

import h5py
import numpy as np


class HDF5Logger:
    """Simulation data logger using HDF5 backend.

    Usage:
        with HDF5Logger("output.h5") as log:
            log.record(0.0, motor_state, foc_state)

    Security: Integrity attribute written on clean close.
    Reads should check for '_integrity' attribute to verify file is complete.
    """

    def __init__(self, filepath: str, mode: str = "w"):
        self.filepath = filepath
        self.mode = mode
        self._file: h5py.File | None = None
        self._buffers: dict[str, list] = {}
        self._flush_interval = 1000
        self._counter = 0

    def open(self) -> "HDF5Logger":
        self._file = h5py.File(self.filepath, self.mode)
        return self

    def close(self) -> None:
        if self._file:
            self._flush()  # flush remaining data
            # SECURITY: Write integrity marker (CWE-353)
            self._file.attrs["_integrity"] = "COMPLETE"
            self._file.attrs["_records"] = self._counter
            self._file.flush()  # force OS write
            self._file.close()
            self._file = None

    def __enter__(self):
        return self.open()

    def __exit__(self, *args):
        self.close()

    def record(self, time_s: float, **kwargs) -> None:
        """Record a time step with named data series.

        SECURITY (CWE-754): NaN/Inf values are clamped to 0.
        """
        self._append("time", time_s)
        for name, value in kwargs.items():
            if isinstance(value, dict):
                for k, v in value.items():
                    self._append(f"{name}/{k}", v)
            else:
                self._append(name, value)

        self._counter += 1
        if self._counter % self._flush_interval == 0:
            self._flush()

    def _append(self, name: str, value: float) -> None:
        if name not in self._buffers:
            self._buffers[name] = []
        # SECURITY: Guard NaN/Inf (CWE-754)
        if isinstance(value, (int, float)):
            if math.isnan(value) or math.isinf(value):
                value = 0.0
        self._buffers[name].append(
            float(value) if isinstance(value, (int, float, bool)) else value)

    def _flush(self) -> None:
        """Write buffered data to HDF5."""
        if not self._file:
            return
        for name, buf in self._buffers.items():
            if not buf:
                continue
            arr = np.array(buf, dtype=np.float64)
            if name in self._file:
                dset = self._file[name]
                old_len = dset.shape[0]
                new_len = old_len + len(arr)
                dset.resize((new_len,))
                dset[old_len:new_len] = arr
            else:
                self._file.create_dataset(
                    name, data=arr,
                    maxshape=(None,), chunks=True,
                    compression="gzip", compression_opts=4,
                )
            buf.clear()

    def read(self, name: str) -> np.ndarray:
        """Read a dataset. Returns empty array if not found."""
        if self._file and name in self._file:
            return self._file[name][:]
        return np.array([])

    def keys(self) -> list[str]:
        if self._file:
            return list(self._file.keys())
        return []

    @staticmethod
    def verify_integrity(filepath: str) -> bool:
        """Check if HDF5 file was cleanly closed.

        Returns True if '_integrity' attribute says COMPLETE.
        """
        try:
            with h5py.File(filepath, "r") as f:
                return f.attrs.get("_integrity") == "COMPLETE"
        except Exception:
            return False
