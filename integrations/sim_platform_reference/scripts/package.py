#!/usr/bin/env python3
"""sim_platform 项目打包脚本.

Usage:
    python scripts/package.py              # 打包到 dist/
    python scripts/package.py --zip        # 创建 zip 归档
    python scripts/package.py --check      # 仅检查完整性
"""

import argparse
import hashlib
import os
import shutil
import sys
import zipfile

# ── Config ────────────────────────────────────────────────
_PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
_DIST = os.path.join(_PROJ, "dist")
_PACKAGE_NAME = "sim_platform_v1.0.0"

# ── Files to include ──────────────────────────────────────
INCLUDE_GLOBS = [
    "core/*.py",
    "models/**/*.py",
    "tools/**/*.py",
    "verification/**/*.py",
    "verification/fault_injection/*.py",
    "examples/**/*.py",
    "examples/**/*.yaml",
    "deployment/**/*.py",
]

INCLUDE_FILES = [
    "pyproject.toml",
    "requirements.txt",
    "README.md",
    "TEAM_GUIDE.md",
    "CHANGELOG.md",
    "output/.gitkeep",  # ensure output dir exists
]

EXCLUDE_PATTERNS = ["__pycache__", ".pyc", "*.pyc"]


def _should_exclude(name: str) -> bool:
    for pat in EXCLUDE_PATTERNS:
        if pat in name or name.endswith(".pyc"):
            return True
    return False


def check_integrity() -> list:
    """Check all expected files exist. Returns list of missing files."""
    missing = []

    # Check core modules
    for mod in ["clock.py", "orchestrator.py", "data_bus.py", "model_registry.py"]:
        path = os.path.join(_PROJ, "core", mod)
        if not os.path.exists(path):
            missing.append(path)

    # Check models
    for mod in ["pmsm_dq.py", "foc.py", "sensors.py", "power_models.py"]:
        found = False
        for root, _, files in os.walk(os.path.join(_PROJ, "models")):
            if mod in files:
                found = True
                break
        if not found:
            missing.append(f"models/**/{mod}")

    # Check verification
    test_files = ["test_foc_mvp.py", "test_tui.py", "stress_test.py"]
    for tf in test_files:
        path = os.path.join(_PROJ, "verification", "test_cases", tf)
        if not os.path.exists(path):
            missing.append(path)

    # Check TUI
    tui_main = os.path.join(_PROJ, "tools", "tui", "app.py")
    if not os.path.exists(tui_main):
        missing.append(tui_main)

    # Check packaging files
    for f in INCLUDE_FILES:
        path = os.path.join(_PROJ, f)
        if not os.path.exists(path):
            missing.append(path)

    return missing


def copy_sources(target_dir: str) -> None:
    """Copy source files to target directory."""
    os.makedirs(target_dir, exist_ok=True)

    # Copy package directory
    src = _PROJ
    dst = os.path.join(target_dir, "sim_platform")

    if os.path.exists(dst):
        shutil.rmtree(dst)

    # Walk and copy all .py files + config
    for root, dirs, files in os.walk(src):
        # Skip hidden dirs, __pycache__, dist, .workbuddy
        dirs[:] = [d for d in dirs
                   if not d.startswith(".") and d != "__pycache__"
                   and d != "dist" and d != ".pytest_cache"]

        for f in files:
            if _should_exclude(f):
                continue
            if not (f.endswith(".py") or f.endswith(".yaml") or f.endswith(".toml")
                    or f.endswith(".txt") or f.endswith(".md")):
                continue

            src_path = os.path.join(root, f)
            rel = os.path.relpath(src_path, src)
            dst_path = os.path.join(target_dir, rel)
            os.makedirs(os.path.dirname(dst_path), exist_ok=True)
            shutil.copy2(src_path, dst_path)

    # Create output dir
    os.makedirs(os.path.join(target_dir, "output"), exist_ok=True)


def create_zip(target_dir: str, output_path: str) -> str:
    """Create zip archive."""
    zip_path = os.path.join(_DIST, output_path)
    os.makedirs(_DIST, exist_ok=True)

    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for root, _, files in os.walk(target_dir):
            for f in files:
                file_path = os.path.join(root, f)
                arcname = os.path.relpath(file_path, target_dir)
                zf.write(file_path, arcname)

    return zip_path


def compute_sha256(filepath: str) -> str:
    """Compute SHA-256 hash of a file."""
    sha256 = hashlib.sha256()
    with open(filepath, "rb") as f:
        for block in iter(lambda: f.read(65536), b""):
            sha256.update(block)
    return sha256.hexdigest()


def main():
    ap = argparse.ArgumentParser(description="sim_platform package script")
    ap.add_argument("--check", action="store_true", help="Only check integrity")
    ap.add_argument("--zip", action="store_true", help="Create zip archive")
    ap.add_argument("--output", default=f"{_PACKAGE_NAME}.zip", help="Output zip name")
    args = ap.parse_args()

    print("[sim_platform] Packaging...")
    print(f"  Source: {_PROJ}")

    # Integrity check
    print("\n[CHECK] File integrity...")
    missing = check_integrity()
    if missing:
        print("  [FAIL] Missing files:")
        for m in missing:
            print(f"    - {m}")
        sys.exit(1)
    print("  [PASS] All files present")

    # Check test status
    print("\n[CHECK] Test summary...")
    test_dir = os.path.join(_PROJ, "verification", "test_cases")
    test_count = 0
    for f in ["test_foc_mvp.py", "test_tui.py", "stress_test.py"]:
        path = os.path.join(test_dir, f)
        if os.path.exists(path):
            with open(path, encoding="utf-8") as fh:
                tests = sum(1 for line in fh if "def test_" in line)
                test_count += tests
    print(f"  Test cases found: {test_count}")

    if args.check:
        print("\n[PASS] Integrity check complete")
        return

    # Copy sources
    print("\n[BUILD] Copying sources...")
    build_dir = os.path.join(_DIST, _PACKAGE_NAME)
    if os.path.exists(build_dir):
        shutil.rmtree(build_dir)
    copy_sources(build_dir)
    print(f"  Copied to: {build_dir}")

    # Optional zip
    if args.zip:
        print("\n[BUILD] Creating zip archive...")
        zip_path = create_zip(build_dir, args.output)
        sha = compute_sha256(zip_path)
        print(f"  Archive:  {zip_path}")
        print(f"  Size:     {os.path.getsize(zip_path) / 1024:.1f} KB")
        print(f"  SHA-256:  {sha}")

        # Write checksum
        sha_path = zip_path + ".sha256"
        with open(sha_path, "w") as f:
            f.write(sha)
        print(f"  Checksum: {sha_path}")

    print("\n[DONE] Package complete")
    if not args.zip:
        print("  Use --zip to create redistributable archive")


if __name__ == "__main__":
    main()
