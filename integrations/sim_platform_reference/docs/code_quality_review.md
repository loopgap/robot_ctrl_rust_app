# sim_platform 代码质量与可维护性审查报告

**审查日期**: 2026-06-04
**审查范围**: core/, models/, tools/, verification/, examples/ (82个Python文件, 16,096行)

---

## 一、项目统计指标

| 指标 | 数值 |
|------|------|
| Python文件总数 | 82 |
| 代码总行数 | 16,096 |
| 生产代码类数 | ~45 |
| 生产代码方法数 | ~200 |
| TODO/FIXME/HACK | 0 |
| bare `except:` | 0 |
| `except Exception:` | 18 (其中6处在orchestrator) |

### 按目录分布

| 目录 | 文件数 | 职责 |
|------|--------|------|
| core/ | 7 | 框架核心(clock, databus, registry, orchestrator, utils, constants) |
| models/motor/ | 5 | 电机模型(PMSM, BLDC, IM) |
| models/controller/ | 4 | 控制器(FOC, MPC, EKF) |
| models/sensor/ | 2 | 传感器模型 |
| models/thermal/ | 2 | 热模型 |
| models/power/ | 2 | 功率电子模型 |
| models/fusion/ | 2 | 传感器融合 |
| tools/ | 16 | 配置/TUI/可视化/日志/性能 |
| verification/ | 26 | 测试与故障注入 |
| examples/ | 3 | 示例应用 |

---

## 二、DRY原则违反 (按严重性排序)

### 🔴 P0: Park/Clarke变换重复实现 (4处)

最严重的DRY违反。同一数学变换在4个文件中各自实现：

| 文件 | 行号 | 实现方式 |
|------|------|----------|
| `models/controller/foc.py` | 38-66 | 独立函数 `clarke_transform()`, `park_transform()`, `inverse_park()` |
| `models/motor/pmsm_dq.py` | 141-164 | 内联在 `update_abc_currents()` 和 `step_abc()` 中 |
| `models/motor/im_dq.py` | 245-288 | 内联在 `step_abc()` 和 `update_abc_currents()` 中 |
| `models/controller/ekf.py` | 345-382 | 内联在 `_observation()` 和 `_observation_jacobian()` 中 |

**影响**: 如果发现变换公式有误或需要优化(如使用CORDIC),需要改4处。

**建议**: 将 `foc.py` 中的 `clarke_transform`, `park_transform`, `inverse_park` 移至 `core/transforms.py`, 所有模块统一导入。

### 🔴 P0: NaN/Inf防护方式不一致

`core/utils.py` 提供了统一的 `guard_numeric()`, 但各模块使用方式不一致：

| 文件 | 使用 `_guard_numeric` | 手动 `math.isfinite()` |
|------|----------------------|----------------------|
| `pmsm_advanced.py` | ✅ 全部使用 | ❌ |
| `bldc.py` | ✅ 全部使用 | ❌ |
| `im_dq.py` | ✅ 全部使用 | ❌ |
| `mpc.py` | ✅ 全部使用 | ❌ |
| `ekf.py` | ✅ 全部使用 | ❌ |
| `foc.py` | ✅ 入口使用 | 部分内联 |
| `pmsm_dq.py` | ❌ 仅导入未使用 | ✅ **全部手动** (8处) |
| `thermal_model.py` | ❌ 未导入 | ✅ 全部手动 |
| `sensor_fusion.py` | ❌ 未导入 | ✅ 全部手动 |
| `sensors.py` | 部分(`__init__`) | ✅ `read()` 方法手动 |
| `power_models.py` | ❌ 未导入 | ✅ 全部手动 |

**关键问题**: `pmsm_dq.py` 导入了 `_guard_numeric` 但从未使用, 而是手动写 `x if math.isfinite(x) else 0.0`。

```python
# pmsm_dq.py:30 — 导入了但未使用
from sim_platform.core.utils import guard_numeric as _guard_numeric

# pmsm_dq.py:81 — 手动实现同样的逻辑
id_s = self.id if math.isfinite(self.id) else 0.0
```

### 🟠 P1: 常量 `2 * math.pi` 重复 ~25次

`2 * math.pi` 在整个代码库中出现约25次, 应定义为常量:

```python
# constants.py 中应添加:
TWO_PI: float = 2 * math.pi
```

出现位置: `bldc.py`(6处), `pmsm_dq.py`(1处), `pmsm_advanced.py`(2处), `im_dq.py`(2处), `ekf.py`(1处), `sensors.py`(2处), 以及所有测试文件。

### 🟠 P1: `math.sqrt(3)` 重复计算

| 文件 | 方式 |
|------|------|
| `foc.py` | ✅ 预计算 `_SQRT3_INV`, `_SQRT3_HALF` (模块级常量) |
| `pmsm_dq.py` | ✅ 预计算 `_SQRT3_INV`, `_SQRT3_HALF` (类级常量) |
| `im_dq.py` | ❌ 内联 `math.sqrt(3)` 3次 |
| `ekf.py` | ❌ 内联 `math.sqrt(3)/2` **6次** |

### 🟠 P1: PI控制器重复实现 (3处)

| 文件 | 实现 |
|------|------|
| `foc.py:123-193` | 完整 `PIController` 类 (含anti-windup, back-calculation) |
| `im_dq.py:390-400` | `_pi_controller()` 方法 (简化版anti-windup) |
| `bldc.py:397-431` | `BLDCController.update()` 内联PI逻辑 |

**建议**: 统一使用 `foc.py` 的 `PIController`, 移至 `core/` 或 `models/controller/base.py`。

### 🟡 P2: dt_ns → dt 转换模式重复 (3处)

```python
# pmsm_dq.py:61-65, bldc.py:106-110, im_dq.py:72-76 — 完全相同的代码
if dt_ns > 0:
    self.dt = dt_ns * 1e-9
else:
    self.dt = _DEFAULT_DT_S
self.dt = max(self.dt, 1e-12)
```

**建议**: 提取为 `core/utils.py` 中的 `resolve_dt(dt_ns: int) -> float` 工具函数。

### 🟡 P2: `convert_enums` 函数重复

- `tools/config/config_manager.py:156-165`
- `tools/config/config_schema.py:462-470`

完全相同的递归枚举转换函数, 定义了两次。

### 🟡 P2: `MAX_CONFIG_SIZE` 重复定义

- `core/constants.py:61`: `MAX_CONFIG_SIZE: int = 10 * 1024 * 1024`
- `tools/config/config_manager.py:38`: `MAX_CONFIG_SIZE = 10 * 1024 * 1024`

`config_manager.py` 没有从 `constants.py` 导入。

### 🟡 P2: 硬编码 π 值

`examples/pmsm_foc_mvp/main.py` 中使用硬编码的 π 值:

```python
# main.py:145
quantization=2 * 3.1415926535 / (2 ** cfg["sensor"]["encoder_quant_bits"])
# main.py:181, 263
cfg['scenario']['speed_ref_value']*60/(2*3.14159)
```

应使用 `math.pi`。

---

## 三、SOLID原则违反

### 🔴 P0: 无公共模型接口 — LSP/ISP违反

电机模型没有共同的基类或接口:

```python
# 三种不同的 step() 签名 — 无法互换
PMSMdqModel.step(vd, vq, tl, dt)          # dq电压
BLDCModel.step(v_bus, tl, dt, direction)   # 直流母线电压
IMdqModel.step(vsd, vsq, omega_e, tl, dt) # dq电压 + 同步速度
```

**影响**:
- 无法实现多态调度
- 添加新电机类型需要修改调用方代码
- `Orchestrator.register_model()` 接受 `Any` 类型, 丧失类型安全

**建议**: 定义抽象基类:

```python
from abc import ABC, abstractmethod

class MotorModel(ABC):
    @abstractmethod
    def step(self, **kwargs) -> None: ...
    @abstractmethod
    def reset(self) -> None: ...
    @abstractmethod
    def get_state(self) -> dict: ...
    @property
    @abstractmethod
    def torque(self) -> float: ...
```

### 🟠 P1: SRP违反 — 电机文件包含控制器

| 文件 | 包含的类 | 问题 |
|------|----------|------|
| `models/motor/bldc.py` | `BLDCModel` + `BLDCController` | 电机和控制器应在不同文件 |
| `models/motor/im_dq.py` | `IMdqModel` + `IMVectorController` | 同上 |

