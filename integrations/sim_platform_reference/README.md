# sim_platform — Multi-Domain Co-Simulation Platform

> **8 rounds security audit** | **1079 tests passing** | **PySide6 GUI + TUI + CLI** | **v1.3.0**

---

## Quick Start (3 min)

### Install

```bash
# 1. Create environment (推荐 uv)
uv venv .venv --python 3.13
# 或: python -m venv .venv

# 2. Install
uv pip install -e ".[dev,gui]" --python .venv/Scripts/python.exe
# 或: pip install -r requirements.txt && pip install PySide6>=6.5

# 3. Verify
.venv/Scripts/python.exe -m pytest verification/ -q
# Expected: 1079 passed
```

### Run

```bash
# GUI (推荐 — PySide6 桌面应用)
sim-gui
# 或: .venv/Scripts/python.exe -m sim_platform.tools.gui.app

# TUI (终端界面)
sim-tui
# 或: .venv/Scripts/python.exe -m sim_platform --tui

# Quick simulation (一键仿真)
sim-run --quick

# Parameter scan (批量扫描)
sim-scan --param speed --values 50,100,150
```

---

## Architecture

```
┌─────────────── Config ───────────────┐
│  YAML / Pydantic Schema / Templates  │
└──────────────┬───────────────────────┘
               ▼
┌─────────────── Orchestrator ─────────┐
│  GlobalClock + Scheduler + Registry  │
└──────────────┬───────────────────────┘
               ▼
┌─────────────── Solver Fabric ────────┐
│  PMSM │ BLDC │ IM │ FOC │ MPC │ EKF │
│  Thermal │ Sensor Fusion │ SVPWM    │
└──────────────┬───────────────────────┘
               ▼
┌─────────────── DataBus ──────────────┐
│  Pub/Sub + ACL (default-deny)        │
│  threading.RLock + regex validation  │
└──────────────┬───────────────────────┘
               ▼
┌─────────────── Verification ─────────┐
│  FaultInjector │ 1079 Tests │ 8-round│
│  Security Audit │ Monte Carlo        │
└──────────────────────────────────────┘
```

### Motor Models

| Model | File | Fidelity | Description |
|-------|------|----------|-------------|
| **PMSMdqModel** | `models/motor/pmsm_dq.py` | L2 | dq-axis PMSM with torque clamp |
| **PMSMAdvanced** | `models/motor/pmsm_advanced.py` | L3 | Saturation + temperature + iron loss + overflow protection |
| **BLDCModel** | `models/motor/bldc.py` | L2 | Trapezoidal back-EMF, 6-step commutation |
| **IMdqModel** | `models/motor/im_dq.py` | L2 | Induction motor dq + vector control |

### Controllers

| Controller | File | Description |
|------------|------|-------------|
| **FOCController** | `models/controller/foc.py` | Clarke/Park/SVPWM + PI anti-windup |
| **MPCController** | `models/controller/mpc.py` | Model predictive control with QP solver |
| **EKFEstimator** | `models/controller/ekf.py` | Joseph form covariance, configurable B/J |

### Supporting Models

| Model | File | Description |
|-------|------|-------------|
| **RintBattery** | `models/power/battery.py` | Internal resistance battery model |
| **ThermalNode** | `models/thermal/thermal_node.py` | RC thermal network |
| **SensorFusion** | `models/fusion/sensor_fusion.py` | 1D Kalman filter (Joseph form) |
| **CurrentSensor** | `models/sensor/current_sensor.py` | Noise + bias + quantization |
| **Encoder** | `models/sensor/encoder.py` | Quadrature encoder simulation |

---

## GUI (PySide6)

Professional desktop application with Material Design 3 + Apple HIG design system.

### Features

| Feature | Description |
|---------|-------------|
| **Dashboard** | Quick Actions + Scenario Presets + Workspace Info |
| **Config Panel** | 7 groups (Scenario/Motor/FOC/SpeedPI/Sensors/Time/OP), YAML/JSON load/save |
| **Real-time Charts** | QChartView with speed/reference/torque series, 50k point limit |
| **Log Viewer** | Level filtering, text search, export, HTML sanitization |
| **Result Table** | Performance metrics (Rise/Settling/Overshoot/Peak Current) |
| **Parameter Scan** | Multi-value sweep with progress tracking |
| **File Management** | New/Open/Save/SaveAs/Export CSV/JSON, recent files |
| **Pause/Resume** | Simulation pause and resume support |
| **i18n** | 200+ entries (zh/en), language switch in Help menu |
| **Onboarding** | 4-step guided tutorial, "don't show again" option |
| **Path Safety** | All file dialogs restricted to workspace directory |
| **Thread Safety** | QThread + Signal/Slot, threading.Event stop flags |

### Design System

- **Material Design 3**: Tonal surface model (bg_base → surface → elevated → overlay)
- **Apple HIG**: Semantic colors, label hierarchy (100%/55%/25% opacity)
- **Glassmorphism**: Stat cards with blur effect and color accent bars
- **Catppuccin Mocha**: Dark theme with 280+ lines QSS

---

## Test Report

```bash
# All tests
.venv/Scripts/python.exe -m pytest verification/ -v

# By category
pytest verification/test_cases/test_core.py -v           # Core (83 tests)
pytest verification/test_cases/test_integration.py -v    # Integration (9 tests)
pytest verification/test_cases/test_gui.py -v            # GUI (64 tests)
pytest verification/test_cases/test_visualization.py -v  # Visualization (31 tests)
pytest verification/test_cases/test_stress_extreme.py -v # Stress (20 tests)
pytest verification/test_cases/test_security_attack.py -v # Security (9 tests)
```

