"""Configuration Manager — handles configuration loading, validation, and versioning.

Provides:
- YAML/JSON configuration loading
- Schema validation using Pydantic
- Configuration versioning and migration
- Configuration templates and defaults

Security:
  - CWE-22: Path traversal prevention
  - CWE-20: Input validation on all configuration fields
  - CWE-754: NaN/Inf guards on numeric fields
"""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Any

import yaml

from .config_schema import (
    FidelityLevel,
    ModelConfig,
    ScenarioConfig,
    SimulationConfig,
    TimeConfig,
    create_default_pmsm_config,
)

logger = logging.getLogger(__name__)

# ── Constants ──────────────────────────────────────────────

CONFIG_VERSION = "1.0"
SUPPORTED_EXTENSIONS = {'.yaml', '.yml', '.json'}
MAX_CONFIG_SIZE = 10 * 1024 * 1024  # 10MB


class ConfigurationManager:
    """Manages simulation configurations.
    
    Features:
    - Load/save YAML/JSON configurations
    - Validate configurations against schema
    - Configuration versioning
    - Configuration templates
    - Configuration merging
    """

    def __init__(self, config_dir: str | None = None):
        """Initialize configuration manager.
        
        Args:
            config_dir: Directory for configuration files
        """
        self.config_dir = Path(config_dir) if config_dir else Path("configs")
        self.config_dir.mkdir(parents=True, exist_ok=True)

        # Configuration cache
        self._config_cache: dict[str, SimulationConfig] = {}

        # Configuration history
        self._history: list[dict[str, Any]] = []

    def load_config(self, config_path: str, validate: bool = True) -> SimulationConfig:
        """Load configuration from file.
        
        Args:
            config_path: Path to configuration file
            validate: Whether to validate configuration
            
        Returns:
            Loaded configuration
            
        Raises:
            FileNotFoundError: If file not found
            ValueError: If configuration is invalid
        """
        path = Path(config_path)

        # Security: Check path traversal
        if '..' in str(path):
            raise ValueError("Path traversal not allowed")

        # Check file exists
        if not path.exists():
            raise FileNotFoundError(f"Configuration file not found: {config_path}")

        # Check file size
        if path.stat().st_size > MAX_CONFIG_SIZE:
            raise ValueError(f"Configuration file too large: {path.stat().st_size} bytes")

        # Check extension
        if path.suffix not in SUPPORTED_EXTENSIONS:
            raise ValueError(f"Unsupported configuration format: {path.suffix}")

        # Load configuration
        try:
            with open(path, encoding='utf-8') as f:
                if path.suffix in ('.yaml', '.yml'):
                    raw_config = yaml.safe_load(f)
                elif path.suffix == '.json':
                    raw_config = json.load(f)
                else:
                    raise ValueError(f"Unsupported format: {path.suffix}")
        except yaml.YAMLError as e:
            raise ValueError(f"Invalid YAML format: {e}")
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON format: {e}")

        # Validate configuration
        if validate:
            config = SimulationConfig(**raw_config)
        else:
            config = SimulationConfig.model_construct(**raw_config)

        # Cache configuration
        self._config_cache[str(path)] = config

        # Record history
        self._record_history("load", str(path), config)

        logger.info("Loaded configuration from %s", config_path)
        return config

    def save_config(self, config: SimulationConfig, config_path: str,
                   create_backup: bool = True) -> None:
        """Save configuration to file.
        
        Args:
            config: Configuration to save
            config_path: Path to save configuration
            create_backup: Whether to create backup of existing file
        """
        path = Path(config_path)

        # Security: Check path traversal
        if '..' in str(path):
            raise ValueError("Path traversal not allowed")

        # Create backup if requested
        if create_backup and path.exists():
            backup_path = path.with_suffix(f'.backup{path.suffix}')
            path.rename(backup_path)
            logger.info("Created backup: %s", backup_path)

        # Create directory if needed
        path.parent.mkdir(parents=True, exist_ok=True)

        # Save configuration
        with open(path, 'w', encoding='utf-8') as f:
            if path.suffix in ('.yaml', '.yml'):
                # Convert to dict with enum values as strings
                def convert_enums(obj):
                    """Recursively convert enum values to strings."""
                    from enum import Enum
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

                yaml.dump(config_dict, f, default_flow_style=False,
                         sort_keys=False, allow_unicode=True)
            elif path.suffix == '.json':
                json.dump(config.model_dump(), f, indent=2, ensure_ascii=False)

        # Update cache
        self._config_cache[str(path)] = config

        # Record history
        self._record_history("save", str(path), config)

        logger.info("Saved configuration to %s", config_path)

    def create_config(self, name: str = "default",
                     template: str = "pmsm_foc") -> SimulationConfig:
        """Create new configuration from template.
        
        Args:
            name: Configuration name
            template: Template name (pmsm_foc, bldc, im, etc.)
            
        Returns:
            New configuration
        """
        if template == "pmsm_foc":
            config = create_default_pmsm_config()
        else:
            config = SimulationConfig(name=name)

        config.name = name

        # Record history
        self._record_history("create", name, config)

        logger.info("Created configuration '%s' from template '%s'", name, template)
        return config

    def merge_configs(self, base: SimulationConfig,
                     override: dict[str, Any]) -> SimulationConfig:
        """Merge configuration with override values.
        
        Args:
            base: Base configuration
            override: Override values
            
        Returns:
            Merged configuration
        """
        # Convert base to dict
        base_dict = base.model_dump()

        # Deep merge
        merged = self._deep_merge(base_dict, override)

        # Create new configuration
        config = SimulationConfig(**merged)

        # Record history
        self._record_history("merge", config.name, config)

        return config

    def validate_config(self, config: SimulationConfig) -> list[str]:
        """Validate configuration and return errors.
        
        Args:
            config: Configuration to validate
            
        Returns:
            List of validation errors
        """
        errors = []

        try:
            # Re-validate to catch any issues
            SimulationConfig(**config.model_dump())
        except Exception as e:
            errors.append(str(e))

        # Check for circular dependencies
        try:
            config._check_dependencies()
        except ValueError as e:
            errors.append(str(e))

        # Check model references
        model_ids = {m.model_id for m in config.models}
        for model in config.models:
            for dep in model.depends_on:
                if dep not in model_ids:
                    errors.append(f"Model '{model.model_id}' depends on unknown model '{dep}'")

        return errors

    def get_config_history(self) -> list[dict[str, Any]]:
        """Get configuration history."""
        return self._history.copy()

    def clear_cache(self) -> None:
        """Clear configuration cache."""
        self._config_cache.clear()
        logger.info("Cleared configuration cache")

    def _deep_merge(self, base: dict[str, Any],
                   override: dict[str, Any]) -> dict[str, Any]:
        """Deep merge two dictionaries."""
        result = base.copy()

        for key, value in override.items():
            if key in result and isinstance(result[key], dict) and isinstance(value, dict):
                result[key] = self._deep_merge(result[key], value)
            else:
                result[key] = value

        return result

    def _record_history(self, action: str, target: str,
                       config: SimulationConfig) -> None:
        """Record configuration history."""
        self._history.append({
            'timestamp': datetime.now().isoformat(),
            'action': action,
            'target': target,
            'config_name': config.name,
            'config_version': config.version,
        })

        # Keep only last 100 entries
        if len(self._history) > 100:
            self._history = self._history[-100:]


