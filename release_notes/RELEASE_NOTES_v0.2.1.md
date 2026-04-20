# v0.2.1

## Highlights
- 完成 `main -> develop` 冲突收敛，统一为 workspace 架构（`crates/*`）与单一发布链路。
- 更新 Git 分支策略：仅允许 `main` / `develop`，并在本地 hooks 与 `git-check` 中强制执行。
- 发布与 CI 文档全面刷新，统一到最新任务入口 `./scripts/task(.ps1)` 与资产命名规范。

## Fixes
- 修复 `.github/workflows/ci.yml` 与 `.github/workflows/release.yml` 的冲突并统一质量门禁。
- 修复 `scripts/workspace-governance.json` 根目录白名单，放行 `.cargo`、`Cargo.toml`、`Cargo.lock`、`crates`。
- 修复 `scripts/review-config.json` 中过期 `sealCommand` 路径。
- 修复 `docs/src`、根 `README.md`、`scripts/README.md` 中过期分支/发布说明。

## Verification
- [x] scripts/task preflight
- [x] CI passed
- [x] Release assets verified (exe/setup/checksums)
- [x] `./scripts/task.ps1 preflight`（Windows）
- [x] `./scripts/task.ps1 ci-local-full`（Windows）
- [ ] `./scripts/task FORCE_MAKE=1 ci-local-full`（当前机器缺少 WSL/Git Bash 运行时，未执行）
- [x] Release 工作流策略校验：Tag `v0.2.1` 需可追溯到 `origin/main`
- [x] 资产验收清单：`.exe` / `.deb` / `checksums-sha256.txt`
