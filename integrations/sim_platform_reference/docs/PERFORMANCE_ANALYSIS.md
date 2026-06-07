# sim_platform 性能分析报告

> 分析日期: 2026-06-04
> 分析范围: 全部模型热路径、内存、算法、IO、Python开销、并发潜力

---

## 一、热路径分析 (Hot Path)

### 1.1 单步操作分解

每个仿真步 (dt=50μs, 20kHz) 的完整调用链：

| 组件 | cos/sin 调用 | isfinite 检查 | 乘除法 | 备注 |
|------|:-----------:|:------------:|:------:|------|
| `FOCController.update()` | 4 | 9 | ~25 | Clarke + Park + InvPark + SVPWM |
| `PIController.update()` ×2 | 0 | 4 | ~12 | 两个PI回路 |
| `PMSMdqModel.step_abc()` | 2 | 6 | ~15 | Clarke+Park变换 + 电气/机械动力学 |
| `PMSMdqModel.step()` | 0 | 6 | ~10 | Forward Euler积分 |
| `update_abc_currents()` | 2 | 0 | ~8 | InvPark变换 |
| `CurrentSensor.read_abc()` | 0 | 3 | ~6 | 3相×(噪声+偏置+饱和) |
| `Encoder.read_angle()` | 0 | 2 | ~3 | 量化+噪声 |
| `AverageInverter.step()` | 0 | 3 | ~6 | duty→电压 |
| **合计** | **~8** | **~33** | **~85** | **每步约120+浮点操作** |

### 1.2 三角函数开销

`math.cos` / `math.sin` 占每步计算的 **~15-20%** 时间。关键路径：
- `foc.py:51-52` (`park_transform`): cos_t, sin_t
- `foc.py:62-63` (`inverse_park`): cos_t, sin_t
- `pmsm_dq.py:160-161` (`step_abc`): cos_t, sin_t
- `pmsm_dq.py:143-144` (`update_abc_currents`): cos_t, sin_t

**问题**: 每步计算了 **4组 cos/sin 对**，其中 `step_abc` 和 `update_abc_currents` 使用相同的 `theta_e`，存在重复计算。

### 1.3 guard_numeric 调用频率

代码库中共有 **~210处** guard_numeric/isfinite 调用。单步热路径中：
- 直接 `math.isfinite()` 调用: ~15次 (FOC/PMSM入口守卫)
- 通过 `_guard_numeric()` 间接调用: ~10次 (PI控制器、传感器)
- **总计: ~25-30次/步**

`math.isfinite()` 虽然比 `isnan()+isinf()` 快，但在如此高频调用下仍有可观开销。

---

## 二、内存使用分析

### 2.1 Signal 对象创建

`DataBus.publish()` 每次调用创建 1 个 `Signal` dataclass 对象 (11个字段)：
- `data_bus.py:291-296`: `publish_scalar()` 创建 Signal
- `data_bus.py:270-273`: 历史记录 append + 列表切片裁剪

**问题**: 如果通过 DataBus 传递每步数据 (如 TUI 模式)，每步创建 10+ Signal 对象 = 200k 对象/秒。

### 2.2 历史数据存储

```python
# data_bus.py:270-273
hist = self._history[topic]
hist.append(signal)
if len(hist) > _MAX_HISTORY:  # 10000
    self._history[topic] = hist[-_MAX_HISTORY:]  # O(n) 切片!
```

**严重问题**: 列表切片 `hist[-10000:]` 在达到上限时每步都创建新列表，是 O(n) 操作。

### 2.3 日志缓冲区

`HDF5Logger` 使用 Python list 作为缓冲区 (`hdf5_logger.py:74-82`)：
- 每 1000 步 flush 一次
- flush 时 `np.array(buf, dtype=np.float64)` 转换 + gzip 压缩
- 缓冲区 15 个 channel × 1000 样本 = ~120KB 内存

### 2.4 示例主循环日志

`examples/pmsm_foc_mvp/main.py:217-231` 将所有数据存入 Python dict of lists：
- 15 个 channel × 20000 步 (1秒) = 300000 个 Python float 对象
- 1 秒仿真 = ~2.4MB Python 对象开销 (每个 float 28 bytes)

