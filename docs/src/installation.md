# 安装指南

## 系统要求

- Windows 10+
- macOS 10.15+
- Debian / Ubuntu 系 Linux
- Rust stable

## 构建依赖

### Windows

- Visual Studio Build Tools
- Windows SDK

### Debian / Ubuntu

```bash
sudo apt update
sudo apt install build-essential pkg-config libudev-dev libgtk-3-dev dpkg-dev
```

### macOS

```bash
xcode-select --install
```

## 本地构建

```powershell
cargo build --release --manifest-path robot_control_rust/Cargo.toml
cargo build --release --manifest-path rust_tools_suite/Cargo.toml
```

## Linux Debian 包

构建：

```bash
chmod +x rust_tools_suite/packaging/package_deb.sh
./rust_tools_suite/packaging/package_deb.sh
```

安装：

```bash
sudo dpkg -i dist/debian/rust-tools-suite_<version>_amd64.deb
```

卸载：

```bash
sudo dpkg -r rust-tools-suite
```

## Linux 验证点

- `rust_tools_suite --help`
- `robot_control_rust --version`
- `/usr/share/applications/*.desktop`
- `/usr/share/icons/hicolor/scalable/apps/*.svg`
- 中文字体 fallback 正常
