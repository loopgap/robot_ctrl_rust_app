#!/usr/bin/env bash
set -euo pipefail

echo "[Preflight] Start"

run_step() {
  local name="$1"
  shift
  echo "[Preflight] Running ${name}: $*"
  "$@"
}

run_step format cargo fmt --check
run_step build_debug cargo build
run_step test_debug cargo test
run_step test_release cargo test --release
run_step clippy cargo clippy --all-targets -- -D warnings
run_step build_release cargo build --release

release_bin="target/release/robot_control_rust"
if [[ -f "$release_bin" ]]; then
  if stat --version >/dev/null 2>&1; then
    size=$(stat -c%s "$release_bin")
  else
    size=$(stat -f%z "$release_bin")
  fi
  echo "[Preflight] Release binary: ${release_bin} (${size} bytes)"
fi

echo "[Preflight] All checks passed"
