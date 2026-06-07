"""Development environment setup script.

Usage:
    python scripts/setup_dev.py

Sets up:
    1. Python virtual environment
    2. Development dependencies
    3. Pre-commit hooks
    4. Verifies installation
"""

import os
import subprocess
import sys
from pathlib import Path


def run(cmd: str, check: bool = True) -> subprocess.CompletedProcess:
    """Run a shell command."""
    print(f"  > {cmd}")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if check and result.returncode != 0:
        print(f"  ERROR: {result.stderr}")
        sys.exit(1)
    return result


def main():
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)
    print(f"Setting up development environment in: {project_root}\n")

    # 1. Check Python version
    print("[1/5] Checking Python version...")
    ver = sys.version_info
    if ver < (3, 10):
        print(f"  ERROR: Python 3.10+ required, got {ver.major}.{ver.minor}")
        sys.exit(1)
    print(f"  OK: Python {ver.major}.{ver.minor}.{ver.micro}\n")

    # 2. Install dependencies
    print("[2/5] Installing dependencies...")
    run(f'"{sys.executable}" -m pip install -e ".[dev]" --quiet')
    print("  OK\n")

    # 3. Install pre-commit
    print("[3/5] Setting up pre-commit hooks...")
    run(f'"{sys.executable}" -m pip install pre-commit --quiet', check=False)
    result = run(f'"{sys.executable}" -m pre_commit install --install-hooks', check=False)
    if result.returncode != 0:
        print("  WARNING: pre-commit install failed (git hooks not set up)")
        print("  You can manually run: pre-commit install")
    else:
        print("  OK\n")

    # 4. Run ruff check
    print("[4/5] Running ruff lint check...")
    result = run(f'"{sys.executable}" -m ruff check . --statistics', check=False)
    if result.returncode == 0:
        print("  OK: No lint issues\n")
    else:
        print(f"  WARNING: {result.stdout.strip()}\n")

    # 5. Run quick tests
    print("[5/5] Running quick tests...")
    result = run(
        f'"{sys.executable}" -m pytest verification/test_cases/test_core.py '
        f'verification/test_cases/test_motor_models.py -x -q --tb=no',
        check=False,
    )
    if result.returncode == 0:
        print(f"  OK: {result.stdout.strip()}\n")
    else:
        print("  WARNING: Some tests failed\n")

    print("=" * 50)
    print("Development environment setup complete!")
    print("\nNext steps:")
    print("  1. Run all tests:    python -m pytest verification/")
    print("  2. Run TUI:          python -m sim_platform --tui")
    print("  3. Run benchmark:    python tools/profiling/benchmark.py")
    print("  4. Run coverage:     python -m pytest verification/ --cov=sim_platform")


if __name__ == "__main__":
    main()
