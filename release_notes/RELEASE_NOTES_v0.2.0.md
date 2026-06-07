# v0.2.0

## Highlights
- Integrated a filtered `sim_platform` reference snapshot under `integrations/sim_platform_reference/` with documented MIT metadata, source baseline, and exclusion policy.
- Added a Rust-native Simulation Lab in `robot_control_rust` with guarded numeric configuration, PMSM/BLDC/IM motor models, FOC/speed/MPC/EKF controllers, thermal and sensor models, background execution, cancellation, parameter scan, and JSON/CSV export.
- Replaced runtime emoji icon strings with painter-rendered `IconKind` icons and refreshed packaged Linux SVG icons.

## Fixes
- Updated release ancestry validation and `smart-bump` branch checks to require `develop` instead of `main`.
- Hardened release notes validation so local notes do not require fake CI or asset-verification claims before the release workflow performs those checks.
- Preserved the v0.1.9 Windows installer path rendering fix and added regression coverage for native NSIS file paths.

## Verification
- [x] ./scripts/windows/task.ps1 preflight
- [x] Local release artifact smoke checks completed
- [ ] GitHub release workflow verifies uploaded artifacts and checksums
