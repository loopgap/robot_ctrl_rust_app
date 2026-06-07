"""Configuration Schema definitions using Pydantic.

Defines the schema for simulation configuration files.
Supports YAML/JSON configuration with validation.

Security:
  - CWE-20: Input validation on all configuration fields
  - CWE-754: NaN/Inf guards on numeric fields
"""

import math
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field, field_validator, model_validator


class FidelityLevel(str, Enum):
    """Model fidelity levels."""
    L0 = "L0"  # Interface stub
    L1 = "L1"  # Empirical/curve-based
    L2 = "L2"  # Lumped parameter
    L3 = "L3"  # Detailed physics
    L4 = "L4"  # High-fidelity (SPICE/FEA)


class TimeScale(str, Enum):
    """Time scale categories."""
    NANOSECOND = "ns"
    MICROSECOND = "us"
    MILLISECOND = "ms"
    SECOND = "s"


class SolverType(str, Enum):
    """Solver types."""
    FORWARD_EULER = "forward_euler"
    BACKWARD_EULER = "backward_euler"
    RUNGE_KUTTA = "runge_kutta"
    ADAPTIVE = "adaptive"


class OutputFormat(str, Enum):
    """Output formats."""
    HDF5 = "hdf5"
    CSV = "csv"
    PARQUET = "parquet"
    JSON = "json"


# ── Model Configuration ──────────────────────────────────────

class ModelConfig(BaseModel):
    """Model configuration schema."""

    model_id: str = Field(..., description="Unique model identifier")
    model_type: str = Field(..., description="Model type (motor, power, sensor, etc.)")
    fidelity: FidelityLevel = Field(default=FidelityLevel.L2, description="Model fidelity level")

    # Model parameters
    parameters: dict[str, Any] = Field(default_factory=dict, description="Model parameters")

    # Time configuration
    time_scale: TimeScale = Field(default=TimeScale.MICROSECOND, description="Model time scale")
    dt_ns: int | None = Field(default=None, description="Time step in nanoseconds")

    # Dependencies
    depends_on: list[str] = Field(default_factory=list, description="Model dependencies")

    # Input/output interfaces
    inputs: list[str] = Field(default_factory=list, description="Input signal names")
    outputs: list[str] = Field(default_factory=list, description="Output signal names")

    @field_validator('model_id')
    @classmethod
    def validate_model_id(cls, v):
        """Validate model ID format."""
        if not v or not isinstance(v, str):
            raise ValueError("model_id must be a non-empty string")
        # Allow alphanumeric, underscore, hyphen
        if not all(c.isalnum() or c in '_-' for c in v):
            raise ValueError("model_id must contain only alphanumeric, underscore, or hyphen")
        return v

    @field_validator('parameters')
    @classmethod
    def validate_parameters(cls, v):
        """Validate model parameters."""
        if not isinstance(v, dict):
            raise ValueError("parameters must be a dictionary")
        # Guard against NaN/Inf in numeric parameters
        for key, value in v.items():
            if isinstance(value, float):
                if math.isnan(value) or math.isinf(value):
                    raise ValueError(f"Parameter '{key}' must not be NaN or Inf")
        return v


# ── Time Configuration ──────────────────────────────────────

