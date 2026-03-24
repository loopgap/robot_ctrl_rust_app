# Base64 工坊

> 标准 / URL Safe Base64 编解码工具

## 功能概述

| 功能 | 说明 |
|------|------|
| 标准 Base64 | 标准的 A-Z, a-z, 0-9, +, / 字符集 |
| URL Safe Base64 | 安全的 - , _ 字符集（无 +, /） |

## Base64 编码表

```
索引 → 字符
────────────────
 0 → A   16 → Q  32 → g  48 → w
 1 → B   17 → R  33 → h  49 → x
 2 → C   18 → S  34 → i  50 → y
 3 → D   19 → T  35 → j  51 → z
 4 → E   20 → U  36 → k  52 → 0
 5 → F   21 → V  37 → l  53 → 1
 6 → G   22 → W  38 → m  54 → 2
 7 → H   23 → X  39 → n  55 → 3
 8 → I   24 → Y  40 → o  56 → 4
 9 → J   25 → Z  41 → p  57 → 5
10 → K   26 → a  42 → q  58 → 6
11 → L   27 → b  43 → r  59 → 7
12 → M   28 → c  44 → s  60 → 8
13 → N   29 → d  45 → t  61 → 9
14 → O   30 → e  46 → u  62 → +
15 → P   31 → f  47 → v  63 → /
```

## 编码原理

### 步骤

1. 将输入转换为 8 位二进制
2. 每 6 位一组，转换为 Base64 索引
3. 根据索引查找对应字符
4. 末尾不足 6 位时用 `=` 填充

### 示例

```
原始: "M" (ASCII 77)
二进制: 01001101

分组: 010011 010000
Base64: k Q (索引 18, 16)

结果: "bQ==" (M → bQ==)
```

## 使用方法

### 交互模式

```powershell
cargo run -- base64
```

### 命令行模式

```powershell
# 标准 Base64 编码
cargo run --release -- base64 --encode "Hello World"

# 标准 Base64 解码
cargo run --release -- base64 --decode "SGVsbG8gV29ybGQ="

# URL Safe Base64 编码
cargo run --release -- base64 --encode "Hello/World+Test" --url-safe

# URL Safe Base64 解码
cargo run --release -- base64 --decode "SGVsbG8~V29ybGQ-Test" --url-safe

# 从文件编码
cargo run --release -- base64 --encode --file data.bin

# 输出到文件
cargo run --release -- base64 --encode --input "test" --output result.txt
```

### 参数选项

| 参数 | 说明 |
|------|------|
| `--encode` | 编码模式 |
| `--decode` | 解码模式 |
| `--input <text>` | 输入字符串 |
| `--file <path>` | 从文件读取 |
| `--output <path>` | 输出到文件 |
| `--url-safe` | 使用 URL Safe 字符集 |

## 标准 vs URL Safe

| 特性 | 标准 Base64 | URL Safe Base64 |
|------|--------------|-----------------|
| 字符集 | A-Za-z0-9+/ | A-Za-z0-9-_ |
| 特殊字符 | +, / | -, _ |
| 填充 | = | = 或省略 |
| 用途 | 邮件、MIME | URL 参数、JSON |

### URL Safe 说明

在 URL 中使用 Base64 时，`+` 和 `/` 需要额外编码。
URL Safe Base64 直接使用 `-` 和 `_`，无需额外编码。

## 输出示例

### 编码结果

```
╔══════════════════════════════════════════╗
║  Base64 Encoder                       ║
╠══════════════════════════════════════════╣
║  Input:   Hello World                 ║
╠══════════════════════════════════════════╣
║  Output:  SGVsbG8gV29ybGQ=            ║
╠══════════════════════════════════════════╣
║  Status:  ● Completed                 ║
╚══════════════════════════════════════════╝
```

### URL Safe 编码

```
╔══════════════════════════════════════════╗
║  Base64 Encoder (URL Safe)            ║
╠══════════════════════════════════════════╣
║  Input:   Hello World+Test           ║
╠══════════════════════════════════════════╣
║  Output:  SGVsbG8gV29ybGQrVGVzdA      ║
╠══════════════════════════════════════════╣
║  Status:  ● Completed                 ║
╚══════════════════════════════════════════╝
```

## 典型应用场景

### 场景 1: API Token 处理

```powershell
# 编码用户信息
cargo run --release -- base64 --encode '{"user":"admin","exp":1705312345}'
# 输出: eyJ1c2VyIjoiYWRtaW4iLCJleHAiOjE3MDUzMTIzNDV9
```

### 场景 2: 数据 URL

```powershell
# 图片 Base64
data:image/png;base64,iVBORw0KGgoAAAANS...
```

### 场景 3: URL 参数

```powershell
# JWT 载荷
cargo run --release -- base64 --encode '{"alg":"HS256","typ":"JWT"}' --url-safe
# 输出: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9
```

## 常见错误

### 无效的 Base64 输入

```
⚠ 错误: 无效的 Base64 字符串
说明: 字符串长度必须是 4 的倍数
位置: ... ABC█
```

### 填充错误

```
⚠ 错误: 无效的填充
说明: 填充字符 = 的位置不正确
```

## 状态流转

```
○ 待处理 → ● 读取中 → ● 编码/解码中 → ● 验证 → ● 已完成
```

## 相关工具

- [URL 编解码](url-codec.md) - URL 参数编解码
- [校验和工坊](checksum.md) - 数据校验