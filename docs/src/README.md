# Rust Serial 工作区智能手册

> 极致性能 · 高度智能化 · 统一工作区

[![CI](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/ci.yml)
[![Security Audit](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/audit.yml)
[![Release](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml/badge.svg)](https://github.com/loopgap/robot_ctrl_rust_app/actions/workflows/release.yml)

## 项目简介

Rust Serial 工作区是一个统一的、**极致性能与高度智能化**的 Rust 串行设备工作区，包含：

- **机器人控制主应用** (`robot_control_rust`) - 工业控制、协议调试、可视化与联调能力
- **微型工具集** (`rust_micro_tools`) - 智能 TUI/CLI 与硬件互动的统一入口
- **独立图形工具** (`rust_indie_tools`) - 可独立交付的 GUI 小工具

## 核心特性

### 🚀 极致性能 UI

使用纯原生的 `egui` (即时模式渲染硬件加速)，提供高达 **144Hz** 的无损实时波形渲染，极大减少内存占用。

### 🔧 多协议支持

| 协议类型 | 支持明细 |
|---------|---------|
| 串行通信 | Serial (RS232/RS485/RS422) |
| 网络协议 | TCP / UDP |
| CAN 总线 | CAN 2.0 / CAN FD / EtherCAT CoE |
| 工业协议 | Modbus RTU / Modbus TCP |
| USB | CDC / HID / MSC / Audio / Video |

### 🧠 智能控制算法

- **经典控制**: PID、增量 PID、Bang-Bang、串级 PID
- **高级控制**: Smith 预测器、ADRC、LADRC、LQR、MPC
- **智能调参**: 模糊 PID、神经网络建议、外部 LLM API 建议

### 🔄 智能 Git 工作流

- `smart-bump.ps1` 支持 Semantic Versioning 自动升号与全量生成 Changelog
- CI 携带自动修复 (Auto-fix) 和自动推回功能
- 本地 Git Hooks 拦截性能退化

### 🌐 零拷贝协议解包

基于 `nom` 将封包解析与内存拷贝降解到 **0 级别**，搭配 `tokio`/`crossbeam` 搭建真正的无锁异步通信骨干网。

### 🔌 跨平台融合 (C-FFI)

控制引擎的协议解析独立执行，导出成 C 动态库 (`.dll`/`.so`)，可直接被 Python/C++ 以零成本对接调用。

## 目录结构

```
rust_serial/
├── robot_control_rust/     # 机器人控制主应用
│   ├── src/
│   │   ├── models/         # 协议模型、算法模型
│   │   ├── services/       # 串口/TCP/UDP/CAN/LLM/MCP 服务
│   │   └── views/          # UI 页面组件
│   └── scripts/            # 打包与预检脚本
├── rust_micro_tools/       # 微型工具集 (TUI/CLI)
│   └── src/tools/          # 7 款工具实现
├── rust_indie_tools/       # 独立图形工具
│   ├── csv_cleaner_gui/    # CSV 清洗工坊
│   ├── jwt_inspector_gui/  # JWT 解析工坊
│   └── regex_workbench_gui/# Regex 巡检工坊
├── docs/                   # mdBook 文档
└── scripts/                # 开发脚本与 Git Hooks
```

## 快速导航

- [快速入门](getting-started.md) - 环境准备与首次运行
- [机器人主控](robot-control/README.md) - 主应用完整功能指南
- [微型工具集](micro-tools/README.md) - TUI 工具使用手册
- [独立图形工具](indie-tools/README.md) - 独立 GUI 工具介绍
- [开发与工作流](workflow.md) - Git 工作流与自动化
- [智能排障](troubleshooting.md) - 常见问题与解决方案

## 智能化开发工作流

### CI/CD 流水线

| 工作流 | 触发条件 | 核心能力 |
|--------|----------|----------|
| **CI** | PR / push 到 main/develop | 格式检查、Clippy 分析、Auto-fix 并推回 |
| **Security Audit** | 每周一 / 依赖变更 | cargo-audit 与 cargo-deny 门禁 |
| **Release** | push tag v* | 全自动化跨平台打包与 Changelog 生成 |

### 本地开发

```powershell
# 格式检查
cargo fmt --check

# Clippy 分析
cargo clippy --all-targets

# 运行测试
cargo test

# 一键预检 (Windows)
.\scripts\preflight.ps1
```

## 失败处理规范

所有阻断型检查都会输出标准化的错误报告：

- **问题摘要** - 简短描述当前错误
- **建议命令** - Auto-fix 执行指令
- **修改方向** - 修复思路指引
- **排查位置** - 下一步排查建议
- **文档链接** - 直通 mdBook 智能排查文档

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../LICENSE) 文件。