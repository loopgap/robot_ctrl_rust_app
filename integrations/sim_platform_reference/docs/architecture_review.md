# sim_platform 深度架构审查报告

**审查人**: 高见远（Gao）· 架构师  
**日期**: 2026-06-04  
**项目版本**: v1.2.0  
**Python**: 3.13  
**测试**: 984 passed

---

## 目录

1. [依赖方向违规](#1-依赖方向违规)
2. [循环依赖](#2-循环依赖)
3. [接口一致性](#3-接口一致性)
4. [状态管理](#4-状态管理)
5. [错误处理](#5-错误处理)
6. [配置管理](#6-配置管理)
7. [性能瓶颈](#7-性能瓶颈)
8. [综合评级与优先修复清单](#8-综合评级与优先修复清单)

---

## 1. 依赖方向违规

**预期依赖方向**: `core/` ← `models/` ← `tools/` ← `verification/`  
（下层不应依赖上层）

### 问题 1.1: tools → verification 依赖 ⚠️ P1

**位置**: `tools/visualization/interactive_runner.py` 第 290-291 行

```python
from sim_platform.verification.fault_injection.injector import (
    FaultConfig, FaultInjector)
```

`interactive_runner.py`（tools 层）直接导入了 `verification/fault_injection/injector.py`（verification 层）。这违反了 `tools ← verification` 的依赖方向。代码中注释说 "lazy import to avoid tools→verification dependency"，但实际上这个 lazy import 仍然建立了运行时依赖关系。

**修复建议**:
- **方案 A（推荐）**: 将 `FaultInjector` 和 `FaultConfig` 提升到 `core/` 层或 `models/fault/` 层，因为 fault injection 是一个通用仿真概念，不应归属 verification。
- **方案 B**: 在 `tools/` 层定义一个 `FaultInjectorProtocol` 接口，由 verification 层实现注入。

### 问题 1.2: examples → verification 依赖 ⚠️ P1

**位置**: `examples/pmsm_foc_mvp/main.py` 第 33 行

```python
from sim_platform.verification.fault_injection.injector import FaultConfig, FaultInjector
```

示例代码（examples 层）直接导入 verification 层。同样违反依赖方向。

**修复建议**: 同 1.1，将 FaultInjector 下移到 models 层。

### 问题 1.3: verification → tools 依赖（双向依赖） ⚠️ P2

**位置**: 多个 verification 测试文件导入 tools 层：
- `verification/test_cases/test_config.py` → `tools.config`
- `verification/test_cases/test_config_deep.py` → `tools.config`
- `verification/test_cases/test_tui.py` → `tools.tui`
- `verification/test_cases/test_tui_ux.py` → `tools.tui`
- `verification/test_cases/test_coverage_boost.py` → `tools.visualization`

这是 **正常的**（上层依赖下层），但结合 1.1，形成了 **tools ↔ verification 双向依赖**。

**修复建议**: 测试对被测模块的依赖是正常的，但应确保 tools 不依赖 verification。

### 问题 1.4: 常量重复定义 ⚠️ P2

**位置**: `core/data_bus.py` 第 30-33 行

```python
_MAX_MODULE_ID_LEN = 256
_MAX_HISTORY = 10000
_MAX_EVENTS = 50000
```

而 `core/constants.py` 第 59-62 行已定义：
```python
MAX_MODULE_ID_LEN: int = 256
MAX_HISTORY: int = 10000
MAX_EVENTS: int = 50000
```

`data_bus.py` 没有从 `constants.py` 导入，而是自行定义了相同的值。同理，`orchestrator.py` 第 32 行也重复定义了 `_MAX_TOTAL_STEPS = 1_000_000_000`。

**修复建议**: `data_bus.py` 和 `orchestrator.py` 应从 `core.constants` 导入这些常量，删除本地重复定义。

---

## 2. 循环依赖

### 问题 2.1: 无模块级循环引用 ✅

经过完整的 import 图分析，项目不存在模块级循环引用。各层之间的依赖方向总体正确：
- `core/` 不依赖任何其他层
- `models/` 只依赖 `core/`
- `tools/` 依赖 `core/` + `models/` + **`verification/`（违规）**
- `verification/` 依赖 `core/` + `models/` + `tools/`

### 问题 2.2: 潜在的运行时循环风险 ⚠️ P3

`tools/visualization/interactive_runner.py` 通过 lazy import 延迟加载 `verification/fault_injection`。如果未来 `FaultInjector` 初始化时需要从 tools 导入任何东西，就会形成真正的循环。

**建议**: 将 FaultInjector 移至 models 层可彻底消除此风险。

---

## 3. 接口一致性

### 问题 3.1: step() 签名不一致 ⚠️ P0

各模型的 `step()` 方法签名差异过大，没有统一接口：

| 模型 | step() 签名 | 备注 |
|------|------------|------|
| `PMSMdqModel` | `step(vd, vq, tl=0.0, dt=None)` | dq 电压输入 |
| `PMSMAdvanced` | `step(vd, vq, tl=0.0, dt=None, winding_temp=None)` | 继承 PMSM + 温度参数 |
| `BLDCModel` | `step(v_bus, tl=0.0, dt=None, direction=1)` | **完全不同的输入**：单一直流母线电压 |
| `IMdqModel` | `step(vsd, vsq, omega_e, tl=0.0, dt=None)` | **额外参数** omega_e |
| `ThermalNode` | `step(P_loss, dt)` | **仅两个参数** |
| `IdealBattery` | `step(i_load=0.0)` | 仅一个参数 |
| `RintBattery` | `step(i_load=0.0)` | 仅一个参数 |
| `AverageInverter` | `step(duty_a, duty_b, duty_c, v_bus=None, ia=0.0, ...)` | 7 个参数 |

**核心问题**: 没有抽象基类或 Protocol 定义统一的 `step()` 接口。Orchestrator 无法通过统一接口调用不同模型。

**修复建议**:
```python
from abc import ABC, abstractmethod
from typing import Protocol

class SteppableModel(Protocol):
    """All simulation models must implement this protocol."""
    def step(self, dt: float) -> None: ...
    def reset(self) -> None: ...
    def get_state(self) -> dict: ...
```

建议创建 `core/interfaces.py` 定义 `Steppable`、`EnergyProvider`、`SensorInterface` 等 Protocol。

### 问题 3.2: update() vs step() 命名不一致 ⚠️ P1

- 控制器使用 `update()`: `FOCController.update()`, `PIController.update()`, `SpeedController.update()`
- 物理模型使用 `step()`: `PMSMdqModel.step()`, `BLDCModel.step()`, `ThermalNode.step()`
- 但 `IMVectorController.update_speed()` 不叫 `step()` 也不叫 `update()`

**修复建议**: 统一命名——物理模型用 `step(dt)`，控制器用 `update()`。

### 问题 3.3: reset() 接口基本一致 ✅ / 缺失 get_state() ⚠️ P1

所有物理模型和控制器都实现了 `reset() -> None`，这部分一致。

但 `ThermalNode`、`IdealBattery`、`RintBattery`、`AverageInverter` **缺少 `get_state()` 方法**。Orchestrator 的 `_energy_audit()` 通过 `hasattr` 检测 `get_power_input` 等方法，这是一种脆弱的鸭子类型。

**修复建议**: 定义统一接口，要求所有模型实现 `get_state() -> dict`。

### 问题 3.4: dt 转换逻辑重复 ⚠️ P2

以下 4 个模型有完全相同的 `dt_ns → dt` 转换代码：

**pmsm_dq.py** 第 60-64 行:
```python
if dt_ns > 0:
    self.dt = dt_ns * 1e-9
else:
    self.dt = 50e-6
self.dt = max(self.dt, 1e-12)
```

**bldc.py** 第 105-109 行、**im_dq.py** 第 71-75 行完全相同。

**修复建议**: 将此逻辑提取到 `core/utils.py`:
```python
def resolve_dt(dt_ns: int, default_dt: float = 50e-6) -> float:
    if dt_ns > 0:
        return max(dt_ns * 1e-9, 1e-12)
    return max(default_dt, 1e-12)
```

---

## 4. 状态管理

### 问题 4.1: PMSMAdvanced.step() 重复计算饱和电感 ⚠️ P1

**位置**: `models/motor/pmsm_advanced.py`

在 `step()` 方法中，第 216 行调用了一次 `_get_saturated_inductance()`，然后在 `get_state()` 中（第 276-277 行）又调用了两次：
```python
"Ld_effective": self._get_saturated_inductance(self.id, self.iq)[0],
"Lq_effective": self._get_saturated_inductance(self.id, self.iq)[1],
```

**修复建议**: 在 `step()` 中将饱和电感缓存到 `self._cached_Ld` 和 `self._cached_Lq`，`get_state()` 直接使用缓存值。

### 问题 4.2: IMdqModel.get_state() 隐式修改状态 ⚠️ P1

**位置**: `models/motor/im_dq.py` 第 302-304 行

```python
def get_state(self) -> dict:
    """Get current state as dictionary."""
    self.update_abc_currents()  # ← 这会修改 self.ia, self.ib, self.ic
    return { ... }
```

`get_state()` 应该是只读操作，但 `IMdqModel.get_state()` 会隐式调用 `update_abc_currents()` 修改内部状态。这违反了查询方法不修改状态的原则。

**修复建议**: 将 `update_abc_currents()` 的调用移到 `step()` 方法内部，使 `get_state()` 成为纯读操作。

### 问题 4.3: EKF predict/update 异常静默降级 ⚠️ P2

**位置**: `models/controller/ekf.py` 第 145-148 行、第 210-213 行

```python
except Exception:
    # Fallback: identity prediction
    x_pred = x.copy()
    P_pred = self.P + self.Q
```

EKF 的 `predict()` 和 `update()` 方法在异常时静默降级（保持原值），不记录日志。这在长时间仿真中可能导致状态估计严重偏离而无法被发现。

**修复建议**: 至少添加 `logger.warning()` 记录异常降级事件。

### 问题 4.4: MPC predict() 异常吞掉具体异常 ⚠️ P2

**位置**: `models/controller/mpc.py` 第 106-110 行

```python
try:
    x_next = model(x_current, u_current)
    x_next = _guard_numeric(x_next, 0.0)
except Exception:
    x_next = 0.0  # ← 将预测值硬编码为 0，可能严重影响控制质量
```

将异常状态直接置零会导致 MPC 控制器在模型函数偶尔出错时产生剧烈控制信号。

**修复建议**: 记录异常并使用前一步的预测值作为 fallback，而非 0.0。

---

## 5. 错误处理

### 问题 5.1: Orchestrator.run() init 失败仅记录日志继续 ⚠️ P1

**位置**: `core/orchestrator.py` 第 176-181 行

```python
for solver_id, init_fn in self._initializers.items():
    try:
        logger.debug("Initializing %s", solver_id)
        init_fn()
    except Exception:
        logger.exception("Init failed for %s, skipping", solver_id)
```

如果某个模型初始化失败，Orchestrator 仅记录日志后跳过。后续 step() 阶段引用未初始化的模型会导致不可预测的错误。

**修复建议**: 初始化失败应抛出 `RuntimeError` 或在 `_steppers` 中标记该 solver 为无效，跳过其 step。

### 问题 5.2: Orchestrator.run() solver 崩溃后继续仿真 ⚠️ P2

**位置**: `core/orchestrator.py` 第 208-216 行

```python
try:
    result = stepper(current_step_ns)
    if not result.converged:
        all_converged = False
except Exception:
    logger.exception("Solver %s crashed at step %d", solver_id, i)
    all_converged = False
```

Solver 崩溃后仅标记不收敛，继续下一个 solver 和下一步。在耦合仿真中（如 FOC + 电机），如果一个 solver 崩溃，其他 solver 的输入已经无效，继续仿真是没有意义的。

**修复建议**: 添加配置项 `abort_on_solver_failure: bool = True`，默认在 solver 崩溃时终止仿真。

### 问题 5.3: 验证层测试中的裸 except ⚠️ P3

**位置**: `verification/test_cases/stress_test.py` 第 183 行

```python
except Exception:
    pass
```

以及 `test_security_deep_attack.py` 多处 `except Exception: pass`。虽然测试中使用 `pass` 可以接受（用于测试异常场景），但 `stress_test.py` 中的 `pass` 会导致压力测试的错误被静默吞掉。

### 问题 5.4: HDF5Logger.verify_integrity() 裸 except ⚠️ P3

**位置**: `tools/replay/hdf5_logger.py` 第 123-127 行

```python
@staticmethod
def verify_integrity(filepath: str) -> bool:
    try:
        with h5py.File(filepath, "r") as f:
            return f.attrs.get("_integrity") == "COMPLETE"
    except Exception:
        return False  # ← 文件不存在 vs 文件损坏 都返回 False，无法区分
```

**修复建议**: 捕获具体异常类型（`FileNotFoundError`, `OSError`, `KeyError`）。

---

## 6. 配置管理

### 问题 6.1: 硬编码 48V 母线电压散布全局 ⚠️ P0

**最严重的配置管理问题。**

`48.0` 作为默认母线电压出现在至少 **30+ 处**：

| 文件 | 行号 | 代码 |
|------|------|------|
| `models/power/power_models.py` | 13, 14, 24, 25, 42, 43 | `v_nom=48.0`, `v_oc=48.0`, `v_bus=48.0` |
| `models/controller/foc.py` | 205, 213 | `v_bus: float = 48.0` |
| `tools/visualization/interactive_runner.py` | 275, 276, 284 | `v_oc=48.0`, `v_bus=48.0` |
| `tools/tui/screens/run.py` | - | `v_bus=48.0` |
| `tools/tui/screens/scan.py` | - | `v_bus=48.0` |
| `tools/visualization/parameter_scan.py` | - | `v_bus=48.0` |
| 30+ 处测试文件 | - | `v_bus=48.0` |

`constants.py` 中已定义 `DEFAULT_V_BUS: float = 48.0`，但**几乎没有文件使用它**。

**修复建议**: 全局搜索替换 `48.0` 默认值 → `DEFAULT_V_BUS`（从 `core.constants` 导入）。

### 问题 6.2: 硬编码 50μs 时间步 ⚠️ P1

`50e-6` 作为默认仿真步长出现在：
- `constants.py` 第 38 行: `DEFAULT_DT_S: float = 50e-6`（已定义但未使用）
- `pmsm_dq.py` 第 63 行: `self.dt = 50e-6`
- `bldc.py` 第 108 行: `self.dt = 50e-6`
- `im_dq.py` 第 74 行: `self.dt = 50e-6`
- `foc.py` 第 212 行: `ts = max(_guard_numeric(ts, 50e-6), 1e-12)`
- `mpc.py` 第 37, 71, 264 行
- `ekf.py` 第 269 行
- 多处测试文件

**修复建议**: 所有 `50e-6` 默认值应替换为 `DEFAULT_DT_S`。

### 问题 6.3: 硬编码安全边界常量 ⚠️ P2

以下值在多个文件中硬编码而非使用 `constants.py`：

| 常量 | constants.py 定义 | 实际使用 |
|------|-----------------|---------|
| `1e-12` (零除保护) | `NUMERIC_EPS` | `foc.py:133`, `foc.py:146`, `foc.py:181`, `im_dq.py:75`, `pmsm_dq.py:64` 等 20+ 处直接使用 `1e-12` |
| `1e-9` (最小电感) | `MOTOR_EPS_L` | 部分使用，部分硬编码 |
| `1e-15` (最小惯量) | `MOTOR_EPS_J` | 部分使用 |
| `200.0` (最大电流) | `DEFAULT_I_MAX` | 仅 `pmsm_advanced.py` 使用 |

**修复建议**: 全量审计硬编码数值，统一从 `constants.py` 导入。

### 问题 6.4: thermal_model.py 和 sensor_fusion.py 完全不使用 constants.py ⚠️ P2

**位置**: `models/thermal/thermal_model.py` 全文, `models/fusion/sensor_fusion.py` 全文

这两个模块没有从 `core.constants` 导入任何常量，所有数值参数直接硬编码：
- `thermal_model.py`: `alpha_cu = 0.00393`（铜温度系数）, `alpha_mag = -0.0012`（磁铁温度系数）
- `sensor_fusion.py`: `Q=0.01`, `R=1.0` 等滤波器参数

**修复建议**: 将物理常数（铜温度系数、磁铁温度系数等）添加到 `constants.py`。

### 问题 6.5: power_models.py 不使用 guard_numeric ⚠️ P2

**位置**: `models/power/power_models.py` 全文

`power_models.py` 是唯一不使用 `core.utils.guard_numeric` 的 models 模块。它使用原始的 `math.isfinite()` 检查：

```python
self.v_nom = v_nom if math.isfinite(v_nom) else 48.0
```

而非：
```python
self.v_nom = _guard_numeric(v_nom, DEFAULT_V_BUS)
```

这违反了 "单一数值安全检查源" 的原则。

**修复建议**: 统一使用 `guard_numeric` 并导入 `DEFAULT_V_BUS` 等常量。

---

## 7. 性能瓶颈

### 问题 7.1: DataBus 历史记录 O(n) 截断 ⚠️ P1

**位置**: `core/data_bus.py` 第 270-272 行

```python
hist = self._history[topic]
hist.append(signal)
if len(hist) > _MAX_HISTORY:
    self._history[topic] = hist[-_MAX_HISTORY:]
```

当历史记录超过 `_MAX_HISTORY`（10000）时，每次溢出都会创建一个新的切片列表，这是 **O(n) 操作**。在高频发布场景下（如 20kHz 仿真），每 10000 次发布就有一次 O(n) 拷贝。

**修复建议**: 使用 `collections.deque(maxlen=_MAX_HISTORY)` 替代 list：
```python
from collections import deque
self._history: Dict[str, deque] = defaultdict(lambda: deque(maxlen=_MAX_HISTORY))
```

### 问题 7.2: FaultInjector.apply() 线性扫描 ⚠️ P2

**位置**: `verification/fault_injection/injector.py` 第 121-135 行

```python
def apply(self, path: str, value: float, sim_time_s: float) -> float:
    for state in self._faults.values():
        if state.config.target_path != path:
            continue
```

`apply()` 每次调用都遍历所有已注册的 fault 配置。在有多个 fault 且高频调用（20kHz）的场景下，这是不必要的开销。

**修复建议**: 使用 `Dict[str, List[FaultState]]` 按 `target_path` 索引，实现 O(1) 查找。

### 问题 7.3: Orchestrator.schedule_fault() deque→list 转换 ⚠️ P2

**位置**: `core/orchestrator.py` 第 143-147 行

```python
fault_list = list(self._fault_queue)
insertion_point = bisect.bisect_left([f[0] for f in fault_list], fault_ns)
fault_list.insert(insertion_point, (fault_ns, fault_fn))
self._fault_queue = deque(fault_list)
```

每次插入 fault 都执行：deque→list 转换 + bisect + list.insert + list→deque 转换。时间复杂度 O(n)。

**修复建议**: 使用 `sortedcontainers.SortedList` 或直接用 `list`（因为 fault 数量通常很少，list 的 O(n) 插入可能比频繁的 deque↔list 转换更快）。

### 问题 7.4: EKF predict/update 中的逐元素 NaN 检查 ⚠️ P3

**位置**: `models/controller/ekf.py` 第 129-130 行

```python
x = np.array([_guard_numeric(xi, 0.0) for xi in x])
u = np.array([_guard_numeric(ui, 0.0) for ui in u])
```

每次 EKF 更新都对 numpy 数组逐元素调用 Python `_guard_numeric()` 函数。对于小维度（4×4）影响不大，但如果扩展到更高维状态，应使用向量化操作：
```python
x = np.where(np.isfinite(x), x, 0.0)
```

### 问题 7.5: MPC 数值梯度计算 O(Nc × Np × max_iter) ⚠️ P3

**位置**: `models/controller/mpc.py` 第 190-204 行

每次 `solve()` 调用执行 `max_iterations × Nc` 次 `predict()` + `compute_cost()`。默认配置 `max_iterations=50, Nc=3, Np=10`，每次 solve 共执行 300 次 predict + compute_cost。

在 20kHz 电流环中这不可接受。建议仅用于离线分析或低频速度环。

**建议**: 在文档中明确标注 MPCCurrentController 的计算开销，建议仅用于 <1kHz 控制频率。

---

## 8. 综合评级与优先修复清单

### 总体评级

| 维度 | 评级 | 说明 |
|------|------|------|
| 依赖方向 | **B+** | 仅 1 处违规（tools→verification），可通过重构消除 |
| 循环依赖 | **A** | 无循环引用 |
| 接口一致性 | **C+** | step() 签名混乱，缺少抽象接口 |
| 状态管理 | **B** | 大部分正确，get_state() 副作用和缓存缺失需修复 |
| 错误处理 | **B+** | 整体较好，但 init 失败静默跳过是隐患 |
| 配置管理 | **D+** | 硬编码 48V 和 50μs 遍布全局，constants.py 形同虚设 |
| 性能瓶颈 | **B** | DataBus 历史截断是主要瓶颈，其余可接受 |

### 优先修复清单（按影响 × 紧急度排序）

| 优先级 | 问题 | 影响 | 工作量 |
|--------|------|------|--------|
| **P0** | 6.1 硬编码 48V 母线电压 | 维护性灾难 | 中（全局搜索替换） |
| **P0** | 3.1 step() 无统一接口 | 无法构建通用调度器 | 大（需设计 Protocol） |
| **P1** | 1.1/1.2 tools→verification 依赖 | 架构腐化 | 中（重构 FaultInjector） |
| **P1** | 6.2 硬编码 50μs | 配置一致性 | 小 |
| **P1** | 4.1 饱和电感重复计算 | 性能 + 可维护性 | 小 |
| **P1** | 4.2 get_state() 副作用 | 正确性风险 | 小 |
| **P1** | 5.1 init 失败静默跳过 | 仿真可靠性 | 小 |
| **P1** | 7.1 DataBus O(n) 截断 | 高频仿真性能 | 小（改用 deque） |
| **P2** | 1.4 常量重复定义 | 维护性 | 小 |
| **P2** | 3.2 update/step 命名不一致 | 代码可读性 | 中 |
| **P2** | 5.2 solver 崩溃后继续仿真 | 仿真正确性 | 小 |
| **P2** | 6.3/6.4 安全边界常量硬编码 | 维护性 | 中 |
| **P2** | 7.2 FaultInjector 线性扫描 | 性能 | 小 |
| **P3** | 5.3/5.4 裸 except | 代码质量 | 小 |

---

## 附录：文件依赖图

```
core/
├── constants.py          ← 不依赖任何模块
├── utils.py              ← 不依赖任何模块  
├── clock.py              ← 不依赖任何模块
├── data_bus.py           ← 不依赖任何模块
├── model_registry.py     ← 不依赖任何模块
└── orchestrator.py       ← clock, data_bus, model_registry

models/
├── motor/
│   ├── pmsm_dq.py        ← core.constants, core.utils
│   ├── pmsm_advanced.py  ← core.constants, core.utils, pmsm_dq
│   ├── bldc.py           ← core.constants, core.utils
│   └── im_dq.py          ← core.constants, core.utils
├── controller/
│   ├── foc.py            ← core.constants, core.utils
│   ├── ekf.py            ← core.constants, core.utils, numpy
│   └── mpc.py            ← core.constants, core.utils
├── sensor/
│   └── sensors.py        ← core.utils
├── thermal/
│   └── thermal_model.py  ← (无 core 依赖!)
├── power/
│   └── power_models.py   ← (无 core 依赖!)
└── fusion/
    └── sensor_fusion.py  ← (无 core 依赖!)

tools/
├── visualization/
│   ├── interactive_runner.py  ← models.*, tools.replay, ❌ verification.fault_injection
│   ├── advanced_plot.py       ← matplotlib
│   ├── plot_log.py            ← matplotlib
│   └── parameter_scan.py      ← models.*, tools.replay
├── config/
│   ├── config_manager.py      ← tools.config.config_schema
│   └── config_schema.py       ← pydantic
├── replay/
│   └── hdf5_logger.py         ← h5py
└── tui/
    └── screens/
        ├── run.py             ← models.*, tools.replay, tools.visualization
        └── results.py         ← tools.visualization

verification/
├── fault_injection/
│   └── injector.py            ← (无 sim_platform 依赖，仅标准库)
└── test_cases/
    └── *.py                   ← core.*, models.*, tools.*, verification.fault_injection
```

---

*审查完成。如有疑问，请联系高见远。*