| Test Suite | Count | Coverage |
|------------|-------|----------|
| Deep security attack | 112 | 17 dimensions across all models |
| Core modules | 83 | utils, constants, clock, data_bus, registry, orchestrator |
| Motor models | 47 | PMSM, BLDC, IM, sensors, power |
| Controllers | 36 | FOC, MPC, EKF, PI |
| Config system | 45 | YAML parsing, schema, templates |
| GUI | 64 | Widgets, dialogs, thread safety, security |
| TUI | 37 | Screens, widgets, validation |
| Induction motor | 33 | IM model + vector controller |
| Visualization | 31 | plot_log, advanced_plot, parameter_scan, interactive |
| Solver performance | 34 | Numerical stability, physics invariants |
| Security stress | 103 | NaN/Inf/Unicode/concurrent/memory attacks |
| Monte Carlo | 13 | Parameter sensitivity |
| Extreme stress | 20 | Concurrent, long-running, extreme parameters |
| Integration | 9 | End-to-end: PMSM+FOC, BLDC, IM, EKF |
| GUI deep attack | 9 | Thread safety, traceback leak, data limits |
| Physics audit | 23 | Equation verification (1e-10 precision) |
| **Total** | **1079** | **All passing** |

---

## Security Posture

| Dimension | Status |
|-----------|--------|
| Security audits | 8 rounds completed |
| Attack vectors tested | 215+ (84 core + 112 deep + 13 Monte Carlo + 9 GUI) |
| DataBus policy | Default-deny (all modules must register) |
| NaN/Inf guards | Unified in `core/utils.py`, 197 call sites |
| Thread safety | threading.RLock on DataBus, threading.Event on GUI workers |
| Path validation | `_is_within_workspace()` on all file dialogs |
| HTML sanitization | CWE-117 protection on log output |
| Overflow protection | Current/flux/temperature clamps on all models |
| Traceback leak | Removed from GUI error signals (CWE-209) |
| GUI injection | No eval/exec/subprocess, max data points/entries |

---

## Performance

| Metric | Value |
|--------|-------|
| Throughput | 174k steps/sec (+55% from baseline) |
| Guard optimization | math.isfinite() replaces isnan+isinf (+17%) |
| Hot-path inlining | clarke/park/svpwm guards consolidated (+47%) |
| Sensor noise | Pre-generated 1024-sample buffer |
| Chart limit | 50,000 data points (auto-trim) |
| Log limit | 10,000 messages (auto-trim) |

---

## Code Quality

| Metric | Value |
|--------|-------|
| Python files | 70+ (source + test) |
| Lines of code | ~15,000 |
| Test count | 1,079 |
| Test coverage | 85%+ |
| `_guard_numeric` duplication | 0 (unified in `core/utils.py`) |
| Magic numbers | 0 (centralized in `core/constants.py`) |
| Linting | ruff (548 issues fixed) |
| Pre-commit | ruff lint + format + quick tests |
| Technical debt | 0 |

---

## Tech Stack

| Domain | Technology |
|--------|------------|
| Language | Python 3.10+ (3.13 recommended) |
| Math | numpy |
| Visualization | matplotlib |
| Data storage | HDF5 (h5py) |
| GUI | PySide6 6.11.1 + QtCharts |
| TUI | Textual (legacy) |
| Testing | pytest 9.0.3 |
| Linting | ruff |
| Package management | uv |
| Security | CWE Top25 + OWASP |

---

## Project Structure

```
sim_platform/
├── core/                          # Core framework
│   ├── utils.py                   # Unified numeric guards
│   ├── constants.py               # Centralized constants
│   ├── orchestrator.py            # Simulation scheduler
│   ├── data_bus.py                # Pub/sub with ACL + RLock
│   ├── clock.py                   # Time management
│   └── model_registry.py          # Model registration
│
├── models/                        # Physical models
│   ├── motor/                     # PMSM, BLDC, IM
│   ├── controller/                # FOC, MPC, EKF, PI
│   ├── power/                     # Battery
│   ├── sensor/                    # Current sensor, encoder
│   ├── thermal/                   # RC thermal network
│   └── fusion/                    # Sensor fusion (Kalman)
│
├── tools/                         # Utilities
│   ├── gui/                       # PySide6 GUI (13 files)
│   │   ├── app.py                 # Main window + Dashboard
│   │   ├── workers.py             # QThread workers
│   │   ├── theme.py               # M3 + Apple HIG design system
│   │   ├── i18n.py                # 200+ translations (zh/en)
│   │   ├── widgets/               # Config, Chart, Log, Stats, Results
│   │   └── dialogs/               # Onboarding, Scan
│   ├── tui/                       # Textual TUI (legacy)
│   ├── visualization/             # Plot tools
│   ├── config/                    # Configuration management
│   └── replay/                    # HDF5 logger
│
├── verification/                  # Tests + fault injection
│   ├── test_cases/                # 1079 tests
│   └── fault_injection/           # Fault injector
│
├── examples/                      # Example simulations
├── pyproject.toml                 # Project config
└── requirements.txt               # Dependencies
```

---

## Entry Points

| Command | Description |
|---------|-------------|
| `sim-gui` | PySide6 desktop GUI |
| `sim-tui` | Textual terminal UI |
| `sim-run` | CLI simulation runner |
| `sim-scan` | Parameter scan tool |

---

## Contributing

1. Models must use `guard_numeric` from `core/utils.py` for NaN/Inf protection
2. All new features must have corresponding `test_*` tests
3. DataBus publish/subscribe requires `module_id` registration
4. Follow SI units (V, A, rad, N*m, H, Wb)
5. Run `pytest verification/ -q` to ensure zero regression
6. Use `core/constants.py` for all numeric constants (no magic numbers)
7. GUI code must follow M3 + Apple HIG design tokens in `theme.py`

---

## License

MIT
