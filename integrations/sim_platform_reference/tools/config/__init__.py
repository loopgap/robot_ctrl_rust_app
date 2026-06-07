"""Configuration tools for sim_platform.

Provides:
- YAML/JSON configuration parsing
- Schema validation using Pydantic
- Configuration management and versioning
- Configuration templates and wizard

Usage:
    from sim_platform.tools.config import ConfigurationManager, SimulationConfig
    
    # Load configuration
    manager = ConfigurationManager()
    config = manager.load_config("config.yaml")
    
    # Create from template
    config = ConfigTemplates.pmsm_foc_template()
    
    # Validate configuration
    errors = manager.validate_config(config)
"""

from .config_manager import (
    # Templates
    ConfigTemplates,
    # Manager
    ConfigurationManager,
    # Wizard
    ConfigWizard,
    create_default_config,
    # Utilities
    load_config_from_file,
    save_config_to_file,
    validate_config,
)
from .config_schema import (
    FaultConfig,
    # Enums
    FidelityLevel,
    # Sub-configurations
    ModelConfig,
    OutputConfig,
    OutputFormat,
    ScenarioConfig,
    # Main configuration
    SimulationConfig,
    SolverType,
    TimeConfig,
    TimeScale,
    config_to_json,
    config_to_yaml,
    # Utilities
    create_default_pmsm_config,
    validate_config_file,
)

__all__ = [
    # Main configuration
    "SimulationConfig",

    # Sub-configurations
    "ModelConfig",
    "TimeConfig",
    "ScenarioConfig",
    "FaultConfig",
    "OutputConfig",

    # Enums
    "FidelityLevel",
    "TimeScale",
    "SolverType",
    "OutputFormat",

    # Manager
    "ConfigurationManager",

    # Templates
    "ConfigTemplates",

    # Wizard
    "ConfigWizard",

    # Utilities
    "create_default_pmsm_config",
    "validate_config_file",
    "config_to_yaml",
    "config_to_json",
    "load_config_from_file",
    "save_config_to_file",
    "create_default_config",
    "validate_config",
]
