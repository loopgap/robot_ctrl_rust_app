"""Solver configuration presets and parameter management.

Provides industry-standard solver preset definitions with:
- Pre-configured solver parameter profiles (Forward Euler, RK4, etc.)
- Parameter freeze/immutability management
- Preset versioning and change tracking
- Configurable solver-level settings (not user-facing)
- Traceability through audit logging

Design: Solver parameters are 'sunk' to the base settings layer
- they are not directly editable in the UI config panel unless
  explicitly unlocked by an advanced user toggle.
"""

from __future__ import annotations

import copy
import hashlib
import json
import os
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

# ── Solver Type Enum ───────────────────────────────────────

class SolverType(Enum):
    """Available numerical integration methods."""
    FORWARD_EULER = "forward_euler"
    RK4 = "rk4"
    RK45 = "rk45"           # Adaptive step size (Dormand-Prince)
    IMPLICIT_EULER = "implicit_euler"
    TRAPEZOIDAL = "trapezoidal"


class IntegrationMode(Enum):
    """Integration mode determines step size handling."""
    FIXED_STEP = "fixed_step"
    ADAPTIVE = "adaptive"
    MULTI_RATE = "multi_rate"


# ── Solver Parameter Model ─────────────────────────────────

@dataclass
class SolverParameters:
    """Core solver configuration parameters (sunk to base layer)."""

    # ── Time Stepping ──────────────────────────────────────
    dt_current: float = 50e-6        # Current loop time step [s]
    dt_speed: float = 1e-3           # Speed loop time step [s]
    dt_thermal: float = 0.1          # Thermal model time step [s]
    dt_mechanical: float = 1e-3      # Mechanical model time step [s]

    # ── Integration ────────────────────────────────────────
    solver_type: SolverType = SolverType.FORWARD_EULER
    integration_mode: IntegrationMode = IntegrationMode.FIXED_STEP
    max_sub_steps: int = 10          # Max sub-steps for adaptive mode
    rel_tolerance: float = 1e-6      # Relative tolerance for adaptive
    abs_tolerance: float = 1e-9      # Absolute tolerance for adaptive

    # ── Numerical Safety ───────────────────────────────────
    divergence_auto_recover: bool = True
    max_divergence_retries: int = 3
    divergence_step_reduction: float = 0.5  # Factor to reduce step on divergence
    nan_guard_enabled: bool = True
    clamp_outputs: bool = True

    # ── Multirate Configuration ────────────────────────────
    multi_rate_enabled: bool = False
    max_rate_ratio: int = 20         # Max dt_ratio between subsystems

    # ── Real-Time Constraints ──────────────────────────────
    realtime_target: bool = False
    max_wallclock_ratio: float = 10.0
    miss_deadline_policy: str = "skip"  # "skip", "warn", "abort"

    # ── Resource Limits ────────────────────────────────────
    max_total_steps: int = 1_000_000_000
    max_memory_mb: float = 1024.0

    def to_dict(self) -> dict:
        """Export to plain dictionary."""
        return {
            "dt_current": self.dt_current,
            "dt_speed": self.dt_speed,
            "dt_thermal": self.dt_thermal,
            "dt_mechanical": self.dt_mechanical,
            "solver_type": self.solver_type.value,
            "integration_mode": self.integration_mode.value,
            "max_sub_steps": self.max_sub_steps,
            "rel_tolerance": self.rel_tolerance,
            "abs_tolerance": self.abs_tolerance,
            "divergence_auto_recover": self.divergence_auto_recover,
            "max_divergence_retries": self.max_divergence_retries,
            "divergence_step_reduction": self.divergence_step_reduction,
            "nan_guard_enabled": self.nan_guard_enabled,
            "clamp_outputs": self.clamp_outputs,
            "multi_rate_enabled": self.multi_rate_enabled,
            "max_rate_ratio": self.max_rate_ratio,
            "realtime_target": self.realtime_target,
            "max_wallclock_ratio": self.max_wallclock_ratio,
            "miss_deadline_policy": self.miss_deadline_policy,
            "max_total_steps": self.max_total_steps,
            "max_memory_mb": self.max_memory_mb,
        }

    @classmethod
    def from_dict(cls, data: dict) -> SolverParameters:
        """Create from dictionary, with type coercion."""
        return cls(
            dt_current=float(data.get("dt_current", 50e-6)),
            dt_speed=float(data.get("dt_speed", 1e-3)),
            dt_thermal=float(data.get("dt_thermal", 0.1)),
            dt_mechanical=float(data.get("dt_mechanical", 1e-3)),
            solver_type=SolverType(data.get("solver_type", "forward_euler")),
            integration_mode=IntegrationMode(data.get("integration_mode", "fixed_step")),
            max_sub_steps=int(data.get("max_sub_steps", 10)),
            rel_tolerance=float(data.get("rel_tolerance", 1e-6)),
            abs_tolerance=float(data.get("abs_tolerance", 1e-9)),
            divergence_auto_recover=bool(data.get("divergence_auto_recover", True)),
            max_divergence_retries=int(data.get("max_divergence_retries", 3)),
            divergence_step_reduction=float(data.get("divergence_step_reduction", 0.5)),
            nan_guard_enabled=bool(data.get("nan_guard_enabled", True)),
            clamp_outputs=bool(data.get("clamp_outputs", True)),
            multi_rate_enabled=bool(data.get("multi_rate_enabled", False)),
            max_rate_ratio=int(data.get("max_rate_ratio", 20)),
            realtime_target=bool(data.get("realtime_target", False)),
            max_wallclock_ratio=float(data.get("max_wallclock_ratio", 10.0)),
            miss_deadline_policy=str(data.get("miss_deadline_policy", "skip")),
            max_total_steps=int(data.get("max_total_steps", 1_000_000_000)),
            max_memory_mb=float(data.get("max_memory_mb", 1024.0)),
        )


