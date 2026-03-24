# csv_cleaner_gui

独立 Rust 小工具：CSV 清洗工坊（GUI）。

## 功能
- 去除空行
- 字段 Trim
- 可选按整行去重
- 统计列数一致性
- 闭环流程：输入 → 校验 → 执行 → 验证 → 导出

## 运行
```bash
cargo run --release
```
