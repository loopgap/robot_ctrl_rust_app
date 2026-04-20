# rusktask

Transitional Go task orchestrator for workspace automation standardization.

## Quick Start

```bash
go run . version
go run . release-notes validate --file ../../release_notes/RELEASE_NOTES_v0.1.8.md --mode draft
go run . check
go run . preflight
```

## Current Scope

- `fmt --check`: formatting check for all workspace projects.
- `clippy`: lint check with warnings denied.
- `test [--release]`: test workspace projects.
- `build [--release]`: build workspace projects.
- `doc`: rustdoc build with warnings denied.
- `audit [--ignore <ID>]`: cargo-audit + cargo-deny checks.
- `check`: fmt --check + clippy + test.
- `preflight`: workspace cleanup + guard + check + release test/build + doc.
- `git-check [--pre-push] [--commit-msg-file <path>]`: git workflow guard (strictly allows only `main`/`develop`, commit message format, staged-file policy, remote sync in pre-push mode).
- `rust-review [--fix] [--skip-tests] [--skip-audit] [--project <name>]...`: Rust review pipeline per selected project(s).
- `review [--quick] [--fix] [--before-push] [--skip-tests] [--skip-audit] [--project <name>]...`: combined review orchestration.
- `install-hooks [--uninstall] [--force]`: install/uninstall managed Git hooks that call rusktask directly.
- `smart-bump [--part patch|minor|major] [--push] [--no-verify] [--allow-dirty] [--no-tag] [--skip-release-state-audit] [--skip-process-cleanup] [--skip-workspace-guard]`: semantic version bump + release notes draft + commit/tag/push workflow.
- `smart-rollback --tag <vX.Y.Z> [--owner <owner>] [--repo <repo>] [--delete-release] [--delete-remote-tag] [--delete-local-tag] [--revert-last-commit] [--push-revert] [--no-verify] [--skip-process-cleanup] [--skip-workspace-guard] [--skip-index-refresh]`: release rollback workflow.
- `pr-helper [--check|--create|--merge] [--draft] [--title <text>] [--body <text>] [--base <branch>] [--head <branch>] [--auto-fill]`: pull request readiness/create/merge helper.
- `docs-bundle [--output-root <dir>] [--create-zip]`: build mdBook + help bundle for offline distribution.
- `release-publish [--owner <owner>] [--repo <repo>] --tag <tag> [--release-name <name>] [--body-file <path>] [--asset <path>]... [--prerelease] [--draft] [--prune-extra-assets]`: create/update GitHub release and upload assets.
- `build-release-slim [--skip-tests] [--skip-clippy]`: run slim release pipeline for workspace crate `robot_control` and trim transient target directories.
- `release-sync [--mode audit|apply] [--prune-local-tags-not-on-remote] [--clean-orphan-notes] [--skip-remote] [--strict]`: release tags/notes state audit and normalization.
- `workflow-seal [--mode audit|apply] [--prune-local-tags-not-on-remote] [--clean-orphan-notes] [--skip-remote]`: one-command workflow seal pipeline.
- `update-release-index [--skip-remote]`: rebuild release_notes/RELEASE_INDEX.md.
- `package-windows-installer [--version <X.Y.Z>] [--build-tag <yyyymmdd>] [--prefer-iexpress] [--skip-build]`: native Windows installer packaging flow (ISCC with iExpress fallback).
- `package-windows-assets [--version <X.Y.Z>] [--output-dir <dir>] [--skip-build]`: native Windows portable assets packaging flow.
- `package-windows-portable-installer [--version <X.Y.Z>] [--output-dir <dir>] [--skip-build]`: build portable installer bundle zip with install/uninstall scripts.
- `workspace-cleanup [--mode audit|apply] [--strict]`: workspace transient artifact governance.
- `workspace-guard [--mode audit|apply] [--strict] [--use-staged-paths]`: workspace structure/path policy guard.
- `release-notes validate`: release notes structure and readiness validation.

More release and rollback commands will be migrated from PowerShell and Make targets in later phases.
