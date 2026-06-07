# 团队上手指南 — TEAM_GUIDE.md

> 本文档帮助新团队成员在 30 分钟内上手 sim_platform v1.3.0

---

## 第一天：快速上手（15分钟）

### Step 1: 环境搭建 (3 min)

```bash
# 推荐使用 uv
uv venv .venv --python 3.13
uv pip install -e ".[dev,gui]" --python .venv/Scripts/python.exe

# 或使用 pip
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -e ".[dev,gui]"
```

### Step 2: 运行第一个仿真 (5 min)

```bash
# 一键快速仿真
sim-run --quick
# 或: .venv/Scripts/python.exe -m sim_platform --quick
```

你应该看到：

```
═══════════════════════════════════════════════════
  ⚡ sim_platform — PMSM FOC Interactive Runner
═══════════════════════════════════════════════════
  STEP 3/3: Running Simulation
  Simulating 30000 steps...
  ████████████████████ 100%
  Results:
  Final Speed │ 99.9 rad/s (954 rpm)
  Speed Error │ 0.15%
═══════════════════════════════════════════════════
```

### Step 3: 启动 GUI (7 min)

```bash
sim-gui
# 或: .venv/Scripts/python.exe -m sim_platform.tools.gui.app
```

**GUI 功能速览:**

| 区域 | 功能 |
|------|------|
| 左侧面板 | 配置参数 (7 组: 场景/电机/FOC/PI/传感器/时间/工况) |
| Dashboard | 快速操作 + 场景预设 + 工作区信息 |
| Chart | 实时仿真图表 (速度/参考/转矩) |
| Log | 日志查看器 (支持筛选/搜索/导出) |
| Results | 性能指标 (上升时间/稳定时间/超调/峰值电流) |

---

## 第二天：深入理解（30分钟）

### 架构概览

```
Config (YAML) → Orchestrator → Solver Fabric → DataBus → Verification
                  ↓                ↓                ↓
              GlobalClock      Motor Models     FaultInjector
                              Controllers
                              Sensors/Thermal
```

### 核心模块

| 模块 | 文件 | 职责 |
|------|------|------|
| Orchestrator | `core/orchestrator.py` | 仿真调度, 模型生命周期管理 |
| DataBus | `core/data_bus.py` | 发布/订阅, ACL, 线程安全 |
| GlobalClock | `core/clock.py` | 仿真时间, 壁钟同步 |
| ModelRegistry | `core/model_registry.py` | 模型注册, 依赖验证 |
| guard_numeric | `core/utils.py` | NaN/Inf 统一防护 |
| constants | `core/constants.py` | 集中常量管理 |

### 电机模型

| 模型 | 文件 | 用途 |
|------|------|------|
| PMSMdqModel | `models/motor/pmsm_dq.py` | 永磁同步电机 (dq 轴) |
| PMSMAdvanced | `models/motor/pmsm_advanced.py` | 高级 PMSM (饱和+铁损+温升) |
| BLDCModel | `models/motor/bldc.py` | 无刷直流电机 (6 步换相) |
| IMdqModel | `models/motor/im_dq.py` | 感应电机 (矢量控制) |

### 控制器

| 控制器 | 文件 | 算法 |
|--------|------|------|
| FOCController | `models/controller/foc.py` | Clarke/Park/SVPWM + PI |
| MPCController | `models/controller/mpc.py` | 模型预测控制 |
| EKFStateEstimator | `models/controller/ekf.py` | 扩展卡尔曼滤波 |
| PIController | `models/controller/pi.py` | PI + anti-windup |

---

## 开发规范

### 1. 安全第一

```python
# 所有数值输入必须经过 guard_numeric
from sim_platform.core.utils import guard_numeric

def step(self, vabc):
    va = guard_numeric(vabc[0], 0.0)
    # ...
```

### 2. 无魔法数字

```python
# ❌ 错误
if current > 200: ...

# ✅ 正确
from sim_platform.core.constants import DEFAULT_I_MAX
if current > DEFAULT_I_MAX: ...
```

### 3. 测试驱动

```bash
# 运行所有测试
.venv/Scripts/python.exe -m pytest verification/ -q

# 运行特定测试
.venv/Scripts/python.exe -m pytest verification/test_cases/test_gui.py -v

# 检查覆盖率
.venv/Scripts/python.exe -m pytest verification/ --cov=sim_platform --cov-report=html
```

### 4. 代码质量

```bash
# lint 检查
.venv/Scripts/python.exe -m ruff check .

# 自动修复
.venv/Scripts/python.exe -m ruff check --fix .

# 格式化
.venv/Scripts/python.exe -m ruff format .
```

### 5. GUI 开发

- 遵循 M3 + Apple HIG 设计 token (`tools/gui/theme.py`)
- 使用 `tr()` 函数实现国际化 (`tools/gui/i18n.py`)
- 文件操作必须通过 `_is_within_workspace()` 验证
- 线程通信使用 QThread + Signal/Slot, 停止使用 threading.Event

---

## 常见问题

### Q: 如何添加新的电机模型？

1. 在 `models/motor/` 下创建新文件
2. 继承基本接口 (`step()` 方法)
3. 使用 `guard_numeric` 保护所有数值输入
4. 在 `core/model_registry.py` 注册
5. 编写对应测试 `verification/test_cases/test_xxx.py`

### Q: 如何添加新的控制器？

1. 在 `models/controller/` 下创建新文件
2. 实现 `compute()` 方法
3. 使用 `core/constants.py` 中的常量
4. 编写测试验证控制效果

### Q: GUI 报错 "No module named 'PySide6'"？

```bash
pip install PySide6>=6.5
# 或: pip install sim_platform[gui]
```

### Q: 测试失败怎么办？

```bash
# 查看详细错误
.venv/Scripts/python.exe -m pytest verification/ -v --tb=long

# 只运行失败的测试
.venv/Scripts/python.exe -m pytest verification/ --lf
```

---

## 文件清单

| 目录 | 文件数 | 说明 |
|------|--------|------|
| core/ | 6 | 核心框架 |
| models/motor/ | 4 | 电机模型 |
| models/controller/ | 5 | 控制器 |
| models/sensor/ | 2 | 传感器 |
| models/thermal/ | 1 | 热模型 |
| models/fusion/ | 1 | 传感器融合 |
| models/power/ | 1 | 电源模型 |
| tools/gui/ | 13 | PySide6 GUI |
| tools/tui/ | 9 | Textual TUI (legacy) |
| tools/visualization/ | 5 | 可视化工具 |
| tools/config/ | 4 | 配置管理 |
| verification/ | 20+ | 测试 |
| examples/ | 1 | 示例仿真 |
| **总计** | **70+** | **~15,000 行代码** |

---

*更新日期: 2026-06-06 | sim_platform v1.3.0*