# ── Configuration Templates ──────────────────────────────────

class ConfigTemplates:
    """Configuration templates for common scenarios."""

    @staticmethod
    def pmsm_foc_template() -> SimulationConfig:
        """PMSM Field-Oriented Control template."""
        return create_default_pmsm_config()

    @staticmethod
    def bldc_six_step_template() -> SimulationConfig:
        """BLDC six-step commutation template."""
        return SimulationConfig(
            name="BLDC Six-Step",
            description="BLDC motor with six-step commutation control",
            time=TimeConfig(
                duration_s=1.0,
                dt_ns=50000,
            ),
            models=[
                ModelConfig(
                    model_id="battery",
                    model_type="power",
                    fidelity=FidelityLevel.L0,
                    parameters={"v_nom": 24.0},
                    outputs=["v_bus"],
                ),
                ModelConfig(
                    model_id="bldc",
                    model_type="motor",
                    fidelity=FidelityLevel.L2,
                    parameters={
                        "Rs": 0.5,
                        "Ls": 1e-3,
                        "Ke": 0.01,
                        "Kt": 0.01,
                        "J": 1e-4,
                        "Pp": 1,
                    },
                    inputs=["v_bus"],
                    outputs=["ia", "ib", "ic", "omega_m", "theta_e", "torque"],
                    depends_on=["battery"],
                ),
                ModelConfig(
                    model_id="hall_sensor",
                    model_type="sensor",
                    fidelity=FidelityLevel.L1,
                    parameters={},
                    inputs=["theta_e"],
                    outputs=["hall_state"],
                    depends_on=["bldc"],
                ),
                ModelConfig(
                    model_id="bldc_controller",
                    model_type="controller",
                    fidelity=FidelityLevel.L2,
                    parameters={"kp_speed": 0.1, "ki_speed": 1.0},
                    inputs=["omega_m", "omega_ref", "hall_state"],
                    outputs=["duty_cycle"],
                    depends_on=["bldc", "hall_sensor"],
                ),
            ],
            scenario=ScenarioConfig(
                name="Speed Control",
                speed_ref_value=100.0,
            ),
        )

    @staticmethod
    def im_vector_control_template() -> SimulationConfig:
        """Induction motor vector control template."""
        return SimulationConfig(
            name="IM Vector Control",
            description="Induction motor with Rotor Flux Oriented Control",
            time=TimeConfig(
                duration_s=1.0,
                dt_ns=50000,
            ),
            models=[
                ModelConfig(
                    model_id="inverter",
                    model_type="power",
                    fidelity=FidelityLevel.L2,
                    parameters={"v_bus": 310.0},
                    inputs=["vd_ref", "vq_ref"],
                    outputs=["va", "vb", "vc"],
                ),
                ModelConfig(
                    model_id="im",
                    model_type="motor",
                    fidelity=FidelityLevel.L2,
                    parameters={
                        "Rs": 0.5,
                        "Rr": 0.5,
                        "Ls": 0.01,
                        "Lr": 0.01,
                        "Lm": 0.009,
                        "J": 0.01,
                        "B": 0.001,
                        "Pp": 2,
                    },
                    inputs=["va", "vb", "vc"],
                    outputs=["ia", "ib", "ic", "omega_m", "theta_e", "torque"],
                    depends_on=["inverter"],
                ),
                ModelConfig(
                    model_id="im_controller",
                    model_type="controller",
                    fidelity=FidelityLevel.L2,
                    parameters={
                        "kp_flux": 5.0,
                        "ki_flux": 500.0,
                        "kp_torque": 5.0,
                        "ki_torque": 500.0,
                        "kp_speed": 0.1,
                        "ki_speed": 1.0,
                    },
                    inputs=["ia", "ib", "ic", "omega_m", "omega_ref"],
                    outputs=["vd_ref", "vq_ref"],
                    depends_on=["im"],
                ),
            ],
            scenario=ScenarioConfig(
                name="Speed Control",
                speed_ref_value=100.0,
            ),
        )

    @staticmethod
    def get_template_list() -> list[str]:
        """Get list of available templates."""
        return ["pmsm_foc", "bldc_six_step", "im_vector_control"]

    @staticmethod
    def get_template(name: str) -> SimulationConfig:
        """Get configuration template by name."""
        templates = {
            "pmsm_foc": ConfigTemplates.pmsm_foc_template,
            "bldc_six_step": ConfigTemplates.bldc_six_step_template,
            "im_vector_control": ConfigTemplates.im_vector_control_template,
        }

        if name not in templates:
            raise ValueError(f"Unknown template: {name}. Available: {list(templates.keys())}")

        return templates[name]()


