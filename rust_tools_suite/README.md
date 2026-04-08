# Rust Tools Suite

`rust_tools_suite` 是当前工作区唯一保留的聚合式桌面工具目录，统一提供 10 款高频工具、响应式布局、文件导入导出、双语与闭环流程面板。

## 当前能力

- 10 款工具统一切换
- 深度的双系统支持 (Win8+ & Ubuntu 20+)：原生 Windows 文件读写锁防呆机制，无阻塞重试，消除 UI 冻结
- 顶部菜单：`File / Edit / View / Tools / Help / Language`
- 宽屏侧栏 + 窄屏流程抽屉
- 文件导入、另存为结果、剪贴板复制
- 深色 / 浅色主题与 UI 缩放
- CJK 字体 fallback
- JWT Header/Payload 解析与可选 HS256 / RS256 验签

## 工具清单

1. 校验和工坊
2. JSON 工坊
3. 日志巡检
4. URL 编解码
5. 时间戳转换
6. Base64 工坊
7. UUID 批量生成
8. CSV 清洗工坊
9. JWT 解析工坊
10. Regex 巡检工坊

## 目录结构

```text
rust_tools_suite/
├── Cargo.toml
├── README.md
├── ARCHITECTURE_AND_USAGE.md
├── packaging/
│   ├── package_deb.sh
│   └── linux/
├── src/
│   ├── app.rs
│   ├── file_ops.rs
│   ├── guide.rs
│   ├── i18n.rs
│   ├── settings.rs
│   ├── theme.rs
│   ├── workflow.rs
│   └── tools/
```

## 运行

```bash
cargo run --release --manifest-path rust_tools_suite/Cargo.toml
```

## 测试

```bash
cargo fmt --check --manifest-path rust_tools_suite/Cargo.toml
cargo clippy --manifest-path rust_tools_suite/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path rust_tools_suite/Cargo.toml
```

## 相关文档

- 架构说明：[`ARCHITECTURE_AND_USAGE.md`](ARCHITECTURE_AND_USAGE.md)
- 工作区总览：[`../README.md`](../README.md)
- 发布手册：[`../docs/src/operations-release.md`](../docs/src/operations-release.md)