---

## 三、算法复杂度分析

### 3.1 MPC 控制器

```
MPCController.solve() 复杂度:
  O(max_iterations × Nc × Np)
  默认: O(50 × 3 × 10) = O(1500) 模型评估/步
```

**关键瓶颈** (`mpc.py:184-221`):
- 数值梯度: 每个控制变量需要 2 次 predict + compute_cost
- 每次 predict 执行 Np 次模型评估
- `max_iterations=50` 通常过高，大多数问题 10-20 次即可收敛

**无提前终止**: 当前实现没有收敛检测，总是跑满 `max_iterations`。

### 3.2 EKF 矩阵运算

EKF 使用 numpy 但矩阵仅 4×4：
- `ekf.py:143`: `F_k @ self.P @ F_k.T` — 两次 4×4 矩阵乘法
- `ekf.py:195`: `np.linalg.inv(S)` — 4×4 矩阵求逆
- `ekf.py:199`: `P_pred @ H_k.T @ S_inv` — 两次 4×4 矩阵乘法

**问题**: numpy 对 4×4 矩阵有显著的调用开销 (~1-2μs/调用)，实际计算仅需 ~100ns。EKF 每步约 10 次 numpy 操作 = ~15μs 开销。

**未使用 BLAS**: numpy 默认使用 OpenBLAS/MKL，但 4×4 矩阵不值得调用 BLAS，应使用手写展开。

### 3.3 SVPWM 扇区判断

当前 SVPWM 实现 (`foc.py:74-118`) 使用简化的 duty cycle 计算：
- 无显式扇区判断 (直接计算 abc duty)
- Overmodulation 使用线性缩放
- **已经足够高效**，无需优化

---

## 四、IO 瓶颈

### 4.1 HDF5 写入策略

```python
# hdf5_logger.py:84-104
def _flush(self):
    for name, buf in self._buffers.items():
        arr = np.array(buf, dtype=np.float64)  # list→ndarray 转换
        dset.resize((new_len,))                 # 动态扩展
        dset[old_len:new_len] = arr             # 写入
```

**问题**:
1. **gzip 压缩** (`compression="gzip", compression_opts=4`): 每次 flush 都压缩，CPU 密集
2. **动态 resize**: HDF5 dataset 扩展需要重新分配块
3. **每 1000 步 flush**: 对于 20kHz 仿真 = 每秒 20 次 flush

### 4.2 Matplotlib 渲染

`plot_log.py` 仅在仿真结束后渲染一次，**不是瓶颈**。

### 4.3 控制台输出

`main.py:177-181` 在循环前打印参数，循环内无 print。**不是瓶颈**。

---

## 五、Python 开销

### 5.1 可 Cython/Numba 加速的热点

| 热点 | 当前实现 | 加速潜力 | 优先级 |
|------|---------|---------|--------|
| `clarke/park/inverse_park` | pure Python math | **5-10x** | **P0** |
| `PIController.update()` | pure Python | **3-5x** | P1 |
| `svpwm()` | pure Python | **3-5x** | P1 |
| `PMSMdqModel.step()` | pure Python | **3-5x** | P1 |
| `guard_numeric()` | pure Python | **2-3x** | P2 |
| `CurrentSensor.read()` | pure Python + random | **2x** | P2 |

### 5.2 numpy 利用程度

- **电机模型**: 纯 Python float 运算，**未使用 numpy**
- **EKF**: 使用 numpy 4×4 矩阵，但开销 > 收益
- **HDF5 Logger**: 使用 numpy 做 list→array 转换
- **传感器噪声**: 使用 Python `random.gauss`，有 1024 样本缓冲

**关键发现**: 热路径 (电机+控制器) 完全是标量 Python 运算，numpy 在此场景下反而因调用开销而更慢。

### 5.3 不必要的 Python 循环

1. **MPC 数值梯度** (`mpc.py:192-205`): Python for 循环遍历 Nc
2. **MPC predict** (`mpc.py:103-116`): Python for 循环遍历 Np
3. **DataBus 历史裁剪** (`data_bus.py:272`): `hist[-MAX_HISTORY:]` O(n) 列表复制
4. **Orchestrator 主循环** (`orchestrator.py:186-250`): 每步检查 stop hooks、faults、energy audit

