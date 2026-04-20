# Robot Control Rust

> 迁移说明：运行时代码已迁移到 workspace crate `crates/robot_control`。
> 当前目录保留文档与历史兼容说明，构建/运行请在仓库根目录执行 `cargo run -p robot_control`。

> 纯 Rust 实现的工业级机器人控制与多协议调试平台
>
> Pure-Rust industrial robot control & multi-protocol debugging platform

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/egui-0.31-blue.svg)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

---

## 目录

- [功能概览](#功能概览)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [快速开始](#快速开始)
- [页面功能](#页面功能)
  - [Dashboard](#dashboard)
  - [Connections](#connections)
  - [Serial Debug](#serial-debug)
  - [Packet Builder / Parser](#packet-builder--parser)
  - [Topology](#topology)
  - [Control Algorithms](#control-algorithms)
  - [NN Tuning](#nn-tuning)
  - [Data Viz](#data-viz)
  - [Protocol Analysis](#protocol-analysis)
  - [CANopen Tools](#canopen-tools)
  - [Modbus Tools](#modbus-tools)
  - [MCP Server](#mcp-server)
- [控制算法详解](#控制算法详解)
- [底盘运动学](#底盘运动学)
- [国际化](#国际化)
- [UI/UX 体验](#uiux-体验)
- [商用特性](#商用特性)
- [跨平台部署](#跨平台部署)
- [测试](#测试)
- [验收清单](#验收清单)
- [架构详情](#架构详情)

---

## 功能概览

| 类别 | 功能 |
|------|------|
| **核心架构** | 深度的双系统支持 (Win8+ & Ubuntu 20+)；基于 `ConnectionProvider` 的无阻塞并发通信基架 (Non-blocking I/O)，消除死锁与 UI 冻结。 |
| **多协议通信** | Serial / TCP (Client+Server) / UDP / CAN 2.0 / CAN FD / USB (12种协议) / Modbus RTU / Modbus TCP |
| **控制算法** | 经典 PID / 增量 PID / Bang-Bang / 模糊 PID / 串级 PID / Smith 预估 / ADRC / LADRC / LQR / MPC |
| **智能调参** | 神经网络自适应调参 + LLM API 在线建议 |
| **协议分析** | 帧深度解剖器 / 事务配对分析 / 工业 KPI 仪表盘 / USB 类协议特化 (CDC/HID/MSC/Audio/Video) |
| **CANopen** | 标准 CAN / CAN FD / EtherCAT CoE 三协议工具链（NMT/SDO/PDO/对象字典/帧分析） |
| **数据可视化** | 折线图 / 散点图 / 柱状图 / 仪表盘 / 直方图 / 表格 (6种类型) |
| **报文工具** | 模板式报文构建器 / 二进制解析器 / 14种字段类型 / 6种校验算法 |
| **外部接口** | MCP Server (JSON-RPC 2.0) / LLM API 集成 |
| **国际化** | 中英文实时切换 |
| **商用级体验** | 企业级菜单栏 / 偏好持久化 / 日志轮转 / 崩溃追溯 / 性能看门狗 / 动效四层级 |

---

## 技术栈

| 组件 | 版本 / 说明 |
|------|------------|
| Rust | 2021 Edition |
| eframe / egui | 0.31 |
| egui_plot | 0.31 |
| serialport | 4.7 |
| serde / serde_json | 1.x |
| chrono | 0.4 |
| tracing / tracing-subscriber | 0.1 / 0.3 |
| ureq | 2.x (LLM HTTP) |
| anyhow / thiserror | 1.x / 2.x |

**零外部 C 依赖**：所有控制算法与协议解析纯标准库实现。

---

## 项目结构

```
robot_control_rust/
├── Cargo.toml
├── README.md
├── ARCHITECTURE_AND_USAGE.md          # 详细架构文档
├── .cargo/config.toml                 # 平台链接优化
├── scripts/
│   ├── preflight.sh                   # Linux/macOS 本地预检
│   └── (packaging 已迁移至 scripts/go/rusktask)
├── src/
│   ├── main.rs                        # 入口 + egui 全局样式 + 动效 + 路由
│   ├── app.rs                         # 状态中心 AppState / UiState
│   ├── i18n.rs                        # 国际化
│   ├── models/
│   │   ├── mod.rs
│   │   ├── pid_controller.rs          # 经典位置式 PID
│   │   ├── incremental_pid.rs         # 增量式 PID
│   │   ├── bang_bang.rs               # Bang-Bang 控制器
│   │   ├── fuzzy_pid.rs              # 模糊自适应 PID
│   │   ├── cascade_pid.rs            # 串级 PID
│   │   ├── smith_predictor.rs        # Smith 预估控制
│   │   ├── adrc.rs                   # ADRC 自抗扰控制
│   │   ├── ladrc.rs                  # LADRC 线性自抗扰
│   │   ├── lqr.rs                    # LQR 线性二次调节
│   │   ├── mpc.rs                    # MPC 模型预测控制
│   │   ├── neural_network.rs         # 神经网络
│   │   ├── control_algorithm.rs      # 算法枚举
│   │   ├── connection.rs             # 连接类型与配置
│   │   ├── packet.rs                 # 报文模板 / 解析器
│   │   ├── data_channel.rs           # 数据通道 / 可视化
│   │   ├── robot_state.rs            # 机器人状态
│   │   ├── robot_topology.rs         # 拓扑配置
│   │   ├── preset.rs                 # 预设管理
│   │   ├── chassis_kinematics.rs     # 底盘运动学
│   │   ├── canopen.rs                # CANopen 协议模型
│   │   └── modbus.rs                 # Modbus 协议模型
│   ├── services/
│   │   ├── mod.rs
│   │   ├── serial_service.rs         # 串口服务
│   │   ├── tcp_service.rs            # TCP 客户端/服务端
│   │   ├── udp_service.rs            # UDP 服务
│   │   ├── can_service.rs            # CAN/CAN FD 模拟
│   │   ├── llm_service.rs            # LLM API 调用
│   │   └── mcp_server.rs             # MCP Server (JSON-RPC)
│   └── views/
│       ├── mod.rs
│       ├── dashboard.rs              # 仪表盘
│       ├── connections.rs            # 连接管理
│       ├── serial_debug.rs           # 串口调试
│       ├── packet_builder.rs         # 报文构建与解析
│       ├── topology.rs               # 拓扑可视化
│       ├── pid_control.rs            # 控制算法页面
│       ├── nn_tuning.rs              # 神经网络调参
│       ├── data_viz.rs               # 数据可视化
│       ├── protocol_analysis.rs      # 协议分析器
│       ├── canopen_view.rs           # CANopen 工具
│       ├── modbus_view.rs            # Modbus 工具
│       └── ui_kit.rs                 # UI 工具组件
└── target/
```

---

## 快速开始

### 环境要求

- Rust 1.70+ (推荐 stable)
- Windows / macOS / Linux

### 构建与运行

```powershell
# 克隆项目
git clone <repo-url>
cd robot_control_rust

# Debug 构建并运行
cargo run

# Release 构建（体积优化 ~5.35MB）
cargo build --release
./target/release/robot_control_rust
```

### 测试

```powershell
cargo test          # Debug 模式测试
cargo test --release # Release 模式测试
cargo clippy        # 静态分析
cargo fmt --check   # 代码格式检查
```

### 本地预检（与 CI 对齐）

```powershell
# Windows
./scripts/task.ps1 preflight

# Linux / macOS
./scripts/task preflight
```

---

## 页面功能

### Dashboard

- **连接状态卡片**：Serial / TCP / UDP / CAN 连接状态与吞吐统计
- **System Check**：启动自检（偏好路径、日志路径、MCP 端口、LLM URL、串口可用性）
- **Runtime Metrics**：连接数、LLM 请求、MCP 连接数实时指标
- **Check Updates**：支持 0.1.x 预发布阶段版本策略（`0.1.patch` 视为修复，`0.minor` 视为功能/变更）；优先读取更新清单（可配置通道/超时/Manifest URL），失败时回退到 `ROBOT_CONTROL_UPDATE_URL` 文档链接
- **快捷动作**：连接、启动/停止控制、急停、刷新端口

### Connections

支持 7 种通信协议的配置与管理：

| 协议 | 配置项 |
|------|--------|
| Serial | 端口、波特率、校验、流控 |
| TCP | Client/Server 模式、Host、Port |
| UDP | 本地/远端地址与端口 |
| CAN 2.0 / CAN FD | 仲裁段波特率(9档)、采样点、SJW、FD数据段波特率(8档)、高级选项(终端电阻/监听/回环/重传/错误报告) |
| USB | 12种协议(CDC ACM/HID/MSC/MIDI/Audio/Video等)、5种速度、VID/PID、端点配置 |
| Modbus RTU/TCP | 功能码、地址、寄存器配置 |
| MCP Server | 端口配置、Token 认证、启动/停止 |

### Serial Debug

- HEX / ASCII / Mixed 显示模式
- 自动滚动
- 发送文本或 HEX
- 换行策略（`\r\n` / `\n` / `\r`）

### Packet Builder / Parser

**Builder 标签**：
- 报文模板管理（新建/删除/切换）
- 14 种字段类型：U8, U16, U32, I8, I16, I32, F32, F64, Bool, ASCII, HexBytes, Padding, BitField8, BitField16
- 6 种校验算法：Sum8 / XOR / CRC-8 / CRC-16 Modbus / CRC-16 CCITT / CRC-32
- 实时 HEX 预览、发送、复制

**Parser 标签**：
- 基于模板的二进制数据包解析
- 自动解析模式
- 校验状态指示（✅/❌）
- 字段详情表格（名称/类型/值/数值/原始HEX）
- 最多保留 200 条解析记录
- **联动 Data Viz**：每个数值字段可一键创建可视化通道

### Topology

- 6 种底盘类型选择（差速/麦轮/3轮全向/4轮全向/阿克曼/履带）
- 几何参数设置
- 电机/关节配置
- ASCII 结构可视化

### Control Algorithms

支持 10 种控制算法在线切换，详见[控制算法详解](#控制算法详解)。

### NN Tuning

- 神经网络结构配置与训练状态
- 单步/批量训练
- Loss 曲线可视化
- 建议参数预测并应用
- **LLM API 调参**：可配置 API URL / Model / API Key，一键获取 PID 参数建议

### Data Viz

**6 种可视化类型**：

| 类型 | 说明 |
|------|------|
| Line（折线图） | 连续数据趋势 |
| Scatter（散点图） | 离散数据分布 |
| Bar（柱状图） | 统计对比 |
| Gauge（仪表盘） | 大字号当前值 + 统计 |
| Histogram（直方图） | 数据分布频率 (20 bins) |
| Table（表格） | 统计数据网格 |

**数据源**：RobotState（位置/速度/电流/温度/误差/PID输出）+ PacketField（解析模板字段）

**统计**：最后值、最小值、最大值、平均值、标准差、数据量

### Protocol Analysis

多协议工业级协议分析器，支持 8 种协议：

| 协议 | 分析能力 |
|------|----------|
| Modbus RTU/TCP | 功能码识别、地址/寄存器/数量提取、CRC 校验验证 |
| CAN 2.0 / CAN FD | COB-ID 角色识别、DLC 检查、帧类型分类 |
| USB | Setup/Descriptor/BOT(CBW/CSW) 解剖、类请求特化解析、事务可视化 |
| Serial / TCP / UDP | 通用帧统计 |

**核心功能**：

- **帧深度解剖器 (Frame Dissector)**
  - 彩色 HEX 字节映射（每个字段不同颜色高亮）
  - 字段分解表格（偏移/长度/名称/值/描述）
  - 位级视图（单字节 8-bit 展开）
  - 支持所有 8 种协议的专业解析

- **事务配对分析 (Transaction Analysis)**
  - Modbus：按事务 ID 或地址自动配对请求→响应
  - CAN：按 COB-ID 关联帧
  - 通用协议：按时间窗口匹配
  - 响应时间统计

- **工业 KPI 仪表盘**
  - Payload Utilization（负载率）
  - Frame Error Rate（帧错误率）
  - BER(est)（估算误码率，ppm）
  - IFG(avg) / Jitter(std)（帧间隔与抖动）
  - Duplicate Payload Ratio（重复负载占比）

- **协议诊断检查**：Modbus CRC 错误检测、CAN DLC 异常、帧格式校验
- **CSV 导出**：分析结果一键导出

**USB 特化能力（新增）**：

- **类识别**：`UsbClassHint` 自动识别 Standard / CDC ACM / HID / Mass Storage / Audio / Video / Vendor
- **请求映射**：支持标准请求 + CDC/HID/MSC/Audio/Video 类请求名映射
- **描述符解析**：Device / Config / Interface / Endpoint / HID / CS_INTERFACE 逐字段解析
- **MSC BOT 解析**：自动识别并解析 CBW/CSW，附带 SCSI opcode 名称
- **CDC 数据阶段解析**：`SET_LINE_CODING` 与 `SET_CONTROL_LINE_STATE` 语义解码

### CANopen Tools

面向 CiA 301/402/401 规范的 CANopen 工具集（支持标准 CAN / CAN FD / EtherCAT CoE）：

- **协议选择器**：同一页面下切换三种工业协议视图
- **标准 CANopen**：NMT/SDO/PDO/对象字典完整工具链
- **CAN FD 模式**：支持 FD DLC 映射（0..15）与 64 字节数据区构建/校验
- **EtherCAT CoE 模式**：支持 CoE SDO 帧构建与帧分析（Mailbox + CoE Header + SDO）

- **NMT 控制**：Start / Stop / Pre-Operational / Reset Node / Reset Communication
- **SDO 客户端**：Upload / Download / Abort，支持 Index/SubIndex/Data 配置
- **PDO 映射管理器**
  - CiA 402 (Motion) / CiA 401 (I/O) 预设一键加载
  - 手动添加/删除映射条目
  - JSON 导入/导出
  - 数据类型：I8/U8/I16/U16/I32/U32/F32/Bool
- **PDO 实时解码器**：输入 HEX 负载 → 按当前 PDO 配置解码为字段值
- **CANopen 帧分析器**
  - 输入 COB-ID + Data → 自动识别帧类型（NMT/SYNC/EMCY/TPDO/RPDO/SDO/Heartbeat）
  - 彩色字段显示
  - 详细字段表格
- **对象字典浏览器**：45+ 标准条目，按类别标签（Communication/PDO Mapping/Device/Motion）分类

### Modbus Tools

- 功能码配置 (FC01~FC16)
- RTU / TCP 帧构建与预览
- CRC 校验自动计算
- 寄存器模拟表
- Modbus 通信日志

### MCP Server

基于 JSON-RPC 2.0 的 TCP 协议接口，支持外部 AI 工具集成：

**支持方法**：

| 方法 | 说明 |
|------|------|
| `initialize` | 握手与版本协商 |
| `tools` / `tools/list` | 列出可用方法 |
| `get_pid_params` | 获取 PID 参数 |
| `set_pid_params` | 设置 PID 参数 |
| `get_robot_state` | 获取机器人状态 |
| `get_state_history` | 获取状态历史 |
| `get_parsed_packets` | 获取解析数据包 |
| `suggest_params` | 参数建议 |

**调用示例**：

```json
// 获取 PID 参数
{"jsonrpc":"2.0","method":"get_pid_params","id":1}
// → {"jsonrpc":"2.0","result":{"kp":1.0,"ki":0.1,"kd":0.01,"setpoint":0.0},"id":1}

// 设置 PID 参数
{"jsonrpc":"2.0","method":"set_pid_params","params":{"kp":2.2,"ki":0.15,"kd":0.03,"setpoint":50.0},"id":2}
// → {"jsonrpc":"2.0","result":{"ok":true},"id":2}
```

---

## 控制算法详解

### 1. 经典位置式 PID

$$u(t) = K_p \cdot e(t) + K_i \int e(t)\,dt + K_d \frac{de(t)}{dt}$$

参数：Kp, Ki, Kd, Setpoint, 限幅, 死区, 微分滤波, 前馈 | 支持抗积分饱和

### 2. 增量式 PID

$$\Delta u(k) = K_p [e(k)-e(k-1)] + K_i \cdot e(k) + K_d [e(k)-2e(k-1)+e(k-2)]$$

无积分饱和，支持增量限幅、斜率限制 | 适用：步进电机、阀门控制

### 3. Bang-Bang 控制器

开关式控制，迟滞带防抖 | 适用：温度控制、继电器驱动

### 4. 模糊自适应 PID

7×7 模糊规则表 (NB/NM/NS/ZO/PS/PM/PB)，双线性插值连续推理 | 适用：非线性/时变系统

### 5. 串级 PID

```
setpoint → [外环 PID] → inner_sp → [内环 PID] → output
```

双闭环位置-速度控制 | 适用：电机位置-速度闭环、温度-功率闭环

### 6. Smith 预估控制

内置一阶惯性过程模型：$G(s) = \frac{K}{Ts+1} e^{-\tau s}$

延迟缓冲补偿 | 适用：化工过程、大惯量系统

### 7. ADRC 自抗扰控制

```
setpoint → [TD] → (v1,v2) → [NLSEF] → u0 → u = u0 - z3/b0
feedback → [ESO] → (z1,z2,z3) ─┘
```

fhan 最速控制 + 三阶 ESO + fal 非线性反馈 | 适用：大干扰/不确定模型

### 8. LADRC 线性自抗扰

带宽参数化：ωc（控制器）+ ωo（观测器）仅 2 个调参旋钮 | 一阶/二阶模式

### 9. LQR 线性二次调节

$$u = -K_1(x_1 - r) - K_2 \dot{x}_1 - K_i \int e \, dt$$

解析 Riccati 求解 + 可选积分环节 | 适用：倒立摆、自平衡机器人

### 10. MPC 模型预测控制

预测时域 Np / 控制时域 Nc，代价函数 $J = \sum Q(r-y)^2 + R u^2 + S \Delta u^2$

自适应梯度下降 QP 求解 | 适用：有约束多变量系统、轨迹跟踪

---

## 底盘运动学

支持 6 种底盘正/逆运动学：

| 底盘类型 | 正运动学 | 逆运动学 | 典型应用 |
|---------|---------|---------|----------|
| 差速驱动 (Differential) | ✅ | ✅ | 巡线机器人、扫地机 |
| 麦克纳姆 (Mecanum) | ✅ | ✅ | RoboMaster 步兵 |
| 三轮全向 (Omni-3) | ✅ | ✅ | RoboCup 小型组 |
| 四轮全向 (Omni-4) | ✅ | ✅ | 全向移动平台 |
| 阿克曼转向 (Ackermann) | ✅ | ✅ | 无人车、AGV |
| 履带式 (Tracked) | — | ✅ | 排爆/探索机器人 |

纯 Rust 实现，无外部依赖。代码示例在控制算法页面底部可展开查看（中英文切换）。

---

## 国际化

- 模块：`src/i18n.rs`
- 语言：English / 中文
- 切换：底部状态栏地球图标按钮，即时生效
- 实现：所有视图通过 `Tr::xxx(lang)` 集中取文案

---

## UI/UX 体验

### 企业级菜单栏

- 顶部 `File / Edit / View / Tools / Help` 五大菜单
- `File`：导出分析 CSV、导入预设、偏好设置、退出
- `Edit`：清空日志、复制最后一帧、重置计数器
- `View`：主题切换、侧栏切换、动效等级、语言切换
- `Tools`：11 个页面直达 + MCP Server 开关
- `Help`：关于、快捷键、文档入口

### 动效四层级

| 层级 | 说明 |
|------|------|
| 极致 | 最丝滑过渡，高刷新率 |
| 标准 | 平衡视觉与性能 |
| 原生 | 接近系统默认节奏 |
| 优化 | 最低功耗，适合弱性能设备 |

- Apple-like 贝塞尔缓动曲线
- 页面切换 slide + fade 过渡
- 侧边栏平滑宽度动画
- 空闲自动降低刷新频率
- 帧耗时持续偏高时自动降级动效层级

### 平台特化

- macOS：更柔和（速度略降、位移略增）
- Windows：基线参数
- Linux：略偏性能

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+1..9` | 快速切换主页面 |
| `Ctrl+B` | 切换侧栏展开/收起 |
| `Ctrl+M` | 循环切换动效层级 |

---

## 商用特性

### 偏好持久化

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%/robot_control_rust/preferences.json` |
| macOS | `~/Library/Application Support/robot_control_rust/preferences.json` |
| Linux | `~/.config/robot_control_rust/preferences.json` |

自动保存：每 3 秒增量保存，退出时强制保存。

恢复项目：语言、主题、侧栏状态、动效层级、最近页面、连接参数、解析模式、LLM/MCP 配置。

### 日志与追溯

- **运行日志**：`logs/app.log`，超过 5MB 自动轮转到 `app.log.1`
- **崩溃日志**：`logs/panic.log`（panic hook 捕获）
- **遥测日志**：`logs/telemetry.log`（tracing 结构化事件）

### 资源上限

| 资源 | 上限 | 超限行为 |
|------|------|----------|
| CAN 帧历史 | 10,000 | 统计丢弃帧并告警 |
| DataChannel 缓冲 | 2,000 | 统计丢弃点并在 Data Viz 提示 |

### 安全

- LLM API Key：支持 `LLM_API_KEY` 环境变量（UI 为掩码输入）
- MCP Token：支持 `MCP_TOKEN` 环境变量或 UI 输入
- 端口校验：TCP/UDP/MCP 端口统一 1~65535 范围校验

### 性能看门狗

- 状态栏显示 `Frame(ms)` 与卡顿计数 `Spike`
- 持续高帧耗时自动下调动效档位（自适应性能保护）

---

## 跨平台部署

### Release 构建

```powershell
cargo build --release
# 输出: target/release/robot_control_rust(.exe)
# Windows 实测 ~5.35 MB
```

体积优化配置：`opt-level="z"` + `lto` + `codegen-units=1` + `strip` + `panic="abort"`

### 字体兼容

| 平台 | CJK 字体候选 |
|------|-------------|
| Windows | msyh.ttc / simhei.ttf / simsun.ttc |
| macOS | PingFang.ttc / STHeiti Light.ttc |
| Linux | NotoSansCJK / wqy-microhei |

### CI 三平台验收

GitHub Actions 工作流 `.github/workflows/platform-validation.yml`：

- 矩阵：`windows-latest` / `ubuntu-latest` / `macos-latest`
- 步骤：fmt → build → test → test --release → clippy → build --release
- 产物：每平台上传 release size 工件

---

## 测试

> 最近闭环验证日期：2026-02-26
> 
> 当前结果：`cargo test` = **353 passed, 0 failed**，`cargo clippy --all-targets` = **0 warning**。

### 测试矩阵

| 项目 | 命令 | 状态 |
|------|------|------|
| 代码格式 | `cargo fmt --check` | ✅ |
| Debug 构建 | `cargo build` | ✅ |
| Debug 测试 | `cargo test` | ✅ |
| Release 测试 | `cargo test --release` | ✅ |
| 静态分析 | `cargo clippy --all-targets` | ✅ 0 warning |

### 单元测试覆盖（重点模块）

| 模块 | 测试数 | 说明 |
|------|--------|------|
| `views::protocol_analysis::tests` | 33 | USB 特化解析、Modbus/CAN/UDP 诊断、事务分析与校验逻辑 |
| `models::canopen::tests` | 14 | CAN/CAN FD/EtherCAT CoE 构建与解析、DLC 映射、SDO/NMT 回归 |
| `models::packet::tests` | 49 | 模板构包、字段类型、校验、解析 roundtrip |
| `models::connection::tests` | 24 | 协议配置与参数边界 |
| `services::can_service::tests` | 15 | CAN 通道行为与帧服务逻辑 |
| **总计（所有模块）** | **353** | 全量测试通过 |

---

## 验收清单

- [x] 左侧导航与底部状态栏显示正常
- [x] 中英文实时切换
- [x] 中文无方框
- [x] 10 种控制算法参数面板正确
- [x] 底盘运动学代码示例可展开
- [x] CAN/CAN FD 完整参数（波特率/采样点/SJW/高级选项）
- [x] USB 12 种协议 + VID/PID + 速度 + 端点配置
- [x] Packet Builder / Parser 双标签
- [x] Parser → Data Viz 联动通道
- [x] Data Viz 六种图表类型
- [x] LLM API 配置与调参建议
- [x] MCP Server 配置、启动/停止
- [x] Protocol Analysis 帧解剖器 + 事务分析 + 工业 KPI
- [x] CANopen 三协议支持（标准 CAN / CAN FD / EtherCAT CoE）
- [x] USB 多类协议特化分析（CDC/HID/MSC/Audio/Video）
- [x] 企业级菜单栏（File/Edit/View/Tools/Help）
- [x] Modbus RTU/TCP 帧构建 + 寄存器模拟
- [x] 动效四层级切换 + Apple-like 过渡
- [x] 偏好持久化（跨平台路径）
- [x] 性能看门狗（Frame(ms) + Spike）
- [x] `cargo build` 编译通过
- [x] `cargo fmt --check` 通过
- [x] `cargo clippy` 0 warning
- [x] `cargo test` 353 测试全部通过
- [x] `cargo test --release` 353 测试全部通过

### target 目录瘦身（交付建议）

推荐一键脚本（Windows）：

```powershell
cd ../scripts/go/rusktask
go run . build-release-slim
```

可选参数：

- `--skip-tests`：跳过测试（仅格式/可选clippy/构建）
- `--skip-clippy`：跳过 clippy

若仅需交付可运行 Release 版本，建议在构建后执行：

```powershell
# 1) 清理历史编译缓存
cargo clean

# 2) 重新仅构建 Release
cargo build --release

# 3) 可选：删除非运行必须产物（仅保留 release exe）
Remove-Item .\target\debug -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item .\target\release\deps -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item .\target\release\build -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item .\target\release\incremental -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item .\target\release\examples -Recurse -Force -ErrorAction SilentlyContinue
```

默认建议最终保留：

- `target/release/robot_control_rust.exe`

---

## 架构详情

详细架构文档请参阅 [ARCHITECTURE_AND_USAGE.md](ARCHITECTURE_AND_USAGE.md)。

---

## License

MIT
