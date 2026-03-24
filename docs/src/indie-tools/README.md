# 独立图形工具

> 每个工具独立目录、独立 Cargo.toml、单独构建与发布

## 简介

`rust_indie_tools` 是独立 Rust GUI 工具目录，每个工具：

- 独立目录结构
- 独立 `Cargo.toml`
- 独立 README 文档
- 独立 GitHub Matrix Action 自动构建打包

UI 风格与主应用一致：**深色工业风 + 蓝色强调色**

## 工具列表

### 1. CSV 清洗工坊 (csv_cleaner_gui)

**功能**: CSV 数据清洗与预处理

| 功能 | 说明 |
|------|------|
| 去除空行 | 自动移除空白行 |
| 字段 Trim | 清除字段首尾空白 |
| 行去重 | 可选按整行内容去重 |
| 列数一致性检查 | 统计并报告列数不一致的行 |

**闭环流程**: `输入 → 校验 → 执行 → 验证 → 导出`

**运行**:
```powershell
cd rust_indie_tools/csv_cleaner_gui
cargo run --release
```

---

### 2. JWT 解析工坊 (jwt_inspector_gui)

**功能**: JWT Token 解析与检查

| 功能 | 说明 |
|------|------|
| Header 解析 | Base64URL 解码并格式化 |
| Payload 解析 | Base64URL 解码并格式化 |
| JSON 格式化 | 美化 JSON 输出 |
| 过期检查 | 快速显示过期时间（仅解析，不验签）|

**适用场景**: API 调试、Token 排障、前端联调

**运行**:
```powershell
cd rust_indie_tools/jwt_inspector_gui
cargo run --release
```

---

### 3. Regex 巡检工坊 (regex_workbench_gui)

**功能**: 正则表达式测试与验证

| 功能 | 说明 |
|------|------|
| 正则输入 | 支持 Rust 正则语法 |
| 文本输入 | 多行文本支持 |
| 命中高亮 | 快速定位匹配内容 |
| 命中统计 | 显示匹配次数和行号 |

**适用场景**: 日志告警规则验证、文本模式匹配、规则调试

**运行**:
```powershell
cd rust_indie_tools/regex_workbench_gui
cargo run --release
```

---

## 统一规范

### UI 风格
- 深色工业风 + 蓝色强调色
- 与 `robot_control_rust` 和 `rust_micro_tools` 保持一致

### 交互流程
所有工具实现统一闭环流程：

```
输入 → 校验 → 执行 → 验证 → 导出
```

### 状态可视化
| 符号 | 状态 |
|------|------|
| `○` | 待处理 |
| `●` | 已完成 |
| `▲` | 需关注 |

## 与微型工具集的关系

| 特性 | rust_indie_tools | rust_micro_tools |
|------|-------------------|------------------|
| 入口方式 | 独立应用 | 统一 TUI 入口 |
| 双语支持 | ❌ | ✅ |
| 语言持久化 | ❌ | ✅ |
| 构建方式 | 每个工具独立 | 统一 |
| 发布方式 | 独立 Matrix 构建 | 统一发布 |
| 复杂度 | 较复杂 GUI | 较简单 TUI |

两者互补：
- `rust_micro_tools` 提供统一入口、双语支持与语言持久化
- `rust_indie_tools` 提供更复杂功能的独立 GUI 应用

## 相关文档

- 微型工具集: [Micro Tools](../micro-tools/README.md)
- 主应用: [Robot Control](../robot-control/README.md)