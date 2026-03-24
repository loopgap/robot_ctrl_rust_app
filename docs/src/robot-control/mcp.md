# MCP Server

> Model Control Protocol 服务端 - 外部应用与机器人控制的桥梁

## 概述

MCP Server 是基于 JSON-RPC 2.0 的服务接口，允许外部应用（如 Python、C++、JavaScript）通过网络调用机器人控制功能。

## 协议规范

### JSON-RPC 2.0

MCP Server 完全兼容 JSON-RPC 2.0 规范：

```
┌─────────────────────────────────────┐
│ Request                             │
├─────────────────────────────────────┤
│ {                                  │
│   "jsonrpc": "2.0",               │
│   "method": "method_name",         │
│   "params": {...},                 │
│   "id": 1                         │
│ }                                  │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ Response (Success)                  │
├─────────────────────────────────────┤
│ {                                  │
│   "jsonrpc": "2.0",               │
│   "result": {...},                 │
│   "id": 1                         │
│ }                                  │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ Response (Error)                    │
├─────────────────────────────────────┤
│ {                                  │
│   "jsonrpc": "2.0",               │
│   "error": {                      │
│     "code": -32600,              │
│     "message": "Error description"│
│   },                              │
│   "id": 1                         │
│ }                                  │
└─────────────────────────────────────┘
```

### 错误码

| 错误码 | 含义 |
|--------|------|
| -32700 | Parse error - 无效 JSON |
| -32600 | Invalid Request - 无效请求 |
| -32601 | Method not found - 方法不存在 |
| -32602 | Invalid params - 无效参数 |
| -32603 | Internal error - 内部错误 |

## API 方法

### 连接管理

#### `connect`

建立连接

```json
{
  "method": "connect",
  "params": {
    "type": "serial",
    "config": {
      "port": "COM1",
      "baudrate": 115200
    }
  },
  "id": 1
}
```

响应：

```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": true,
    "connection_id": "conn_001"
  },
  "id": 1
}
```

#### `disconnect`

断开连接

```json
{
  "method": "disconnect",
  "params": {
    "connection_id": "conn_001"
  },
  "id": 2
}
```

#### `get_connections`

获取所有连接状态

```json
{
  "method": "get_connections",
  "id": 3
}
```

### 数据操作

#### `send_data`

发送数据

```json
{
  "method": "send_data",
  "params": {
    "connection_id": "conn_001",
    "data": "01 02 03 04",
    "format": "hex"
  },
  "id": 4
}
```

#### `receive_data`

接收数据

```json
{
  "method": "receive_data",
  "params": {
    "connection_id": "conn_001",
    "timeout": 1000
  },
  "id": 5
}
```

### 控制算法

#### `set_algorithm`

设置控制算法

```json
{
  "method": "set_algorithm",
  "params": {
    "algorithm": "pid",
    "params": {
      "kp": 1.0,
      "ki": 0.1,
      "kd": 0.05
    }
  },
  "id": 6
}
```

#### `start_control`

启动控制

```json
{
  "method": "start_control",
  "params": {
    "setpoint": 100.0
  },
  "id": 7
}
```

#### `stop_control`

停止控制

```json
{
  "method": "stop_control",
  "id": 8
}
```

### 状态查询

#### `get_status`

获取系统状态

```json
{
  "method": "get_status",
  "id": 9
}
```

响应：

```json
{
  "jsonrpc": "2.0",
  "result": {
    "connections": 2,
    "control_active": true,
    "algorithm": "pid",
    "setpoint": 100.0,
    "process_value": 98.5,
    "output": 45.2
  },
  "id": 9
}
```

#### `get_logs`

获取日志

```json
{
  "method": "get_logs",
  "params": {
    "level": "info",
    "limit": 100
  },
  "id": 10
}
```

### 订阅事件

#### `subscribe`

订阅事件通知

```json
{
  "method": "subscribe",
  "params": {
    "events": ["data_received", "connection_status", "control_update"]
  },
  "id": 11
}
```

服务器将主动推送事件：

```json
{
  "jsonrpc": "2.0",
  "method": "event",
  "params": {
    "event": "data_received",
    "data": {
      "connection_id": "conn_001",
      "data": "AA BB CC DD"
    }
  }
}
```

## 使用示例

### Python 调用示例

```python
import json
import socket

class MCPClient:
    def __init__(self, host, port):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((host, port))
        self.request_id = 1

    def call(self, method, params=None):
        request = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or {},
            "id": self.request_id
        }
        self.request_id += 1

        self.sock.send(json.dumps(request).encode())
        response = json.loads(self.sock.recv(4096).decode())
        return response.get("result")

    def connect_serial(self, port, baudrate):
        return self.call("connect", {
            "type": "serial",
            "config": {"port": port, "baudrate": baudrate}
        })

    def send_command(self, connection_id, data):
        return self.call("send_data", {
            "connection_id": connection_id,
            "data": data,
            "format": "hex"
        })

    def close(self):
        self.sock.close()

# 使用
client = MCPClient("127.0.0.1", 8080)
result = client.connect_serial("COM1", 115200)
print(result)
client.close()
```

### C++ 调用示例

```cpp
#include <jsonrpccpp/client.h>
#include <jsonrpccpp/client/connectors/httpclient.h>

using namespace jsonrpc;
using namespace std;

class MCPClient {
private:
    HttpClient client;
    RpcClient rpc;

public:
    MCPClient(const string& url) : client(url), rpc(client) {}

    Json::Value connect(const string& type, const Json::Value& config) {
        Json::Value params;
        params["type"] = type;
        params["config"] = config;
        return rpc.CallMethod("connect", params);
    }

    Json::Value sendData(const string& connId, const string& data) {
        Json::Value params;
        params["connection_id"] = connId;
        params["data"] = data;
        params["format"] = "hex";
        return rpc.CallMethod("send_data", params);
    }
};

int main() {
    MCPClient client("http://127.0.0.1:8080");
    Json::Value result = client.connect("serial", ...);
    return 0;
}
```

## 配置

### 启动参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| 端口 | 8080 | 监听端口 |
| 最大连接数 | 10 | 最大并发连接 |
| 超时 | 30s | 请求超时 |
| 日志级别 | info | 日志详细程度 |

### 配置文件

```json
{
  "mcp_server": {
    "port": 8080,
    "max_connections": 10,
    "request_timeout": 30000,
    "log_level": "info"
  }
}
```

## 安全性

### 认证 (可选)

```json
{
  "auth": {
    "enabled": true,
    "type": "token",
    "token": "your-secret-token"
  }
}
```

使用时在请求头中添加：

```
Authorization: Bearer your-secret-token
```

### 速率限制

| 限制 | 值 |
|------|-----|
| 每秒请求数 | 100 |
| 单请求最大数据 | 1MB |
| 并发连接数 | 10 |

## 故障排除

### 连接被拒绝

1. 确认 MCP Server 已启动
2. 检查端口是否被占用
3. 验证防火墙设置

### 请求超时

1. 增加超时时间
2. 检查网络延迟
3. 确认服务器负载

### 无效响应

1. 检查 JSON-RPC 版本 (必须是 "2.0")
2. 验证请求格式
3. 查看错误码和错误信息