class TimeConfig(BaseModel):
    """Time configuration schema."""

    duration_s: float = Field(default=1.0, gt=0, description="Simulation duration in seconds")
    dt_ns: int = Field(default=50000, gt=0, description="Base time step in nanoseconds")

    # Multi-rate configuration
    multi_rate: bool = Field(default=False, description="Enable multi-rate scheduling")
    rate_scales: dict[str, TimeScale] = Field(default_factory=dict, description="Rate scales for models")

    # Real-time configuration
    realtime: bool = Field(default=False, description="Enable real-time mode")
    realtime_factor: float = Field(default=1.0, gt=0, description="Real-time factor")

    @field_validator('duration_s')
    @classmethod
    def validate_duration(cls, v):
        """Validate simulation duration."""
        if math.isnan(v) or math.isinf(v):
            raise ValueError("duration_s must not be NaN or Inf")
        if v <= 0:
            raise ValueError("duration_s must be positive")
        if v > 1e6:  # Max 1 million seconds
            raise ValueError("duration_s exceeds maximum allowed value")
        return v

    @field_validator('dt_ns')
    @classmethod
    def validate_dt(cls, v):
        """Validate time step."""
        if v <= 0:
            raise ValueError("dt_ns must be positive")
        if v > 1e12:  # Max 1 second
            raise ValueError("dt_ns exceeds maximum allowed value")
        return v


# ── Fault Injection Configuration ──────────────────────────

class FaultConfig(BaseModel):
    """Fault injection configuration schema."""

    enabled: bool = Field(default=False, description="Enable fault injection")
    fault_id: str = Field(default="", description="Fault identifier")
    fault_type: str = Field(default="bias", description="Fault type")
    target_path: str = Field(default="", description="Target signal path")
    magnitude: float = Field(default=0.0, description="Fault magnitude")
    start_time_s: float = Field(default=0.0, ge=0, description="Fault start time")
    duration_s: float = Field(default=0.0, ge=0, description="Fault duration")
    probability: float = Field(default=0.0, ge=0, le=1, description="Fault probability")

    @field_validator('fault_id')
    @classmethod
    def validate_fault_id(cls, v):
        """Validate fault ID format."""
        if v and not all(c.isalnum() or c in '_-' for c in v):
            raise ValueError("fault_id must contain only alphanumeric, underscore, or hyphen")
        return v

    @field_validator('magnitude')
    @classmethod
    def validate_magnitude(cls, v):
        """Validate fault magnitude."""
        if math.isnan(v) or math.isinf(v):
            raise ValueError("magnitude must not be NaN or Inf")
        return v


# ── Output Configuration ──────────────────────────────────

class OutputConfig(BaseModel):
    """Output configuration schema."""

    enabled: bool = Field(default=True, description="Enable output recording")
    format: OutputFormat = Field(default=OutputFormat.HDF5, description="Output format")
    filename: str = Field(default="simulation_output", description="Output filename")
    directory: str = Field(default="output", description="Output directory")

    # Recording configuration
    record_interval_ns: int = Field(default=50000, gt=0, description="Recording interval in ns")
    compress: bool = Field(default=True, description="Enable compression")

    # Visualization
    plot_enabled: bool = Field(default=True, description="Enable plotting")
    plot_format: str = Field(default="png", description="Plot format")

    @field_validator('filename')
    @classmethod
    def validate_filename(cls, v):
        """Validate output filename."""
        if not v or not isinstance(v, str):
            raise ValueError("filename must be a non-empty string")
        # Prevent path traversal
        if '..' in v or '/' in v or '\\' in v:
            raise ValueError("filename must not contain path separators")
        return v


# ── Scenario Configuration ──────────────────────────────────

class ScenarioConfig(BaseModel):
    """Scenario configuration schema."""

    name: str = Field(default="default", description="Scenario name")
    description: str = Field(default="", description="Scenario description")

    # Speed reference
    speed_ref_type: str = Field(default="step", description="Speed reference type")
    speed_ref_value: float = Field(default=100.0, description="Speed reference value")
    speed_ref_time_s: float = Field(default=0.1, description="Speed reference time")

    # Load torque
    load_torque_value: float = Field(default=0.0, description="Load torque value")
    load_torque_time_s: float = Field(default=0.5, description="Load torque time")

    @field_validator('speed_ref_value')
    @classmethod
    def validate_speed_ref(cls, v):
        """Validate speed reference."""
        if math.isnan(v) or math.isinf(v):
            raise ValueError("speed_ref_value must not be NaN or Inf")
        return v


