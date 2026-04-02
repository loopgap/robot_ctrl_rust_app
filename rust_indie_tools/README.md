# rust_indie_tools

独立 Rust 小工具目录（每个工具单独目录、单独 `Cargo.toml`、单独 `README`）。

## 工具列表

- `csv_cleaner_gui`：CSV 清洗工坊  
  文档：[`csv_cleaner_gui/README.md`](csv_cleaner_gui/README.md)

- `jwt_inspector_gui`：JWT 解析工坊（仅解析，不验签）  
  文档：[`jwt_inspector_gui/README.md`](jwt_inspector_gui/README.md)

- `regex_workbench_gui`：Regex 巡检工坊  
  文档：[`regex_workbench_gui/README.md`](regex_workbench_gui/README.md)

## 统一规范

- UI 风格：深色工业风 + 蓝色强调色（与主应用一致）
- 流程闭环：`输入 → 校验 → 执行 → 验证 → 导出`
- 每个工具都可独立构建与发布
- 与 `rust_micro_tools` 互补：`rust_micro_tools` 提供统一入口、双语与语言持久化

## 聚合与独立关系

- `rust_micro_tools` 已提供统一管理界面，聚合了本目录三款工具能力（CSV/JWT/Regex）。
- 本目录仍保留独立工程，便于单工具开发、调试和按需独立构建。
