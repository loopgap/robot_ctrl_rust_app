# rust_serial Workspace (Intelligent & High-Performance Edition)

> ⚠️ **重要规范提示**：本工作区的所有开发、功能拓展及维护必须严格遵守根目录下的 [**`route.md`**](route.md) 开发路线与规范文档。该文档定义了目录分类、验证规则、自动化流转及各子项目的关联标准，是项目工程化的“宪法”。

[![CI](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml)
[![Security Audit](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml)
[![Release](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml)

本仓库是一个统一的、**极致性能与高度智能化的** Rust 串行设备工作区，包含机器人控制主应用、统一入口的桌面工具套件，以及在线文档与发布自动化。

工作流的最高目标是：**统一覆盖、严格阻断、智能修复、极致性能**。

## 🌟 最新核心特性 (Features)

- **智能 Git 工作流**：`smart-bump.ps1` 支持 SemVer 升号、annotated tag 和发布说明草稿生成。
- **极致性能 UI**：使用纯原生的 `egui` (即时模式渲染硬件加速) 提供流畅的实时桌面体验，减少内存占用。
- **统一桌面工具套件**：`rust_tools_suite` 聚合 10 款高频工具，提供双语界面、响应式布局、文件导入导出与闭环流程面板。
- **跨平台融合体验 (C-FFI)**：控制引擎协议解析导出为 C 动态库 (`.dll`/`.so`)，可被 Python/C++ 直接调用。
- **零拷贝协议解包 (Zero-copy Pipeline)**：基于 `nom` 的高效封包解析，结合 `crossbeam` 构建低开销通信骨干。

## 子目录导航

- `robot_control_rust`
  主应用，覆盖工业控制、协议调试、可视化与联调能力。
  文档：[`robot_control_rust/README.md`](robot_control_rust/README.md)

- `rust_tools_suite`
  当前工作区唯一保留的聚合式桌面工具目录，统一提供 10 款高频工具、双语支持和响应式工作流。
  文档：[`rust_tools_suite/README.md`](rust_tools_suite/README.md)

- `docs`
  使用 `mdBook` 生成的在线交互式说明站点，覆盖安装、操作、发布与排障。
  另外保留 `docs/help/index.html` 作为桌面程序“文档”菜单优先打开的本地 HTML 帮助页；它不替代 mdBook 首页。

## 智能化开发工作流

### GitHub Actions

| 工作流 | 触发条件 | 智能特性概览 |
|--------|----------|------|
| **CI** | PR / push 到 `main`/`develop` | 格式、Clippy、测试、文档全量阻断（失败即终止，不自动回推） |
| **Security Audit** | 每周一 / 依赖变更 / 手动触发 | `cargo-audit` 与 `cargo-deny` 严格门禁 |
| **Release** | push tag `v*` | 校验 tag 策略后发布可用 Windows 资产（`robot_control_rust_windows_x64_portable.zip`、`rust_tools_suite_windows_x64_portable.zip`、`RobotControlSuite_Setup.exe`、`checksums-sha256.txt`），并同步 `release_notes/RELEASE_NOTES_vX.Y.Z.md` 到远端 Release 正文 |

### 本地终端与交互测试

```powershell
# Windows PowerShell
.\make.ps1 check
.\scripts\smart-bump.ps1 -Part patch

# 在确认无误后推送分支和 tag（将触发 Release 工作流）
.\scripts\smart-bump.ps1 -Part patch -Push

# 直接运行统一工具套件
cargo run --release --manifest-path rust_tools_suite/Cargo.toml
```

## 失败后的建议格式与智能修复

所有阻断型检查都应该输出以下五项内容：

- `问题摘要`
- `建议命令 / Auto-fix 执行指令`
- `修改方向`
- `如需继续排查先看哪里`
- `[如果命中了特征库] 直通智能排查文档 (mdBook) 的 URL 链接`

## Git Hooks

运行 `.\scripts\install-hooks.ps1` 安装本地钩子。钩子会在提交或推送前执行工作流校验和性能退化拦截。

## 发布流程

1. 在 `main/master` 分支完成并通过 `.\make.ps1 preflight`。
2. 执行 `.\scripts\smart-bump.ps1 -Part patch` 生成版本提交与 tag。
3. 推送分支与 tag，触发 Release 工作流。
4. 在 Release 页面验证四个必需资产：`robot_control_rust_windows_x64_portable.zip`、`rust_tools_suite_windows_x64_portable.zip`、`RobotControlSuite_Setup.exe`、`checksums-sha256.txt`。

发布失败可用以下命令回滚：

```powershell
.\scripts\smart-rollback.ps1 -Tag vX.Y.Z -DeleteRemoteTag -DeleteLocalTag -RevertLastCommit -PushRevert -NoVerify
```
