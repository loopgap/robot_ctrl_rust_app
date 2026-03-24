# 配置参考

> Rust Serial 工作区配置详解

## 项目配置

### Cargo.toml 结构

```toml
[package]
name = "robot_control_rust"
version = "0.1.1"
edition = "2021"
authors = ["Antigravity Workspace"]
license = "MIT"

[dependencies]
# GUI
eframe = "0.31"
egui = "0.31"
egui_plot = "0.31"

# 串口通信
serialport = "4.7"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 数据序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 网络请求
ureq = "2"

# 错误处理
anyhow = "1"
thiserror = "2"
```

### 特性标志 (Feature Flags)

| 特性 | 说明 | 默认启用 |
|------|------|----------|
| `default` | 默认特性集 | ✅ |
| `serial` | 串口支持 | ✅ |
| `can` | CAN 总线支持 | ✅ |
| `usb` | USB 支持 | ✅ |
| `llm` | LLM API 支持 | ✅ |
| `mcp` | MCP Server 支持 | ✅ |

## 应用配置

### 配置文件位置

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%/rust_serial/` |
| macOS | `~/Library/Application Support/rust_serial/` |
| Linux | `~/.config/rust_serial/` |

### 配置示例

```json
{
  "language": "zh-CN",
  "theme": "dark",
  "serial": {
    "default_baudrate": 115200,
    "timeout_ms": 1000
  },
  "mcp": {
    "port": 8080,
    "enabled": false
  },
  "llm": {
    "api_url": "http://localhost:8080/api",
    "api_key": ""
  }
}
```

### 配置字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `language` | string | 界面语言 (`zh-CN` / `en-US`) |
| `theme` | string | 主题 (`dark` / `light`) |
| `serial.default_baudrate` | number | 默认波特率 |
| `serial.timeout_ms` | number | 串口超时 (毫秒) |
| `mcp.port` | number | MCP Server 端口 |
| `mcp.enabled` | boolean | 是否启用 MCP |
| `llm.api_url` | string | LLM API 地址 |
| `llm.api_key` | string | LLM API 密钥 |

## 环境变量

### 应用环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `RUST_LOG` | 日志级别 | `info` |
| `RUST_SERIAL_CONFIG` | 配置文件路径 | 平台默认 |
| `ROBOT_CONTROL_UPDATE_URL` | 更新检查 URL | 官方地址 |
| `MCP_PORT` | MCP Server 端口 | `8080` |

### 日志级别

| 级别 | 说明 |
|------|------|
| `error` | 仅错误 |
| `warn` | 警告及以上 |
| `info` | 信息及以上 (默认) |
| `debug` | 调试及以上 |
| `trace` | 所有日志 |

### 设置日志

```powershell
# PowerShell
$env:RUST_LOG = "debug"

# CMD
set RUST_LOG=debug

# Linux/macOS
export RUST_LOG=debug
```

## 服务配置

### MCP Server 配置

```json
{
  "mcp_server": {
    "host": "127.0.0.1",
    "port": 8080,
    "max_connections": 10,
    "request_timeout_ms": 30000,
    "auth": {
      "enabled": false,
      "token": ""
    }
  }
}
```

### LLM Service 配置

```json
{
  "llm_service": {
    "api_url": "http://localhost:8080/v1/chat",
    "api_key": "",
    "model": "gpt-3.5-turbo",
    "timeout_ms": 30000,
    "max_retries": 3
  }
}
```

## 串口配置

### 默认波特率

| 波特率 | 典型应用 |
|--------|----------|
| 9600 | 慢速设备、老旧设备 |
| 19200 | 工业仪表 |
| 38400 | 串口打印机 |
| 57600 | 工业 Modbus |
| 115200 | 高速串口 (常用) |
| 460800 | 高速数据采集 |
| 921600 | 高速工业设备 |
| 3000000 | 最大速率 |

### 数据格式

| 参数 | 可选值 |
|------|--------|
| 数据位 | 5, 6, 7, 8 |
| 停止位 | 1, 2 |
| 校验位 | None, Odd, Even |

## CAN 配置

### CAN 波特率

| 波特率 | 典型应用 |
|--------|----------|
| 125k | 长距离 CAN |
| 250k | 标准工业 CAN |
| 500k | 高速 CAN |
| 1M | 高速设备 |

### CAN FD 配置

| 参数 | 说明 |
|------|------|
| 仲裁波特率 | 协商阶段波特率 |
| 数据波特率 | 数据阶段波特率 (最高 8M) |
| 采样点 | 数据阶段采样点 (75%/87.5%) |

## 网络配置

### TCP 配置

```json
{
  "tcp": {
    "connect_timeout_ms": 5000,
    "read_timeout_ms": 30000,
    "write_timeout_ms": 10000,
    "keepalive": true
  }
}
```

### UDP 配置

```json
{
  "udp": {
    "broadcast_enabled": true,
    "recv_buffer_size": 65536,
    "send_buffer_size": 65536
  }
}
```

## 主题配置

### 深色主题 (默认)

```json
{
  "theme": {
    "name": "dark",
    "primary_color": "#007ACC",
    "background_color": "#1E1E1E",
    "surface_color": "#252526",
    "text_color": "#D4D4D4",
    "accent_color": "#3794FF"
  }
}
```

### 浅色主题

```json
{
  "theme": {
    "name": "light",
    "primary_color": "#0078D4",
    "background_color": "#FFFFFF",
    "surface_color": "#F3F3F3",
    "text_color": "#333333",
    "accent_color": "#0066CC"
  }
}
```

## 快捷键配置

### 默认快捷键

| 功能 | Windows/Linux | macOS |
|------|---------------|-------|
| 新建连接 | Ctrl+N | Cmd+N |
| 打开连接 | Ctrl+O | Cmd+O |
| 关闭连接 | Ctrl+W | Cmd+W |
| 发送数据 | Ctrl+Enter | Cmd+Enter |
| 清除日志 | Ctrl+L | Cmd+L |
| 设置 | Ctrl+, | Cmd+, |
| 退出 | Alt+F4 | Cmd+Q |

### 自定义快捷键

```json
{
  "keybindings": {
    "new_connection": "ctrl+n",
    "open_connection": "ctrl+o",
    "close_connection": "ctrl+w",
    "send_data": "ctrl+enter",
    "clear_log": "ctrl+l",
    "settings": "ctrl+comma"
  }
}
```

## 相关文档

- [快速入门](getting-started.md) - 环境准备与首次运行
- [机器人主控](robot-control/README.md) - 主应用使用指南
- [智能排障](troubleshooting.md) - 配置问题排查