---

## 六、并发潜力

### 6.1 可并行化的计算

| 组件 | 并行方式 | 预期收益 | 可行性 |
|------|---------|---------|--------|
| MPC 双轴 (id/iq) | 多线程 | 2x | 高 (独立计算) |
| EKF predict+update | 流水线 | 1.3x | 中 |
| 多电机仿真 | 多进程 | N× | 高 (完全独立) |
| Monte Carlo 批量 | 多进程 | N× | 高 (完全独立) |

### 6.2 GPU 加速适用性

- **单步仿真**: 不适合 GPU (标量运算，启动开销 > 计算)
- **批量仿真** (Monte Carlo): 适合 GPU (大量独立轨迹)
- **MPC 优化**: 可用 GPU 做批量梯度计算

### 6.3 多模型仿真

当前 `Orchestrator.run()` 串行执行所有 stepper (`orchestrator.py:208-216`)。
对于独立模型 (如多电机)，可用 `concurrent.futures.ProcessPoolExecutor` 并行。

---

## 七、热点排名 (按性能影响)

| 排名 | 热点 | 影响程度 | 当前开销估算 |
|:----:|------|:--------:|:----------:|
| 1 | **guard_numeric / isfinite 过度调用** | 高 | ~25次/步，~3μs/步 |
| 2 | **重复 cos/sin 计算** | 高 | 8次/步，~2μs/步 |
| 3 | **MPC 无收敛检测** | 高 | 固定50迭代，~100μs/步 |
| 4 | **Python 标量运算开销** | 中 | ~15μs/步 (20kHz) |
| 5 | **DataBus Signal 对象创建** | 中 | 每次 publish ~0.5μs |
| 6 | **DataBus 历史列表切片** | 中 | 每次达到上限 ~5μs |
| 7 | **EKF numpy 小矩阵开销** | 低 | ~15μs/步 (仅EKF模式) |
| 8 | **HDF5 gzip 压缩** | 低 | flush时 ~5ms/次 |

---

## 八、优化建议 (按收益排序)

### P0 — 高收益、低风险

#### 8.1 合并 cos/sin 计算 (预估提速: 15-20%)

```python
# 当前: step_abc 和 update_abc_currents 各算一次 cos/sin
# 优化: 缓存 cos/sin 结果

class PMSMdqModel:
    def __init__(self, ...):
        self._cos_theta = 1.0
        self._sin_theta = 0.0

    def _update_trig(self):
        """在 theta 变化后调用一次"""
        self._cos_theta = math.cos(self.theta_e)
        self._sin_theta = math.sin(self.theta_e)

    def step(self, ...):
        # ... 积分后:
        self._update_trig()  # 只算一次

    def step_abc(self, ...):
        cos_t, sin_t = self._cos_theta, self._sin_theta  # 复用
        # ...

    def update_abc_currents(self):
        cos_t, sin_t = self._cos_theta, self._sin_theta  # 复用
        # ...
```

**影响范围**: `pmsm_dq.py`, `pmsm_advanced.py`, `im_dq.py`
**预估**: 每步减少 2 组 cos/sin = ~0.5μs

#### 8.2 热路径守卫精简 (预估提速: 10-15%)

```python
# 当前: 每个函数都独立检查 isfinite
# 优化: 仅在入口点检查，内部函数信任调用者

# foc.py - 当前实现已经做得较好 (注释说明)
# 但 PMSMdqModel.step() 内部仍有冗余守卫

# 建议: step() 内部移除状态变量的重复检查
# 仅保留入口点 (vd, vq, tl) 和最终状态的检查
```

**影响范围**: `pmsm_dq.py:109-111`, `pmsm_dq.py:121-122`
**预估**: 每步减少 ~10 次 isfinite = ~0.3μs

#### 8.3 MPC 收敛检测 + 提前终止 (预估提速: 50-80% for MPC)

