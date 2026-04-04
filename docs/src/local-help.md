# 本地帮助页说明

`docs/help/index.html` 的职责很窄，但它不是无效文件。

## 它负责什么

- 这是桌面程序在缺少完整手册构建产物时使用的本地浏览器帮助页。
- `robot_control_rust` 和 `rust_tools_suite` 点击“文档 / Documentation”时，会优先打开包内 `docs/index.html`，没有完整手册时再回退到它。
- Windows 安装包和 portable bundle 会同时包含 `help_index.html` 与完整的 `docs/` 目录。

## 它不负责什么

- 它不是 `mdBook` 站点首页。
- 它不会替代 `docs/src` 下的章节内容。
- 它不会自动改写仓库根目录 `README.md`。

## 当前三套入口的关系

1. `docs/help/index.html`
   面向桌面程序的本地 HTML 帮助入口，适合浏览器直接打开。
2. `docs/src/*.md`
   面向 `mdBook` 的章节化手册，适合在线阅读、搜索和维护长文档。
3. `README.md`
   面向仓库首页的总览说明，负责工作区介绍和快速导航。

## 什么时候应该改它

- 需要补充“帮助菜单打开后第一屏看到什么”时，改这份 HTML。
- 需要补充安装、工作流、功能细节时，优先改 `docs/src` 里的 mdBook 章节。
- 需要改仓库首页展示时，改根目录 `README.md`。

## 当前程序中的实际查找位置

程序会优先尝试以下位置：

- 可执行文件同级的 `docs/index.html`
- 可执行文件同级的 `help_index.html`
- 仓库中的 `docs/help/index.html`

如果这些都不存在，程序才会回退到 GitHub 上的文档链接。



