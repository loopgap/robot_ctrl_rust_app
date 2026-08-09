# v0.2.1

## Highlights
- Feature-gated dependency architecture: `rust_tools_suite` gains `gui`/`cli`/`jwt`/`full` interface features and `robot_control_rust` gains `hardware`/`llm`/`mcp` capability features, so lean builds no longer pull heavy optional stacks (aws-lc-rs, clap CLI stack) and every declared combination is compile-checked by a new CI feature-matrix gate (11 combinations).
- Unified error handling: new `AppError`/`AppResult` (robot_control_rust) and `ToolError`/`ToolResult` (rust_tools_suite) types replace stringly-typed errors across simulation validation, file operations, and tool status surfaces while keeping user-visible status text verbatim.
- UI/UX audit round: `canopen_view` received a full bilingual pass (~30 previously hard-coded labels) plus red-flag feedback with bilingual hover explanations on 10 numeric/hex input fields that previously fell back silently to defaults or dropped malformed bytes on send.
- Single-source release versioning: crate versions now inherit from `[workspace.package]` and the release tooling understands workspace inheritance, eliminating version-drift between crates.

## Fixes
- Restored the missing `at32_boot_entry` sources (protocol core + GUI tool) that were referenced by the workspace but never committed, which had left `develop` unbuildable from a fresh clone.
- Repaired i18n breaks: 13 orphaned translation keys were restored and wired into connections, dashboard, nn_tuning, and pid_control views; eight Chinese-only input placeholders in the tools suite are now localized.
- checksum tool: verify pass/fail state is now carried by a dedicated `verify_ok` flag instead of matching localized text, fixing English-mode rendering a passing verification in red and mislabeling the workflow step.
- Security: bumped `anyhow` to 1.0.104 (RUSTSEC-2026-0190) and `memmap2` to 0.9.11 (RUSTSEC-2026-0186); documented policy exceptions for the Linux-only transitive `quick-xml` advisories (RUSTSEC-2026-0194/0195) pinned by the egui 0.31 dependency tree.
- CI/tooling: workspace-guard required-file path corrected to `scripts/nsis/installer.nsi`; GitHub Actions upgraded off deprecated Node 20 runtimes (`checkout@v5`, `setup-go@v6`); `.gitattributes` added for deterministic line endings.

## Verification
- [x] ./scripts/windows/task.ps1 preflight
- [x] Local release artifact smoke checks completed
- [ ] GitHub release workflow verifies uploaded artifacts and checksums
