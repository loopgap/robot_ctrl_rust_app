# 日志巡检

> 日志分析与正则筛选工具

## 功能概述

| 功能 | 说明 |
|------|------|
| 包含筛选 | 显示匹配正则的行 |
| 排除筛选 | 隐藏匹配正则的行 |
| 命中统计 | 统计匹配次数和位置 |
| 多规则组合 | AND/OR 规则组合 |

## 使用方法

### 交互模式

```powershell
cargo run -- log
```

### 输入日志

支持多行日志输入：

```
2024-01-15 10:23:45 INFO  Starting service on port 8080
2024-01-15 10:23:46 DEBUG Initializing database connection
2024-01-15 10:23:47 ERROR Failed to connect to database: timeout
2024-01-15 10:23:48 WARN  Retrying connection (attempt 1/3)
2024-01-15 10:23:49 ERROR Connection failed again
```

### 正则表达式

#### 基础语法

| 模式 | 匹配 |
|------|------|
| `ERROR` | 包含 "ERROR" 的行 |
| `WARN\|ERROR` | 包含 "WARN" 或 "ERROR" |
| `\d{4}-\d{2}-\d{2}` | 日期格式 |
| `timeout\|failed` | 错误关键词 |

#### 高级模式

```regex
# 匹配 IP 地址
\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}

# 匹配时间戳
\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}

# 匹配错误级别
(ERROR|WARN|FATAL)

# 匹配特定字段
\[.*?\]\s*(.*?)\s*at\s+.*?
```

### 命令行模式

```powershell
# 包含筛选
cargo run --release -- log --include "ERROR|WARN"

# 排除筛选
cargo run --release -- log --exclude "DEBUG|TRACE"

# 统计模式
cargo run --release -- log --count --include "ERROR"

# 从文件读取
cargo run --release -- log --file app.log --include "ERROR"
```

### 参数选项

| 参数 | 说明 |
|------|------|
| `--include <regex>` | 包含匹配的行 |
| `--exclude <regex>` | 排除匹配的行 |
| `--count` | 显示命中统计 |
| `--file <path>` | 从文件读取日志 |
| `--ignore-case` | 忽略大小写 |

## 输出示例

### 包含筛选

```
╔══════════════════════════════════════════╗
║  Log Inspector - Include Filter        ║
╠══════════════════════════════════════════╣
║  Pattern: ERROR|WARN                   ║
╠══════════════════════════════════════════╣
║  [3] 2024-01-15 10:23:47 ERROR ...    ║
║  [4] 2024-01-15 10:23:48 WARN  ...    ║
║  [5] 2024-01-15 10:23:49 ERROR ...    ║
╠══════════════════════════════════════════╣
║  Total: 3 matches                     ║
╚══════════════════════════════════════════╝
```

### 命中统计

```
╔══════════════════════════════════════════╗
║  Match Statistics                      ║
╠══════════════════════════════════════════╣
║  Pattern: ERROR                        ║
║  Total matches: 2                     ║
║  Occurrences: 5                       ║
╠══════════════════════════════════════════╣
║  Line 3:  1 occurrence               ║
║  Line 5:  1 occurrence               ║
║  Total:   2 occurrences              ║
╚══════════════════════════════════════════╝
```

## 典型应用场景

### 场景 1: 错误追踪

```powershell
# 查看所有错误
cargo run -- log --include "ERROR"

# 查看错误和警告
cargo run -- log --include "ERROR|WARN"

# 查看特定模块错误
cargo run -- log --include "ERROR.*database|ERROR.*auth"
```

### 场景 2: 性能分析

```powershell
# 查看慢查询
cargo run -- log --include "Slow query|timeout"

# 查看请求延迟
cargo run -- log --include "latency.*\d{3,}ms"
```

### 场景 3: 安全审计

```powershell
# 查看登录失败
cargo run -- log --include "Login failed|Authentication error"

# 查看异常访问
cargo run -- log --include "403|404|500"
```

## 状态流转

```
○ 待处理 → ● 读取中 → ● 筛选中 → ● 统计中 → ● 已完成
```

## 相关工具

- [JSON 工坊](json.md) - JSON 数据处理
- [校验和工坊](checksum.md) - 数据校验