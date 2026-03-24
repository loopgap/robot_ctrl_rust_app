# 安装指南

> Rust Serial 工作区完整安装说明

## 系统要求

### 最低要求

| 项目 | 要求 |
|------|------|
| 操作系统 | Windows 10+, macOS 10.15+, Ubuntu 20.04+ |
| 内存 | 4 GB RAM |
| 磁盘空间 | 2 GB 可用空间 |
| Rust | 1.70+ |

### 推荐配置

| 项目 | 推荐 |
|------|------|
| 操作系统 | Windows 11 / macOS 14+ / Ubuntu 22.04+ |
| 内存 | 8 GB RAM+ |
| 磁盘空间 | 10 GB+ |
| Rust | 最新稳定版 |

## 安装步骤

### 1. 安装 Rust

#### Windows

1. 下载 [rustup-init.exe](https://win.rustup.rs)
2. 运行安装程序
3. 按照提示完成安装
4. 重启终端

#### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

#### 验证安装

```powershell
rustc --version
cargo --version
```

### 2. 安装构建依赖

#### Windows

安装 Visual Studio Build Tools：

1. 下载 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio)
2. 安装时选择 **"C++ 构建工具"** 工作负载
3. 确保勾选 Windows 10/11 SDK

#### Ubuntu / Debian

```bash
sudo apt update
sudo apt install build-essential pkg-config libudev-dev
```

#### macOS

```bash
xcode-select --install
```

### 3. 克隆项目

```powershell
git clone https://github.com/loopgap/robot_ctrl_rust_app.git
cd rust_serial
```

### 4. 构建项目

#### 构建所有子项目

```powershell
# Debug 构建
cargo build --workspace

# Release 构建 (推荐用于发布)
cargo build --workspace --release
```

#### 构建特定项目

```powershell
# 机器人控制主应用
cd robot_control_rust
cargo build --release

# 微型工具集
cd ../rust_micro_tools
cargo build --release
```

## 可选组件安装

### Git Hooks (推荐)

```powershell
cd rust_serial
.\scripts\install-hooks.ps1
```

### mdBook 文档工具 (可选)

```bash
cargo install mdbook
```

### 预检脚本

预检脚本用于在提交前检查代码质量：

```powershell
# Windows
.\scripts\preflight.ps1

# Linux/macOS
chmod +x ./scripts/preflight.sh
./scripts/preflight.sh
```

## 运行应用

### 机器人控制主应用

```powershell
cd robot_control_rust
cargo run --release
```

### 微型工具集

```powershell
cd rust_micro_tools
cargo run --release
```

### 独立 GUI 工具

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

## Docker 部署 (可选)

### Dockerfile 示例

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/robot_control_rust /usr/local/bin/
CMD ["robot_control_rust"]
```

### 构建镜像

```bash
docker build -t rust-serial:latest .
```

### 运行容器

```bash
docker run -it --rm \
  --device /dev/ttyUSB0 \
  rust-serial:latest
```

## 卸载

### 移除构建产物

```bash
# Debug 构建
cargo clean

# Release 构建
rm -rf target/release
```

### 完全卸载

1. 删除项目目录
2. 移除 Rust (通过 rustup)
3. 移除 Visual Studio Build Tools (如不需要)

## 故障排除

### 编译错误: 找不到 serialport

**解决方案**: 安装 Windows 构建工具或 Linux libudev 开发库。

### 运行时错误: 权限不足

**解决方案**:
- Linux: 将用户加入 dialout 组 `sudo usermod -a -G dialout $USER`
- Windows: 以管理员权限运行

### 其他问题

请参阅 [智能排障百科](troubleshooting.md) 获取更多帮助。

## 下一步

- [快速入门](getting-started.md) - 了解如何使用
- [开发与工作流](workflow.md) - 学习项目规范
- [智能排障百科](troubleshooting.md) - 常见问题解决