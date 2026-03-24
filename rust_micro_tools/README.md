# Rust Micro Tools Suite

一个面向实际交付场景的 Rust 小工具合集，UI 风格与 `robot_control_rust` 保持一致（深色工业风、蓝色强调色、统一控件密度）。

当前版本提供 **7 款工具**，并且全部支持：

- 双语界面（中文 / English）
- App 内置使用引导文档（Guide）
- 语言偏好持久化（重启后自动恢复）

## 代码结构（已拆分提升可读性）

- `src/main.rs`：入口与启动
- `src/app.rs`：总 UI 容器与工具切换
- `src/theme.rs`：统一主题
- `src/workflow.rs`：闭环流程状态面板
- `src/i18n.rs`：语言定义（中文 / English）
- `src/settings.rs`：语言偏好持久化
- `src/tools/`：各工具实现（`checksum` / `json_workshop` / `log_inspector` / `url_codec` / `time_converter` / `base64_workshop` / `uuid_batch`）

## 工具清单（市场高频场景）

1. **校验和工坊**
   - 算法：CRC32、FNV1a-64、SHA256
   - 场景：接口联调、报文验签、数据入库一致性

2. **JSON 工坊**
   - 功能：JSON 校验、格式化、压缩
   - 场景：配置治理、跨环境参数分发、回归排查

3. **日志巡检**
   - 功能：包含/排除正则筛选、命中统计
   - 场景：故障定位、值班告警复核、工单证据提取

4. **URL 编解码**
   - 功能：URL Encode / Decode，错误提示
   - 场景：参数透传、回调链接联调、跨系统编码对齐

5. **时间戳转换**
   - 功能：Unix 秒/毫秒与时间字符串互转
   - 场景：日志对时、链路回放、跨时区排障

6. **Base64 工坊**
   - 功能：标准/URL Safe Base64 编码与解码
   - 场景：Token 排障、二进制字段联调、网关参数处理

7. **UUID 批量生成**
   - 功能：批量生成 UUID（支持大写与去连字符）
   - 场景：测试数据准备、批量主键生成、链路追踪 ID 预置

## 闭环流程（与主应用一致）

每个工具都实现统一流程面板：

`输入 → 校验 → 执行 → 验证 → 导出`

状态可视化：
- `○` 待处理
- `●` 已完成
- `▲` 需关注

## 运行

```bash
cargo run --release
```

## 相关目录

- 独立工具集合：[`../rust_indie_tools/README.md`](../rust_indie_tools/README.md)
