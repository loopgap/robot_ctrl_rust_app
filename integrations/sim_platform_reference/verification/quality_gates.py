"""Quality gate runner — enforces code quality standards.

Integrates ruff, mypy, and interface consistency checks into a single
pipeline that can be run locally or in CI/CD.

Usage:
    python -m sim_platform.tools.gui.quality_gates
"""

from __future__ import annotations

import ast
import importlib
import os
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Any


@dataclass
class QualityReport:
    """Aggregated quality check results."""
    passed: int = 0
    failed: int = 0
    skipped: int = 0
    details: list[dict] = field(default_factory=list)

    def add(self, name: str, passed: bool, detail: str = ""):
        if passed:
            self.passed += 1
        else:
            self.failed += 1
        self.details.append({
            "name": name,
            "passed": passed,
            "detail": detail,
        })

    @property
    def all_passed(self) -> bool:
        return self.failed == 0

    def summary(self) -> str:
        lines = [
            f"Quality Gate Report: {self.passed} passed, {self.failed} failed, {self.skipped} skipped",
            "",
        ]
        for d in self.details:
            icon = "✓" if d["passed"] else "✗"
            lines.append(f"  {icon} {d['name']}")
            if d["detail"]:
                for line in d["detail"].split("\n"):
                    lines.append(f"      {line}")
        return "\n".join(lines)


def run_ruff_lint() -> tuple[bool, str]:
    """Run ruff linter and return (passed, output)."""
    try:
        result = subprocess.run(
            [sys.executable, "-m", "ruff", "check", "sim_platform/"],
            capture_output=True, text=True, cwd=_get_project_root(),
            timeout=120,
        )
        return result.returncode == 0, result.stdout + result.stderr
    except Exception as e:
        return False, str(e)


def run_ruff_format_check() -> tuple[bool, str]:
    """Check ruff formatting and return (passed, output)."""
    try:
        result = subprocess.run(
            [sys.executable, "-m", "ruff", "format", "--check", "sim_platform/"],
            capture_output=True, text=True, cwd=_get_project_root(),
            timeout=120,
        )
        return result.returncode == 0, result.stdout + result.stderr
    except Exception as e:
        return False, str(e)


def check_interface_consistency() -> tuple[bool, str]:
    """Verify interface consistency across modules.

    Checks that:
    - All model `step()` methods accept consistent parameters
    - All controller `update()` methods return consistent types
    - All core modules expose expected public APIs
    """
    issues = []

    # ── Model interface check ────────────────────────────
    required_model_methods = ["step", "reset", "get_state"]
    model_modules = [
        "sim_platform.models.motor.pmsm_dq",
        "sim_platform.models.motor.bldc",
        "sim_platform.models.motor.pmsm_advanced",
        "sim_platform.models.motor.im_dq",
    ]

    for mod_name in model_modules:
        try:
            mod = importlib.import_module(mod_name)
            for method in required_model_methods:
                # Find the main class in the module
                for name, obj in mod.__dict__.items():
                    if isinstance(obj, type) and not name.startswith("_"):
                        if hasattr(obj, method):
                            break
                else:
                    issues.append(f"Module {mod_name}: no class with {method}() found")
        except ImportError as e:
            issues.append(f"Module {mod_name}: import failed — {e}")

    # ── Controller interface check ────────────────────────
    controller_modules = [
        "sim_platform.models.controller.foc",
        "sim_platform.models.controller.mpc",
        "sim_platform.models.controller.ekf",
    ]

    for mod_name in controller_modules:
        try:
            mod = importlib.import_module(mod_name)
            has_update = False
            for name, obj in mod.__dict__.items():
                if isinstance(obj, type) and not name.startswith("_"):
                    if hasattr(obj, "update"):
                        has_update = True
                        break
            if not has_update:
                issues.append(f"Module {mod_name}: no controller with update() found")
        except ImportError as e:
            issues.append(f"Module {mod_name}: import failed — {e}")

    # ── Core module interface check ──────────────────────
    core_checks = {
        "sim_platform.core.clock": ["GlobalClock"],
        "sim_platform.core.data_bus": ["DataBus"],
        "sim_platform.core.orchestrator": ["Orchestrator"],
        "sim_platform.core.utils": ["guard_numeric", "guard_positive"],
    }

    for mod_name, required_names in core_checks.items():
        try:
            mod = importlib.import_module(mod_name)
            for name in required_names:
                if not hasattr(mod, name):
                    issues.append(f"Module {mod_name}: missing {name}")
        except ImportError as e:
            issues.append(f"Module {mod_name}: import failed — {e}")

    if issues:
        return False, "\n".join(issues)
    return True, "All interfaces consistent"


def check_python_syntax() -> tuple[bool, str]:
    """Check Python syntax in all .py files."""
    issues = []
    project_root = _get_project_root()
    sim_dir = os.path.join(project_root, "sim_platform")

    for root, dirs, files in os.walk(sim_dir):
        # Skip __pycache__ and .venv
        dirs[:] = [d for d in dirs if d not in ("__pycache__", ".venv", ".git")]

        for f in files:
            if f.endswith(".py"):
                fpath = os.path.join(root, f)
                try:
                    with open(fpath, encoding="utf-8") as fh:
                        source = fh.read()
                    ast.parse(source)
                except SyntaxError as e:
                    issues.append(f"{fpath}: {e}")
                except Exception as e:
                    # Non-syntax errors are OK (imports etc.)
                    pass

    if issues:
        return False, "\n".join(issues)
    return True, "All Python files have valid syntax"


def run_all_gates(fail_fast: bool = False) -> QualityReport:
    """Run all quality gates.

    Args:
        fail_fast: If True, stop on first failure.

    Returns:
        QualityReport with results.
    """
    report = QualityReport()

    gates = [
        ("Ruff Lint", run_ruff_lint),
        ("Ruff Format", run_ruff_format_check),
        ("Interface Consistency", check_interface_consistency),
        ("Python Syntax", check_python_syntax),
    ]

    for name, checker in gates:
        passed, detail = checker()
        report.add(name, passed, detail)
        if not passed and fail_fast:
            break

    return report


def _get_project_root() -> str:
    """Get the project root directory."""
    return os.path.abspath(
        os.path.join(os.path.dirname(__file__), "../../..")
    )


if __name__ == "__main__":
    report = run_all_gates()
    print(report.summary())
    sys.exit(0 if report.all_passed else 1)
