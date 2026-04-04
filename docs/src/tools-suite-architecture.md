# 工具套件架构

`rust_tools_suite` 是当前工作区唯一保留的聚合式工具目录。

## 结构

- `src/app.rs`
  全局壳层、菜单、响应式布局、状态栏、快捷键、偏好持久化。
- `src/file_ops.rs`
  文件导入与另存为结果。
- `src/theme.rs`
  深浅色主题与 CJK 字体 fallback。
- `src/workflow.rs`
  闭环流程面板。
- `src/tools/*.rs`
  单工具实现与测试。

## 响应式布局

- 宽屏：右侧流程面板
- 紧凑：底部流程抽屉
- 工具切换区：按钮组 / 下拉选择自动切换
- JWT 工具：宽屏双栏，窄屏上下堆叠

## 文件 I/O

适合文本流的工具统一支持：

- 导入输入文件
- 剪贴板复制
- 另存为结果

## JWT 范围

- Header / Payload 解析
- HS256 / RS256 可选验签
- 文本框或文件导入密钥

更详细的产品内架构说明见 [`rust_tools_suite/ARCHITECTURE_AND_USAGE.md`](../../rust_tools_suite/ARCHITECTURE_AND_USAGE.md)。
