# 仿真实验室

`Simulation Lab` 是 v0.2.0 引入的 Rust-native 仿真子系统。它吸收 `D:\Destop\test_ui\sim_platform` 的模型边界和测试思路，但发布运行时不携带 Python sidecar。

## 参考快照

参考资料位于 `integrations/sim_platform_reference/`：

- 保留 Python 源码、配置、示例、文档、测试、`pyproject.toml`、`requirements.txt`、`uv.lock`。
- 过滤 `.git`、虚拟环境、缓存、构建产物、字节码和生成位图。
- `NOTICE.md` 记录源路径、MIT 元数据、过滤策略和 baseline：2026-06-07 在源工作树执行 `pytest --collect-only -q verification`，实际收集 1128 个测试。

该目录只作为可审计参考，不参与 v0.2.0 runtime 启动路径。

## Rust Runtime 架构

核心实现位于 `robot_control_rust/src/simulation.rs`，UI 状态位于 `robot_control_rust/src/app/simulation_lab.rs`，页面位于 `robot_control_rust/src/views/simulation_lab.rs`。

| 层 | 职责 |
|----|------|
| Numeric guard | 拒绝 NaN/Inf、零步长、超大 step count，并对除法、电机参数、温度状态设置有限域保护 |
| Config schema | `SimulationConfig` 聚合场景时长、步长、速度给定、负载、PMSM 与 FOC 参数 |
| Registry / Bus / Clock | `ModelRegistry` 阻断重复模型 ID，`DataBus` 默认拒绝未授权 topic 写入，`GlobalClock` 统一仿真时间 |
| Models | PMSM dq、PMSM advanced、BLDC、IM dq、电池、逆变器、传感器、热节点 |
| Controllers | PI、FOC、速度环、MPC、EKF |
| Orchestrator | `run_pmsm_foc_with_hooks` 提供进度回调和取消钩子，`run_parameter_scan` 提供参数扫描 |
| Export | v0.2.0 输出 JSON/CSV，HDF5 行为保留在参考快照文档中，不进入本次 release 风险面 |

## UI 数据流

1. 用户在 `Simulation Lab` tab 输入场景参数。
2. `SimulationLabState::sync_config_from_text` 将文本转换为 `SimulationConfig` 并调用 `validate`。
3. Run/Scan 启动后台线程，UI 通过 mpsc channel 接收进度、结果或错误。
4. 取消只对可取消的 run worker 生效，取消结果会在 metrics 中标记 `cancelled`，不会假装成功。
5. 页面展示指标表、速度曲线、扫描表和 JSON/CSV 预览。

## 验证策略

仿真测试必须断言数值、边界和不变量，不接受只验证 “不 panic” 的空测试。

当前覆盖范围：

- Golden envelope：PMSM FOC step response 的最终速度、速度误差、峰值转矩和采样数量。
- 模型覆盖：BLDC six-step hall progression、IM dq finite step、thermal heat/cool、MPC control selection、EKF finite estimate。
- 对抗输入：NaN/Inf、零/负 dt、过大 step count、重复 model ID、未授权 bus topic、取消路径。
- 导出：CSV header、JSON metrics、参数扫描排序和非空结果。
- UI worker：文本参数解析、扫描值解析、导出预览截断。

发布前的本地门禁仍以 workspace 为单位执行：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release
$env:RUSTDOCFLAGS="-D warnings"; cargo doc --workspace --no-deps
.\scripts\task.ps1 preflight
```
