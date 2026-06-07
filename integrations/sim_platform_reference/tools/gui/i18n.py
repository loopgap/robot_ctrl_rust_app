"""Internationalization (i18n) module — Chinese/English bilingual support.

Provides:
- Centralized translation dictionary
- Language switching at runtime
- tr() function for all UI strings
- Language persistence via QSettings
"""

from __future__ import annotations

# ── Current language state ─────────────────────────────────
_current_lang = "zh"  # Default: Chinese

# ── Translation table ─────────────────────────────────────
_TRANSLATIONS: dict[str, dict[str, str]] = {
    # ═══════════════════════════════════════════════════════
    # App / Window
    # ═══════════════════════════════════════════════════════
    "app.title": {
        "zh": "多物理域联合仿真平台",
        "en": "Multi-Domain Co-Simulation Platform",
    },
    "app.version": {
        "zh": "版本",
        "en": "Version",
    },
    "app.ready": {
        "zh": "就绪",
        "en": "Ready",
    },

    # ═══════════════════════════════════════════════════════
    # Tabs
    # ═══════════════════════════════════════════════════════
    "tab.home": {
        "zh": "首页",
        "en": "Home",
    },
    "tab.chart": {
        "zh": "图表",
        "en": "Chart",
    },
    "tab.log": {
        "zh": "日志",
        "en": "Log",
    },
    "tab.results": {
        "zh": "结果",
        "en": "Results",
    },

    # ═══════════════════════════════════════════════════════
    # Menu — File
    # ═══════════════════════════════════════════════════════
    "menu.file": {
        "zh": "文件(&F)",
        "en": "&File",
    },
    "menu.file.new": {
        "zh": "新建配置(&N)",
        "en": "&New Config",
    },
    "menu.file.open": {
        "zh": "打开配置(&O)...",
        "en": "&Open Config...",
    },
    "menu.file.save": {
        "zh": "保存配置(&S)",
        "en": "&Save Config",
    },
    "menu.file.save_as": {
        "zh": "另存为(&A)...",
        "en": "Save &As...",
    },
    "menu.file.export_csv": {
        "zh": "导出结果 (CSV)...",
        "en": "Export Results (CSV)...",
    },
    "menu.file.export_json": {
        "zh": "导出结果 (JSON)...",
        "en": "Export Results (JSON)...",
    },
    "menu.file.recent": {
        "zh": "最近文件",
        "en": "Recent Files",
    },
    "menu.file.recent.empty": {
        "zh": "(无)",
        "en": "(empty)",
    },
    "menu.file.exit": {
        "zh": "退出(&Q)",
        "en": "E&xit",
    },

    # ═══════════════════════════════════════════════════════
    # Menu — Simulation
    # ═══════════════════════════════════════════════════════
    "menu.sim": {
        "zh": "仿真(&S)",
        "en": "&Simulation",
    },
    "menu.sim.run": {
        "zh": "运行",
        "en": "Run",
    },
    "menu.sim.pause": {
        "zh": "暂停",
        "en": "Pause",
    },
    "menu.sim.resume": {
        "zh": "继续",
        "en": "Resume",
    },
    "menu.sim.stop": {
        "zh": "停止",
        "en": "Stop",
    },

    # ═══════════════════════════════════════════════════════
    # Menu — View
    # ═══════════════════════════════════════════════════════
    "menu.view": {
        "zh": "视图(&V)",
        "en": "&View",
    },
    "menu.view.config": {
        "zh": "切换配置面板",
        "en": "Toggle Config Panel",
    },
    "menu.view.toolbar": {
        "zh": "切换工具栏",
        "en": "Toggle Toolbar",
    },
    "menu.view.show": {
        "zh": "显示 {0}",
        "en": "Show {0}",
    },
    "menu.view.reset": {
        "zh": "重置布局",
        "en": "Reset Layout",
    },

    # ═══════════════════════════════════════════════════════
    # Menu — Tools
    # ═══════════════════════════════════════════════════════
    "menu.tools": {
        "zh": "工具(&T)",
        "en": "&Tools",
    },
    "menu.tools.scan": {
        "zh": "参数扫描(&P)...",
        "en": "&Parameter Scanner...",
    },

    # ═══════════════════════════════════════════════════════
    # Menu — Help
    # ═══════════════════════════════════════════════════════
    "menu.help": {
        "zh": "帮助(&H)",
        "en": "&Help",
    },
    "menu.help.shortcuts": {
        "zh": "快捷键参考(&K)",
        "en": "&Keyboard Shortcuts",
    },
    "menu.help.about": {
        "zh": "关于(&A)",
        "en": "&About",
    },
    "menu.help.language": {
        "zh": "语言(&L)",
        "en": "&Language",
    },

    # ═══════════════════════════════════════════════════════
    # Toolbar
    # ═══════════════════════════════════════════════════════
    "toolbar.home": {
        "zh": "🏠 首页",
        "en": "🏠 Home",
    },
    "toolbar.run": {
        "zh": "▶ 运行",
        "en": "▶ Run",
    },
    "toolbar.pause": {
        "zh": "⏸ 暂停",
        "en": "⏸ Pause",
    },
    "toolbar.resume": {
        "zh": "▶ 继续",
        "en": "▶ Resume",
    },
    "toolbar.stop": {
        "zh": "⏹ 停止",
        "en": "⏹ Stop",
    },
    "toolbar.results": {
        "zh": "📊 结果",
        "en": "📊 Results",
    },

    # ═══════════════════════════════════════════════════════
    # Dashboard
    # ═══════════════════════════════════════════════════════
    "dashboard.welcome": {
        "zh": "多物理域联合仿真平台",
        "en": "Multi-Domain Co-Simulation Platform",
    },
    "dashboard.subtitle": {
        "zh": "PMSM FOC · BLDC · 感应电机 · 一站式仿真工具",
        "en": "PMSM FOC · BLDC · Induction Motor · All-in-One Simulation",
    },
    "dashboard.quick_start": {
        "zh": "  快速开始  ",
        "en": "  Quick Start  ",
    },
    "dashboard.quick_actions": {
        "zh": "快捷操作",
        "en": "Quick Actions",
    },
    "dashboard.new_sim": {
        "zh": "新建仿真",
        "en": "New Simulation",
    },
    "dashboard.new_sim.desc": {
        "zh": "使用当前配置运行 FOC 仿真",
        "en": "Run FOC simulation with current config",
    },
    "dashboard.open_config": {
        "zh": "打开配置",
        "en": "Open Config",
    },
    "dashboard.open_config.desc": {
        "zh": "加载 YAML/JSON 配置文件",
        "en": "Load a YAML/JSON configuration file",
    },
    "dashboard.load_results": {
        "zh": "加载结果",
        "en": "Load Results",
    },
    "dashboard.load_results.desc": {
        "zh": "查看 HDF5 仿真结果",
        "en": "View HDF5 simulation results",
    },
    "dashboard.scan": {
        "zh": "参数扫描",
        "en": "Parameter Scan",
    },
    "dashboard.scan.desc": {
        "zh": "扫描参数并对比结果",
        "en": "Sweep parameters and compare results",
    },
    "dashboard.scenarios": {
        "zh": "场景预设",
        "en": "Scenario Presets",
    },
    "dashboard.workspace": {
        "zh": "工作区信息",
        "en": "Workspace Info",
    },

    # ═══════════════════════════════════════════════════════
    # Scenario names
    # ═══════════════════════════════════════════════════════
    "scenario.step": {
        "zh": "阶跃响应",
        "en": "Step Response",
    },
    "scenario.step.desc": {
        "zh": "经典速度阶跃响应 — 评估跟踪和超调",
        "en": "Classic speed step response — evaluate tracking and overshoot",
    },
    "scenario.ramp": {
        "zh": "斜坡测试",
        "en": "Ramp Test",
    },
    "scenario.ramp.desc": {
        "zh": "线性速度斜坡 — 评估跟踪滞后",
        "en": "Linear speed ramp — evaluate tracking lag",
    },
    "scenario.load": {
        "zh": "负载扰动",
        "en": "Load Disturbance",
    },
    "scenario.load.desc": {
        "zh": "t=0.5s 阶跃负载 — 评估抗扰性能",
        "en": "Step load at t=0.5s — evaluate disturbance rejection",
    },
    "scenario.sag": {
        "zh": "电压跌落",
        "en": "Voltage Sag",
    },
    "scenario.sag.desc": {
        "zh": "电压跌落穿越 — 评估鲁棒性",
        "en": "Voltage dip ride-through — evaluate robustness",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel
    # ═══════════════════════════════════════════════════════
    "config.file": {
        "zh": "配置文件",
        "en": "Configuration File",
    },
    "config.load": {
        "zh": "加载",
        "en": "Load",
    },
    "config.save": {
        "zh": "保存",
        "en": "Save",
    },
    "config.reset": {
        "zh": "重置",
        "en": "Reset",
    },
    "config.scenario": {
        "zh": "仿真场景",
        "en": "Scenario",
    },
    "config.motor": {
        "zh": "电机参数",
        "en": "Motor Parameters",
    },
    "config.foc": {
        "zh": "FOC 控制器",
        "en": "FOC Controller",
    },
    "config.speed_pi": {
        "zh": "速度环 PI",
        "en": "Speed Loop PI",
    },
    "config.sensors": {
        "zh": "传感器",
        "en": "Sensors",
    },
    "config.time": {
        "zh": "时间与求解器",
        "en": "Time & Solver",
    },
    "config.operating": {
        "zh": "工作点",
        "en": "Operating Point",
    },

    # ═══════════════════════════════════════════════════════
    # Stat Cards
    # ═══════════════════════════════════════════════════════
    "stat.speed": {
        "zh": "转速",
        "en": "Speed",
    },
    "stat.torque": {
        "zh": "转矩",
        "en": "Torque",
    },
    "stat.throughput": {
        "zh": "吞吐量",
        "en": "Throughput",
    },
    "stat.progress": {
        "zh": "进度",
        "en": "Progress",
    },

    # ═══════════════════════════════════════════════════════
    # Result Table
    # ═══════════════════════════════════════════════════════
    "result.metric": {
        "zh": "指标",
        "en": "Metric",
    },
    "result.value": {
        "zh": "值",
        "en": "Value",
    },
    "result.unit": {
        "zh": "单位",
        "en": "Unit",
    },
    "result.speed_section": {
        "zh": "── 转速 ──",
        "en": "── Speed ──",
    },
    "result.target_speed": {
        "zh": "目标转速",
        "en": "Target Speed",
    },
    "result.final_speed": {
        "zh": "最终转速",
        "en": "Final Speed",
    },
    "result.speed_error": {
        "zh": "转速误差",
        "en": "Speed Error",
    },
    "result.rise_time": {
        "zh": "上升时间 (10-90%)",
        "en": "Rise Time (10-90%)",
    },
    "result.settling_time": {
        "zh": "调节时间 (2%)",
        "en": "Settling Time (2%)",
    },
    "result.overshoot": {
        "zh": "超调量",
        "en": "Overshoot",
    },
    "result.torque_section": {
        "zh": "── 转矩 ──",
        "en": "── Torque ──",
    },
    "result.final_torque": {
        "zh": "最终转矩",
        "en": "Final Torque",
    },
    "result.peak_torque": {
        "zh": "峰值转矩",
        "en": "Peak Torque",
    },
    "result.current_section": {
        "zh": "── 电流 ──",
        "en": "── Currents ──",
    },
    "result.timing_section": {
        "zh": "── 时间 ──",
        "en": "── Timing ──",
    },
    "result.duration": {
        "zh": "仿真时长",
        "en": "Duration",
    },
    "result.data_points": {
        "zh": "数据点数",
        "en": "Data Points",
    },

    # ═══════════════════════════════════════════════════════
    # Log Widget
    # ═══════════════════════════════════════════════════════
    "log.filter.all": {
        "zh": "全部",
        "en": "All",
    },
    "log.filter.info": {
        "zh": "信息",
        "en": "Info",
    },
    "log.filter.warning": {
        "zh": "警告",
        "en": "Warning",
    },
    "log.filter.error": {
        "zh": "错误",
        "en": "Error",
    },
    "log.filter.success": {
        "zh": "成功",
        "en": "Success",
    },
    "log.search": {
        "zh": "搜索日志... (Ctrl+F)",
        "en": "Search logs... (Ctrl+F)",
    },
    "log.export": {
        "zh": "导出",
        "en": "Export",
    },
    "log.clear": {
        "zh": "清空",
        "en": "Clear",
    },
    "log.placeholder": {
        "zh": "仿真日志将在此显示...",
        "en": "Simulation log will appear here...",
    },

    # ═══════════════════════════════════════════════════════
    # Status Bar
    # ═══════════════════════════════════════════════════════
    "status.running": {
        "zh": "仿真运行中...",
        "en": "Simulation running...",
    },
    "status.paused": {
        "zh": "仿真已暂停",
        "en": "Simulation paused",
    },
    "status.stopping": {
        "zh": "正在停止...",
        "en": "Stopping...",
    },
    "status.complete": {
        "zh": "仿真完成",
        "en": "Simulation complete",
    },
    "status.failed": {
        "zh": "仿真失败",
        "en": "Simulation failed",
    },
    "status.scenario_loaded": {
        "zh": "场景已加载: {0}",
        "en": "Scenario loaded: {0}",
    },

    # ═══════════════════════════════════════════════════════
    # Dialogs
    # ═══════════════════════════════════════════════════════
    "dialog.about.title": {
        "zh": "关于 sim_platform",
        "en": "About sim_platform",
    },
    "dialog.about.desc": {
        "zh": "PySide6 桌面仿真平台\n替代 Textual TUI 的现代化界面",
        "en": "PySide6 Desktop Simulation Platform\nModern GUI replacing Textual TUI",
    },
    "dialog.close": {
        "zh": "关闭",
        "en": "Close",
    },
    "dialog.confirm_exit": {
        "zh": "确认退出",
        "en": "Confirm Exit",
    },
    "dialog.confirm_exit.msg": {
        "zh": "仿真仍在运行。确定要停止并退出吗？",
        "en": "Simulation is still running. Stop and exit?",
    },
    "dialog.access_denied": {
        "zh": "访问被拒绝",
        "en": "Access Denied",
    },
    "dialog.access_denied.msg": {
        "zh": "文件必须保存在项目工作区内。",
        "en": "Files must be saved within the project workspace.",
    },
    "dialog.no_data": {
        "zh": "没有数据",
        "en": "No Data",
    },
    "dialog.no_data.msg": {
        "zh": "没有仿真结果可导出。请先运行仿真。",
        "en": "No simulation results to export. Run a simulation first.",
    },
    "dialog.config_error": {
        "zh": "配置错误",
        "en": "Configuration Error",
    },
    "dialog.sim_error": {
        "zh": "仿真错误",
        "en": "Simulation Error",
    },
    "dialog.load_error": {
        "zh": "加载错误",
        "en": "Load Error",
    },
    "dialog.save_error": {
        "zh": "保存错误",
        "en": "Save Error",
    },
    "dialog.export_error": {
        "zh": "导出错误",
        "en": "Export Error",
    },
    "dialog.warning": {
        "zh": "警告",
        "en": "Warning",
    },
    "dialog.cannot_reset_running": {
        "zh": "仿真运行中无法重置配置。",
        "en": "Cannot reset while simulation is running.",
    },
    "dialog.file_not_found": {
        "zh": "文件未找到:\n{0}",
        "en": "File not found:\n{0}",
    },
    "dialog.failed_load": {
        "zh": "加载失败:\n{0}",
        "en": "Failed to load:\n{0}",
    },
    "dialog.failed_save": {
        "zh": "保存失败:\n{0}",
        "en": "Failed to save:\n{0}",
    },
    "dialog.failed_export": {
        "zh": "导出失败:\n{0}",
        "en": "Failed to export:\n{0}",
    },

    # ═══════════════════════════════════════════════════════
    # Shortcuts Dialog
    # ═══════════════════════════════════════════════════════
    "shortcuts.title": {
        "zh": "快捷键参考",
        "en": "Keyboard Shortcuts",
    },

    # ═══════════════════════════════════════════════════════
    # Onboarding
    # ═══════════════════════════════════════════════════════
    "onboarding.title": {
        "zh": "欢迎使用 sim_platform",
        "en": "Welcome to sim_platform",
    },
    "onboarding.welcome": {
        "zh": "欢迎使用多物理域联合仿真平台！\n这是一个专业的电机控制仿真工具，支持 PMSM FOC、BLDC、感应电机等多种拓扑。",
        "en": "Welcome to the Multi-Domain Co-Simulation Platform!\nA professional motor control simulation tool supporting PMSM FOC, BLDC, IM and more.",
    },
    "onboarding.step1.title": {
        "zh": "1. 配置参数",
        "en": "1. Configure Parameters",
    },
    "onboarding.step1.desc": {
        "zh": "在左侧面板中选择电机预设和仿真场景，调整控制器增益和传感器参数。所有参数均可通过 YAML 文件保存和加载。",
        "en": "Select motor presets and simulation scenarios in the left panel. Adjust controller gains and sensor parameters. All parameters can be saved/loaded via YAML files.",
    },
    "onboarding.step2.title": {
        "zh": "2. 运行仿真",
        "en": "2. Run Simulation",
    },
    "onboarding.step2.desc": {
        "zh": "点击 ▶ 运行按钮或按 F5 开始仿真。支持暂停 (F6) 和停止 (Shift+F5)。实时图表会显示转速和转矩曲线。",
        "en": "Click ▶ Run or press F5 to start. Supports Pause (F6) and Stop (Shift+F5). Real-time charts show speed and torque curves.",
    },
    "onboarding.step3.title": {
        "zh": "3. 分析结果",
        "en": "3. Analyze Results",
    },
    "onboarding.step3.desc": {
        "zh": "仿真完成后查看结果标签页：上升时间、调节时间、超调量等性能指标。支持导出 CSV/JSON 和 HDF5 日志。",
        "en": "After completion, check Results tab: rise time, settling time, overshoot and more. Export to CSV/JSON and HDF5 logs.",
    },
    "onboarding.step4.title": {
        "zh": "4. 进阶功能",
        "en": "4. Advanced Features",
    },
    "onboarding.step4.desc": {
        "zh": "• 参数扫描：批量扫描控制器增益\n• 故障注入：模拟电压跌落等场景\n• HDF5 回放：加载历史结果进行对比\n• 快捷键：Ctrl+1/2/3/4 切换标签页",
        "en": "• Parameter Scan: sweep controller gains\n• Fault Injection: simulate voltage sag etc.\n• HDF5 Replay: load and compare past results\n• Shortcuts: Ctrl+1/2/3/4 to switch tabs",
    },
    "onboarding.got_it": {
        "zh": "知道了，开始使用！",
        "en": "Got it, let's go!",
    },
    "onboarding.dont_show": {
        "zh": "不再显示",
        "en": "Don't show again",
    },

    # ═══════════════════════════════════════════════════════
    # Worker messages
    # ═══════════════════════════════════════════════════════
    "worker.starting": {
        "zh": "开始仿真...",
        "en": "Starting simulation...",
    },
    "worker.duration": {
        "zh": "  时长: {0}s | dt_c: {1}us | dt_s: {2}ms",
        "en": "  Duration: {0}s | dt_c: {1}us | dt_s: {2}ms",
    },
    "worker.motor": {
        "zh": "  电机: {0}",
        "en": "  Motor: {0}",
    },
    "worker.battery": {
        "zh": "  电池: {0}V / {1}Ohm",
        "en": "  Battery: {0}V / {1}Ohm",
    },
    "worker.target": {
        "zh": "  目标: {0} rad/s ({1} rpm)",
        "en": "  Target: {0} rad/s ({1} rpm)",
    },
    "worker.stopped": {
        "zh": "仿真已被用户停止。",
        "en": "Simulation stopped by user.",
    },
    "worker.results": {
        "zh": "结果:",
        "en": "Results:",
    },
    "worker.final_speed": {
        "zh": "  最终转速: {0} rad/s ({1} rpm)",
        "en": "  Final Speed: {0} rad/s ({1} rpm)",
    },
    "worker.error_label": {
        "zh": "  误差: {0}%",
        "en": "  Error: {0}%",
    },
    "worker.peak_torque": {
        "zh": "  峰值转矩: {0} N*m",
        "en": "  Peak Torque: {0} N*m",
    },

    # ═══════════════════════════════════════════════════════
    # Config validation
    # ═══════════════════════════════════════════════════════
    "validation.speed_range": {
        "zh": "转速参考必须在 5-500 rad/s 范围内",
        "en": "Speed reference must be 5-500 rad/s",
    },
    "validation.duration_range": {
        "zh": "仿真时长必须在 0.1-10s 范围内",
        "en": "Duration must be 0.1-10s",
    },
    "validation.load_range": {
        "zh": "负载转矩必须在 0-5 N*m 范围内",
        "en": "Load torque must be 0-5 N*m",
    },

    # ═══════════════════════════════════════════════════════
    # Scan Dialog
    # ═══════════════════════════════════════════════════════
    "scan.title": {
        "zh": "参数扫描",
        "en": "Parameter Scanner",
    },
    "scan.parameter": {
        "zh": "参数:",
        "en": "Parameter:",
    },
    "scan.values": {
        "zh": "扫描值:",
        "en": "Scan Values:",
    },
    "scan.run": {
        "zh": "运行扫描",
        "en": "Run Scan",
    },
    "scan.stop": {
        "zh": "停止",
        "en": "Stop",
    },
    "scan.duration": {
        "zh": "每次仿真时长 (s):",
        "en": "Duration per run (s):",
    },

    # ═══════════════════════════════════════════════════════
    # Scan Dialog (extended)
    # ═══════════════════════════════════════════════════════
    "scan.group.parameter": {
        "zh": "参数",
        "en": "Parameter",
    },
    "scan.select_param": {
        "zh": "选择要扫描的参数:",
        "en": "Select parameter to scan:",
    },
    "scan.placeholder": {
        "zh": "例如: 50, 100, 150, 200",
        "en": "e.g. 50, 100, 150, 200",
    },
    "scan.col.value": {
        "zh": "值",
        "en": "Value",
    },
    "scan.col.speed": {
        "zh": "转速 (rad/s)",
        "en": "Speed (rad/s)",
    },
    "scan.col.error": {
        "zh": "误差 (%)",
        "en": "Error (%)",
    },

    # ═══════════════════════════════════════════════════════
    # Tooltips (general)
    # ═══════════════════════════════════════════════════════
    "tooltip.new": {
        "zh": "重置配置为默认值",
        "en": "Reset configuration to defaults",
    },
    "tooltip.open": {
        "zh": "从 YAML/JSON 文件加载配置",
        "en": "Load configuration from YAML/JSON file",
    },
    "tooltip.save": {
        "zh": "保存当前配置",
        "en": "Save current configuration",
    },
    "tooltip.export_csv": {
        "zh": "导出仿真结果为 CSV 文件",
        "en": "Export simulation results to CSV",
    },
    "tooltip.home": {
        "zh": "返回首页 (Ctrl+1)",
        "en": "Go to dashboard (Ctrl+1)",
    },
    "tooltip.run": {
        "zh": "开始仿真 (F5)",
        "en": "Start simulation (F5)",
    },
    "tooltip.pause": {
        "zh": "暂停/继续仿真 (F6)",
        "en": "Pause/Resume simulation (F6)",
    },
    "tooltip.stop": {
        "zh": "停止仿真 (Shift+F5)",
        "en": "Stop simulation (Shift+F5)",
    },
    "tooltip.results": {
        "zh": "显示结果表格 (Ctrl+3)",
        "en": "Show results table (Ctrl+3)",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel tooltips
    # ═══════════════════════════════════════════════════════
    "tooltip.load_config": {
        "zh": "从 YAML/JSON 文件加载配置",
        "en": "Load configuration from YAML/JSON file",
    },
    "tooltip.save_config": {
        "zh": "保存当前配置到文件",
        "en": "Save current configuration to file",
    },
    "tooltip.reset_config": {
        "zh": "重置所有参数为默认值",
        "en": "Reset all parameters to defaults",
    },
    "tooltip.select_scenario": {
        "zh": "选择预设场景自动填充参数。",
        "en": "Select a preset scenario to auto-fill parameters.",
    },
    "tooltip.profile": {
        "zh": "参考信号波形类型。",
        "en": "Reference signal profile type.",
    },
    "tooltip.motor_preset": {
        "zh": "选择电机预设。",
        "en": "Select motor preset.",
    },
    "tooltip.dt_current": {
        "zh": "电流控制环时间步长",
        "en": "Current control loop time step",
    },
    "tooltip.dt_speed": {
        "zh": "速度控制环时间步长",
        "en": "Speed control loop time step",
    },
    "tooltip.duration": {
        "zh": "仿真时长 (0.1-10s)",
        "en": "Simulation duration (0.1-10s)",
    },
    "tooltip.solver": {
        "zh": "数值积分方法",
        "en": "Numerical integration method",
    },
    "tooltip.speed_ref": {
        "zh": "目标转速参考 (5-500 rad/s)",
        "en": "Target speed reference (5-500 rad/s)",
    },
    "tooltip.load_torque": {
        "zh": "负载转矩 (0-5 N*m)",
        "en": "Load torque (0-5 N*m)",
    },
    "tooltip.battery_v": {
        "zh": "电池电压 (12-800V)",
        "en": "Battery voltage (12-800V)",
    },
    "tooltip.battery_r": {
        "zh": "电池内阻",
        "en": "Battery internal resistance",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel labels
    # ═══════════════════════════════════════════════════════
    "config.label.scenario": {
        "zh": "场景:",
        "en": "Scenario:",
    },
    "config.label.profile": {
        "zh": "波形:",
        "en": "Profile:",
    },
    "config.label.preset": {
        "zh": "预设:",
        "en": "Preset:",
    },
    "config.label.duration": {
        "zh": "时长:",
        "en": "Duration:",
    },
    "config.label.solver": {
        "zh": "求解器:",
        "en": "Solver:",
    },
    "config.label.speed_ref": {
        "zh": "目标转速:",
        "en": "Speed Ref:",
    },
    "config.label.load_torque": {
        "zh": "负载转矩:",
        "en": "Load Torque:",
    },
    "config.label.battery_v": {
        "zh": "电池电压:",
        "en": "Battery V:",
    },
    "config.label.battery_r": {
        "zh": "电池内阻:",
        "en": "Battery R:",
    },
    "config.default": {
        "zh": "(默认)",
        "en": "(default)",
    },
    "config.forward_euler": {
        "zh": "前向欧拉",
        "en": "Forward Euler",
    },
    "config.runge_kutta": {
        "zh": "四阶龙格-库塔",
        "en": "Runge-Kutta 4",
    },

    # ═══════════════════════════════════════════════════════
    # Log Widget tooltips
    # ═══════════════════════════════════════════════════════
    "tooltip.filter_level": {
        "zh": "按日志级别筛选",
        "en": "Filter by log level",
    },
    "tooltip.search_log": {
        "zh": "搜索日志消息 (文本匹配)",
        "en": "Search log messages (text match)",
    },
    "tooltip.export_log": {
        "zh": "导出日志到文本文件",
        "en": "Export log to text file",
    },
    "tooltip.clear_log": {
        "zh": "清空所有日志消息",
        "en": "Clear all log messages",
    },

    # ═══════════════════════════════════════════════════════
    # Chart Widget
    # ═══════════════════════════════════════════════════════
    "chart.title": {
        "zh": "仿真结果",
        "en": "Simulation Results",
    },
    "chart.series.speed": {
        "zh": "转速 (rad/s)",
        "en": "Speed (rad/s)",
    },
    "chart.series.ref": {
        "zh": "目标转速",
        "en": "Speed Ref",
    },
    "chart.series.torque": {
        "zh": "转矩 (N*m)",
        "en": "Torque (N*m)",
    },
    "chart.axis.time": {
        "zh": "时间 (s)",
        "en": "Time (s)",
    },
    "chart.axis.speed": {
        "zh": "转速 (rad/s)",
        "en": "Speed (rad/s)",
    },
    "chart.axis.torque": {
        "zh": "转矩 (N*m)",
        "en": "Torque (N*m)",
    },

    # ═══════════════════════════════════════════════════════
    # Result Table (extended)
    # ═══════════════════════════════════════════════════════
    "result.final_id": {
        "zh": "最终 Id",
        "en": "Final Id",
    },
    "result.final_iq": {
        "zh": "最终 Iq",
        "en": "Final Iq",
    },
    "result.peak_current": {
        "zh": "峰值相电流",
        "en": "Peak Phase Current",
    },

    # ═══════════════════════════════════════════════════════
    # File dialogs
    # ═══════════════════════════════════════════════════════
    "dialog.open_config": {
        "zh": "打开配置文件",
        "en": "Open Configuration",
    },
    "dialog.save_config": {
        "zh": "保存配置文件",
        "en": "Save Configuration",
    },
    "dialog.export_csv": {
        "zh": "导出 CSV",
        "en": "Export CSV",
    },
    "dialog.export_json": {
        "zh": "导出 JSON",
        "en": "Export JSON",
    },
    "dialog.export_log": {
        "zh": "导出日志",
        "en": "Export Log",
    },
    "dialog.load_hdf5": {
        "zh": "加载 HDF5 结果",
        "en": "Load HDF5 Results",
    },
    "dialog.hdf5_no_data": {
        "zh": "HDF5 文件不包含转速/时间数据。",
        "en": "HDF5 file does not contain speed/time data.",
    },
    "dialog.hdf5_error": {
        "zh": "加载 HDF5 失败:\n{0}",
        "en": "Failed to load HDF5:\n{0}",
    },

    # ═══════════════════════════════════════════════════════
    # Toolbar title
    # ═══════════════════════════════════════════════════════
    "toolbar.title": {
        "zh": "主工具栏",
        "en": "Main Toolbar",
    },

    # ═══════════════════════════════════════════════════════
    # Status bar (extended)
    # ═══════════════════════════════════════════════════════
    "status.resumed": {
        "zh": "仿真已继续",
        "en": "Simulation resumed",
    },

    # ═══════════════════════════════════════════════════════
    # Worker messages (extended)
    # ═══════════════════════════════════════════════════════
    "worker.foc_params": {
        "zh": "  FOC: kp_id={0} ki_id={1} kp_iq={2} ki_iq={3}",
        "en": "  FOC: kp_id={0} ki_id={1} kp_iq={2} ki_iq={3}",
    },
    "worker.speed_pi_params": {
        "zh": "  速度PI: kp={0} ki={1}",
        "en": "  Speed PI: kp={0} ki={1}",
    },
    "worker.progress": {
        "zh": "  步进 {0}/{1} ({2}%) -- {3} 步/秒 -- 剩余 {4}s",
        "en": "  Step {0}/{1} ({2}%) -- {3} steps/s -- ETA {4}s",
    },

    # ═══════════════════════════════════════════════════════
    # Dashboard workspace info
    # ═══════════════════════════════════════════════════════
    "workspace.dir": {
        "zh": "工作区目录",
        "en": "Workspace",
    },
    "workspace.configs": {
        "zh": "配置目录",
        "en": "Configs",
    },
    "workspace.output": {
        "zh": "输出目录",
        "en": "Output",
    },
    "workspace.logs": {
        "zh": "日志目录",
        "en": "Logs",
    },

    # ═══════════════════════════════════════════════════════
    # Config access denied (from config_panel)
    # ═══════════════════════════════════════════════════════
    "dialog.config_load_error": {
        "zh": "加载配置失败:\n{0}",
        "en": "Failed to load config:\n{0}",
    },
    "dialog.config_save_error": {
        "zh": "保存配置失败:\n{0}",
        "en": "Failed to save config:\n{0}",
    },

    # ═══════════════════════════════════════════════════════
    # Log export
    # ═══════════════════════════════════════════════════════
    "log.export_title": {
        "zh": "导出日志",
        "en": "Export Log",
    },

    # ═══════════════════════════════════════════════════════
    # Config file dialog (from config_panel)
    # ═══════════════════════════════════════════════════════
    "config.open_title": {
        "zh": "打开配置文件",
        "en": "Load Configuration",
    },
    "config.save_title": {
        "zh": "保存配置文件",
        "en": "Save Configuration",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel — Parameter descriptions (physical meaning)
    # ═══════════════════════════════════════════════════════
    "param.Rs.desc": {
        "zh": "定子相电阻 [Ω]。影响铜损和电流响应速度。典型值: 0.01-1.0 Ω",
        "en": "Stator phase resistance [Ω]. Affects copper loss and current response. Typical: 0.01-1.0 Ω",
    },
    "param.Ld.desc": {
        "zh": "d轴电感 [H]。影响d轴电流环带宽和弱磁能力。典型值: 0.1-10 mH",
        "en": "d-axis inductance [H]. Affects d-axis current loop bandwidth. Typical: 0.1-10 mH",
    },
    "param.Lq.desc": {
        "zh": "q轴电感 [H]。影响转矩输出和MTPA策略。典型值: 0.1-10 mH",
        "en": "q-axis inductance [H]. Affects torque output and MTPA strategy. Typical: 0.1-10 mH",
    },
    "param.flux_pm.desc": {
        "zh": "永磁体磁链 [Wb]。决定反电动势常数和最大转矩。典型值: 0.01-1.0 Wb",
        "en": "PM flux linkage [Wb]. Determines back-EMF and max torque. Typical: 0.01-1.0 Wb",
    },
    "param.J.desc": {
        "zh": "转动惯量 [kg·m²]。影响加速度和速度环响应。典型值: 1e-5 ~ 1.0",
        "en": "Moment of inertia [kg·m²]. Affects acceleration and speed response. Typical: 1e-5 ~ 1.0",
    },
    "param.B.desc": {
        "zh": "粘性摩擦系数 [N·m·s]。提供阻尼，防止振荡。典型值: 1e-5 ~ 0.1",
        "en": "Viscous friction [N·m·s]. Provides damping to prevent oscillation. Typical: 1e-5 ~ 0.1",
    },
    "param.Pp.desc": {
        "zh": "极对数。决定电频率与机械频率的比值。典型值: 2-8",
        "en": "Pole pairs. Determines electrical/mechanical frequency ratio. Typical: 2-8",
    },
    "param.kp_id.desc": {
        "zh": "d轴电流环比例增益。增大可加快响应，但过大会振荡。建议: 1-50",
        "en": "d-axis current P gain. Faster response but may oscillate if too high. Suggested: 1-50",
    },
    "param.ki_id.desc": {
        "zh": "d轴电流环积分增益。消除稳态误差，过大会超调。建议: 100-5000",
        "en": "d-axis current I gain. Eliminates steady-state error. Suggested: 100-5000",
    },
    "param.kp_iq.desc": {
        "zh": "q轴电流环比例增益。直接影响转矩响应速度。建议: 1-50",
        "en": "q-axis current P gain. Directly affects torque response. Suggested: 1-50",
    },
    "param.ki_iq.desc": {
        "zh": "q轴电流环积分增益。消除转矩稳态误差。建议: 100-5000",
        "en": "q-axis current I gain. Eliminates torque steady-state error. Suggested: 100-5000",
    },
    "param.spd_kp.desc": {
        "zh": "速度环比例增益。决定速度跟踪的快速性。建议: 0.01-1.0",
        "en": "Speed loop P gain. Determines speed tracking quickness. Suggested: 0.01-1.0",
    },
    "param.spd_ki.desc": {
        "zh": "速度环积分增益。消除速度稳态误差。建议: 0.01-10.0",
        "en": "Speed loop I gain. Eliminates speed steady-state error. Suggested: 0.01-10.0",
    },
    "param.current_noise.desc": {
        "zh": "电流传感器噪声标准差 [A]。模拟ADC量化和EMI干扰。典型: 0.01-0.5 A",
        "en": "Current sensor noise std [A]. Simulates ADC quantization and EMI. Typical: 0.01-0.5 A",
    },
    "param.current_bias.desc": {
        "zh": "电流传感器零偏 [A]。模拟传感器温漂和校准误差。典型: 0-0.1 A",
        "en": "Current sensor bias [A]. Simulates thermal drift and calibration error. Typical: 0-0.1 A",
    },
    "param.encoder_noise.desc": {
        "zh": "编码器角度噪声 [rad]。模拟位置传感器精度限制。典型: 0.0001-0.01 rad",
        "en": "Encoder angle noise [rad]. Simulates position sensor precision. Typical: 0.0001-0.01 rad",
    },

    # ═══════════════════════════════════════════════════════
    # Dashboard — Getting Started guide
    # ═══════════════════════════════════════════════════════
    "dashboard.getting_started": {
        "zh": "  快速入门",
        "en": "  Getting Started",
    },
    "dashboard.step1": {
        "zh": "① 在左侧配置面板选择电机预设和仿真场景",
        "en": "① Select motor preset and scenario in the left config panel",
    },
    "dashboard.step2": {
        "zh": "② 点击 ▶ 运行 或按 F5 开始仿真",
        "en": "② Click ▶ Run or press F5 to start simulation",
    },
    "dashboard.step3": {
        "zh": "③ 在图表/日志/结果标签页查看仿真数据",
        "en": "③ View data in Chart/Log/Results tabs",
    },
    "dashboard.step4": {
        "zh": "④ 使用参数扫描工具进行批量对比分析",
        "en": "④ Use Parameter Scanner for batch comparison",
    },

    # ═══════════════════════════════════════════════════════
    # Chart — Empty state & interaction hints
    # ═══════════════════════════════════════════════════════
    "chart.empty": {
        "zh": "暂无数据 — 请先运行仿真 (F5)",
        "en": "No data — Run a simulation first (F5)",
    },
    "chart.hint.zoom": {
        "zh": "滚轮缩放 · 拖拽平移",
        "en": "Scroll to zoom · Drag to pan",
    },

    # ═══════════════════════════════════════════════════════
    # Scan Dialog (extended)
    # ═══════════════════════════════════════════════════════
    "scan.start": {
        "zh": "开始扫描",
        "en": "Start Scan",
    },
    "scan.values_label": {
        "zh": "扫描值 (逗号分隔):",
        "en": "Values (comma-separated):",
    },
    "scan.error.parse": {
        "zh": "解析值出错: {0}",
        "en": "Error parsing values: {0}",
    },
    "scan.error.need2": {
        "zh": "至少需要 2 个值",
        "en": "Need at least 2 values",
    },
    "scan.error.max100": {
        "zh": "最多 100 个值",
        "en": "Maximum 100 values allowed",
    },
    "scan.error.nan": {
        "zh": "值不能为 NaN 或 Inf",
        "en": "Values cannot be NaN or Inf",
    },
    "scan.completed": {
        "zh": "扫描完成: {0} 个结果",
        "en": "Scan complete: {0} results",
    },
    "scan.stopped": {
        "zh": "扫描已停止",
        "en": "Scan stopped",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel — Motor presets (bilingual)
    # ═══════════════════════════════════════════════════════
    "preset.small_pmsm": {
        "zh": "小型 PMSM (200W)",
        "en": "Small PMSM (200W)",
    },
    "preset.medium_pmsm": {
        "zh": "中型 PMSM (2kW)",
        "en": "Medium PMSM (2kW)",
    },
    "preset.large_pmsm": {
        "zh": "大型 PMSM (20kW)",
        "en": "Large PMSM (20kW)",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel — Scenario names (bilingual)
    # ═══════════════════════════════════════════════════════
    "scenario.step_name": {
        "zh": "阶跃响应",
        "en": "Step Response",
    },
    "scenario.ramp_name": {
        "zh": "斜坡测试",
        "en": "Ramp Test",
    },
    "scenario.load_name": {
        "zh": "负载扰动",
        "en": "Load Disturbance",
    },
    "scenario.sag_name": {
        "zh": "电压跌落穿越",
        "en": "Voltage Sag Ride-Through",
    },

    # ═══════════════════════════════════════════════════════
    # Scan parameter names (bilingual)
    # ═══════════════════════════════════════════════════════
    "scan.param.speed": {
        "zh": "转速参考",
        "en": "Speed Reference",
    },
    "scan.param.kp_id": {
        "zh": "FOC d轴 Kp",
        "en": "FOC kp_id",
    },
    "scan.param.ki_id": {
        "zh": "FOC d轴 Ki",
        "en": "FOC ki_id",
    },
    "scan.param.spd_kp": {
        "zh": "速度环 Kp",
        "en": "Speed Loop Kp",
    },
    "scan.param.load": {
        "zh": "负载转矩",
        "en": "Load Torque",
    },

    # ═══════════════════════════════════════════════════════
    # Config Panel — additional labels
    # ═══════════════════════════════════════════════════════
    "config.label.dt_current": {
        "zh": "电流环步长:",
        "en": "dt_current:",
    },
    "config.label.dt_speed": {
        "zh": "速度环步长:",
        "en": "dt_speed:",
    },
    "config.label.kp_id": {
        "zh": "d轴 Kp:",
        "en": "d-axis Kp:",
    },
    "config.label.ki_id": {
        "zh": "d轴 Ki:",
        "en": "d-axis Ki:",
    },
    "config.label.kp_iq": {
        "zh": "q轴 Kp:",
        "en": "q-axis Kp:",
    },
    "config.label.ki_iq": {
        "zh": "q轴 Ki:",
        "en": "q-axis Ki:",
    },
    "config.label.spd_kp": {
        "zh": "Kp:",
        "en": "Kp:",
    },
    "config.label.spd_ki": {
        "zh": "Ki:",
        "en": "Ki:",
    },
    "config.label.current_noise": {
        "zh": "电流噪声:",
        "en": "Current Noise:",
    },
    "config.label.current_bias": {
        "zh": "电流零偏:",
        "en": "Current Bias:",
    },
    "config.label.encoder_noise": {
        "zh": "编码器噪声:",
        "en": "Encoder Noise:",
    },

    # ═══════════════════════════════════════════════════════
    # Result Table — empty state
    # ═══════════════════════════════════════════════════════
    "result.empty": {
        "zh": "暂无结果 — 请先运行仿真",
        "en": "No results — Run a simulation first",
    },

    # ═══════════════════════════════════════════════════════
    # Onboarding — extended steps
    # ═══════════════════════════════════════════════════════
    "onboarding.version": {
        "zh": "版本 {0}",
        "en": "Version {0}",
    },
    "onboarding.features": {
        "zh": "支持: PMSM (L2/L3) · BLDC · 感应电机\n控制器: FOC · MPC · EKF · PI\n采样率: 174k 步/秒",
        "en": "Supports: PMSM (L2/L3) · BLDC · Induction Motor\nControllers: FOC · MPC · EKF · PI\nThroughput: 174k steps/sec",
    },

    # ═══════════════════════════════════════════════════════
    # Result table re-run button
    # ═══════════════════════════════════════════════════════
    "result.rerun": {
        "zh": "▶ 重新运行",
        "en": "▶ Re-run",
    },
    "result.rerun.tooltip": {
        "zh": "使用相同配置重新运行仿真 (Ctrl+R)",
        "en": "Re-run simulation with same config (Ctrl+R)",
    },

    # ═══════════════════════════════════════════════════════
    # Status messages for config feedback
    # ═══════════════════════════════════════════════════════
    "status.config_modified": {
        "zh": "配置已修改",
        "en": "Config modified",
    },
    "status.config_saved": {
        "zh": "配置已保存: {0}",
        "en": "Config saved: {0}",
    },
    "status.config_loaded": {
        "zh": "配置已加载: {0}",
        "en": "Config loaded: {0}",
    },

    # ═══════════════════════════════════════════════════════
    # Scan cancel confirmation
    # ═══════════════════════════════════════════════════════
    "scan.cancel_confirm": {
        "zh": "确定要取消扫描吗？",
        "en": "Are you sure you want to cancel the scan?",
    },
    "scan.cancel_confirm.title": {
        "zh": "取消扫描",
        "en": "Cancel Scan",
    },

    # ═══════════════════════════════════════════════════════
    # Conflict Resolution Dialog
    # ═══════════════════════════════════════════════════════
    "conflict.title": {
        "zh": "参数冲突解析器 — 多策略冲突处理",
        "en": "Conflict Resolver — Multi-Strategy Resolution",
    },
    "conflict.subtitle": {
        "zh": "检测到参数配置存在潜在问题。请为每个冲突选择处理策略。",
        "en": "Potential issues detected in parameter configuration. Select a resolution strategy for each conflict.",
    },
    "conflict.batch_label": {
        "zh": "批量处理:",
        "en": "Batch:",
    },
    "conflict.auto_all": {
        "zh": "全部自动修复",
        "en": "Auto-Fix All",
    },
    "conflict.ignore_all": {
        "zh": "全部忽略",
        "en": "Ignore All",
    },
    "conflict.remember": {
        "zh": "记住我的选择，下次不再询问此类冲突",
        "en": "Remember my choices, don't ask for similar conflicts again",
    },
    "conflict.apply": {
        "zh": "应用并继续",
        "en": "Apply & Continue",
    },
    "conflict.impact.accuracy": {
        "zh": "精度",
        "en": "Accuracy",
    },
    "conflict.impact.stability": {
        "zh": "稳定性",
        "en": "Stability",
    },
    "conflict.impact.convergence": {
        "zh": "收敛性",
        "en": "Convergence",
    },
    "conflict.strategy.ask": {
        "zh": "每次询问",
        "en": "Ask Each Time",
    },
    "conflict.strategy.auto_fix": {
        "zh": "自动修复",
        "en": "Auto-Fix",
    },
    "conflict.strategy.manual": {
        "zh": "手动调整",
        "en": "Manual Adjust",
    },
    "conflict.strategy.ignore_this": {
        "zh": "本次忽略",
        "en": "Ignore This Run",
    },
    "conflict.strategy.ignore_always": {
        "zh": "始终忽略",
        "en": "Ignore Always",
    },

    # ═══════════════════════════════════════════════════════
    # Solver Presets
    # ═══════════════════════════════════════════════════════
    "solver.title": {
        "zh": "求解器配置",
        "en": "Solver Configuration",
    },
    "solver.preset": {
        "zh": "求解器预设",
        "en": "Solver Preset",
    },
    "solver.standard": {
        "zh": "标准 (Forward Euler)",
        "en": "Standard (Forward Euler)",
    },
    "solver.standard.desc": {
        "zh": "快速稳定的 Forward Euler 积分，适用于大多数 PMSM/BLDC/IM 仿真。50us 电流环，1ms 速度环。",
        "en": "Fast, stable Forward Euler integration for most PMSM/BLDC/IM simulations.",
    },
    "solver.high_precision": {
        "zh": "高精度 (RK4)",
        "en": "High Precision (RK4)",
    },
    "solver.high_precision.desc": {
        "zh": "四阶 Runge-Kutta 积分，25us 电流环，500us 速度环。精度高但速度约为 Forward Euler 的 1/4。",
        "en": "4th-order Runge-Kutta. Higher accuracy, ~4x slower than Forward Euler.",
    },
    "solver.adaptive": {
        "zh": "自适应步长 (RK45)",
        "en": "Adaptive Step (RK45)",
    },
    "solver.adaptive.desc": {
        "zh": "自适应 Dormand-Prince RK5(4)，自动调整步长。适用于刚性系统或变动态场景。",
        "en": "Adaptive Dormand-Prince RK5(4) with auto step size control.",
    },
    "solver.realtime": {
        "zh": "实时 (HIL)",
        "en": "Real-Time (HIL)",
    },
    "solver.realtime.desc": {
        "zh": "实时硬件在环求解器。严格固定步长 Forward Euler 100us。超时即中止。",
        "en": "Real-time Hardware-in-the-Loop solver. Fixed-step 100us. Aborts on deadline miss.",
    },
    "solver.frozen_hint": {
        "zh": "此参数在当前预设中被锁定",
        "en": "This parameter is locked in the current preset",
    },
    "solver.config_saved": {
        "zh": "求解器配置已保存",
        "en": "Solver configuration saved",
    },
    "solver.preset_applied": {
        "zh": "求解器预设已应用: {0}",
        "en": "Solver preset applied: {0}",
    },

    # ═══════════════════════════════════════════════════════
    # Guided Tour
    # ═══════════════════════════════════════════════════════
    "tour.start": {
        "zh": "开始导览",
        "en": "Start Tour",
    },
    "tour.welcome.title": {
        "zh": "欢迎使用仿真平台",
        "en": "Welcome to Sim Platform",
    },
    "tour.welcome.desc": {
        "zh": "这是一个工业级多物理域联合仿真平台。本导览将带您了解主要功能。",
        "en": "Industrial-grade multi-domain co-simulation platform. This tour will guide you through the main features.",
    },
    "tour.dashboard.title": {
        "zh": "仪表板首页",
        "en": "Dashboard",
    },
    "tour.dashboard.desc": {
        "zh": "首页提供快速启动、场景预设和操作指引。您可以在这里一键开始仿真或选择预定义场景。",
        "en": "The dashboard provides quick start, scenario presets, and operation guides.",
    },
    "tour.config.title": {
        "zh": "配置面板",
        "en": "Configuration Panel",
    },
    "tour.config.desc": {
        "zh": "左侧配置面板包含电机参数、控制器增益、传感器设置等。所有参数支持中英文双语描述和实时验证。",
        "en": "Left panel contains motor parameters, controller gains, sensor settings, etc.",
    },
    "tour.run.title": {
        "zh": "运行仿真",
        "en": "Run Simulation",
    },
    "tour.run.desc": {
        "zh": "设置好参数后，点击工具栏的运行按钮或按 F5 开始仿真。进度条会显示仿真进度。",
        "en": "After configuring parameters, click Run or press F5 to start simulation.",
    },
    "tour.chart.title": {
        "zh": "实时图表",
        "en": "Real-Time Chart",
    },
    "tour.chart.desc": {
        "zh": "仿真运行时，图表标签页会实时显示速度、转矩和电流曲线。支持缩放和拖拽。",
        "en": "During simulation, the chart tab shows real-time speed, torque and current curves.",
    },
    "tour.results.title": {
        "zh": "结果分析",
        "en": "Results Analysis",
    },
    "tour.results.desc": {
        "zh": "仿真完成后，结果标签页会展示详细的性能指标：上升时间、调节时间、超调量等。",
        "en": "After simulation, the results tab shows detailed performance metrics.",
    },
    "tour.export.title": {
        "zh": "导出数据",
        "en": "Export Data",
    },
    "tour.export.desc": {
        "zh": "完整的仿真结果可以导出为 CSV、JSON 或 HDF5 格式，用于进一步分析和报告。",
        "en": "Export results to CSV, JSON, or HDF5 for further analysis.",
    },
    "tour.shortcuts.title": {
        "zh": "快捷键",
        "en": "Keyboard Shortcuts",
    },
    "tour.shortcuts.desc": {
        "zh": "常用操作都有键盘快捷键：F5 运行、F6 暂停、Ctrl+S 保存、Ctrl+O 打开、Ctrl+1-4 切换标签页。F1 查看完整快捷键。",
        "en": "Common operations have keyboard shortcuts: F5 Run, F6 Pause, Ctrl+S Save, etc.",
    },
    "tour.complete.title": {
        "zh": "准备就绪",
        "en": "Ready",
    },
    "tour.complete.desc": {
        "zh": "您已了解平台的基本功能。点击「知道了」关闭导览，开始使用仿真平台吧！",
        "en": "You've learned the basics. Click OK to start using the platform!",
    },

    # ═══════════════════════════════════════════════════════
    # Icon system (toolbar actions with new icons)
    # ═══════════════════════════════════════════════════════
    "toolbar.home": {
        "zh": "首页",
        "en": "Home",
    },
    "toolbar.run": {
        "zh": "运行",
        "en": "Run",
    },
    "toolbar.pause": {
        "zh": "暂停",
        "en": "Pause",
    },
    "toolbar.resume": {
        "zh": "继续",
        "en": "Resume",
    },
    "toolbar.stop": {
        "zh": "停止",
        "en": "Stop",
    },
    "toolbar.results": {
        "zh": "结果",
        "en": "Results",
    },
    "toolbar.title": {
        "zh": "工具栏",
        "en": "Toolbar",
    },

    # ═══════════════════════════════════════════════════════
    # Context Help
    # ═══════════════════════════════════════════════════════
    "help.context": {
        "zh": "上下文帮助",
        "en": "Context Help",
    },
    "help.no_context": {
        "zh": "当前没有可用的上下文帮助。",
        "en": "No context help available.",
    },

    # ═══════════════════════════════════════════════════════
    # Quality Gates & Error Messages
    # ═══════════════════════════════════════════════════════
    "quality.check_failed": {
        "zh": "代码质量检查未通过",
        "en": "Code quality check failed",
    },
    "quality.interface_mismatch": {
        "zh": "模块接口不一致: {0}",
        "en": "Module interface mismatch: {0}",
    },

}

# ── Language management functions ───────────────────────────


def tr(key: str, *args) -> str:
    """Translate a key to the current language.

    Args:
        key: Translation key (e.g. "app.title")
        *args: Format arguments for {0}, {1}, etc.

    Returns:
        Translated string, or key if translation not found.
    """
    entry = _TRANSLATIONS.get(key)
    if entry is None:
        return key
    text = entry.get(_current_lang, entry.get("en", key))
    if args:
        try:
            text = text.format(*args)
        except (IndexError, KeyError):
            pass
    return text


def set_language(lang: str) -> None:
    """Set the current language ("zh" or "en")."""
    global _current_lang
    if lang in ("zh", "en"):
        _current_lang = lang
        try:
            from PySide6.QtCore import QSettings
            settings = QSettings("SimPlatform", "GUI")
            settings.setValue("language", lang)
        except Exception:
            pass


def get_language() -> str:
    """Get the current language code."""
    return _current_lang


def load_language() -> str:
    """Load saved language from QSettings. Returns the loaded language."""
    global _current_lang
    try:
        from PySide6.QtCore import QSettings
        settings = QSettings("SimPlatform", "GUI")
        saved = settings.value("language", "zh")
        if saved in ("zh", "en"):
            _current_lang = saved
    except Exception:
        pass
    return _current_lang


def get_supported_languages() -> list[tuple[str, str]]:
    """Return list of (code, display_name) tuples for supported languages."""
    return [("zh", "中文"), ("en", "English")]
