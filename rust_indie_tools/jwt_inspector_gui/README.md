# jwt_inspector_gui

独立 Rust 小工具：JWT 解析工坊（GUI）。

## 功能
- 解析 JWT Header / Payload（Base64URL）
- JSON 格式化展示
- 到期时间字段快速检查（仅展示，不验签）
- 闭环流程：输入 → 校验 → 执行 → 验证 → 导出

## 运行
```bash
cargo run --release
```