```python
# mpc.py - 当前: 固定 max_iterations=50
# 优化: 添加收敛检测

def solve(self, x0, x_ref, model, u_init=None):
    # ...
    prev_cost = float('inf')
    for iteration in range(num_iterations):
        x_pred = self.predict(x0, u_seq, model)
        cost = self.compute_cost(x_pred, x_ref, u_seq)

        # 提前终止
        if abs(prev_cost - cost) < 1e-6 * (abs(prev_cost) + 1e-12):
            break
        prev_cost = cost
        # ... 梯度计算
```

**影响范围**: `mpc.py`
**预估**: 典型场景从 50 次降至 10-15 次迭代 = **3-5x MPC 加速**

#### 8.4 MPC 解析梯度替代数值梯度 (预估提速: 2-3x for MPC)

```python
# 当前: 数值梯度需要 2*Nc 次 predict
# 优化: 对于线性 RL 模型，可推导解析梯度

# 对于 di/dt = (v - R*i) / L 模型:
# dJ/du = Σ 2*Q*(x-xref)*dx/du + 2*R*u
# 其中 dx/du 可通过链式法则解析计算
```

**影响范围**: `mpc.py`
**预估**: 每次迭代从 2*Nc predict 降至 1 predict

### P1 — 中收益、中风险

#### 8.5 DataBus 历史缓冲改用 deque (预估提速: IO 密集场景 5-10%)

```python
# 当前: list + 切片裁剪 (O(n))
# 优化: collections.deque(maxlen=MAX_HISTORY)

from collections import deque

class DataBus:
    def __init__(self):
        # ...
        self._history: Dict[str, deque] = defaultdict(
            lambda: deque(maxlen=_MAX_HISTORY))

    def publish(self, topic, signal, module_id=""):
        # ...
        self._history[topic].append(signal)  # O(1) 自动丢弃旧数据
```

**影响范围**: `data_bus.py`
**风险**: 需验证 `read_history()` 的返回类型兼容性

#### 8.6 Signal 使用 __slots__ (预估内存减少 30-40%)

```python
@dataclass
class Signal:
    __slots__ = ('source', 'signal_type', 'timestamp_ns', 'value',
                 'unit', 'coordinate_frame', 'sample_rate_hz',
                 'latency_ns', 'validity', 'quality',
                 'safety_level', 'sequence_id')
    # ...
```

**影响范围**: `data_bus.py`
**预估**: 每个 Signal 对象从 ~400 bytes 降至 ~250 bytes

#### 8.7 EKF 小矩阵手写展开 (预估提速: 3-5x for EKF)

```python
# 当前: numpy 4×4 矩阵运算 (调用开销 >> 计算)
# 优化: 手写展开的 4×4 矩阵乘法

def _mat4_mul(A, B):
    """4×4 矩阵乘法，手写展开"""
    C = [0.0] * 16
    for i in range(4):
        for k in range(4):
            aik = A[i*4+k]
            for j in range(4):
                C[i*4+j] += aik * B[k*4+j]
    return C
```

**影响范围**: `ekf.py`
**预估**: EKF 每步从 ~15μs 降至 ~3μs

#### 8.8 HDF5 写入优化

```python
# 1. 使用 lzf 替代 gzip (快 5-10x，压缩率略低)
self._file.create_dataset(
    name, data=arr,
    maxshape=(None,), chunks=True,
    compression="lzf",  # 替代 gzip
)

# 2. 增大 flush 间隔 (1000 → 5000)
self._flush_interval = 5000

# 3. 预分配 dataset 大小 (避免频繁 resize)
```

**影响范围**: `hdf5_logger.py`
**预估**: IO 密集场景提速 3-5x

### P2 — 低收益、可选优化

#### 8.9 传感器噪声使用 numpy 批量生成

```python
# 当前: random.gauss 1024 样本缓冲
# 优化: numpy.random.Generator 批量生成

import numpy as np
_rng = np.random.default_rng()

def _get_noise_batch(n, std):
    return _rng.normal(0, std, n)
```

**预估**: 噪声生成提速 3-5x，但噪声本身不是瓶颈

#### 8.10 guard_numeric 内联