# ── Solver Preset Definitions ──────────────────────────────

@dataclass
class SolverPreset:
    """A named, versioned solver configuration preset."""
    name: str
    description: str
    version: str
    parameters: SolverParameters
    frozen_fields: list[str] = field(default_factory=list)
    metadata: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        """Export preset to dictionary."""
        return {
            "name": self.name,
            "description": self.description,
            "version": self.version,
            "parameters": self.parameters.to_dict(),
            "frozen_fields": list(self.frozen_fields),
            "metadata": self.metadata,
        }

    @classmethod
    def from_dict(cls, data: dict) -> SolverPreset:
        """Create preset from dictionary."""
        return cls(
            name=data["name"],
            description=data.get("description", ""),
            version=data.get("version", "1.0.0"),
            parameters=SolverParameters.from_dict(data.get("parameters", {})),
            frozen_fields=data.get("frozen_fields", []),
            metadata=data.get("metadata", {}),
        )


# ── Built-in Presets ───────────────────────────────────────

BUILTIN_PRESETS = {
    "standard": SolverPreset(
        name="Standard (Forward Euler)",
        description="Fast, stable Forward Euler integration for most PMSM/BLDC/IM simulations. 50us current loop, 1ms speed loop.",
        version="1.0.0",
        parameters=SolverParameters(
            dt_current=50e-6,
            dt_speed=1e-3,
            solver_type=SolverType.FORWARD_EULER,
            integration_mode=IntegrationMode.FIXED_STEP,
        ),
        frozen_fields=["nan_guard_enabled", "clamp_outputs", "divergence_step_reduction"],
        metadata={"suitable_for": ["PMSM_FOC", "BLDC_SIX_STEP", "IM_VECTOR"]},
    ),
    "high_precision": SolverPreset(
        name="High Precision (RK4)",
        description="4th-order Runge-Kutta for high-accuracy simulations. 25us current loop, 500us speed loop. ~4x slower than Forward Euler.",
        version="1.0.0",
        parameters=SolverParameters(
            dt_current=25e-6,
            dt_speed=500e-6,
            solver_type=SolverType.RK4,
            integration_mode=IntegrationMode.FIXED_STEP,
            rel_tolerance=1e-8,
            abs_tolerance=1e-12,
        ),
        frozen_fields=["nan_guard_enabled", "clamp_outputs"],
        metadata={"suitable_for": ["PMSM_ADVANCED", "HIGH_ACCURACY"]},
    ),
    "adaptive": SolverPreset(
        name="Adaptive Step (RK45)",
        description="Adaptive Dormand-Prince RK5(4) with automatic step size control. Best for stiff systems or variable dynamics.",
        version="1.0.0",
        parameters=SolverParameters(
            dt_current=50e-6,
            dt_speed=1e-3,
            solver_type=SolverType.RK45,
            integration_mode=IntegrationMode.ADAPTIVE,
            max_sub_steps=20,
            rel_tolerance=1e-6,
            abs_tolerance=1e-9,
        ),
        frozen_fields=["clamp_outputs"],
        metadata={"suitable_for": ["STIFF_SYSTEMS", "VARIABLE_DYNAMICS"]},
    ),
    "realtime_hil": SolverPreset(
        name="Real-Time (HIL)",
        description="Real-time solver for Hardware-in-the-Loop. Strictly fixed-step Forward Euler at 100us. Aborts on deadline miss.",
        version="1.0.0",
        parameters=SolverParameters(
            dt_current=100e-6,
            dt_speed=1e-3,
            solver_type=SolverType.FORWARD_EULER,
            integration_mode=IntegrationMode.FIXED_STEP,
            realtime_target=True,
            max_wallclock_ratio=1.2,
            miss_deadline_policy="abort",
            divergence_auto_recover=False,
        ),
        frozen_fields=[
            "nan_guard_enabled", "clamp_outputs",
            "realtime_target", "miss_deadline_policy",
        ],
        metadata={"suitable_for": ["HIL", "HARDWARE_TESTING"]},
    ),
}