# ── Main Configuration ──────────────────────────────────────

class SimulationConfig(BaseModel):
    """Main simulation configuration schema."""

    # Metadata
    version: str = Field(default="1.0", description="Configuration version")
    name: str = Field(default="simulation", description="Simulation name")
    description: str = Field(default="", description="Simulation description")

    # Time configuration
    time: TimeConfig = Field(default_factory=TimeConfig, description="Time configuration")

    # Models
    models: list[ModelConfig] = Field(default_factory=list, description="Model configurations")

    # Scenario
    scenario: ScenarioConfig = Field(default_factory=ScenarioConfig, description="Scenario configuration")

    # Fault injection
    fault_injection: FaultConfig = Field(default_factory=FaultConfig, description="Fault injection configuration")

    # Output
    output: OutputConfig = Field(default_factory=OutputConfig, description="Output configuration")

    # Solver configuration
    solver: SolverType = Field(default=SolverType.FORWARD_EULER, description="Solver type")

    # Logging
    log_level: str = Field(default="INFO", description="Logging level")
    log_file: str | None = Field(default=None, description="Log file path")

    @model_validator(mode='after')
    def validate_config(self):
        """Validate entire configuration."""
        # Check for duplicate model IDs
        model_ids = [m.model_id for m in self.models]
        if len(model_ids) != len(set(model_ids)):
            raise ValueError("Duplicate model IDs found")

        # Check for circular dependencies
        self._check_dependencies()

        return self

    def _check_dependencies(self):
        """Check for circular dependencies in model graph."""
        # Build adjacency list
        graph = {}
        for model in self.models:
            graph[model.model_id] = model.depends_on

        # DFS to detect cycles
        visited = set()
        rec_stack = set()

        def has_cycle(node):
            visited.add(node)
            rec_stack.add(node)

            for neighbor in graph.get(node, []):
                if neighbor not in visited:
                    if has_cycle(neighbor):
                        return True
                elif neighbor in rec_stack:
                    return True

            rec_stack.discard(node)
            return False

        for node in graph:
            if node not in visited:
                if has_cycle(node):
                    raise ValueError(f"Circular dependency detected involving model '{node}'")

    def get_model_by_id(self, model_id: str) -> ModelConfig | None:
        """Get model configuration by ID."""
        for model in self.models:
            if model.model_id == model_id:
                return model
        return None

    def get_total_steps(self) -> int:
        """Calculate total simulation steps."""
        duration_ns = int(self.time.duration_s * 1e9)
        return duration_ns // self.time.dt_ns


# ── Default Configurations ──────────────────────────────────

