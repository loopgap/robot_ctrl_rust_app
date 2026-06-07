"""Model Registry — metadata-driven model management.

Every model in the simulation platform is registered here with
a unique ID, version, fidelity level, I/O ports, units, and
validation status.

Security:
  - CWE-20: model_id format validation (M-11)
  - CWE-209: Generic error messages (L-11)
"""

import logging
import re
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

logger = logging.getLogger(__name__)

# Security constants (M-11)
_MAX_MODEL_ID_LEN = 256
_MODEL_ID_PATTERN = re.compile(r'^[a-zA-Z][a-zA-Z0-9_]*://[a-zA-Z0-9_./-]+$')


class FidelityLevel(Enum):
    L0_STUB = 0          # stub — I/O only
    L1_EMPIRICAL = 1     # empirical — curves / lookup tables
    L2_LUMPED = 2        # lumped parameter — ODE with R/L/C/J/B
    L3_PHYSICS = 3       # physics-based — nonlinear, switching
    L4_HIGH_FIDELITY = 4 # high fidelity — FEM, CFD, full SPICE


class Domain(Enum):
    POWER = "power"
    MOTOR = "motor"
    SENSOR = "sensor"
    CONTROLLER = "controller"
    MECHANICAL = "mechanical"
    THERMAL = "thermal"
    GRID = "grid"
    FPGA = "fpga"
    ML = "ml"
    BATTERY = "battery"


@dataclass
class Port:
    """Model input/output port definition."""
    name: str
    unit: str                       # SI unit: "V", "A", "rad", "N.m" ...
    dtype: str = "float64"
    range_min: float = -float("inf")
    range_max: float = float("inf")
    description: str = ""

    def __post_init__(self):
        """Validate port name (CWE-20)."""
        if not self.name or not self.name.strip():
            raise ValueError("Port name cannot be empty")
        if ".." in self.name or "\x00" in self.name:
            raise ValueError(f"Invalid port name: '{self.name}'")


@dataclass
class ModelMetadata:
    """Complete metadata for a simulation model."""

    model_id: str                   # "mdl://motor/pmsm/dq/v1"
    model_name: str                 # human-readable
    domain: Domain
    fidelity: FidelityLevel

    input_ports: list[Port] = field(default_factory=list)
    output_ports: list[Port] = field(default_factory=list)

    sim_step_ns: int = 50000        # recommended simulation step
    latency_ns: int = 1000          # model computation latency
    is_realtime_capable: bool = False
    is_hil_capable: bool = False

    assumptions: list[str] = field(default_factory=list)
    valid_range: dict[str, Any] = field(default_factory=dict)

    version: str = "1.0.0"
    author: str = ""
    dependencies: list[str] = field(default_factory=list)
    validation_status: str = "NOT_TESTED"

    def to_dict(self) -> dict:
        return {
            "model_id": self.model_id,
            "model_name": self.model_name,
            "domain": self.domain.value,
            "fidelity": self.fidelity.value,
            "version": self.version,
            "step_ns": self.sim_step_ns,
        }


class ModelRegistry:
    """Central registry for all simulation models.

    Security (M-11): model_id validated on registration.
    """

    def __init__(self):
        self._models: dict[str, Any] = {}       # model_id → model instance
        self._metadata: dict[str, ModelMetadata] = {}  # model_id → metadata

    # ── registration ─────────────────────────────────────────

    def register(self, model: Any, metadata: ModelMetadata) -> None:
        """Register a model with validated model_id (M-11, CWE-20)."""
        mid = metadata.model_id
        if not mid or not mid.strip():
            raise ValueError("model_id cannot be empty")
        if len(mid) > _MAX_MODEL_ID_LEN:
            raise ValueError(f"model_id exceeds {_MAX_MODEL_ID_LEN} characters")
        if ".." in mid or "\x00" in mid:
            raise ValueError(f"Invalid model_id: path traversal in '{mid}'")
        if mid in self._models:
            raise ValueError("Model already registered")
        self._models[mid] = model
        self._metadata[mid] = metadata

    def get(self, model_id: str) -> Any:
        """Get model by ID. SECURITY (CWE-209): generic error message."""
        if model_id not in self._models:
            raise KeyError("Model not found in registry")
        return self._models[model_id]

    def get_metadata(self, model_id: str) -> ModelMetadata:
        """Get metadata by ID. SECURITY (CWE-209): generic error message."""
        if model_id not in self._metadata:
            raise KeyError("Model metadata not found")
        return self._metadata[model_id]

    # ── queries ──────────────────────────────────────────────

    def list_by_domain(self, domain: Domain) -> dict[str, ModelMetadata]:
        return {mid: m for mid, m in self._metadata.items()
                if m.domain == domain}

    def list_by_fidelity(self, level: FidelityLevel) -> dict[str, ModelMetadata]:
        return {mid: m for mid, m in self._metadata.items()
                if m.fidelity == level}

    def list_all(self) -> dict[str, ModelMetadata]:
        return dict(self._metadata)

    @property
    def model_count(self) -> int:
        return len(self._models)

    def validate_dependencies(self) -> list[str]:
        """Check all model dependencies are satisfied. Returns generic descriptions."""
        missing = []
        for meta in self._metadata.values():
            for dep in meta.dependencies:
                if dep not in self._models:
                    # L-11: Don't expose internal model_id in external reports
                    missing.append("model depends on unregistered dependency")
        return missing
