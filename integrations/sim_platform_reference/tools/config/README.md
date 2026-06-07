# Configuration Management System

## Overview

The configuration management system provides YAML/JSON configuration parsing, schema validation using Pydantic, and configuration versioning for the sim_platform.

## Features

- **YAML/JSON Configuration**: Support for both YAML and JSON configuration formats
- **Schema Validation**: Pydantic-based validation with detailed error messages
- **Configuration Templates**: Pre-built templates for common simulation scenarios
- **Configuration Versioning**: Track configuration changes and history
- **Configuration Merging**: Merge base configurations with overrides
- **Security**: Input validation, path traversal prevention, NaN/Inf guards

## Quick Start

### Loading Configuration

```python
from sim_platform.tools.config import ConfigurationManager

# Initialize manager
manager = ConfigurationManager()

# Load from YAML file
config = manager.load_config("config.yaml")

# Load from JSON file
config = manager.load_config("config.json")
```

### Creating Configuration from Template

```python
from sim_platform.tools.config import ConfigTemplates

# Create PMSM FOC configuration
config = ConfigTemplates.pmsm_foc_template()

# Create BLDC six-step configuration
config = ConfigTemplates.bldc_six_step_template()

# Create IM vector control configuration
config = ConfigTemplates.im_vector_control_template()
```

### Saving Configuration

```python
from sim_platform.tools.config import ConfigurationManager, SimulationConfig

# Create configuration
config = SimulationConfig(name="my_simulation")

# Save to YAML
manager = ConfigurationManager()
manager.save_config(config, "my_config.yaml")

# Save to JSON
manager.save_config(config, "my_config.json")
```

### Validating Configuration

```python
from sim_platform.tools.config import ConfigurationManager

manager = ConfigurationManager()
config = manager.load_config("config.yaml")

# Validate configuration
errors = manager.validate_config(config)
if errors:
    print("Validation errors:")
    for error in errors:
        print(f"  - {error}")
else:
    print("Configuration is valid")
```

### Merging Configurations

```python
from sim_platform.tools.config import ConfigurationManager

manager = ConfigurationManager()
base_config = manager.load_config("base.yaml")

# Override specific values
overrides = {
    'time': {'duration_s': 2.0},
    'scenario': {'speed_ref_value': 200.0},
}

# Merge configurations
merged_config = manager.merge_configs(base_config, overrides)
```

## Configuration Schema

### SimulationConfig

Main configuration schema with the following fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `version` | str | "1.0" | Configuration version |
| `name` | str | "simulation" | Simulation name |
| `description` | str | "" | Simulation description |
| `time` | TimeConfig | - | Time configuration |
| `models` | List[ModelConfig] | [] | Model configurations |
| `scenario` | ScenarioConfig | - | Scenario configuration |
| `fault_injection` | FaultConfig | - | Fault injection configuration |
| `output` | OutputConfig | - | Output configuration |
| `solver` | SolverType | "forward_euler" | Solver type |
| `log_level` | str | "INFO" | Logging level |
| `log_file` | str | null | Log file path |

### TimeConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `duration_s` | float | 1.0 | Simulation duration in seconds |
| `dt_ns` | int | 50000 | Base time step in nanoseconds |
| `multi_rate` | bool | false | Enable multi-rate scheduling |
| `realtime` | bool | false | Enable real-time mode |
| `realtime_factor` | float | 1.0 | Real-time factor |

### ModelConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `model_id` | str | required | Unique model identifier |
| `model_type` | str | required | Model type (motor, power, sensor, etc.) |
| `fidelity` | FidelityLevel | "L2" | Model fidelity level |
| `parameters` | dict | {} | Model parameters |
| `time_scale` | TimeScale | "us" | Model time scale |
| `dt_ns` | int | null | Time step in nanoseconds |
| `depends_on` | List[str] | [] | Model dependencies |
| `inputs` | List[str] | [] | Input signal names |
| `outputs` | List[str] | [] | Output signal names |

### ScenarioConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | str | "default" | Scenario name |
| `description` | str | "" | Scenario description |
| `speed_ref_type` | str | "step" | Speed reference type |
| `speed_ref_value` | float | 100.0 | Speed reference value |
| `speed_ref_time_s` | float | 0.1 | Speed reference time |
| `load_torque_value` | float | 0.0 | Load torque value |
| `load_torque_time_s` | float | 0.5 | Load torque time |

### FaultConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | false | Enable fault injection |
| `fault_id` | str | "" | Fault identifier |
| `fault_type` | str | "bias" | Fault type |
| `target_path` | str | "" | Target signal path |
| `magnitude` | float | 0.0 | Fault magnitude |
| `start_time_s` | float | 0.0 | Fault start time |
| `duration_s` | float | 0.0 | Fault duration |
| `probability` | float | 0.0 | Fault probability |

### OutputConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | true | Enable output recording |
| `format` | OutputFormat | "hdf5" | Output format |
| `filename` | str | "simulation_output" | Output filename |
| `directory` | str | "output" | Output directory |
| `record_interval_ns` | int | 50000 | Recording interval in ns |
| `compress` | bool | true | Enable compression |
| `plot_enabled` | bool | true | Enable plotting |
| `plot_format` | str | "png" | Plot format |

## Enum Types

### FidelityLevel

- `L0`: Interface stub
- `L1`: Empirical/curve-based
- `L2`: Lumped parameter
- `L3`: Detailed physics
- `L4`: High-fidelity (SPICE/FEA)

### TimeScale

- `ns`: Nanosecond
- `us`: Microsecond
- `ms`: Millisecond
- `s`: Second

### SolverType

- `forward_euler`: Forward Euler method
- `backward_euler`: Backward Euler method
- `runge_kutta`: Runge-Kutta method
- `adaptive`: Adaptive step size

### OutputFormat

- `hdf5`: HDF5 format
- `csv`: CSV format
- `parquet`: Parquet format
- `json`: JSON format

## Configuration Templates

### PMSM FOC Template

```python
from sim_platform.tools.config import ConfigTemplates

config = ConfigTemplates.pmsm_foc_template()
```

Default PMSM Field-Oriented Control configuration with:
- Battery (48V ideal voltage source)
- Three-phase inverter (average model)
- PMSM motor (dq-axis model)
- Current sensor with noise
- Encoder with quantization
- FOC current controller
- Speed PI controller

### BLDC Six-Step Template

```python
from sim_platform.tools.config import ConfigTemplates

config = ConfigTemplates.bldc_six_step_template()
```

BLDC motor with six-step commutation control.

### IM Vector Control Template

```python
from sim_platform.tools.config import ConfigTemplates

config = ConfigTemplates.im_vector_control_template()
```

Induction motor with Rotor Flux Oriented Control.

## Error Handling

The configuration system provides detailed error messages for validation failures:

```python
from sim_platform.tools.config import ConfigurationManager, SimulationConfig

try:
    config = SimulationConfig(
        models=[
            {"model_id": "motor", "model_type": "motor"},
            {"model_id": "motor", "model_type": "motor"},  # Duplicate ID
        ]
    )
except ValueError as e:
    print(f"Validation error: {e}")
    # Output: Validation error: Duplicate model IDs found
```

## Security Features

- **Path Traversal Prevention**: Blocks `..` in file paths
- **Input Validation**: Validates all configuration fields
- **NaN/Inf Guards**: Rejects NaN and Inf values in numeric fields
- **File Size Limits**: Maximum configuration file size of 10MB
- **Format Validation**: Validates YAML/JSON format before parsing

## Examples

See `examples/pmsm_foc_mvp/config_example.yaml` for a complete configuration example.

## Testing

Run the configuration tests:

```bash
python -m pytest verification/test_cases/test_config.py -v
```

## API Reference

### ConfigurationManager

Main configuration manager class.

#### Methods

- `load_config(config_path: str, validate: bool = True) -> SimulationConfig`
- `save_config(config: SimulationConfig, config_path: str, create_backup: bool = True) -> None`
- `create_config(name: str = "default", template: str = "pmsm_foc") -> SimulationConfig`
- `merge_configs(base: SimulationConfig, override: Dict[str, Any]) -> SimulationConfig`
- `validate_config(config: SimulationConfig) -> List[str]`
- `get_config_history() -> List[Dict[str, Any]]`
- `clear_cache() -> None`

### ConfigTemplates

Configuration templates for common scenarios.

#### Methods

- `pmsm_foc_template() -> SimulationConfig`
- `bldc_six_step_template() -> SimulationConfig`
- `im_vector_control_template() -> SimulationConfig`
- `get_template_list() -> List[str]`
- `get_template(name: str) -> SimulationConfig`

### ConfigWizard

Interactive configuration wizard.

#### Methods

- `create_interactive() -> SimulationConfig`

### Utility Functions

- `create_default_pmsm_config() -> SimulationConfig`
- `validate_config_file(config_path: str) -> SimulationConfig`
- `config_to_yaml(config: SimulationConfig) -> str`
- `config_to_json(config: SimulationConfig) -> str`
- `load_config_from_file(config_path: str) -> SimulationConfig`
- `save_config_to_file(config: SimulationConfig, config_path: str) -> None`
- `create_default_config() -> SimulationConfig`
- `validate_config(config: SimulationConfig) -> List[str]`