def create_default_pmsm_config() -> SimulationConfig:
    """Create default PMSM FOC configuration."""
    return SimulationConfig(
        name="PMSM FOC MVP",
        description="Default PMSM Field-Oriented Control simulation",
        time=TimeConfig(
            duration_s=1.5,
            dt_ns=50000,  # 50us
        ),
        models=[
            ModelConfig(
                model_id="battery",
                model_type="power",
                fidelity=FidelityLevel.L0,
                parameters={"v_nom": 48.0},
                outputs=["v_bus"],
            ),
            ModelConfig(
                model_id="inverter",
                model_type="power",
                fidelity=FidelityLevel.L2,
                parameters={"v_bus": 48.0},
                inputs=["duty_a", "duty_b", "duty_c", "v_bus"],
                outputs=["va", "vb", "vc"],
                depends_on=["battery"],
            ),
            ModelConfig(
                model_id="pmsm",
                model_type="motor",
                fidelity=FidelityLevel.L2,
                parameters={
                    "Rs": 0.1,
                    "Ld": 5e-4,
                    "Lq": 1e-3,
                    "flux_pm": 0.03,
                    "J": 0.001,
                    "B": 0.0001,
                    "Pp": 4,
                },
                inputs=["va", "vb", "vc"],
                outputs=["ia", "ib", "ic", "omega_m", "theta_e", "torque"],
                depends_on=["inverter"],
            ),
            ModelConfig(
                model_id="current_sensor",
                model_type="sensor",
                fidelity=FidelityLevel.L1,
                parameters={"noise_std": 0.05, "bias": 0.01},
                inputs=["ia", "ib", "ic"],
                outputs=["ia_meas", "ib_meas", "ic_meas"],
                depends_on=["pmsm"],
            ),
            ModelConfig(
                model_id="encoder",
                model_type="sensor",
                fidelity=FidelityLevel.L1,
                parameters={"noise_std": 0.001, "quantization": 0.0015339807878856412},
                inputs=["theta_e", "omega_m"],
                outputs=["theta_e_meas", "omega_m_meas"],
                depends_on=["pmsm"],
            ),
            ModelConfig(
                model_id="foc",
                model_type="controller",
                fidelity=FidelityLevel.L2,
                parameters={"kp": 1.0, "ki": 100.0},
                inputs=["ia_meas", "ib_meas", "ic_meas", "theta_e_meas", "id_ref", "iq_ref"],
                outputs=["vd_ref", "vq_ref"],
                depends_on=["current_sensor", "encoder"],
            ),
            ModelConfig(
                model_id="speed_pi",
                model_type="controller",
                fidelity=FidelityLevel.L2,
                parameters={"kp": 0.1, "ki": 1.0},
                inputs=["omega_m_meas", "omega_ref"],
                outputs=["iq_ref"],
                depends_on=["encoder"],
            ),
        ],
        scenario=ScenarioConfig(
            name="Speed Step Response",
            description="Speed step response test",
            speed_ref_type="step",
            speed_ref_value=100.0,
            speed_ref_time_s=0.1,
            load_torque_value=0.0,
            load_torque_time_s=0.5,
        ),
        output=OutputConfig(
            filename="pmsm_foc_output",
            record_interval_ns=50000,
        ),
    )


# ── Validation Utilities ──────────────────────────────────

def validate_config_file(config_path: str) -> SimulationConfig:
    """Validate configuration file and return parsed config.
    
    Args:
        config_path: Path to configuration file (YAML or JSON)
        
    Returns:
        Validated SimulationConfig object
        
    Raises:
        ValueError: If configuration is invalid
        FileNotFoundError: If file not found
    """
    import json
    from pathlib import Path

    import yaml

    path = Path(config_path)
    if not path.exists():
        raise FileNotFoundError(f"Configuration file not found: {config_path}")

    # Load configuration
    with open(path, encoding='utf-8') as f:
        if path.suffix in ('.yaml', '.yml'):
            raw_config = yaml.safe_load(f)
        elif path.suffix == '.json':
            raw_config = json.load(f)
        else:
            raise ValueError(f"Unsupported configuration format: {path.suffix}")

    # Validate and return
    return SimulationConfig(**raw_config)


def config_to_yaml(config: SimulationConfig) -> str:
    """Convert configuration to YAML string.
    
    Args:
        config: SimulationConfig object
        
    Returns:
        YAML string representation
    """
    from enum import Enum

    import yaml

    # Convert to dict with enum values as strings
    def convert_enums(obj):
        """Recursively convert enum values to strings."""
        if isinstance(obj, Enum):
            return obj.value
        elif isinstance(obj, dict):
            return {k: convert_enums(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [convert_enums(item) for item in obj]
        return obj

    # Convert to dict with enum values
    config_dict = config.model_dump()
    config_dict = convert_enums(config_dict)

    return yaml.dump(config_dict, default_flow_style=False, sort_keys=False)


def config_to_json(config: SimulationConfig) -> str:
    """Convert configuration to JSON string.
    
    Args:
        config: SimulationConfig object
        
    Returns:
        JSON string representation
    """
    import json
    return json.dumps(config.model_dump(), indent=2)
