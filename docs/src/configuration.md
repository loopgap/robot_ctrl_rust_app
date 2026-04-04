# 配置参考

## `robot_control_rust`

当前主要依赖：

- `eframe 0.31`
- `egui 0.31`
- `egui_plot 0.31`
- `serialport 4.8`
- `serde / serde_json`
- `chrono`
- `tracing`
- `ureq 3`

配置主要通过应用内偏好持久化保存，重点包括：

- 语言
- 主题
- UI 缩放
- 连接参数
- MCP 配置
- 更新检查参数

## `rust_tools_suite`

偏好持久化字段：

- `language`
- `dark_mode`
- `ui_scale_percent`
- `workflow_drawer_open`
- `active_tool_key`

典型保存路径：

- Windows: `%APPDATA%/rust_tools_suite/preferences.json`
- macOS: `~/Library/Application Support/rust_tools_suite/preferences.json`
- Linux: `~/.config/rust_tools_suite/preferences.json`

## 环境变量

- `RUST_LOG`
- `LLM_API_KEY`（主应用可选）
