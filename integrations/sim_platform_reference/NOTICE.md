# sim_platform Reference Snapshot

This directory contains a filtered reference snapshot from `D:\Destop\test_ui\sim_platform` used to port the simulation platform into the Rust-native `robot_control_rust` app for v0.2.0.

Snapshot policy:
- Source state: current `sim_platform` working tree at integration time, not only the last committed `master` revision.
- Included: Python source, configs, docs, examples, tests, project metadata, and lock files.
- Excluded: `.git`, `.venv`, caches, `__pycache__`, `build`, `dist`, bytecode, and generated bitmap artifacts.

License metadata:
- Upstream project metadata declares MIT license text in `pyproject.toml`.
- This GPL-3.0-only workspace uses the snapshot as reference material for a Rust-native implementation; it is not shipped as a Python runtime sidecar in v0.2.0.

Verification baseline:
- `pytest --collect-only -q verification` collected 1128 tests from the source worktree on 2026-06-07.
