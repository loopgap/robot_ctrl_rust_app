# v0.1.7

## Reissue Notice
- This tag was reissued to replace previously broken executables that exited immediately.
- Root cause: binary entrypoint was a placeholder main implementation and passed weak smoke checks.
- This remains `v0.1.7`; it is a corrective reissue that replaces the earlier `v0.1.7` release rather than introducing a new version number.

## Fixes
- Restored functional GUI entrypoint in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs).
- Recovered and normalized [robot_control_rust/src/app.rs](robot_control_rust/src/app.rs) for a compilable runtime path.
- Updated ureq v3 API calls in runtime services to fix build/runtime compatibility.
- Implemented a complete top menu bar with actions in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs): File/Edit/View/Tools/Help/Language.
- Added language switching, preferences dialog, help dialogs, and keyboard shortcuts in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs).
- Added system CJK font fallback loading to improve Chinese text rendering consistency in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs).
- Replaced placeholder protocol analysis page with a functional filtered analyzer and CSV export in [robot_control_rust/src/views/protocol_analysis.rs](robot_control_rust/src/views/protocol_analysis.rs).
- Added reusable log CSV export API in [robot_control_rust/src/app.rs](robot_control_rust/src/app.rs) and connected it to menu actions.
- Integrated former indie tools (CSV Cleaner / JWT Inspector / Regex Workbench) into unified tool management in [rust_tools_suite/src/app.rs](rust_tools_suite/src/app.rs) and [rust_tools_suite/src/tools/mod.rs](rust_tools_suite/src/tools/mod.rs).
- Added persistent UI scale and terminal display preferences in [robot_control_rust/src/main.rs](robot_control_rust/src/main.rs), [robot_control_rust/src/app.rs](robot_control_rust/src/app.rs), and [robot_control_rust/src/i18n.rs](robot_control_rust/src/i18n.rs).
- Raised the default UI baseline to `150%`, changed scale application to deferred apply/reset behavior, and added `Ctrl + mouse wheel` zoom support in both desktop apps.
- Switched in-app help entrypoints to local UTF-8 HTML help so documentation is rendered by the system browser instead of the embedded text view.
- Expanded Windows CJK font fallback candidates in both apps to keep installed Chinese text rendering stable on common systems.
- Removed retired `rust_indie_tools` from the active workspace and cleanup policy after its retained capabilities were absorbed into `rust_tools_suite`.
- Extended release asset pipeline to include `rust_tools_suite.exe` in [.github/workflows/release.yml](.github/workflows/release.yml).
- Switched remote Release body source to [release_notes/RELEASE_NOTES_v0.1.7.md](release_notes/RELEASE_NOTES_v0.1.7.md) via workflow `body_path`.
- Cleaned mojibake comments in [robot_control_rust/src/services/llm_service.rs](robot_control_rust/src/services/llm_service.rs) and removed orphan non-UTF8 file [robot_control_rust/src/views/protocol_analysis_utf8.rs](robot_control_rust/src/views/protocol_analysis_utf8.rs).
- Hardened release smoke checks in [ .github/workflows/release.yml ](.github/workflows/release.yml):
- `plain_start` now requires process survival for a minimum runtime window.
- Arg-based smoke checks still run, and placeholder output is rejected.

## Release Asset Baseline
- Expected remote assets for this reroll:
- `robot_control_rust.exe`
- `rust_tools_suite.exe`
- `RobotControlSuite_Setup.exe`
- `checksums-sha256.txt`

## Verification
- [x] `cargo build --release --manifest-path robot_control_rust/Cargo.toml`
- [x] `cargo test --manifest-path robot_control_rust/Cargo.toml`
- [x] `cargo clippy --manifest-path robot_control_rust/Cargo.toml --all-targets -- -D warnings`
- [x] `cargo clippy --manifest-path rust_tools_suite/Cargo.toml --all-targets -- -D warnings`
- [x] `cargo test --manifest-path rust_tools_suite/Cargo.toml`
- [x] `./make.ps1 check`
- [x] Local smoke equivalent passed (`--version`, `--help`, `plain_start >= 2s`)
- [ ] Remote release workflow passed for reissued `v0.1.7`
- [ ] Remote assets verified (robot/suite/setup/checksums)
- [ ] Remote release body verified (matches this release note)
