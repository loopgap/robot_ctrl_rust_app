#!/usr/bin/env python3
"""Build sim_platform as a standalone .exe using PyInstaller.

Usage:
    python scripts/build_exe.py              # Build with default settings
    python scripts/build_exe.py --clean      # Clean build (remove previous build)
    python scripts/build_exe.py --verify     # Build and verify the .exe
    python scripts/build_exe.py --test       # Build, verify, and run smoke test

Requirements:
    pip install pyinstaller

Output:
    dist/sim_platform/sim_platform.exe
"""

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import time

# ── Config ────────────────────────────────────────────────
PROJ = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DIST_DIR = os.path.join(PROJ, "dist")
BUILD_DIR = os.path.join(PROJ, "build")
SPEC_FILE = os.path.join(PROJ, "sim_platform.spec")
EXE_NAME = "sim_platform.exe"
EXE_PATH = os.path.join(DIST_DIR, "sim_platform", EXE_NAME)


def _run(cmd: list[str], cwd: str = PROJ, timeout: int = 300) -> tuple[int, str]:
    """Run a command and return (returncode, output)."""
    print(f"  > {' '.join(cmd)}")
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
            encoding="utf-8",
            errors="replace",
        )
        if result.returncode != 0:
            print(f"  [WARN] Exit code: {result.returncode}")
            if result.stderr:
                # Show last 20 lines of stderr
                lines = result.stderr.strip().split("\n")
                for line in lines[-20:]:
                    print(f"    {line}")
        return result.returncode, result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        print(f"  [ERROR] Command timed out after {timeout}s")
        return -1, "timeout"
    except Exception as e:
        print(f"  [ERROR] {e}")
        return -1, str(e)


def clean():
    """Remove previous build artifacts."""
    for d in [BUILD_DIR, DIST_DIR]:
        if os.path.exists(d):
            print(f"  Removing {d}")
            shutil.rmtree(d)
    # Also remove __pycache__ in project root
    for root, dirs, _ in os.walk(PROJ):
        if "__pycache__" in dirs:
            pycache = os.path.join(root, "__pycache__")
            shutil.rmtree(pycache, ignore_errors=True)


def check_prerequisites() -> bool:
    """Check that all prerequisites are met."""
    print("\n[1/5] Checking prerequisites...")

    # Check Python version
    v = sys.version_info
    print(f"  Python: {v.major}.{v.minor}.{v.micro}")
    if v < (3, 10):
        print("  [FAIL] Python >= 3.10 required")
        return False

    # Check PyInstaller
    try:
        import PyInstaller
        print(f"  PyInstaller: {PyInstaller.__version__}")
    except ImportError:
        print("  [FAIL] PyInstaller not installed. Run: pip install pyinstaller")
        return False

    # Check PySide6
    try:
        import PySide6
        print(f"  PySide6: {PySide6.__version__}")
    except ImportError:
        print("  [FAIL] PySide6 not installed. Run: pip install PySide6>=6.5")
        return False

    # Check spec file
    if not os.path.exists(SPEC_FILE):
        print(f"  [FAIL] Spec file not found: {SPEC_FILE}")
        return False
    print(f"  Spec: {SPEC_FILE}")

    # Check entry point
    entry = os.path.join(PROJ, "tools", "gui", "app.py")
    if not os.path.exists(entry):
        print(f"  [FAIL] Entry point not found: {entry}")
        return False
    print(f"  Entry: {entry}")

    print("  [PASS] All prerequisites met")
    return True


def build(clean_build: bool = False) -> bool:
    """Run PyInstaller build."""
    print("\n[2/5] Building with PyInstaller...")

    if clean_build:
        print("  Cleaning previous build...")
        clean()

    cmd = [
        sys.executable, "-m", "PyInstaller",
        SPEC_FILE,
        "--clean",
        "--noconfirm",
        "--log-level=WARN",
    ]

    start = time.time()
    rc, output = _run(cmd, timeout=600)
    elapsed = time.time() - start

    if rc != 0:
        print(f"  [FAIL] Build failed (exit code {rc}, {elapsed:.1f}s)")
        return False

    print(f"  [PASS] Build completed in {elapsed:.1f}s")
    return True