```python
# 当前: 函数调用开销 (~70ns/调用)
# 优化: 在热点路径内联

# 适用于: PMSMdqModel.step() 内部的重复守卫
# 将 _guard_numeric(x, 0.0) 替换为: x if math.isfinite(x) else 0.0
```

**预估**: 每次调用节省 ~30ns

#### 8.11 使用 `__slots__` 优化所有模型类

```python
class PMSMdqModel:
    __slots__ = ('Rs', 'Ld', 'Lq', 'flux_pm', 'J', 'B', 'Pp',
                 'dt', 'id', 'iq', 'omega_m', 'theta_e', 'torque',
                 'ia', 'ib', 'ic', '_cos_theta', '_sin_theta')
```

**预估**: 模型实例内存减少 30-40%，属性访问提速 ~10%

---

## 九、预估总提速

| 优化项 | 预估提速 | 实施难度 | 实施优先级 |
|--------|:--------:|:--------:|:----------:|
| 合并 cos/sin | 15-20% | 低 | P0 |
| 热路径守卫精简 | 10-15% | 低 | P0 |
| MPC 收敛检测 | 50-80% (MPC) | 低 | P0 |
| MPC 解析梯度 | 2-3x (MPC) | 中 | P0 |
| DataBus deque | 5-10% (IO) | 低 | P1 |
| Signal __slots__ | 内存-30% | 低 | P1 |
| EKF 手写矩阵 | 3-5x (EKF) | 中 | P1 |
| HDF5 lzf 压缩 | 3-5x (IO) | 低 | P1 |
| Cython/Numba 热点 | **5-10x** | 高 | P2 |

### 综合预估 (FOC + PMSM 闭环场景)

| 场景 | 当前 | P0 优化后 | P0+P1 后 | P0+P1+P2 后 |
|------|:----:|:---------:|:--------:|:-----------:|
| 纯仿真 (无日志) | ~174k steps/s | ~220k steps/s | ~250k steps/s | ~500k+ steps/s |
| 带 HDF5 日志 | ~80k steps/s | ~100k steps/s | ~150k steps/s | ~300k+ steps/s |
| 带 MPC 控制 | ~20k steps/s | ~50k steps/s | ~60k steps/s | ~150k+ steps/s |
| 带 EKF 估计 | ~60k steps/s | ~75k steps/s | ~120k steps/s | ~200k+ steps/s |

---

## 十、快速实施路线图

### Phase 1 (1-2天): 零风险优化
1. 合并 PMSMdqModel 的 cos/sin 缓存
2. MPC 添加收敛检测 + 提前终止
3. DataBus 历史改用 deque

### Phase 2 (3-5天): 低风险优化
4. Signal/模型类添加 `__slots__`
5. EKF 小矩阵手写展开
6. HDF5 使用 lzf 压缩

### Phase 3 (1-2周): 架构优化
7. 热点函数 Cython/Numba JIT
8. 批量仿真多进程并行
9. GPU Monte Carlo 加速

---

## 附录: 关键代码位置索引

| 热点 | 文件 | 行号 |
|------|------|------|
| FOC 热路径 | `models/controller/foc.py` | 227-247 |
| Clarke/Park 变换 | `models/controller/foc.py` | 38-66 |
| SVPWM | `models/controller/foc.py` | 74-118 |
| PMSM step | `models/motor/pmsm_dq.py` | 88-136 |
| PMSM step_abc | `models/motor/pmsm_dq.py` | 154-164 |
| PMSM update_abc | `models/motor/pmsm_dq.py` | 141-152 |
| MPC solve | `models/controller/mpc.py` | 150-224 |
| EKF predict | `models/controller/ekf.py` | 115-154 |
| EKF update | `models/controller/ekf.py` | 156-219 |
| DataBus publish | `core/data_bus.py` | 229-280 |
| Signal dataclass | `core/data_bus.py` | 62-121 |
| guard_numeric | `core/utils.py` | 14-31 |
| Orchestrator 主循环 | `core/orchestrator.py` | 186-250 |
| HDF5 flush | `tools/replay/hdf5_logger.py` | 84-104 |
| 传感器噪声缓冲 | `models/sensor/sensors.py` | 31-45 |
| Benchmark | `tools/profiling/benchmark.py` | 25-69 |
