# v0.1.9

## Highlights
- 收敛 `v0.1.9` 修复线的版本、发布说明、依赖审计和发布资产治理。
- 将 MCP server info 版本改为跟随 crate package version，避免服务版本与发布版本漂移。
- 强化 release-sync 远端 tag 检测，避免本地旧 tag 与远端 tag 不一致时阻断远端状态读取。
- 加固 GitHub Release workflow，使手动发布、tag 校验、Windows 打包和资产校验与本地流程保持一致。

## Fixes
- 补齐 `v0.2.0` 历史 release notes，消除 release-sync/workflow-seal orphan tag 阻断。
- 更新低/中风险依赖并保留 `eframe`/`egui` GUI 大版本到后续版本处理。
- 为 `cargo-deny` 增加明确 license allowlist，使完整依赖策略检查可作为发布门禁。
- 统一发布文档中的 release notes 命令、资产命名和 `v0.1.9` 验收清单。
- 修复 Release workflow 的 tag 解析、失败 draft job 依赖、Windows 安装器打包路径和 Debian control 元数据。

## Verification
- [x] ./scripts/windows/task.ps1 preflight
- [x] CI passed
- [x] Release assets verified (exe/setup/checksums)

## Notes
- `eframe` / `egui` / `egui_plot` 大版本升级留到后续 minor 版本，以降低修复版 GUI 回归风险。
