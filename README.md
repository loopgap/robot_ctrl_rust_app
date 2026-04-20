# rust_serial Workspace (Intelligent & High-Performance Edition)

> ⚠️ **重要规范提示**：本工作区的所有开发、功能拓展及维护必须严格遵守根目录下的 [**`route.md`**](route.md) 开发路线与规范文档。该文档定义了目录分类、验证规则、自动化流转及各子项目的关联标准，是项目工程化的“宪法”。

[![CI](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml)
[![Security Audit](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml)
[![Release](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml)

本仓库是一个统一的、**极致性能与高度智能化的** Rust 串行设备工作区，包含机器人控制主应用、统一入口的桌面工具套件，以及在线文档与发布自动化。

工作流的最高目标是：**统一覆盖、严格阻断、智能修复、极致性能**。

## 🌟 最新核心特性 (Features)

- **深度的双系统支持 (Win8+ & Ubuntu 20+)**：在主应用与工具套件中引入定制的 Wayland/X11 后端自动降级策略，并在文件 I/O 与串口枚举底层构建带回退重试的智能防呆机制，彻底消除死锁。
- **无阻塞的并发通信基架 (Non-blocking I/O)**：通过 `ConnectionProvider` 统一收拢协议层，利用 `std::sync::mpsc` 与 `thread::spawn` 将计算与通信从 UI 渲染帧中完全剥离，界面响应刷新再创新高。
- **智能 Git 工作流**：`rusktask smart-bump` 支持 SemVer 升号、annotated tag 和发布说明草稿生成。
- **极致性能 UI**：使用纯原生的 `egui` (即时模式渲染硬件加速) 提供流畅的实时桌面体验，减少内存占用。
- **统一桌面工具套件**：`rust_tools_suite` 聚合 10 款高频工具，提供双语界面、响应式布局、文件导入导出与闭环流程面板。
- **跨平台融合体验 (C-FFI)**：控制引擎协议解析导出为 C 动态库 (`.dll`/`.so`)，可被 Python/C++ 直接调用。
- **零拷贝协议解包 (Zero-copy Pipeline)**：基于 `nom` 的高效封包解析，结合 `crossbeam` 构建低开销通信骨干。

## 子目录导航

- `crates/robot_control`
  机器人控制主应用源码（workspace crate）。

- `crates/tools_suite`
  聚合工具套件源码（workspace crate）。

- `crates/devtools`
  工作区验证与发布辅助工具（workspace crate）。

- `robot_control_rust`
  历史兼容目录，保留补充说明文档与迁移参考。

- `docs`
  使用 `mdBook` 生成的在线交互式说明站点，覆盖安装、操作、发布与排障。
  另外保留 `docs/help/index.html` 作为桌面程序“文档”菜单优先打开的本地 HTML 帮助页；它不替代 mdBook 首页。

## 智能化开发工作流

### GitHub Actions

| 工作流 | 触发条件 | 智能特性概览 |
|--------|----------|------|
| **CI** | PR / push 到 `main`/`develop` | Workspace 校验、格式、Clippy、测试、文档全量阻断（失败即终止，不自动回推） |
| **Security Audit** | 每周一 / 依赖变更 / 手动触发 | `cargo-audit` 与 `cargo-deny` 严格门禁 |
| **Release** | push tag `v*` | 校验 tag 策略后发布可用资产（`robot_control_suite_*_windows_x64-setup.exe`、`robot_control_suite_*_amd64.deb`、`checksums-sha256.txt`），并同步 `release_notes/RELEASE_NOTES_vX.Y.Z.md` 到远端 Release 正文 |

### 本地终端与交互测试

```powershell
# Windows PowerShell
.\scripts\task.ps1 check
.\scripts\task.ps1 smart-bump -BumpPart patch

# 在确认无误后推送分支和 tag（将触发 Release 工作流）
.\scripts\task.ps1 smart-bump -BumpPart patch -BumpPush

# 直接运行统一工具套件
cargo run --release -p tools_suite
```

## 失败后的建议格式与智能修复

所有阻断型检查都应该输出以下五项内容：

- `问题摘要`
- `建议命令 / Auto-fix 执行指令`
- `修改方向`
- `如需继续排查先看哪里`
- `[如果命中了特征库] 直通智能排查文档 (mdBook) 的 URL 链接`

## Git Hooks

运行 `.\scripts\task.ps1 go-install-hooks` 安装本地钩子。卸载可执行 `cd scripts/go/rusktask; go run . install-hooks --uninstall`。

## 发布流程

1. 在 `main` 分支完成并通过 `.\scripts\task.ps1 preflight`。
2. 执行 `.\scripts\task.ps1 smart-bump -BumpPart patch` 生成版本提交与 tag。
3. 推送分支与 tag，触发 Release 工作流。
4. 在 Release 页面验证必需资产：`robot_control_suite_*_windows_x64-setup.exe`、`robot_control_suite_*_amd64.deb`、`checksums-sha256.txt`。

## Release v0.2.1 验证清单

当发布标签为 `v0.2.1` 时，至少完成以下核验：

1. 校验 Tag 归属：`v0.2.1` 必须可追溯到 `origin/main`。
2. 校验 Release Notes：存在并通过 `release_notes/RELEASE_NOTES_v0.2.1.md` 结构校验。
3. 校验资产完整性：
  - `robot_control_suite_*_windows_x64-setup.exe`
  - `robot_control_suite_*_amd64.deb`
  - `checksums-sha256.txt`
4. 下载并对比 SHA256，确保与 `checksums-sha256.txt` 一致。

发布失败可用以下命令回滚：

```powershell
.\scripts\task.ps1 smart-rollback -RollbackTag vX.Y.Z -RollbackDeleteRemoteTag -RollbackDeleteLocalTag -RollbackRevertLastCommit -RollbackPushRevert -RollbackNoVerify
```
