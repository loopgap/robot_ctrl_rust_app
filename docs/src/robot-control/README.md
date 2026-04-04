# 机器人主控应用 - 概述

> 纯 Rust 实现的工业级机器人控制与多协议调试平台

## 项目简介

`robot_control_rust` 是工作区的核心应用，提供：

- **多协议通信支持**: 覆盖工业现场常见的 Serial / TCP / UDP / CAN / USB / Modbus 等协议
- **控制算法库**: 10+ 种控制算法，从经典 PID 到高级 MPC
- **协议分析工具**: 深度帧解析、事务分析、KPI 仪表盘
- **智能调参**: 神经网络 + LLM API 双重辅助
- **跨平台 GUI**: 基于 egui 的高性能原生界面

## 目录结构

```
robot_control_rust/
├── src/
│   ├── main.rs              # 应用入口、主题配置、路由
│   ├── app.rs               # 状态中心 (AppState / UiState)
│   ├── i18n.rs              # 国际化 (zh-CN / en-US)
│   ├── models/              # 协议模型与算法实现
│   │   ├── control_algorithm.rs   # 算法枚举与统一接口
│   │   ├── connection.rs          # 连接类型与配置
│   │   ├── canopen.rs            # CANopen 协议
│   │   ├── modbus.rs             # Modbus 协议
│   │   ├── packet.rs             # 报文模板与解析
│   │   └── ...                   # 其他数据模型
│   ├── services/             # 服务层
│   │   ├── serial_service.rs     # 串口服务
│   │   ├── tcp_service.rs       # TCP 服务
│   │   ├── udp_service.rs       # UDP 服务
│   │   ├── can_service.rs       # CAN/CAN FD 服务
│   │   ├── llm_service.rs       # LLM API 调用
│   │   └── mcp_server.rs        # MCP Server
│   └── views/                # UI 页面
│       ├── dashboard.rs          # 仪表盘
│       ├── connections.rs        # 连接管理
│       ├── protocol_analysis.rs  # 协议分析
│       └── ...                   # 其他页面
└── scripts/                  # 构建与打包脚本
```

## 技术栈

| 组件 | 版本 | 用途 |
|------|------|------|
| Rust | 2021 Edition | 开发语言 |
| eframe / egui | 0.31 | GUI 框架 |
| serialport | 4.8 | 串口通信 |
| ureq | 3.x | HTTP / LLM API 调用 |
| serde | 1.x | 数据序列化 |
| tracing | 日志 | 追踪日志 |

## 核心特性

### 多协议通信

| 协议 | 支持类型 | 说明 |
|------|----------|------|
| Serial | RS232/RS485/RS422 | 高速串口，波特率最高 3Mbps |
| TCP | Client/Server | 主动连接或被动监听 |
| UDP | Unicast/Broadcast | 轻量级无连接 |
| CAN | CAN 2.0 | 标准帧/扩展帧，11-bit/29-bit ID |
| CAN FD | CAN FD | 更快速率，更大 payload (64B) |
| USB | 12 类协议 | CDC/HID/MSC/Audio/Video 等 |
| Modbus RTU | Master/Slave | RS485 接口 |
| Modbus TCP | Master/Slave | Ethernet 接口 |

### 控制算法

| 算法 | 类型 | 适用场景 |
|------|------|----------|
| 经典 PID | 位置式 | 通用场景 |
| 增量 PID | 增量式 | 需要 bumpless 切换 |
| Bang-Bang | 开关控制 | 温度控制、简单逻辑 |
| 模糊 PID | 自适应 | 非线性、时变系统 |
| 串级 PID | 双闭环 | 电机控制、过程控制 |
| Smith 预估 | 时滞补偿 | 传输延迟系统 |
| ADRC | 自抗扰 | 高性能需求 |
| LADRC | 线性自抗扰 | 需要参数整定简单 |
| LQR | 最优控制 | 状态反馈 |
| MPC | 预测控制 | 多约束优化 |

## 页面导航

### 主要页面

| 页面 | 说明 |
|------|------|
| [Dashboard](dashboard.md) | 系统状态总览、快捷操作入口 |
| [Connections](connections.md) | 8 种连接类型的配置详解 |
| [Serial Debug](serial_debug.md) | 串口数据收发与调试 |
| [Protocol Analysis](protocol.md) | 帧解析、事务分析与 KPI |
| [PID Control](pid_control.md) | 控制算法选择与参数配置 |

### 专业工具

| 页面 | 说明 |
|------|------|
| [CANopen Tools](canopen.md) | CANopen 协议三协议统一工具 |
| [Modbus Tools](modbus.md) | Modbus RTU/TCP 双协议支持 |
| [Packet Builder](packet_builder.md) | 报文模板构建与解析 |
| [Data Viz](data_viz.md) | 实时数据可视化 |
| [NN Tuning](nn_tuning.md) | 神经网络辅助调参 |
| [Topology](topology.md) | 机器人拓扑结构可视化 |

### 服务接口

| 页面 | 说明 |
|------|------|
| [MCP Server](mcp.md) | JSON-RPC 2.0 外部接口 |
| [控制算法](algorithms.md) | 10+ 种算法原理与调参指南 |

## 快速开始

### 构建

```powershell
# Debug 构建
cargo build

# Release 构建 (推荐)
cargo build --release
```

### 运行

```powershell
# Debug 运行
cargo run

# Release 运行
cargo run --release
```

### 测试

```powershell
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --lib models::pid_controller

# 带日志运行
cargo test -- --nocapture
```

## 相关文档

- [架构设计](../robot_control_rust/ARCHITECTURE_AND_USAGE.md) - 内部架构详解
- [安装指南](../installation.md) - 环境安装
- [配置参考](../configuration.md) - 应用配置
- [贡献指南](../contributing.md) - 开发规范
