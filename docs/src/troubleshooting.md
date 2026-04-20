# 智能排障百科

> 常见问题与解决方案手册

## 构建问题

### Q1: Windows 平台编译失败，提示缺少 serialport 依赖

**症状**:
```
error: could not find native static library `libudev`, or its dependencies
```

**原因**: 缺少 `libudev` 开发库（Linux 平台需要）

**解决方案**:
1. 安装 Visual Studio Build Tools，并确保选择 **"C++ 构建工具"** 工作负载
2. 或者使用预编译的 serialport crate

### Q2: cargo build 速度很慢

**症状**: 首次构建耗时过长

**解决方案**:
1. 配置国内 Cargo 镜像加速依赖下载
2. 使用 `cargo check` 替代完整构建进行快速检查
3. 复用之前的 target 目录缓存

**镜像配置** (在 `~/.cargo/config.toml` 中):
```toml
[source.crates-io]
replace-with = "ustc"

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
```

### Q3: 依赖安全警告 (cargo audit)

**症状**:
```
warning: Security reports have been reported for the following crates
```

**解决方案**:
1. 检查是否有修复版本可用
2. 更新依赖版本
3. 如需暂时忽略，添加白名单配置

## 运行问题

### Q4: 串口无法打开

**症状**: 连接串口时提示 "端口不存在" 或 "无法访问"

**排查步骤**:
1. 确认串口设备已连接
2. 检查端口名称是否正确（Windows: `COM1`, Linux: `/dev/ttyUSB0`）
3. 确认端口未被其他程序占用
4. 检查用户权限（Linux: 需要 dialout 组权限）

**解决方案**:
```powershell
# Windows: 查看可用串口
python -m serial.tools.list_ports

# Linux: 添加用户到 dialout 组
sudo usermod -a -G dialout $USER
```

### Q5: TCP/UDP 连接失败

**症状**: 无法建立 TCP/UDP 连接

**排查步骤**:
1. 确认目标 IP 和端口是否正确
2. 检查防火墙设置
3. 确认目标服务已启动
4. 检查网络连通性 (`ping`, `telnet`)

**解决方案**:
```powershell
# 测试端口连通性
telnet 192.168.1.100 8080

# 检查防火墙规则 (Windows)
netsh advfirewall firewall show rule name=all
```

### Q6: CAN 总线通信异常

**症状**: CAN 报文发送/接收失败

**排查步骤**:
1. 确认 CAN 波特率匹配（常用: 125k, 250k, 500k, 1M）
2. 检查终端电阻是否正确配置（120Ω）
3. 确认 CAN_H 和 CAN_L 线序正确
4. 检查总线负载是否过高

**解决方案**:
- 使用 CAN 分析仪检测总线电平
- 确保总线两端添加终端电阻
- 降低总线负载或增加 CAN 网关

## 代码质量问题

### Q7: Clippy 警告无法理解

**症状**: Clippy 给出复杂的警告信息

**解决方案**:
1. 使用 `cargo clippy --explain WARNING_CODE` 查看详细解释
2. 添加 `#![allow(clippy::warning_name)]` 忽略特定警告（谨慎使用）
3. 查阅 [Clippy 文档](https://rust-lang.github.io/rust-clippy/)

### Q8: 测试失败

**症状**: `cargo test` 显示测试用例失败

**排查步骤**:
1. 查看失败的测试名称和错误信息
2. 确认是否是新代码引入的问题
3. 检查测试环境是否正确配置

**解决方案**:
```powershell
# 运行特定测试
cargo test test_name

# 查看详细输出
cargo test -- --nocapture

# 运行 Debug 测试
cargo test --no-fail-fast
```

### Q9: 格式化检查失败

**症状**: `cargo fmt --check` 报错

**解决方案**:
```powershell
# 自动修复格式问题
cargo fmt

# 或使用审查脚本
cd .\scripts\go\rusktask
go run . review --quick --fix
```

## 应用功能问题

### Q10: MCP Server 无法启动

**症状**: 启用 MCP Server 后无响应

**排查步骤**:
1. 确认端口未被占用（默认: 8080）
2. 检查防火墙允许该端口
3. 确认 JSON-RPC 请求格式正确

**解决方案**:
```powershell
# 检查端口占用
netstat -ano | findstr 8080

# 查看 MCP 服务日志
RUST_LOG=debug cargo run
```

### Q11: LLM API 调用失败

**症状**: 神经网络调参或 LLM 建议功能无法使用

**排查步骤**:
1. 确认 API URL 配置正确
2. 检查网络代理设置
3. 验证 API Key 有效性
4. 查看 API 配额是否耗尽

### Q12: 数据可视化刷新慢

**症状**: 波形/图表刷新不及时

**解决方案**:
1. 降低数据采样率
2. 减少同时显示的数据点数
3. 使用 Release 模式运行
4. 检查是否有内存泄漏

## 诊断命令速查

### 环境诊断

```powershell
# Rust 环境
rustc --version
cargo --version

# 串口可用性 (Windows)
python -m serial.tools.list_ports

# 端口占用检查
netstat -ano | findstr <PORT>

# 网络连通性
ping <HOST>
telnet <HOST> <PORT>
```

### 代码诊断

```powershell
# 完整预检
.\scripts\task.ps1 preflight

# 格式检查
cargo fmt --check

# Clippy 分析
cargo clippy --all-targets

# 运行测试
cargo test

# 安全审计
cargo audit
```

## 错误信息对照表

| 错误代码 | 含义 | 解决方案 |
|----------|------|----------|
| `EACCES` | 权限不足 | 以管理员权限运行或检查文件权限 |
| `ENOENT` | 文件/路径不存在 | 检查路径配置是否正确 |
| `ECONNREFUSED` | 连接被拒绝 | 确认目标服务已启动 |
| `ETIMEDOUT` | 连接超时 | 检查网络连通性和防火墙 |
| `EADDRINUSE` | 端口已被占用 | 更换端口或停止占用进程 |

## 获取帮助

### 社区支持

- 提交 GitHub Issue
- 查看项目 Wiki

### 调试日志

启用详细日志获取更多信息：

```powershell
# 设置日志级别
$env:RUST_LOG = "debug"

# 运行应用
cargo run --release
```

## 相关文档

- [快速入门](getting-started.md) - 环境准备与首次运行
- [开发与工作流](workflow.md) - Git 工作流与自动化