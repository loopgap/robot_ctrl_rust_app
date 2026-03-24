# JSON 工坊

> JSON 校验、格式化、压缩工具

## 功能概述

| 功能 | 说明 | 使用场景 |
|------|------|----------|
| 校验 | 语法检查与错误定位 | API 调试、配置检查 |
| 格式化 | 美化 JSON 输出 | 日志查看、代码审查 |
| 压缩 | 移除多余空白 | 数据传输、存储优化 |

## 功能详解

### JSON 校验

**检查项**:
- 语法正确性
- 引号匹配
- 括号匹配
- 数据类型有效性

**错误示例**:
```json
// ❌ 错误 JSON
{
  "name": "test",
  "value": 123,
  "items": [1, 2, 3  // 缺少 ]
}

// ✅ 正确 JSON
{
  "name": "test",
  "value": 123,
  "items": [1, 2, 3]
}
```

**错误输出**:
```
╔══════════════════════════════════════════╗
║  JSON Validation Failed                 ║
╠══════════════════════════════════════════╣
║  Error: Unexpected token at line 4     ║
║  Expected: ] or ,                       ║
║  Position: ... "items": [1, 2, 3█     ║
╚══════════════════════════════════════════╝
```

### JSON 格式化

**输入**:
```json
{"name":"admin","roles":["user","admin"],"config":{"timeout":30,"debug":true}}
```

**输出**:
```json
{
  "name": "admin",
  "roles": [
    "user",
    "admin"
  ],
  "config": {
    "timeout": 30,
    "debug": true
  }
}
```

### JSON 压缩

**输入**:
```json
{
  "name": "admin",
  "value": 123
}
```

**输出**:
```json
{"name":"admin","value":123}
```

## 使用方法

### 交互模式

```powershell
cargo run -- json
```

1. 选择操作 (1: 校验, 2: 格式化, 3: 压缩)
2. 输入 JSON 数据
3. 查看结果

### 命令行模式

```powershell
# 校验
cargo run --release -- json --validate

# 格式化
cargo run --release -- json --format

# 压缩
cargo run --release -- json --minify

# 指定输入文件
cargo run --release -- json --file input.json --format

# 输出到文件
cargo run --release -- json --file input.json --format --output output.json
```

### 参数选项

| 参数 | 说明 |
|------|------|
| `--validate` | 仅校验 |
| `--format` | 格式化输出 |
| `--minify` | 压缩输出 |
| `--file <path>` | 从文件读取 |
| `--output <path>` | 输出到文件 |
| `--indent <n>` | 自定义缩进 (默认 2) |

## 典型应用场景

### 场景 1: API 响应检查

```powershell
# 模拟 API 响应
$response = '{"status":"ok","data":{"id":1,"name":"test"}}'

# 格式化查看
echo $response | cargo run --release -- json --format
```

### 场景 2: 配置文件验证

```json
// config.json
{
  "server": {
    "host": "localhost",
    "port": 8080
  },
  "database": {
    "url": "postgresql://localhost:5432/test",
    "pool_size": 10
  }
}
```

### 场景 3: 数据对比

```powershell
# 对比两个 JSON 文件
diff (json-tool --format file1.json) (json-tool --format file2.json)
```

## 状态流转

```
○ 待处理 → ● 解析中 → ● 校验/格式化/压缩 → ● 验证成功 / ▲ 发现错误 → ● 已完成
```

## 技术细节

### 解析器特性

- 支持 UTF-8 编码
- 支持嵌套结构
- 支持所有 JSON 数据类型
- 支持 Unicode 转义

### 性能指标

| 指标 | 数值 |
|------|------|
| 最大输入大小 | 10 MB |
| 最大嵌套深度 | 100 层 |
| 处理速度 | ~1 MB/s |

## 相关工具

- [校验和工坊](checksum.md) - 数据校验
- [URL 编解码](url-codec.md) - URL 参数处理