**建议**: 将 `BLDCController` 移至 `models/controller/`, 将 `IMVectorController` 移至 `models/controller/`。

### 🟠 P1: 依赖反转违反 — Orchestrator 接受 Any

```python
# orchestrator.py:91-92
def register_model(self, model: Any, metadata) -> None:
    """Register a model. (M-01: Accept Any for MVP — type checked at step time.)"""
```

`metadata` 参数甚至没有类型注解。

### 🟡 P2: OCP部分违反

添加新电机模型需要:
1. 创建新类(✅ 不修改现有代码)
2. 手动注册到 orchestrator(✅ 没问题)
3. 但 step() 签名不同, 调用方必须适配(❌ 违反OCP)

---

## 四、命名一致性

### ✅ 做得好的地方

| 规范 | 状态 |
|------|------|
| 类名 PascalCase | ✅ 全部遵循 |
| 常量 UPPER_SNAKE_CASE | ✅ `constants.py` 中全部遵循 |
| 私有字段 `_` 前缀 | ✅ `_wall_start_ns`, `_paused`, `_diverged` 等 |
| 私有方法 `_` 前缀 | ✅ `_sync_wallclock`, `_apply_faults` 等 |
| 函数/方法 snake_case | ✅ 基本遵循 |

### 🟡 P2: 别名不一致

```python
# sensors.py:28 — 简写别名
from sim_platform.core.utils import guard_numeric as _guard_num

# 其他所有文件 — 完整别名
from sim_platform.core.utils import guard_numeric as _guard_numeric
```

### 🟡 P2: 变量命名风格

- `Pp` (pole pairs) — 工程惯例, 可接受
- `Ld`, `Lq`, `Rs`, `Rr` — 工程惯例, 可接受
- `id_` (ekf.py:306) — 用尾部下划线避免遮蔽内置 `id`, 可接受但应统一

---

## 五、方法复杂度

### 🔴 P0: `Orchestrator.run()` — 101行, 4层嵌套

**文件**: `core/orchestrator.py:151-251`

```
run()                         # 101行
├── for i in range(total_steps):
│   ├── for hook in self._stop_hooks:    # 2层
│   │   └── try/except                   # 3层
│   ├── for solver_id, stepper:          # 2层
│   │   └── try/except                   # 3层
│   ├── if not all_converged:            # 2层
│   │   └── if halving_count < ...:      # 3层
│   │       └── continue                 # 4层
│   └── if progress_callback:            # 2层
│       └── try/except                   # 3层
```

**建议**: 提取 `_run_step()`, `_check_stop_conditions()`, `_step_solvers()` 等方法。

### 🟠 P1: 超过50行的方法

| 方法 | 行数 | 文件 |
|------|------|------|
| `Orchestrator.run()` | 101 | `orchestrator.py:151-251` |
| `MPCController.solve()` | 75 | `mpc.py:150-224` |
| `main()` (example) | 155 | `examples/pmsm_foc_mvp/main.py:124-281` |

### 🟡 P2: 参数过多的方法 (>5个)

| 方法 | 参数数 | 文件 |
|------|--------|------|
| `PMSMAdvanced.__init__()` | 14 | `pmsm_advanced.py:48-61` |
| `PMSMEKF.__init__()` | 11 | `ekf.py:246-263` |
| `IMVectorController.__init__()` | 8 | `im_dq.py:348-352` |
| `FOCController.__init__()` | 8 | `foc.py:204-207` |
| `AverageInverter.step()` | 7 | `power_models.py:46-48` |
| `PMSMEKF.estimate()` | 6 | `ekf.py:384-386` |
| `FOCController.update()` | 6 | `foc.py:227-228` |

**建议**: 对于 `__init__` 参数超过8个的, 考虑使用 dataclass 配置对象。

---

## 六、错误处理

### ✅ 做得好的地方

- **无 bare `except:`** — 全部使用 `except Exception:` 或更精确的异常类型
- **orchestrator.py** 中所有 `except Exception:` 都有 `logger.exception()` 日志
- **data_bus.py** 中 subscriber 回调异常有日志记录
- 输入验证全面: `ValueError`, `TypeError`, `PermissionError` 使用恰当