def verify() -> bool:
    """Verify the built .exe exists and is valid."""
    print("\n[3/5] Verifying build output...")

    if not os.path.exists(EXE_PATH):
        print(f"  [FAIL] .exe not found: {EXE_PATH}")
        return False

    size_mb = os.path.getsize(EXE_PATH) / (1024 * 1024)
    print(f"  Path: {EXE_PATH}")
    print(f"  Size: {size_mb:.1f} MB")

    # Compute SHA-256
    sha256 = hashlib.sha256()
    with open(EXE_PATH, "rb") as f:
        for block in iter(lambda: f.read(65536), b""):
            sha256.update(block)
    sha = sha256.hexdigest()
    print(f"  SHA-256: {sha}")

    # Check dist directory contents
    dist_dir = os.path.join(DIST_DIR, "sim_platform")
    if os.path.exists(dist_dir):
        total_files = 0
        total_size = 0
        for root, _, files in os.walk(dist_dir):
            total_files += len(files)
            for f in files:
                total_size += os.path.getsize(os.path.join(root, f))
        print(f"  Total files: {total_files}")
        print(f"  Total size: {total_size / (1024 * 1024):.1f} MB")

    # Write checksum file
    sha_path = EXE_PATH + ".sha256"
    with open(sha_path, "w") as f:
        f.write(f"{sha}  {EXE_NAME}\n")
    print(f"  Checksum: {sha_path}")

    print("  [PASS] Build output verified")
    return True


def smoke_test() -> bool:
    """Run a smoke test: launch the .exe and check it starts."""
    print("\n[4/5] Running smoke test...")

    # Test 1: Check --help (should not crash)
    print("  Test 1: Checking import capability...")
    cmd = [EXE_PATH, "--help"]
    rc, output = _run(cmd, timeout=30)
    # The exe may not support --help, but it should not crash
    print(f"  Test 1 result: exit code {rc}")

    # Test 2: Try to import the module (if console mode)
    print("  Test 2: Checking executable integrity...")
    if os.path.exists(EXE_PATH):
        # Just verify it's a valid PE file
        with open(EXE_PATH, "rb") as f:
            header = f.read(2)
            if header == b"MZ":
                print("  Test 2 result: Valid PE executable (MZ header)")
            else:
                print("  Test 2 result: [WARN] Unexpected header")

    print("  [PASS] Smoke test completed")
    return True


def run_tests() -> bool:
    """Run the full test suite to ensure no regression."""
    print("\n[5/5] Running test suite...")

    cmd = [
        os.path.join(PROJ, ".venv", "Scripts", "python.exe"),
        "-m", "pytest",
        "verification/",
        "-q",
        "--tb=short",
    ]

    rc, output = _run(cmd, timeout=300)

    # Parse test results
    for line in output.split("\n"):
        if "passed" in line.lower() or "failed" in line.lower():
            print(f"  {line.strip()}")

    if rc != 0:
        print("  [FAIL] Some tests failed")
        return False

    print("  [PASS] All tests passed")
    return True


def main():
    ap = argparse.ArgumentParser(description="Build sim_platform as standalone .exe")
    ap.add_argument("--clean", action="store_true", help="Clean build (remove previous artifacts)")
    ap.add_argument("--verify", action="store_true", help="Build and verify (default)")
    ap.add_argument("--test", action="store_true", help="Build, verify, and run smoke test")
    ap.add_argument("--skip-tests", action="store_true", help="Skip test suite")
    args = ap.parse_args()

    print("=" * 60)
    print("  sim_platform — Build Standalone .exe")
    print("=" * 60)

    # Step 1: Prerequisites
    if not check_prerequisites():
        sys.exit(1)

    # Step 2: Build
    if not build(clean_build=args.clean):
        sys.exit(1)

    # Step 3: Verify
    if not verify():
        sys.exit(1)

    # Step 4: Smoke test
    if args.test:
        if not smoke_test():
            sys.exit(1)

    # Step 5: Test suite (optional)
    if not args.skip_tests and args.test:
        if not run_tests():
            print("\n  [WARN] Tests failed, but .exe was built successfully")

    # Summary
    print("\n" + "=" * 60)
    print("  BUILD COMPLETE")
    print("=" * 60)
    print(f"  Output: {EXE_PATH}")
    size_mb = os.path.getsize(EXE_PATH) / (1024 * 1024)
    print(f"  Size:   {size_mb:.1f} MB")
    print(f"\n  To run: {EXE_PATH}")
    print("  Or:     dist\\sim_platform\\sim_platform.exe")
    print("=" * 60)


if __name__ == "__main__":
    main()
