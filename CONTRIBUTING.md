# Development Standards & Governance

> **Single source of truth** for all contributors and automated tooling.
> Every rule below is enforced by a machine-executable check (pre-commit hook, CI job, or clippy lint).
> If a rule cannot be enforced automatically, it is marked **\[manual\]**.

---

## 1. Architecture Principles

| Principle | Enforcement |
|---|---|
| Workspace members share dependencies via `[workspace.dependencies]` | `workspace-governance.json` required-files + `cargo check` |
| Crate versions inherit from `[workspace.package]` | `rusktask resolveReleaseVersion` understands `version.workspace = true` |
| Feature gates: `gui`/`cli`/`jwt`/`full` (tools), `hardware`/`llm`/`mcp` (robot) | CI feature-matrix job (11 combinations) |
| `compile_error!` when no interface feature is enabled | `cargo check` with no features |

## 2. Error Handling

| Rule | Enforcement |
|---|---|
| Robot app: use `AppError`/`AppResult` from `robot_control_rust::error` | Clippy `pedantic` + **\[manual\]** review |
| Tools suite: use `ToolError`/`ToolResult` from `rust_tools_suite::error` | Clippy `pedantic` + **\[manual\]** review |
| No `Result<_, String>` in new public API functions | **\[manual\]** PR checklist |
| UI-facing error text: strip `AppError::Display` prefix via `ui_err_text()` | **\[manual\]** PR checklist |
| Status messages: use `status_message` + `last_error_time` burst-limiter | **\[manual\]** review |

## 3. Internationalization (i18n)

| Rule | Enforcement |
|---|---|
| Robot app: new UI strings → `Tr::key_name(lang)` | **\[manual\]** PR checklist |
| Tools suite: new UI strings → `lang.tr("中文", "English")` | **\[manual\]** PR checklist |
| No raw Chinese in `Language::En` match arms | **\[manual\]** review |
| No raw English in `Language::Zh` match arms | **\[manual\]** review |
| `hint_text` and `on_hover_text` are bilingual | **\[manual\]** PR checklist |
| Technical terms identical in both languages (RPM, PID, CAN, etc.) may stay untranslated | N/A |

### i18n arg order

- **robot_control_rust**: `Tr::key(lang)` — key defined via `tr!(key, "English", "中文")`
- **rust_tools_suite**: `lang.tr("中文", "English")` — Chinese first, English second

## 4. Input Validation UX

| Rule | Enforcement |
|---|---|
| Numeric/hex inputs with silent fallback → red `flagged_edit()` feedback | **\[manual\]** PR checklist |
| `on_hover_text` explains the fallback behavior in both languages | **\[manual\]** PR checklist |
| Range constraints (e.g., Node ID 1–127) validated before send | **\[manual\]** review |

## 5. Code Quality

| Rule | Enforcement |
|---|---|
| `cargo fmt --all -- --check` | Pre-commit + CI |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pre-commit + CI |
| Workspace clippy lints in `[workspace.lints.clippy]` | Clippy (inherited by all crates) |
| `cargo test --workspace` — all tests pass | Pre-commit + CI |
| Feature matrix: 11 combinations compile | CI feature-matrix job |
| Cognitive complexity ≤ 30 per function | `clippy.toml` |
| Function body ≤ 200 lines | `clippy.toml` |
| Max 8 function arguments | `clippy.toml` |

### Clippy lint policy

- **`pedantic` + `nursery`**: enabled as `warn` (CI `-D warnings` promotes to deny)
- Noisy sub-lints explicitly `allow`ed in `[workspace.lints.clippy]` with justification comments
- New code should NOT introduce violations of the `warn`-level lints
- **\[manual\]**: reviewers should check that new code doesn't add `allow` suppressions without justification

## 6. Security & Dependencies

| Rule | Enforcement |
|---|---|
| `cargo audit` — zero errors | CI Security Audit job |
| `cargo deny check` — zero errors | CI Security Audit job |
| Advisory exceptions in `.cargo/audit.toml` AND `deny.toml` must match | Pre-commit sync check |
| Every exception needs a justification comment | **\[manual\]** review |
| TLS verification enabled (`http.sslverify = true`) | `git config --local` |
| No secrets in git history | **\[manual\]** `.gitignore` covers `.env`, `.env.*`, `.env.local` |

## 7. Workspace Hygiene

| Rule | Enforcement |
|---|---|
| Root directory entries must be in `workspace-governance.json` allowlist | Pre-commit + CI workspace-guard |
| No `release_artifacts/`, `smoke_logs/`, `logs/`, `tmp/`, `temp/` committed | workspace-guard blocked paths |
| `.gitattributes` defines line endings for all file types | Git (LF canonical, CRLF for .ps1/.bat) |
| Build artifacts (`*.exe`, `*.pdb`, `*.rlib`) not committed | `.gitignore` |

## 8. Git & Release Workflow

| Rule | Enforcement |
|---|---|
| Conventional Commits: `feat/fix/refactor/chore/ci(scope): description` | **\[manual\]** PR review |
| `develop` is the integration branch; `main` is the release branch | Branch protection |
| Tags must be on `origin/develop` (or ancestor of it) | CI `release-validate` job |
| Release notes: `release_notes/RELEASE_NOTES_vX.Y.Z.md` must exist and validate | CI `release-validate` job |
| `RELEASE_INDEX.md` updated via `rusktask update-release-index` | **\[manual\]** pre-release |
| `rusktask smart-bump` only edits root `Cargo.toml` (skips `version.workspace = true`) | Go tool logic |

### Commit message format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `refactor`, `chore`, `ci`, `docs`, `test`
Scopes: `arch`, `error`, `i18n`, `ux`, `tools-ux`, `release`, `deps`, `git`

## 9. CI Pipeline

| Job | Gate | Trigger |
|---|---|---|
| Workspace Governance | Root entries, required files, workspace members | push + PR |
| Format Check | `cargo fmt --all -- --check` | push + PR |
| Clippy (3 OS) | `cargo clippy --workspace --all-targets -- -D warnings` | push + PR |
| Test (3 OS) | `cargo test --workspace` | push + PR |
| Feature Matrix | 11 feature combinations compile | push + PR |
| Docs | `cargo doc --workspace --no-deps` | push + PR |
| Release Smoke | Release build smoke check | push + PR |
| Security Audit | `cargo audit` + `cargo deny check` | push + PR (on Cargo.* changes) |
| Release Validate | Tag format, branch ancestry, quality gate | tag push |
| Release Build | 3 platform builds + publish | tag push |

## 10. Pre-commit Hook

The `.githooks/pre-commit` hook runs 6 checks:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. Workspace structure guard (via rusktask)
5. Advisory config sync (audit.toml ↔ deny.toml)
6. Feature matrix spot-check (fast compilation)

Install: `git config core.hookspath .githooks`

---

## Quick Reference

```bash
# Full preflight (what CI runs locally)
./scripts/windows/task.ps1 preflight

# Just the gates
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace

# Feature matrix
for f in "" hardware llm mcp "hardware,llm" "hardware,mcp"; do
  cargo check -p robot_control_rust --no-default-features ${f:+--features "$f"}
done
for f in gui cli "gui,cli" "gui,jwt" full; do
  cargo check -p rust_tools_suite --no-default-features --features "$f"
done

# Security
cargo audit && cargo deny check

# Release notes validation
cd scripts/go/rusktask && go run . release-notes validate --file ../../release_notes/RELEASE_NOTES_v0.2.1.md --mode release
```
