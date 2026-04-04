# Robot Control Rust - 架构与使用文档

> 项目路径：`robot_control_rust/`  
> 目标：跨平台（Windows/macOS/Linux）机器人调试与控制 GUI（eframe/egui）  
> 更新时间：2026-02-26

---

## 1. 项目概览

`robot_control_rust` 是一个纯 Rust 工业控制与协议工具平台，核心能力如下：

- 多协议连接：Serial / TCP / UDP / CAN 2.0 / CAN FD / USB / Modbus RTU / Modbus TCP
- 控制算法：PID、增量 PID、Bang-Bang、模糊 PID、串级 PID、Smith、ADRC、LADRC、LQR、MPC
- 智能调参：神经网络建议 + 外部 LLM API 建议
- 协议分析：多协议过滤、深度解剖、事务分析、工业 KPI
- CANopen 工具：标准 CAN / CAN FD / EtherCAT CoE 三协议统一页面
- USB 特化分析：CDC/HID/MSC/Audio/Video 类请求与描述符/BOT 解析
- 企业级 UI：顶部菜单栏（File/Edit/View/Tools/Help）+ 侧栏 + 状态栏
- 工程能力：MCP Server、日志轮转、偏好持久化、性能看门狗、双语切换

---

## 2. 目录结构

```text
robot_control_rust/
├─ Cargo.toml
├─ README.md
├─ ARCHITECTURE_AND_USAGE.md
├─ scripts/
│  ├─ preflight.ps1
│  ├─ preflight.sh
│  └─ package_windows_x64_iexpress_installer.ps1
├─ src/
│  ├─ main.rs
│  ├─ app.rs
│  ├─ i18n.rs
│  ├─ models/
│  │  ├─ canopen.rs
│  │  ├─ connection.rs
│  │  ├─ control_algorithm.rs
│  │  ├─ packet.rs
│  │  ├─ data_channel.rs
│  │  ├─ modbus.rs
│  │  └─ ...
│  ├─ services/
│  │  ├─ serial_service.rs
│  │  ├─ tcp_service.rs
│  │  ├─ udp_service.rs
│  │  ├─ can_service.rs
│  │  ├─ llm_service.rs
│  │  └─ mcp_server.rs
│  └─ views/
│     ├─ protocol_analysis.rs
│     ├─ canopen_view.rs
│     ├─ connections.rs
│     ├─ modbus_view.rs
│     └─ ...
└─ target/
```

---

## 3. 技术栈

- Rust 2021 Edition
- GUI：`eframe 0.31` / `egui 0.31` / `egui_plot 0.31`
- 串口：`serialport 4.8`
- 数据序列化：`serde` / `serde_json`
- 时间与日志：`chrono` / `tracing` / `tracing-subscriber`
- 网络请求：`ureq 3.x`
- 错误处理：`anyhow` / `thiserror`

---

## 4. 核心架构

### 4.1 状态中心（`src/app.rs`）

应用采用单状态中心（`AppState`）+ UI 临时状态（`UiState`）模式：

- `AppState`：连接状态、日志、算法实例、协议数据、MCP/LLM 状态
- `UiState`：输入框、开关、下拉、页面局部偏好
- `ActiveTab`：11 个主页面路由枚举

关键方法（与本次升级相关）：

- `connect_active()/disconnect_active()`：按连接类型分发
- `send_data()/poll_data()`：统一收发与轮询
- `compute_active_algorithm()`：算法分发执行
- `start_mcp_server()/stop_mcp_server()/sync_mcp_state()`：MCP 生命周期
- `toggle_mcp_server()`：菜单栏/连接页共用 MCP 切换
- `reset_counters()`：重置 serial/tcp/udp 统计计数

### 4.2 UI 组织（`src/main.rs` + `src/views/*.rs`）

`main.rs` 负责全局容器与导航：

- 顶部：企业级 `TopBottomPanel::top("menu_bar")` 菜单栏
- 左侧：图标导航 + 可折叠标签栏
- 底部：状态栏（连接、吞吐、语言、性能）
- 中央：按 `ActiveTab` 路由到各页面 `show()`

### 4.3 分层约束

- `models/`：协议模型、算法模型、数据结构（不依赖 UI）
- `services/`：串口/TCP/UDP/CAN/LLM/MCP 能力层
- `views/`：交互与可视化，调用 `AppState` 完成状态变更

---

## 5. 企业级菜单栏设计（新增）

顶部菜单共 5 个一级菜单：

- `File`：Export Logs CSV、Import Preset、Preferences、Quit
- `Edit`：Clear All Logs、Copy Last Frame、Reset Counters
- `View`：Theme、Sidebar、Motion Level、Language
- `Tools`：11 个页面直达、Toggle MCP Server
- `Help`：About、Keyboard Shortcuts、Documentation

国际化：新增 23 个菜单翻译键（中英文）。

---

## 6. CANopen 三协议融合（新增）

### 6.1 目标

将 CANopen 工具从单一标准 CAN 扩展到三协议统一入口：

- 标准 CAN
- CAN FD
- EtherCAT CoE

### 6.2 模型层新增（`src/models/canopen.rs`）

- `CanProtocolType`
- `CanStdFrame`
- `EcatCoeSdoRequest`
- `EcatCoeFrame`
- `EcatCoeAnalysis`
- `MultiProtocolFrame`

关键能力：

- CAN FD DLC 映射与长度合法性校验
- EtherCAT CoE SDO 帧构建（Mailbox + CoE + SDO）
- 三协议统一构帧与摘要

### 6.3 视图层新增（`src/views/canopen_view.rs`）

