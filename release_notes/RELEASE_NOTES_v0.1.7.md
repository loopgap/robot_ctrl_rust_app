# v0.1.7

## Reissue Notice
- This tag was reissued to replace previously broken executables that exited immediately.
- Root cause: binary entrypoint was a placeholder main implementation and passed weak smoke checks.

## Fixes
- Restored functional GUI entrypoint in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs).
- Recovered and normalized [robot_control_rust/src/app.rs](robot_control_rust/src/app.rs) for a compilable runtime path.
- Updated ureq v3 API calls in runtime services to fix build/runtime compatibility.
- Implemented a complete top menu bar with actions in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs): File/Edit/View/Tools/Help/Language.
- Added language switching, preferences dialog, help dialogs, and keyboard shortcuts in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs).
- Added system CJK font fallback loading to improve Chinese text rendering consistency in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs).
- Replaced placeholder protocol analysis page with a functional filtered analyzer and CSV export in [robot_control_rust/src/views/protocol_analysis.rs](robot_control_rust/src/views/protocol_analysis.rs).
- Added reusable log CSV export API in [robot_control_rust/src/app.rs](robot_control_rust/src/app.rs) and connected it to menu actions.
- Cleaned mojibake comments in [robot_control_rust/src/services/llm_service.rs](robot_control_rust/src/services/llm_service.rs) and removed orphan non-UTF8 file [robot_control_rust/src/views/protocol_analysis_utf8.rs](robot_control_rust/src/views/protocol_analysis_utf8.rs).
- Hardened release smoke checks in [ .github/workflows/release.yml ](.github/workflows/release.yml):
- `plain_start` now requires process survival for a minimum runtime window.
- Arg-based smoke checks still run, and placeholder output is rejected.

## Local Asset Verification
- Local archive path: [release_notes/archive_assets/v0.1.7](release_notes/archive_assets/v0.1.7)
- SHA256:
- `9f896ff7670ecc24536e802c79b6b74b3833715f85d2927df11d6a5679d1d44e  robot_control_rust.exe`
- `ebdace9c168fd41f86e74904f56a384ef00e2e031254fd0a452e4434aceaf3c2  RobotControlSuite_Setup.exe`

## Verification
- [x] `cargo build --release --manifest-path robot_control_rust/Cargo.toml`
- [x] `cargo test --manifest-path robot_control_rust/Cargo.toml`
- [x] `cargo clippy --manifest-path robot_control_rust/Cargo.toml --all-targets -- -D warnings`
- [x] `./make.ps1 check`
- [x] Local smoke equivalent passed (`--version`, `--help`, `plain_start >= 2s`)
- [ ] Remote release workflow passed for reissued `v0.1.7`
- [ ] Remote assets verified (exe/setup/checksums)