### 🟠 P1: 静默异常吞噬 (无日志)

| 文件 | 行号 | 问题 |
|------|------|------|
| `ekf.py:146` | `except Exception:` | 预测步骤失败时静默回退到恒等预测, **无日志** |
| `ekf.py:211` | `except Exception:` | 更新步骤失败时静默保持预测值, **无日志** |
| `mpc.py:110` | `except Exception:` | 模型评估失败时静默返回0.0, **无日志** |
| `hdf5_logger.py:126` | `except Exception:` | 完整性检查失败时静默返回False |

**建议**: 至少添加 `logger.debug()` 或 `logger.warning()` 以便调试。

### 🟡 P2: `except Exception:` 过宽

在生产代码中, `except Exception:` 捕获了所有非系统异常。对于数值计算代码, 应考虑捕获更具体的异常:

```python
# 当前 (ekf.py:146)
except Exception:
    x_pred = x.copy()

# 建议
except (ValueError, ArithmeticError, np.linalg.LinAlgError) as e:
    logger.warning("EKF prediction failed: %s", e)
    x_pred = x.copy()
```

---

## 七、类型注解

### ✅ 做得好的地方

- 所有公共方法基本都有参数类型注解
- 返回类型注解覆盖率高
- `constants.py` 全部有类型注解
- `data_bus.py` Signal dataclass 字段全部有类型

### 🟡 P2: 缺失或不完整的类型注解

| 位置 | 问题 |
|------|------|
| `orchestrator.py:91` | `metadata` 参数无类型 (`register_model(self, model: Any, metadata)`) |
| `orchestrator.py:82` | `_steppers: Dict[str, Callable[[int], StepResult]]` — OK |
| `foc.py:38` | `clarke_transform()` 返回 `tuple` 而非 `Tuple[float, float]` |
| `foc.py:47` | `park_transform()` 返回 `tuple` 而非 `Tuple[float, float, float]` |
| `sensors.py:76` | `read_abc()` 返回 `tuple` 而非 `Tuple[float, float, float]` |
| `pmsm_dq.py:141` | `update_abc_currents()` 返回 `tuple` 而非具体类型 |
| `model_registry.py:107` | `_models: Dict[str, Any]` — 值类型丢失 |

---

## 八、Magic Numbers

### 🔴 P0: `constants.py` 未覆盖的硬编码值

以下数值直接硬编码在源文件中, 未提取到 `constants.py`:

#### thermal_model.py (6处)

```python
# :72 — 温度上限溢出系数
self.T = max(self.T_ambient, min(self.T_max * 1.5, self.T))

# :87 — 降额公式
return max(0.0, 1.0 - (self.T - self.T_max) / self.T_max)

# :122 — 铁损分配到磁体的比例
self.magnet.step(iron_loss_W * 0.3, dt)

# :130 — 铜电阻温度系数
alpha_cu = 0.00393

# :138 — 磁体温度系数
alpha_mag = -0.0012

# :139 — 最小磁通系数
return max(0.5, 1.0 + alpha_mag * (self.magnet.T - T_ref))
```

#### clock.py (3处)

```python
# :114 — 最大等待倍数
max_wait_ns = dt_ns * 10

# :123 — 提前唤醒阈值 (1ms)
sleep_s = max(0, (sleep_ns - 1_000_000)) / 1e9

# :142 — 最大实时因子
self.realtime_factor = min(raw_factor, 1000.0)
```

#### orchestrator.py (1处)

```python
# :55 — EnergyAudit 中的 epsilon
total = self.power_input_j + 1e-12
```

#### bldc.py (3处)

```python
# :126-127 — EMF形状参数
self._emf_flat_top = 120.0  # degrees
self._emf_slope = 60.0      # degrees

# :381 — 控制器默认参数
i_max: float = 10.0, v_bus: float = 24.0
```

#### im_dq.py (3处)

```python
# :134 — flux clamping
rd = max(min(self.psi_rd, 1e100), -1e100)

# :422, 433, 439 — PI控制器限幅
limit=100.0, limit=200.0
```

#### pmsm_advanced.py (3处)

