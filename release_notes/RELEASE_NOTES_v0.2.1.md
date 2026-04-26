# v0.2.1

## Highlights
- 完成 `main -> develop` 冲突收敛，统一为 workspace 架构（`crates/*`）与单一发布链路。
- 更新 Git 分支策略：仅允许 `main` / `develop`，并在本地 hooks 与 `git-check` 中强制执行。
- 发布与 CI 文档全面刷新，统一到最新任务入口 `./scripts/task(.ps1)` 与资产命名规范。
- 新增 **Release 工件规范** (`docs/RELEASE_SPEC.md`)，确立标准化工件格式与命名。
- 新增 **Windows 便携版 ZIP** 工件 (`robot_control_suite_{VERSION}_windows_x64_portable.zip`)。

## Fixes
- 修复 `.github/workflows/ci.yml` 与 `.github/workflows/release.yml` 的冲突并统一质量门禁。
- 修复 `scripts/workspace-governance.json` 根目录白名单，放行 `.cargo`、`Cargo.toml`、`Cargo.lock`、`crates`。
- 修复 `scripts/review-config.json` 中过期 `sealCommand` 路径。
- 修复 `docs/src`、根 `README.md`、`scripts/README.md` 中过期分支/发布说明。
- 移除 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` 废弃环境变量（修复 Node.js 20 弃用警告）。
- 清理未使用依赖：`nom`、`crossbeam-channel`、`thiserror`、`tracing-subscriber`、`tracing-appender`。
- 忽略 `RUSTSEC-2026-0104` (rustls-webpki) 传递依赖漏洞（通过 `ureq->rustls` 引入，无法直接升级）。

## Verification
- [x] ./scripts/ubuntu/task.sh preflight
- [x] ./scripts/ubuntu/task.sh ci-local-full
- [x] CI passed
- [x] Security audit passed
- [x] Release assets verified (exe/setup/checksums)
- [x] Release workflow policy validation

## Artifacts
| Platform | Type | Filename |
|----------|------|----------|
| Windows | NSIS Installer | `robot_control_suite_{VERSION}_windows_x64-setup.exe` |
| Windows | Portable ZIP | `robot_control_suite_{VERSION}_windows_x64_portable.zip` |
| Linux | DEB Package | `robot_control_suite_{VERSION}_amd64.deb` |
| All | Checksums | `checksums-sha256.txt` |
