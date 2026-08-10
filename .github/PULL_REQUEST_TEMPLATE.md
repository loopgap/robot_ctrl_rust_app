## What changed


## Why


## How was this tested
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` — all 891+ tests pass
- [ ] Feature matrix: `for f in "" hardware llm mcp "hardware,llm" "hardware,mcp"; do cargo check -p robot_control_rust --no-default-features ${f:+--features "$f"}; done`
- [ ] Feature matrix: `for f in gui cli "gui,cli" "gui,jwt" full; do cargo check -p rust_tools_suite --no-default-features --features "$f"; done`

## i18n checklist (if touching UI/views)
- [ ] New user-visible strings go through `Tr::key(lang)` (robot) or `lang.tr("中文","English")` (tools)
- [ ] No raw Chinese in `Language::En` arms, no raw English in `Language::Zh` arms
- [ ] hint_text / on_hover_text are bilingual

## Error handling checklist (if touching error types)
- [ ] New errors use `AppError`/`AppResult` (robot) or `ToolError`/`ToolResult` (tools), not `String`
- [ ] UI-facing error text stripped of Display prefix if needed (`ui_err_text()`)

## Release checklist (if bumping version)
- [ ] `scripts/workspace-governance.json` required-files list is current
- [ ] `release_notes/RELEASE_NOTES_vX.Y.Z.md` exists and validates via `rusktask release-notes validate`
- [ ] `release_notes/RELEASE_INDEX.md` updated via `rusktask update-release-index`
