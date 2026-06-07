"""Parameter conflict detection and multi-strategy resolution system.

Provides a comprehensive framework for handling parameter coupling conflicts
and warnings in industrial simulation platforms. Unlike simple yes/no dialogs,
this system supports:

- Multi-strategy conflict resolution (auto-fix, manual override, ignore, rollback)
- User-definable conflict rules with priority ordering
- Conflict impact scope visualization (which subsystems are affected)
- Configurable resolution policies per conflict category
- Audit trail of all resolution decisions

Architecture:
    ConflictDetector → ConflictResolutionEngine → ResolutionUI
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

from sim_platform.models.physics_constraints import (
    ConstraintViolation,
    PhysicsValidator,
)

# ── Enums ──────────────────────────────────────────────────

class ConflictSeverity(Enum):
    """Severity levels for parameter conflicts."""
    BLOCKER = "blocker"     # Must be resolved; simulation cannot proceed
    CRITICAL = "critical"   # High-risk; strongly recommended to fix
    WARNING = "warning"     # Potential issue; user should be informed
    INFO = "info"           # Informational; no action required


class ResolutionStrategy(Enum):
    """Available resolution strategies for conflicts."""
    AUTO_FIX = "auto_fix"           # Automatically apply suggested fix
    MANUAL_OVERRIDE = "manual"      # Let user manually adjust parameters
    IGNORE_THIS_RUN = "ignore"      # Ignore for current run only
    IGNORE_ALWAYS = "ignore_always" # Ignore and remember for future
    ROLLBACK = "rollback"           # Revert to last known good config
    ASK_EACH_TIME = "ask"           # Always prompt (default)


class ConflictDomain(Enum):
    """Subsystem domains affected by a conflict."""
    MOTOR = "motor"
    CONTROLLER = "controller"
    POWER = "power"
    THERMAL = "thermal"
    SENSOR = "sensor"
    SOLVER = "solver"
    OPERATING_POINT = "operating_point"
    CROSS_DOMAIN = "cross_domain"


# ── Data Models ────────────────────────────────────────────

@dataclass
class ConflictImpact:
    """Describes the impact scope of a parameter conflict."""
    domain: ConflictDomain
    affected_parameters: list[str]
    subsystem_effects: list[str]  # Human-readable effect descriptions
    severity: ConflictSeverity

    # Quantified impact estimates (0.0 = no impact, 1.0 = critical)
    result_accuracy_impact: float = 0.0
    convergence_impact: float = 0.0
    stability_impact: float = 0.0


@dataclass
class ConflictResolution:
    """A single resolution decision."""
    violation: ConstraintViolation
    strategy: ResolutionStrategy
    resolved_by: str = "user"  # "user", "auto", "policy"
    timestamp: float = field(default_factory=time.time)
    auto_fix_value: Any = None

    def to_dict(self) -> dict:
        return {
            "parameter": self.violation.parameter,
            "strategy": self.strategy.value,
            "resolved_by": self.resolved_by,
            "timestamp": self.timestamp,
            "auto_fix_value": self.auto_fix_value,
        }


@dataclass
class ConflictRule:
    """User-defined rule for handling specific conflict patterns.

    Rules are pattern-matched against incoming violations and dictate
    the resolution strategy automatically.
    """
    rule_id: str
    name: str
    description: str
    parameter_pattern: str  # Simple wildcard match: "motor.*", "foc.kp_*"
    severity_filter: list[ConflictSeverity] = field(default_factory=list)
    default_strategy: ResolutionStrategy = ResolutionStrategy.ASK_EACH_TIME
    priority: int = 50  # Lower = higher priority (1-100)
    enabled: bool = True

    def matches(self, violation: ConstraintViolation) -> bool:
        """Check if this rule matches a violation."""
        # Pattern matching
        param = violation.parameter
        pattern = self.parameter_pattern

        if pattern.endswith(".*"):
            prefix = pattern[:-2]
            if not param.startswith(prefix):
                return False
        elif pattern.endswith(".*"):
            prefix = pattern[:-1]
            if not param.startswith(prefix):
                return False
        elif pattern != param:
            return False

        # Severity filter
        if self.severity_filter:
            sev_map = {
                "error": ConflictSeverity.BLOCKER,
                "warning": ConflictSeverity.WARNING,
                "info": ConflictSeverity.INFO,
            }
            violation_sev = sev_map.get(violation.severity, ConflictSeverity.WARNING)
            if violation_sev not in self.severity_filter:
                return False

        return True


# ── Conflict Detector ──────────────────────────────────────

class ConflictDetector:
    """Enhanced detector that classifies conflicts into structured impacts.

    Extends PhysicsValidator with impact analysis and domain classification.
    """

    def __init__(self):
        self._validator = PhysicsValidator()

    def detect(self, config: dict) -> list[tuple[ConstraintViolation, ConflictImpact]]:
        """Detect all conflicts and attach impact analysis.

        Returns:
            List of (violation, impact) tuples.
        """
        violations = self._validator.validate(config)
        results = []

        for v in violations:
            impact = self._analyze_impact(v, config)
            results.append((v, impact))

        return results

    def detect_errors(self, config: dict) -> list[ConstraintViolation]:
        """Detect only error-level violations."""
        violations = self._validator.validate(config)
        return [v for v in violations if v.severity == "error"]

    def detect_warnings(self, config: dict) -> list[ConstraintViolation]:
        """Detect only warning-level violations."""
        violations = self._validator.validate(config)
        return [v for v in violations if v.severity == "warning"]

    def get_blockers(self, config: dict) -> list[ConstraintViolation]:
        """Get violations that prevent simulation from running."""
        return self.detect_errors(config)

    def get_impact_summary(self, config: dict) -> dict[str, Any]:
        """Generate a structured impact summary.

        Returns:
            Dict with domains as keys and impact scores as values.
        """
        conflicts = self.detect(config)
        summary = {
            "total_conflicts": len(conflicts),
            "blockers": 0,
            "warnings": 0,
            "info": 0,
            "domains_affected": {},
            "can_proceed": True,
        }

        for v, impact in conflicts:
            sev = v.severity
            if sev == "error":
                summary["blockers"] += 1
            elif sev == "warning":
                summary["warnings"] += 1
            else:
                summary["info"] += 1

            domain = impact.domain.value
            if domain not in summary["domains_affected"]:
                summary["domains_affected"][domain] = {
                    "count": 0,
                    "max_severity": "info",
                    "accuracy_impact": 0.0,
                    "stability_impact": 0.0,
                }

            d = summary["domains_affected"][domain]
            d["count"] += 1
            d["max_severity"] = max(
                d["max_severity"], sev,
                key=lambda s: ["info", "warning", "error"].index(s)
            )
            d["accuracy_impact"] = max(d["accuracy_impact"], impact.result_accuracy_impact)
            d["stability_impact"] = max(d["stability_impact"], impact.stability_impact)

        if summary["blockers"] > 0:
            summary["can_proceed"] = False

        return summary

    def _analyze_impact(self, v: ConstraintViolation, config: dict) -> ConflictImpact:
        """Analyze the impact scope of a violation."""
        param = v.parameter

        # Domain classification
        if param in ("Rs", "Ld", "Lq", "flux_pm", "J", "B", "Pp"):
            domain = ConflictDomain.MOTOR
            affected = [param]
            effects = [
                f"电机参数 {param} 超出推荐范围",
                f"当前值={v.current_value}, 限制={v.limit}",
            ]
            accuracy = 0.6
            convergence = 0.3
            stability = 0.5
        elif param.startswith("kp_") or param.startswith("ki_"):
            domain = ConflictDomain.CONTROLLER
            affected = [param, param.replace("kp", "ki").replace("ki", "kp")]
            effects = [
                f"控制器增益 {param} 设置不当",
                "可能导致电流/速度环不稳定",
            ]
            accuracy = 0.3
            convergence = 0.7
            stability = 0.8
        elif param in ("spd_kp", "spd_ki"):
            domain = ConflictDomain.CONTROLLER
            affected = [param]
            effects = ["速度环参数设置不当"]
            accuracy = 0.2
            convergence = 0.6
            stability = 0.7
        elif param in ("dt_c", "dt_s", "duration"):
            domain = ConflictDomain.SOLVER
            affected = [param]
            effects = [
                f"求解器时间参数 {param} 设置不当",
                "可能影响仿真精度和计算时间",
            ]
            accuracy = 0.8
            convergence = 0.5
            stability = 0.4
        elif param in ("speed_ref", "load_torque") or "battery" in str(param).lower():
            domain = ConflictDomain.OPERATING_POINT
            affected = [param]
            effects = ["工作点参数超出电机能力范围"]
            accuracy = 0.5
            convergence = 0.3
            stability = 0.2
        else:
            domain = ConflictDomain.CROSS_DOMAIN
            affected = [param]
            effects = ["跨域参数冲突"]
            accuracy = 0.4
            convergence = 0.4
            stability = 0.4

        severity_map = {
            "error": ConflictSeverity.BLOCKER,
            "warning": ConflictSeverity.WARNING,
            "info": ConflictSeverity.INFO,
        }
        severity = severity_map.get(v.severity, ConflictSeverity.WARNING)

        return ConflictImpact(
            domain=domain,
            affected_parameters=affected,
            subsystem_effects=effects,
            severity=severity,
            result_accuracy_impact=accuracy,
            convergence_impact=convergence,
            stability_impact=stability,
        )


# ── Conflict Resolution Engine ─────────────────────────────

class ConflictResolutionEngine:
    """Manages conflict resolution rules and decision-making.

    Supports:
    - Auto-fix rules (pre-configured parameter adjustments)
    - User-defined rules (persisted)
    - Default policies per severity/domain
    - Resolution audit trail
    """

    def __init__(self):
        self._rules: list[ConflictRule] = []
        self._resolutions: list[ConflictResolution] = []
        self._detector = ConflictDetector()

        # Default rules
        self._init_default_rules()

    def _init_default_rules(self):
        """Initialize sensible default conflict rules."""
        defaults = [
            ConflictRule(
                rule_id="default.motor.range",
                name="电机参数范围警告",
                description="电机参数超出推荐范围时的默认处理",
                parameter_pattern="Rs",
                severity_filter=[ConflictSeverity.WARNING],
                default_strategy=ResolutionStrategy.ASK_EACH_TIME,
                priority=100,
            ),
            ConflictRule(
                rule_id="default.controller.stability",
                name="控制器稳定性警告",
                description="电流环带宽接近奈奎斯特频率时的处理",
                parameter_pattern="kp_iq/Ld",
                severity_filter=[ConflictSeverity.WARNING],
                default_strategy=ResolutionStrategy.ASK_EACH_TIME,
                priority=90,
            ),
            ConflictRule(
                rule_id="default.solver.dt",
                name="求解器时间步长警告",
                description="时间步长设置不当时的处理",
                parameter_pattern="dt_*",
                severity_filter=[ConflictSeverity.WARNING],
                default_strategy=ResolutionStrategy.ASK_EACH_TIME,
                priority=80,
            ),
            ConflictRule(
                rule_id="default.op.back_emf",
                name="反电动势超限",
                description="反电动势接近母线电压时的处理",
                parameter_pattern="speed_ref",
                severity_filter=[ConflictSeverity.BLOCKER],
                default_strategy=ResolutionStrategy.ASK_EACH_TIME,
                priority=10,
            ),
        ]

        for rule in defaults:
            self.add_rule(rule)

    def add_rule(self, rule: ConflictRule) -> None:
        """Add or update a conflict resolution rule."""
        # Remove existing rule with same ID
        self._rules = [r for r in self._rules if r.rule_id != rule.rule_id]
        self._rules.append(rule)
        # Sort by priority (lower = higher)
        self._rules.sort(key=lambda r: r.priority)

    def remove_rule(self, rule_id: str) -> None:
        """Remove a rule by ID."""
        self._rules = [r for r in self._rules if r.rule_id != rule_id]

    def get_rules(self) -> list[ConflictRule]:
        """Get all rules sorted by priority."""
        return list(self._rules)

    def resolve(
        self,
        config: dict,
        auto_apply: bool = False,
    ) -> list[ConflictResolution]:
        """Resolve all conflicts in config using configured rules.

        Args:
            config: Simulation configuration dictionary.
            auto_apply: If True, auto-apply fixes following rules.
                        If False, only return resolution decisions without applying.

        Returns:
            List of resolution decisions.
        """
        conflicts = self._detector.detect(config)
        resolutions = []

        for violation, impact in conflicts:
            resolution = self._resolve_one(violation, impact, auto_apply)
            resolutions.append(resolution)

        self._resolutions = resolutions
        return resolutions

    def _resolve_one(
        self,
        violation: ConstraintViolation,
        impact: ConflictImpact,
        auto_apply: bool,
    ) -> ConflictResolution:
        """Resolve a single conflict."""
        # Find matching rules
        matching_rules = [r for r in self._rules if r.matches(violation) and r.enabled]

        if matching_rules:
            # Use highest priority matching rule
            rule = matching_rules[0]
            strategy = rule.default_strategy
        else:
            # Default: ask every time
            strategy = ResolutionStrategy.ASK_EACH_TIME

        resolution = ConflictResolution(
            violation=violation,
            strategy=strategy,
            resolved_by="auto" if auto_apply else "rule",
        )

        # Auto-fix if applicable
        if strategy == ResolutionStrategy.AUTO_FIX:
            fix_value = self._compute_auto_fix(violation)
            resolution.auto_fix_value = fix_value

        return resolution

    def _compute_auto_fix(self, violation: ConstraintViolation) -> Any:
        """Compute the suggested auto-fix value for a violation."""
        param = violation.parameter
        limit = violation.limit

        if param in ("Rs", "Ld", "Lq", "flux_pm", "J", "B"):
            return limit
        elif param in ("kp_id", "ki_id", "kp_iq", "ki_iq", "kp", "ki"):
            return max(limit, 0.01)
        elif param in ("dt_c", "dt_s") or param in ("speed_ref",) or param in ("load_torque",):
            return limit
        else:
            return limit

    def apply_resolution(
        self,
        config: dict,
        resolution: ConflictResolution,
    ) -> dict:
        """Apply a single resolution to the config.

        Args:
            config: Current configuration (will create a copy).
            resolution: Resolution decision to apply.

        Returns:
            Modified configuration copy.
        """
        import copy
        new_config = copy.deepcopy(config)

        v = resolution.violation
        param = v.parameter

        if resolution.strategy == ResolutionStrategy.AUTO_FIX and resolution.auto_fix_value is not None:
            # Navigate nested config to set the value
            if param in new_config:
                new_config[param] = resolution.auto_fix_value
            elif "motor_params" in new_config and param in new_config["motor_params"]:
                new_config["motor_params"][param] = resolution.auto_fix_value
            elif "foc" in new_config and param in new_config["foc"]:
                new_config["foc"][param] = resolution.auto_fix_value
            elif "speed_pi" in new_config and param in new_config["speed_pi"]:
                new_config["speed_pi"][param] = resolution.auto_fix_value

        self._resolutions.append(resolution)
        return new_config

    def get_audit_trail(self) -> list[dict]:
        """Get full audit trail of resolution decisions."""
        return [r.to_dict() for r in self._resolutions]

    def get_resolution_summary(self) -> dict:
        """Get a summary of resolution decisions."""
        if not self._resolutions:
            return {"total": 0, "strategies": {}}

        strategy_counts = {}
        for r in self._resolutions:
            s = r.strategy.value
            strategy_counts[s] = strategy_counts.get(s, 0) + 1

        return {
            "total": len(self._resolutions),
            "strategies": strategy_counts,
        }

    def clear_audit(self):
        """Clear the resolution audit trail."""
        self._resolutions.clear()


# ── Impact Visualization Helper ────────────────────────────

def generate_impact_heatmap(
    violations: list[ConstraintViolation],
    impacts: list[ConflictImpact],
) -> str:
    """Generate an ASCII heatmap showing which subsystems are impacted.

    Returns:
        Formatted string suitable for display in log or tooltip.
    """
    if not violations or not impacts:
        return "No conflicts detected."

    lines = []
    lines.append("╔═══════════════ Conflict Impact Heatmap ═══════════════╗")
    lines.append("║ Domain        │ Score │ Sev │ Parameters             ║")
    lines.append("╠═══════════════╪═══════╪═════╪════════════════════════╣")

    domain_order = [
        ConflictDomain.OPERATING_POINT,
        ConflictDomain.CONTROLLER,
        ConflictDomain.MOTOR,
        ConflictDomain.SOLVER,
        ConflictDomain.POWER,
        ConflictDomain.CROSS_DOMAIN,
    ]

    seen_domains = set()
    for v, impact in zip(violations, impacts):
        domain = impact.domain
        if domain in seen_domains:
            continue
        seen_domains.add(domain)

        bars = "█" * int(impact.result_accuracy_impact * 10)
        name = domain.value[:13].ljust(13)
        score = f"{impact.result_accuracy_impact:.1f}".ljust(5)
        sev = v.severity[:5].ljust(5)
        params = ", ".join(impact.affected_parameters[:3])

        lines.append(f"║ {name} │ {score} │ {sev} │ {params[:22].ljust(22)} ║")

    lines.append("╚═══════════════╧═══════╧═════╧════════════════════════╝")
    return "\n".join(lines)