- 协议选择标签（标准 CAN / CAN FD / EtherCAT CoE）
- 各协议子面板独立展示与参数输入
- 保持 NMT/SDO/PDO/对象字典与日志能力

---

## 7. USB 协议特化分析（新增）

实现位置：`src/views/protocol_analysis.rs`

### 7.1 识别与映射

- `UsbClassHint`：Standard / CDC ACM / HID / MassStorage / Audio / Video / Vendor / Unknown
- `usb_class_name()` / `usb_descriptor_type_name()` / `usb_pid_name()`
- `usb_class_request_name()`：按类分发请求名

### 7.2 深度解析器

- 设备描述符：`dissect_usb_device_descriptor()`
- 配置描述符：`dissect_usb_config_descriptor()`（支持子描述符遍历）
- CDC Line Coding：`dissect_usb_cdc_line_coding()`
- MSC BOT：`dissect_usb_msc_cbw()` / `dissect_usb_msc_csw()`
- SCSI opcode 映射：`scsi_opcode_name()`

### 7.3 自动检测与诊断

- `detect_usb_class()`：启发式类识别
- `detect_usb_descriptor_type()`：描述符检测
- `detect_usb_bot_frame()`：CBW/CSW 签名检测
- `usb_checks()`：长度/范围/签名/类请求合法性等检查

### 7.4 可视化增强

- USB Transaction Analyzer 新增类标识徽章
- 类专属详情面板（CDC/HID/MSC）
- BOT 帧信息可视化

---

## 8. 主要页面能力

### 8.1 Connections

- Serial/TCP/UDP/CAN/USB/Modbus/MCP 参数管理
- CAN/CAN FD：波特率、采样点、SJW、高级选项
- USB：12 类协议、5 档速度、VID/PID、端点配置

### 8.2 Protocol Analysis

- 协议筛选：Serial/TCP/UDP/CAN/CAN FD/Modbus RTU/Modbus TCP/USB
- 工具链：关键词过滤、方向过滤、CSV 导出、统计指标
- 诊断：CRC/DLC/长度/格式一致性校验

### 8.3 CANopen Tools

- NMT 控制、SDO 读写、PDO 映射、对象字典
- 三协议视图切换（标准 CAN / CAN FD / EtherCAT CoE）

### 8.4 其余页面

- Packet Builder/Parser（14 字段类型，6 校验算法）
- Data Viz（6 可视化类型 + 统计）
- NN Tuning（NN + LLM 参数建议）
- Modbus Tools（RTU/TCP 构建与寄存器面板）

---

## 9. 国际化

- 模块：`src/i18n.rs`
- 语言：`English` / `Chinese`
- 方式：`Tr::xxx(lang)` 统一取文案
- 菜单栏相关翻译键已全量补齐

---

## 10. 运行、构建与预检

```powershell
cargo build
cargo run
cargo test
cargo clippy --all-targets
```

本地一键预检：

- Windows：`./scripts/preflight.ps1`
- Linux/macOS：`./scripts/preflight.sh`

---

## 11. 闭环测试结果（当前基线）

### 11.1 测试矩阵（2026-04-03）

| 项目 | 命令 | 结果 |
|------|------|------|
| 代码格式 | `cargo fmt --check` | ✅ 通过 |
| Debug 构建 | `cargo build` | ✅ 通过 |
| Debug 测试 | `cargo test` | ✅ 321 passed |
| Release 测试 | `cargo test --release` | 按需执行 |
| 静态分析 | `cargo clippy --all-targets` | ✅ 0 warning |

### 11.2 分模块测试计数

| 模块 | 数量 |
|------|------|
| `app::tests` | 13 |
| `i18n::tests` | 6 |
| `models::adrc::tests` | 10 |
| `models::bang_bang::tests` | 9 |
| `models::canopen::tests` | 14 |
| `models::cascade_pid::tests` | 9 |
| `models::chassis_kinematics::tests` | 13 |
| `models::connection::tests` | 24 |
| `models::control_algorithm::tests` | 4 |
| `models::data_channel::tests` | 18 |
| `models::fuzzy_pid::tests` | 11 |
| `models::incremental_pid::tests` | 10 |
| `models::ladrc::tests` | 9 |
| `models::lqr::tests` | 10 |
| `models::modbus::tests` | 13 |
| `models::mpc::tests` | 10 |
| `models::neural_network::tests` | 16 |
| `models::packet::tests` | 49 |
| `models::pid_controller::tests` | 9 |
| `models::preset::tests` | 5 |
| `models::robot_state::tests` | 3 |
| `models::robot_topology::tests` | 12 |
| `models::smith_predictor::tests` | 11 |
| `services::can_service::tests` | 15 |
| `services::llm_service::tests` | 13 |
| `services::mcp_server::tests` | 4 |
| `views::protocol_analysis::tests` | 已并入当前测试统计 |
| **总计** | **321** |

---

## 12. 交付清单

- [x] CANopen 工具兼容标准 CAN / CAN FD / EtherCAT CoE
- [x] USB 协议多类特化分析（CDC/HID/MSC/Audio/Video）
- [x] 企业级菜单栏（File/Edit/View/Tools/Help）
- [x] UI 性能与交互体验优化（动效分级、状态可观测）
- [x] 全量闭环验证（构建、clippy、测试）
- [x] 文档更新（README + 本架构文档）

---

## 13. 后续可选增强

- 引入集成测试（连接→发包→解析→控制→停止完整链路）
- USB 实设备通信链路（`rusb`）
- 协议分析性能基准与采样火焰图
- 发布产物签名与自动升级策略
