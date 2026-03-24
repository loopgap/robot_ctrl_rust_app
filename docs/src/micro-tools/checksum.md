# 校验和工坊

> 计算数据校验和：CRC32、FNV1a-64、SHA256

## 功能概述

校验和工坊提供三种常用校验算法，用于验证数据完整性和报文验签。

| 算法 | 输出长度 | 特点 | 典型应用 |
|------|----------|------|----------|
| CRC32 | 32 位 (8 hex) | 检错能力强，计算快 | 网络协议、文件传输 |
| FNV1a-64 | 64 位 (16 hex) | 哈希分布均匀 | 哈希表、 bloom filter |
| SHA256 | 256 位 (64 hex) | 加密安全 | 数据签名、密码存储 |

## 算法详解

### CRC32

**原理**: 循环冗余校验，使用生成多项式 `0xEDB88320`

**特性**:
- 检测能力：可检测 1 位、2 位错误，所有奇数位错误，长度 ≤ 32 的突发错误
- 计算速度：极快（硬件支持）

**示例**:
```
输入: "Hello World"
CRC32: E8B7BE43
```

### FNV1a-64

**原理**: Fowler-Noll-Vo 哈希算法的 64 位变体

**公式**:
```
hash = FNV_offset_basis
for each byte:
    hash = hash XOR byte
    hash = hash * FNV_prime
```

**特性**:
- 64 位输出，碰撞概率极低
- 适合作为哈希表的哈希函数

**示例**:
```
输入: "Hello World"
FNV1a-64: 34A361CE8B7BE43
```

### SHA256

**原理**: 安全哈希算法，SHA-2 家族成员

**特性**:
- 256 位输出
- 抗强碰撞：无可实用攻击
- 适合数据完整性验证

**示例**:
```
输入: "Hello World"
SHA256: 0x7F83B1657FF1FC538F836...
```

## 使用方法

### 交互模式

```powershell
cargo run -- checksum
```

1. 选择算法 (1: CRC32, 2: FNV1a-64, 3: SHA256)
2. 输入数据 (支持 HEX 字符串或普通文本)
3. 查看结果

### 命令行模式

```powershell
# 使用 STDIN
echo "Hello World" | cargo run --release -- checksum

# 指定算法和输入
cargo run --release -- checksum --algorithm crc32 --input "Hello World"

# HEX 输入
cargo run --release -- checksum --algorithm sha256 --hex "48656C6C6F20576F726C64"
```

### 参数选项

| 参数 | 说明 | 可选值 |
|------|------|--------|
| `--algorithm` | 选择算法 | crc32, fnv1a64, sha256 |
| `--input` | 输入数据 | 字符串 |
| `--hex` | HEX 格式输入 | HEX 字符串 |
| `--file` | 从文件读取 | 文件路径 |

## 输出格式

### 结果展示

```
╔══════════════════════════════════════════╗
║         Checksum Workshop                ║
╠══════════════════════════════════════════╣
║  Algorithm: CRC32                       ║
║  Input:    "Hello World"                ║
╠══════════════════════════════════════════╣
║  Result:   E8B7BE43                     ║
╠══════════════════════════════════════════╣
║  Status:   ● Completed                  ║
╚══════════════════════════════════════════╝
```

### 批量处理

支持批量计算多个数据的校验和：

```
Input           │ Algorithm │ Result
────────────────┼───────────┼─────────────────
Hello           │ CRC32     │ E8B7BE43
World           │ CRC32     │ F7DCDB8A
Hello World     │ CRC32     │ E8B7BE43
```

## 典型应用场景

### 场景 1: 接口联调验签

**需求**: 验证接收数据与发送数据一致

```
发送方:
原始数据: {"user": "admin", "action": "login"}
CRC32: A1B2C3D4

接收方:
收到数据: {"user": "admin", "action": "login"}
计算 CRC32: A1B2C3D4
比对结果: ✅ 匹配
```

### 场景 2: 文件完整性检查

```powershell
# 下载文件时附带 MD5/SHA256
# 验证本地文件
certutil -hashfile filename SHA256
```

### 场景 3: 数据入库一致性

```rust
// 数据库存储前计算校验和
let checksum = calculate_crc32(data);
store_to_database(data, checksum);

// 读取时验证
let (stored_data, stored_checksum) = read_from_database();
let current_checksum = calculate_crc32(stored_data);
assert_eq!(stored_checksum, current_checksum);
```

## 状态流转

```
○ 待处理 → ● 计算中 → ● 验证成功 / ▲ 验证失败 → ● 已完成
```

## 相关工具

- [JSON 工坊](json.md) - JSON 数据处理
- [Base64 工坊](base64.md) - Base64 编解码