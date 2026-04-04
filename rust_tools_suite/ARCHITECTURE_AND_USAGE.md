# Rust Tools Suite - 架构与使用文档

## 1. 定位

`rust_tools_suite` 是工作区统一保留的聚合式桌面工具目录，用于承载高频文本处理、格式化、检查和调试工具。它取代了旧的 `rust_micro_tools` 与 `rust_indie_tools` 双目录模式。

## 2. 结构

```text
rust_tools_suite/
├─ Cargo.toml
├─ README.md
├─ ARCHITECTURE_AND_USAGE.md
├─ packaging/
│  ├─ package_deb.sh
│  └─ linux/
├─ src/
│  ├─ app.rs
│  ├─ file_ops.rs
│  ├─ guide.rs
│  ├─ i18n.rs
│  ├─ main.rs
│  ├─ settings.rs
│  ├─ theme.rs
│  ├─ workflow.rs
│  └─ tools/
```

## 3. 架构分层

- `main.rs`
  入口、CLI 参数、窗口配置、字体 fallback 初始化。
- `app.rs`
  全局应用壳层，负责菜单、状态栏、响应式布局、对话框、快捷键、偏好持久化和活动工具调度。
- `file_ops.rs`
  文件导入与另存为结果。
- `theme.rs`
  深浅色主题和 CJK 字体 fallback。
- `workflow.rs`
  闭环流程面板。
- `tools/*.rs`
  单工具实现，每个工具负责自己的 UI、执行逻辑、输入输出接口和测试。

## 4. 响应式布局

- 宽屏模式：
  主内容在中央，流程面板在右侧。
- 紧凑模式：
  主内容优先，流程面板切换为底部抽屉。
- 工具切换区：
  宽度充足时显示按钮组，宽度不足时退化为下拉选择。
- 长文本区域：
  通过外层滚动容器避免溢出与重叠。
- JWT 工具：
  宽屏双栏显示 Header / Payload，窄屏自动上下堆叠。

## 5. 菜单与全局能力

- `File`
  导入输入文件、加载示例、另存为结果、偏好设置、退出。
- `Edit`
  复制当前结果、清空当前工具。
- `View`
  流程面板开关、主题切换、UI 缩放。
- `Tools`
  10 款工具直达。
- `Help`
  About、快捷键、文档。
- `Language`
  中文 / English 切换。

## 6. 文件 I/O 约定

- 文本类工具统一支持：
  - 导入输入文件
  - 剪贴板复制
  - 另存为结果
- 对不适合文件输入的工具，例如 UUID 批量生成，菜单中的导入动作会保持禁用。

## 7. JWT 工具范围

- 解析：
  - Header
  - Payload
- 验签：
  - HS256
  - RS256
- 密钥输入：
  - 文本框粘贴
  - 文件导入
- 当前刻意不做：
  - ES256
  - JWK 集管理
  - 复杂 claims 策略校验

## 8. 打包与发布

- Windows：
  发布 `rust_tools_suite.exe`。
- Debian：
  通过 `packaging/package_deb.sh` 构建单个 `.deb`，同时安装 `robot_control_rust` 与 `rust_tools_suite` 两个桌面应用。
- Release 资产：
  - `robot_control_rust.exe`
  - `rust_tools_suite.exe`
  - `rust-tools-suite_<version>_amd64.deb`
  - `RobotControlSuite_Setup.exe`
  - `checksums-sha256.txt`

## 9. 测试策略

- 单元测试：
  核心工具逻辑、JWT 验签、CSV/Regex 回归、响应式断点。
- 质量门禁：
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- 发布前验证：
  - 默认窗口、最小窗口、窄宽度
  - 125% / 150% UI 缩放
  - 中文 / English 切换
  - Linux 字体 fallback
  - Debian 包安装与卸载
