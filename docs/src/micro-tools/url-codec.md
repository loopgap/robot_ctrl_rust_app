# URL 编解码

> URL Encode / Decode 工具

## 功能概述

| 功能 | 说明 |
|------|------|
| URL Encode | 特殊字符转义为 %XX 格式 |
| URL Decode | %XX 格式转义还原 |
| 错误提示 | 无效输入自动提示 |

## 编码规则

### 需要编码的字符

根据 RFC 3986，以下字符需要编码：

| 字符 | 编码 | 说明 |
|------|------|------|
| 空格 | %20 | 空格转为加号或 %20 |
| 中文 | %XX%XX... | UTF-8 编码 |
| 特殊符号 | %XX | 如 `!` → `%21` |
| 非 ASCII | %XX%XX... | 多字节字符 |

### 编码示例

| 原文 | 编码结果 |
|------|----------|
| `Hello World` | `Hello%20World` |
| `name=张三` | `name%3D%E5%BC%A0%E4%B8%89` |
| `a=1&b=2` | `a%3D1%26b%3D2` |
| `https://example.com` | `https%3A%2F%2Fexample.com` |

## 使用方法

### 交互模式

```powershell
cargo run -- url
```

### 命令行模式

```powershell
# URL 编码
cargo run --release -- url --encode "Hello World"

# URL 解码
cargo run --release -- url --decode "Hello%20World"

# 从文件读取
cargo run --release -- url --encode --file input.txt

# 输出到文件
cargo run --release -- url --encode --input "test" --output result.txt
```

### 参数选项

| 参数 | 说明 |
|------|------|
| `--encode` | 编码模式 |
| `--decode` | 解码模式 |
| `--input <text>` | 输入字符串 |
| `--file <path>` | 从文件读取 |
| `--output <path>` | 输出到文件 |
| `--plus-as-space` | 将 + 视为空格 |

## 输出示例

### 编码结果

```
╔══════════════════════════════════════════╗
║  URL Encoder                            ║
╠══════════════════════════════════════════╣
║  Input:   Hello World 你好              ║
╠══════════════════════════════════════════╣
║  Output:  Hello%20World%20%E4%BD%A0%... ║
╠══════════════════════════════════════════╣
║  Status:  ● Completed                  ║
╚══════════════════════════════════════════╝
```

### 解码结果

```
╔══════════════════════════════════════════╗
║  URL Decoder                            ║
╠══════════════════════════════════════════╣
║  Input:   Hello%20World%20%E4%BD%A0%...║
╠══════════════════════════════════════════╣
║  Output:  Hello World 你好              ║
╠══════════════════════════════════════════╣
║  Status:  ● Completed                  ║
╚══════════════════════════════════════════╝
```

## 典型应用场景

### 场景 1: API 参数处理

```powershell
# 编码查询参数
cargo run --release -- url --encode "name=张三&age=30"
# 输出: name%3D%E5%BC%A0%E4%B8%89%26age%3D30

# 编码后的 URL
GET /api?query=name%3D%E5%BC%A0%E4%B8%89%26age%3D30
```

### 场景 2: 回调链接处理

```powershell
# 原始回调
https://example.com/callback?code=AB%20CD&state=123

# 需要编码 state 参数中的特殊字符
cargo run --release -- url --encode "AB CD"
# 输出: AB%20CD
```

### 场景 3: 路径参数

```powershell
# 编码中文路径
cargo run --release -- url --encode "/用户/张三"
# 输出: %2F%E7%94%A8%E6%88%B7%2F%E5%BC%A0%E4%B8%89
```

## 常见错误

### 无效的百分号编码

```
⚠ 错误: 无效的百分号编码
位置: ... %XY ...
说明: %XY 不是有效的 UTF-8 序列
```

### 不完整的编码

```
⚠ 错误: 输入不完整
位置: ... %20
说明: 百分号后缺少 2 位十六进制数字
```

## 状态流转

```
○ 待处理 → ● 解析中 → ● 编码/解码 → ● 验证 → ● 已完成
```

## 相关工具

- [Base64 工坊](base64.md) - Base64 编解码
- [JSON 工坊](json.md) - JSON 数据处理