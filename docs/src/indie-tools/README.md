# 独立图形工具迁移说明

`rust_indie_tools` 已从当前工作区的正式构建与发布流中退役，原有 CSV / JWT / Regex 能力已并入 `rust_tools_suite`。

## 当前结论

- 不再维护独立 GUI 工具的单独发布链。
- 不再为独立 GUI 工具保留活动工作区入口。
- 用户应改为使用 `rust_tools_suite` 获取统一的桌面工具体验。

## 对应关系

| 原独立工具 | 当前归属 |
|------|------|
| CSV 清洗工坊 | `rust_tools_suite` |
| JWT 解析工坊 | `rust_tools_suite` |
| Regex 巡检工坊 | `rust_tools_suite` |

## 迁移后的好处

- 统一双语界面与偏好持久化
- 统一响应式布局和流程面板
- 统一测试、发布和资产命名

## 相关文档

- [工具套件总览](../micro-tools/README.md)
- [工具套件架构](../tools-suite-architecture.md)
- [主应用文档](../robot-control/README.md)
