# 快速入门

> 帮助你快速上手 Rust Serial 工作区

## 环境要求

### 必需依赖

| 依赖项 | 版本要求 | 说明 |
|--------|----------|------|
| Rust | 1.70+ | 推荐使用 rustup 安装最新稳定版 |
| Cargo | 与 Rust 配套 | Rust 包管理器 |

### 可选依赖

| 依赖项 | 说明 |
|--------|------|
| Git | 版本控制（用于安装 Git Hooks） |
| Visual Studio Build Tools | Windows 平台编译串口依赖 |
| GCC / Clang | Linux/macOS 平台编译 |

## 环境安装

### 1. 安装 Rust

访问 [rustup.rs](https://rustup.rs) 或运行以下命令：

```powershell
# Windows (PowerShell)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s

# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s
```

### 2. 验证安装

```powershell
rustc --version
cargo --version
```

## 项目构建

### 克隆项目

```powershell
git clone https://github.com/loopgap/robot_ctrl_rust_app.git
cd rust_serial
```

### 构建所有项目

```powershell
# Debug 构建
cargo build

# Release 构建 (推荐用于发布)
cargo build --release
```

### 构建特定子项目

```powershell
# 机器人控制主应用
cd robot_control_rust
cargo build --release

# 微型工具集
cd ../rust_micro_tools
cargo build --release

# 独立工具 - CSV 清洗工坊
cd ../rust_indie_tools/csv_cleaner_gui
cargo build --release
```

## 运行应用

### 运行机器人控制主应用

```powershell
cd robot_control_rust
cargo run --release
```

### 运行微型工具集

```powershell
cd rust_micro_tools
cargo run --release
```

### 运行独立 GUI 工具

```powershell
# CSV 清洗工坊
cd rust_indie_tools/csv_cleaner_gui
cargo run --release

# JWT 解析工坊
cd rust_indie_tools/jwt_inspector_gui
cargo run --release

# Regex 巡检工坊
cd rust_indie_tools/regex_workbench_gui
cargo run --release
```

## 本地预检

在提交代码或创建 PR 之前，运行预检脚本确保代码质量：

### Windows

```powershell
.\scripts\preflight.ps1
```

### Linux/macOS

```bash
./scripts/preflight.sh
```

预检项目包括：
- 代码格式检查 (`cargo fmt`)
- Clippy 静态分析
- 依赖安全审计
- 测试套件运行

## 安装 Git Hooks

本地安装开发钩子，实现提交前自动校验：

```powershell
.\scripts\install-hooks.ps1
```

安装后，以下钩子将自动执行：
- **pre-commit**: 提交前格式检查与测试
- **pre-push**: 推送前完整预检
- **commit-msg**: 提交信息规范检查

## 常见问题

### Q: Windows 平台编译失败，提示缺少 serialport 依赖

**解决方案**: 安装 Visual Studio Build Tools，并确保选择 "C++ 构建工具" 工作负载。

### Q: cargo build 速度很慢

**解决方案**:
1. 使用 `cargo build --release` 而非 debug 构建
2. 配置 cargo 镜像源加速依赖下载
3. 使用 `cargo check` 替代完整构建进行快速检查

### Q: 如何查看更详细的构建日志

**解决方案**:
```powershell
RUST_LOG=debug cargo build --release
```

## 下一步

- 深入了解 [机器人主控应用](robot-control/README.md) 的完整功能
- 探索 [微型工具集](micro-tools/README.md) 的使用方式
- 查看 [开发与工作流](workflow.md) 了解项目规范