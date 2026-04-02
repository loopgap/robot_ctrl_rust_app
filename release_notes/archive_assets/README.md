# archive_assets

This folder stores archived historical release binaries by tag version.

## Layout

- `vX.Y.Z/robot_control_rust.exe`
- `vX.Y.Z/rust_micro_tools.exe`
- `vX.Y.Z/RobotControlSuite_Setup.exe`
- `vX.Y.Z/checksums-sha256.txt`
- `vX.Y.Z/<optional source package or legacy assets>`

## Notes

- Current CI release workflow publishes runtime artifacts from root `release_artifacts/`.
- Current CI release workflow publishes 4 required assets from root `release_artifacts/`.
- Root `release_artifacts/` is treated as a temporary pipeline output path.
- This folder is for long-term repository archive and audit traceability.