# ── Solver Preset Manager ──────────────────────────────────

class SolverPresetManager:
    """Manages solver presets with persistence and version tracking.

    Features:
    - Load/save presets to YAML
    - Preset versioning and change tracking
    - Frozen parameter enforcement
    - Audit logging of configuration changes
    """

    def __init__(self, presets_dir: str | None = None):
        self._presets: dict[str, SolverPreset] = copy.deepcopy(BUILTIN_PRESETS)
        self._current_preset: str | None = None
        self._current_params: SolverParameters = SolverParameters()
        self._change_log: list[dict] = []
        self._presets_dir = presets_dir

    @property
    def current_parameters(self) -> SolverParameters:
        """Get current active solver parameters."""
        return self._current_params

    @property
    def current_preset_name(self) -> str | None:
        """Get name of current active preset."""
        return self._current_preset

    def list_presets(self) -> list[str]:
        """List all available preset names."""
        return list(self._presets.keys())

    def get_preset(self, name: str) -> SolverPreset | None:
        """Get a preset by name."""
        return self._presets.get(name)

    def apply_preset(self, name: str) -> SolverParameters:
        """Apply a preset and return the resulting parameters.

        Args:
            name: Preset name.

        Returns:
            SolverParameters instance.

        Raises:
            KeyError: If preset not found.
        """
        preset = self._presets.get(name)
        if not preset:
            raise KeyError(f"Preset not found: {name}")

        self._current_preset = name
        self._current_params = copy.deepcopy(preset.parameters)
        self._log_change("apply_preset", name, preset.version)
        return self._current_params

    def update_parameter(self, key: str, value: Any) -> bool:
        """Update a single solver parameter.

        Args:
            key: Parameter name (attribute of SolverParameters).
            value: New value.

        Returns:
            True if the parameter was updated, False if it's frozen.

        Raises:
            AttributeError: If key is not a valid parameter.
        """
        if not hasattr(self._current_params, key):
            raise AttributeError(f"Invalid solver parameter: {key}")

        # Check if frozen
        if self._current_preset:
            preset = self._presets.get(self._current_preset)
            if preset and key in preset.frozen_fields:
                return False  # Frozen — cannot modify

        old_value = getattr(self._current_params, key)
        setattr(self._current_params, key, value)
        self._log_change("update_param", key, {"from": old_value, "to": value})
        return True

    def is_frozen(self, key: str) -> bool:
        """Check if a parameter is frozen in the current preset."""
        if not self._current_preset:
            return False
        preset = self._presets.get(self._current_preset)
        if preset:
            return key in preset.frozen_fields
        return False

    def get_frozen_fields(self) -> list[str]:
        """Get list of currently frozen parameter names."""
        if not self._current_preset:
            return []
        preset = self._presets.get(self._current_preset)
        return preset.frozen_fields if preset else []

    def add_preset(self, preset: SolverPreset) -> None:
        """Add a new preset or update an existing one."""
        self._presets[preset.name] = preset
        self._log_change("add_preset", preset.name, {"version": preset.version})

    def remove_preset(self, name: str) -> bool:
        """Remove a preset. Built-in presets cannot be removed."""
        if name in BUILTIN_PRESETS:
            return False  # Cannot remove built-in presets
        if name in self._presets:
            del self._presets[name]
            self._log_change("remove_preset", name, {})
            return True
        return False

    def save_presets(self, path: str | None = None) -> str:
        """Save all presets to a YAML file.

        Args:
            path: Output path. If None, uses presets_dir/presets.yaml.

        Returns:
            Path to saved file.
        """
        if path is None:
            if self._presets_dir:
                os.makedirs(self._presets_dir, exist_ok=True)
                path = os.path.join(self._presets_dir, "solver_presets.yaml")
            else:
                raise ValueError("No path specified and no presets_dir configured")

        import yaml
        data = {
            "version": "1.0.0",
            "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
            "presets": {
                name: preset.to_dict()
                for name, preset in self._presets.items()
            },
        }

        with open(path, 'w', encoding='utf-8') as f:
            yaml.dump(data, f, default_flow_style=False, sort_keys=False)

        return path

    def load_presets(self, path: str) -> int:
        """Load presets from a YAML file.

        Args:
            path: Path to presets YAML file.

        Returns:
            Number of presets loaded.
        """
        import yaml
        if not os.path.exists(path):
            return 0

        with open(path, encoding='utf-8') as f:
            data = yaml.safe_load(f)

        presets_data = data.get("presets", {})
        count = 0
        for name, pdata in presets_data.items():
            # Don't overwrite built-in presets
            if name not in BUILTIN_PRESETS:
                self._presets[name] = SolverPreset.from_dict(pdata)
                count += 1

        return count

    def get_change_log(self, limit: int = 50) -> list[dict]:
        """Get recent configuration changes for audit trail."""
        return self._change_log[-limit:]

    def get_config_hash(self) -> str:
        """Get a hash of current solver configuration for traceability."""
        config_dict = self._current_params.to_dict()
        config_str = json.dumps(config_dict, sort_keys=True)
        return hashlib.sha256(config_str.encode()).hexdigest()[:16]

    def reset_to_defaults(self) -> SolverParameters:
        """Reset to standard Forward Euler defaults."""
        return self.apply_preset("standard")

    def _log_change(self, action: str, target: str, details: dict):
        """Log a configuration change."""
        self._change_log.append({
            "timestamp": time.time(),
            "action": action,
            "target": target,
            "details": details,
            "config_hash": self.get_config_hash(),
        })

        # Trim log if too long
        if len(self._change_log) > 1000:
            self._change_log = self._change_log[-500:]


# ── Module-level conveniences ──────────────────────────────

_default_manager = SolverPresetManager()


def get_preset_manager() -> SolverPresetManager:
    """Get the default solver preset manager instance."""
    return _default_manager
