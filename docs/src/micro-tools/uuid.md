# UUID 批量生成器

> 批量生成 UUID (通用唯一标识符)

## 功能概述

| 功能 | 说明 |
|------|------|
| 批量生成 | 支持生成多个 UUID |
| 格式控制 | 大小写、连字符开关 |
| 版本选择 | UUID v1, v4 |

## UUID 版本

### UUID v1 (时间戳版本)

```
xxxx-xxxx-xxxx-xxxx
 ││││   ││││   ││││   ││││
 节点ID  时间序列  时钟序列
```

**特点**:
- 基于时间戳和节点 ID 生成
- 可以在一定程度上反推生成时间
- 需要 MAC 地址或随机节点 ID

### UUID v4 (随机版本)

```
xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
                   ││││
                   版本  变体
```

**特点**:
- 完全随机生成
- 无法反推任何信息
- 碰撞概率极低 (~2^122)

## 格式选项

| 选项 | 说明 | 示例 |
|------|------|------|
| 连字符 | 包含 - 分隔符 | `550e8400-e29b-41d4-a716-446655440000` |
| 无连字符 | 纯十六进制 | `550e8400e29b41d4a716446655440000` |
| 大写 | 字母大写 | `550E8400-E29B-41D4-A716-446655440000` |
| 小写 | 字母小写 | `550e8400-e29b-41d4-a716-446655440000` |

## 使用方法

### 交互模式

```powershell
cargo run -- uuid
```

1. 选择 UUID 版本 (1: v1, 2: v4)
2. 输入生成数量
3. 选择格式选项
4. 查看结果

### 命令行模式

```powershell
# 生成默认 UUID (v4, 小写, 有连字符)
cargo run --release -- uuid

# 指定数量
cargo run --release -- uuid --count 10

# 无连字符
cargo run --release -- uuid --no-dash

# 大写
cargo run --release -- uuid --uppercase

# v1 版本
cargo run --release -- uuid --version 1

# 组合选项
cargo run --release -- uuid --count 5 --uppercase --no-dash

# 输出到文件
cargo run --release -- uuid --count 100 --output uuids.txt
```

### 参数选项

| 参数 | 说明 |
|------|------|
| `--count <n>` | 生成数量 (默认 1) |
| `--version <v>` | UUID 版本 (1 或 4) |
| `--uppercase` | 输出大写 |
| `--no-dash` | 无连字符 |
| `--output <path>` | 输出到文件 |

## 输出示例

### 默认格式

```
╔══════════════════════════════════════════╗
║  UUID Generator                       ║
╠══════════════════════════════════════════╣
║  Version:  4 (Random)               ║
║  Count:    5                        ║
╠══════════════════════════════════════════╣
║  1. 550e8400-e29b-41d4-a716-446655440000
║  2. 6ba7b810-9dad-11d1-80b4-00c04fd430c8
║  3. f47ac10b-58cc-4372-a567-0e02b2c3d479
║  4. 7c9e6679-7425-40de-944b-e07fc1f90ae7
║  5. 4f17ba7a-0cbb-4ed1-b890-3a1cd4f6c3c1
╠══════════════════════════════════════════╣
║  Status:  ● Completed               ║
╚══════════════════════════════════════════╝
```

### 大写无连字符

```
550E8400E29B41D4A716446655440000
6BA7B8109DAD11D180B400C04FD430C8
F47AC10B58CC4372A5670E02B2C3D479
7C9E6679742540DE944BE07FC1F90AE7
4F17BA7A0CBB4ED1B8903A1CD4F6C3C1
```

## 典型应用场景

### 场景 1: 测试数据准备

```powershell
# 生成 100 个 UUID 作为测试 ID
cargo run --release -- uuid --count 100 --output test_ids.txt
```

### 场景 2: 批量主键生成

```bash
# 生成唯一 ID
for i in {1..10}; do
  uuid=$(cargo run --release -- uuid --no-dash --uppercase)
  echo "ID_$i: $uuid"
done
```

### 场景 3: 链路追踪 ID

```powershell
# 生成带连字符的标准格式 (适合日志追踪)
cargo run --release -- uuid --count 5
```

## UUID 结构详解

### v4 UUID 位域

```
┌──────────────────────────────────────────────────────────────┐
│                      UUID v4 结构                            │
├──────────┬──────────┬──────────┬────────────────────────────┤
│ 32 bits  │ 16 bits  │  4 bits  │  4 bits  │  12 bits       │
│          │          │  (版本)   │ (变体)   │                │
│ 随机数   │ 随机数   │    4     │    8- B  │ 随机数         │
├──────────┴──────────┴──────────┴──────────┴─────────────────┤
│ xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx                        │
└──────────────────────────────────────────────────────────────┘
```

### 版本和变体

| 版本 | 变体 | 说明 |
|------|------|------|
| 4xxx | 8xxx | NCS 向后兼容 |
| 4xxx | 9xxx | RFC 4122 |
| 4xxx | Axxx | RFC 4122 |
| 4xxx | Bxxx | Microsoft GUID |

## 状态流转

```
○ 待处理 → ● 生成中 → ● 格式化 → ● 输出 → ● 已完成
```

## 相关工具

- [校验和工坊](checksum.md) - 数据校验
- [时间戳转换](time-converter.md) - 时间处理