# ── Configuration Wizard ──────────────────────────────────

class ConfigWizard:
    """Interactive configuration wizard."""

    def __init__(self, config_manager: ConfigurationManager):
        """Initialize configuration wizard.
        
        Args:
            config_manager: Configuration manager instance
        """
        self.config_manager = config_manager

    def create_interactive(self) -> SimulationConfig:
        """Create configuration interactively.
        
        Returns:
            Created configuration
        """
        print("\n=== Simulation Configuration Wizard ===\n")

        # Get basic information
        name = input("Simulation name [default]: ").strip() or "default"
        description = input("Description: ").strip()

        # Get time configuration
        print("\n--- Time Configuration ---")
        duration_s = float(input("Duration (seconds) [1.0]: ").strip() or "1.0")
        dt_ns = int(input("Time step (nanoseconds) [50000]: ").strip() or "50000")

        # Get template
        print("\n--- Model Template ---")
        templates = ConfigTemplates.get_template_list()
        print(f"Available templates: {', '.join(templates)}")
        template_name = input("Select template [pmsm_foc]: ").strip() or "pmsm_foc"

        # Create configuration from template
        config = ConfigTemplates.get_template(template_name)

        # Update with user inputs
        config.name = name
        config.description = description
        config.time.duration_s = duration_s
        config.time.dt_ns = dt_ns

        # Get scenario configuration
        print("\n--- Scenario Configuration ---")
        speed_ref = float(input("Speed reference (rad/s) [100.0]: ").strip() or "100.0")
        config.scenario.speed_ref_value = speed_ref

        # Get output configuration
        print("\n--- Output Configuration ---")
        filename = input("Output filename [simulation_output]: ").strip() or "simulation_output"
        config.output.filename = filename

        print("\n=== Configuration Created ===")
        print(f"Name: {config.name}")
        print(f"Models: {len(config.models)}")
        print(f"Duration: {config.time.duration_s}s")
        print(f"Time step: {config.time.dt_ns}ns")

        return config


# ── Utility Functions ──────────────────────────────────────

def load_config_from_file(config_path: str) -> SimulationConfig:
    """Load configuration from file.
    
    Args:
        config_path: Path to configuration file
        
    Returns:
        Loaded configuration
    """
    manager = ConfigurationManager()
    return manager.load_config(config_path)


def save_config_to_file(config: SimulationConfig, config_path: str) -> None:
    """Save configuration to file.
    
    Args:
        config: Configuration to save
        config_path: Path to save configuration
    """
    manager = ConfigurationManager()
    manager.save_config(config, config_path)


def create_default_config() -> SimulationConfig:
    """Create default configuration."""
    return ConfigTemplates.pmsm_foc_template()


def validate_config(config: SimulationConfig) -> list[str]:
    """Validate configuration.
    
    Args:
        config: Configuration to validate
        
    Returns:
        List of validation errors
    """
    manager = ConfigurationManager()
    return manager.validate_config(config)
