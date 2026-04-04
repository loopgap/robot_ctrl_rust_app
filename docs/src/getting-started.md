# 快速入门

## 克隆与构建

```powershell
git clone https://github.com/loopgap/robot_ctrl_rust_app.git
cd rust_serial
cargo build --release --manifest-path robot_control_rust/Cargo.toml
cargo build --release --manifest-path rust_tools_suite/Cargo.toml
```

## 运行两个正式产品

```powershell
# 机器人控制主应用
cargo run --release --manifest-path robot_control_rust/Cargo.toml

# 工具套件
cargo run --release --manifest-path rust_tools_suite/Cargo.toml
```

启动后如果从菜单点击“文档 / Documentation”，程序会优先打开包内的 `docs/index.html`；若未构建完整手册，再回退到仓库内的 `docs/help/index.html` 或安装目录内的 `help_index.html`。

## 本地验证

```powershell
.\make.ps1 workflow-seal
.\make.ps1 check
cargo test --manifest-path robot_control_rust/Cargo.toml
cargo test --manifest-path rust_tools_suite/Cargo.toml
```

## 推荐阅读

- [机器人主控](robot-control/README.md)
- [工具套件](micro-tools/README.md)
- [本地帮助页说明](local-help.md)
- [工具套件架构](tools-suite-architecture.md)
- [发布操作手册](operations-release.md)
