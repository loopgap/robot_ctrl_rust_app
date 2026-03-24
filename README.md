# rust_serial Workspace (Intelligent & High-Performance Edition)

> ⚠️ **重要规范提示**：本工作区的所有开发、功能拓展及维护必须严格遵守根目录下的 [**`route.md`**](route.md) 开发路线与规范文档。该文档定义了目录分类、验证规则、自动化流转及各子项目的关联标准，是项目工程化的“宪法”。

[![CI](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml)
[![Security Audit](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml)
[![Release](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml)

本仓库是一个统一的、**极致性能与高度智能化的** Rust 串行设备工作区，包含机器人控制主应用、统一入口的小工具合集，以及多个可独立交付的 GUI 工具。

工作流的最高目标是：**统一覆盖、严格阻断、智能修复、极致性能**。

## 🌟 最新核心特性 (Features)

- **智能 Git 工作流**：新增 `smart-bump.ps1` 支持 Semantic Versioning 自动升号与全量生成 Changelog，CI 携带自动修复 (Auto-fix) 和自动推回功能。
- **极致性能 UI**：弃用冗余的 Web GUI，使用纯原生的 `egui` (即时模式渲染硬件加速) 提供高达 144Hz 的无损实时波形渲染，极大减少内存占用。
- **智能排障与设备感知 CLI**：
  - 微型工具新增“交互式 TUI 命令接口 (`rust_micro_tools`)”，用户无需记忆繁杂命令即可一键补全、智能查找设备接口，体验炫酷的渐变彩色动画。
  - 内置 `doctor` 指令提供从驱动到网络的全栈排障指南。
- **跨平台融合体验 (C-FFI)**：
  - 控制引擎的协议解析不仅独立执行，更导出成 C 动态库 (`.dll`/`.so`)。直接被外围生态（如 Python/C++）以零成本对接调用。
- **零拷贝协议解包 (Zero-copy Pipeline)**：基于 `nom` 将封包解析与内存拷贝降解到 0 级别，搭配 `tokio`/`crossbeam` 搭建真正的无锁异步通信骨干网。

## 子目录导航

- `robot_control_rust`
  主应用，覆盖工业控制、协议调试、可视化与联调能力（支持零拷贝极速图表组件和 C-FFI 跨环境外接API）。
  文档：[`robot_control_rust/README.md`](robot_control_rust/README.md)

- `rust_micro_tools`
  包含基于 `inquire` 构建的智能 TUI / CLI 与硬件互动的合集，强调一致 UI、双语支持和闭环流程面板。
  文档：[`rust_micro_tools/README.md`](rust_micro_tools/README.md)

- `rust_indie_tools`
  独立 Rust GUI 工具目录，每个工具单独维护、单独跨平台 Matrix Action 自动构建打包。
  文档：[`rust_indie_tools/README.md`](rust_indie_tools/README.md)
  
- `docs`
  使用 `mdBook` 生成的在线交互式智能说明站点（包含所有项目的入门说明和 `doctor` 故障大全）。

## 智能化开发工作流

### GitHub Actions

| 工作流 | 触发条件 | 智能特性概览 |
|--------|----------|------|
| **CI** | PR / push 到 `main`/`develop` | 格式检查、Clippy 分析并具备**全自动修改 (Auto-fix) 并重定向推回**的能力 |
| **Security Audit** | 每周一 / 依赖变更 / 手动触发 | `cargo-audit` 与 `cargo-deny` 严格门禁 |
| **Release** | push tag `v*` | 全自动化跨平台打包为多个压缩文件并打入自动提取的 Changelog 信息 |

### 本地终端与交互测试

通过附带的 `mdBook`，你可以随时呼出本地知识库：

```powershell
# Windows PowerShell
.\make.ps1 check
.\scripts\smart-bump.ps1 -Project All # 智能跨越版本并撰写 README

# 直接调用交互式终端：
cd rust_micro_tools
cargo run -- connect
# [在未传入参数时, 提示你选择端口和波特率，展示 Spinner 动画提示]
```

## 失败后的建议格式与智能修复

所有阻断型检查都应该输出以下五项内容：

- `问题摘要`
- `建议命令 / Auto-fix 执行指令`
- `修改方向`
- `如需继续排查先看哪里`
- `[如果命中了特征库] 直通智能排查文档 (mdBook) 的 URL 链接`

## Git Hooks

运行 `.\scripts\install-hooks.ps1` 安装本地钩子。钩子会在提交或推送前执行工作流校验和性能退化拦截，让你不再“瞎猜”哪行代码引发了卡顿。