```python
# :157-158 — 铁损计算限幅
freq = min(freq, 1e6)      # Max 1 MHz
B_peak = min(B_peak, 10.0) # Max 10 T

# :249 — 机械速度限幅
omega_c = max(min(self.omega_m, 1e6), -1e6)
```

#### pmsm_dq.py (1处)

```python
# :52 — 摩擦警告阈值
if self.B < 1e-6:
```

#### examples/main.py (3处)

```python
# :145 — 硬编码π
quantization=2 * 3.1415926535 / ...

# :181, 263 — 硬编码π
2*3.14159
```

---

## 九、改进建议 (按优先级)

### P0 — 必须修复

1. **创建 `core/transforms.py`**: 统一 Park/Clarke/InversePark/InverseClarke 变换
2. **统一 NaN/Inf 防护**: 所有模块统一使用 `_guard_numeric`, 消除手动 `math.isfinite` 检查
3. **定义 `MotorModel` 抽象基类**: 统一电机模型接口, 支持多态调度
4. **提取 `Orchestrator.run()` 子方法**: 降低复杂度到3层嵌套以内

### P1 — 应该修复

5. **在 `constants.py` 添加 `TWO_PI`**: 消除25处 `2 * math.pi` 重复
6. **统一 PI 控制器**: 将 `PIController` 移至 `models/controller/base.py`, 所有控制器共用
7. **分离电机/控制器**: 将 `BLDCController` 从 `bldc.py` 移出, `IMVectorController` 从 `im_dq.py` 移出
8. **为静默异常添加日志**: `ekf.py:146,211` 和 `mpc.py:110` 至少添加 `logger.debug()`
9. **`thermal_model.py` 提取物理常数**: 温度系数、降额公式参数等移至 `constants.py`

### P2 — 建议修复

10. **提取 `resolve_dt()` 工具函数**: 消除 dt_ns→dt 转换的3处重复
11. **消除 `convert_enums` 重复**: 保留 `config_schema.py` 中的版本, `config_manager.py` 导入它
12. **`config_manager.py` 导入 `MAX_CONFIG_SIZE`**: 从 `constants.py` 导入而非重新定义
13. **修复硬编码π**: `examples/main.py` 中的 `3.1415926535` → `math.pi`
14. **统一别名**: `sensors.py` 中的 `_guard_num` → `_guard_numeric`
15. **补充返回类型注解**: `tuple` → `Tuple[float, float]` 等
16. **为 `Orchestrator.register_model` 添加 `metadata` 类型注解**

---

## 十、总体评价

### 优势

| 维度 | 评分 | 说明 |
|------|------|------|
| 安全性 | ⭐⭐⭐⭐⭐ | CWE标注全面, 输入验证严格, 无裸except |
| 文档 | ⭐⭐⭐⭐⭐ | 每个模块都有Security/Numerical Limitations文档, constants.py有调优指南 |
| 错误防护 | ⭐⭐⭐⭐ | NaN/Inf防护覆盖全面(虽然实现方式不统一) |
| 测试覆盖 | ⭐⭐⭐⭐ | 26个测试文件, 覆盖安全/数值/集成/压力测试 |
| 命名规范 | ⭐⭐⭐⭐ | 基本统一, 少量不一致 |
| 类型注解 | ⭐⭐⭐⭐ | 公共API覆盖良好, 少量缺失 |

### 劣势

| 维度 | 评分 | 说明 |
|------|------|------|
| DRY | ⭐⭐⭐ | Park/Clarke变换4处重复, NaN防护方式不统一 |
| SOLID | ⭐⭐⭐ | 无公共模型接口, SRP违反(电机文件含控制器) |
| 复杂度控制 | ⭐⭐⭐ | Orchestrator.run() 过长, 嵌套过深 |
| Magic Numbers | ⭐⭐⭐ | constants.py 设计良好但未完全覆盖(thermal_model尤其严重) |

### 综合评分: **B+** (良好, 有改进空间)

代码安全性、文档和测试做得非常出色, 达到工业级水准。主要技术债务集中在DRY违反(特别是Park/Clarke变换重复)和缺少统一模型接口。建议优先处理P0项, 预计2-3天工作量可显著提升可维护性。
