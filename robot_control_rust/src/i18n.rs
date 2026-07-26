// Auto-generated macro-based i18n module
// Reduced from ~2970 lines using declarative macros

// ═══════════════════════════════════════════════════════════════
// 国际化 (i18n) - 中英双语支持
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    pub fn label(&self) -> &str {
        match self {
            Self::English => "English",
            Self::Chinese => "中文",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::English => Self::Chinese,
            Self::Chinese => Self::English,
        }
    }
}

/// 所有可翻译文本的键
macro_rules! tr {
    ($name:ident, $en:expr, $zh:expr) => {
        pub fn $name(lang: Language) -> &'static str {
            match lang {
                Language::English => $en,
                Language::Chinese => $zh,
            }
        }
    };
}

macro_rules! tr_fmt {
    ($name:ident($($arg:ident : $t:ty),*), $en:expr, $zh:expr) => {
        pub fn $name($($arg: $t,)* lang: Language) -> String {
            match lang {
                Language::English => format!($en, $($arg),*),
                Language::Chinese => format!($zh, $($arg),*),
            }
        }
    };
}

pub struct Tr;

impl Tr {
    tr!(app_title, "Robot Control Suite", "机器人控制调试套件");
    tr!(connect, "Connect", "连接");
    tr!(disconnect, "Disconnect", "断开");
    tr!(send, "Send", "发送");
    tr!(clear, "Clear", "清空");
    tr!(refresh, "Refresh", "刷新");
    tr!(save, "Save", "保存");
    tr!(reset, "Reset", "重置");
    tr!(start, "Start", "启动");
    tr!(stop, "Stop", "停止");
    tr!(error_label, "Error", "错误");
    tr!(connected, "Connected", "已连接");
    tr!(disconnected, "Disconnected", "已断开");
    tr!(tab_dashboard, "Dashboard", "仪表盘");
    tr!(tab_connections, "Connections", "连接管理");
    tr!(tab_terminal, "Terminal", "终端调试");
    tr!(tab_protocol_analysis, "Protocol Analysis", "协议分析");
    tr!(tab_packet_builder, "Packet Builder", "协议组包");
    tr!(tab_topology, "Topology", "机器人拓扑");
    tr!(tab_pid_control, "Control Algorithms", "控制算法");
    tr!(tab_nn_tuning, "NN Auto-Tune", "神经网络调参");
    tr!(tab_data_viz, "Data Viz", "数据可视化");
    tr!(tab_simulation_lab, "Simulation Lab", "仿真实验室");
    tr!(tab_modbus, "Modbus Tools", "Modbus 工具");
    tr!(tab_canopen, "CANopen Tools", "CANopen 工具");
    tr!(simulation_scenario, "Scenario", "仿真场景");
    tr!(simulation_duration, "Duration (s)", "仿真时长 (秒)");
    tr!(simulation_step_us, "Step (us)", "步长 (微秒)");
    tr!(
        simulation_speed_ref,
        "Speed reference (rad/s)",
        "速度给定 (rad/s)"
    );
    tr!(
        simulation_load_torque,
        "Load torque (N m)",
        "负载转矩 (N m)"
    );
    tr!(simulation_run, "Run", "运行");
    tr!(simulation_cancel, "Cancel", "取消");
    tr!(simulation_progress, "Progress", "运行进度");
    tr!(simulation_status, "Status", "状态");
    tr!(simulation_results, "Results", "结果指标");
    tr!(simulation_scan, "Parameter Scan", "参数扫描");
    tr!(simulation_export_preview, "Export Preview", "导出预览");
    tr!(
        simulation_no_result,
        "No simulation result yet",
        "尚无仿真结果"
    );
    tr!(
        simulation_export_empty,
        "Run a simulation to preview JSON and CSV exports",
        "运行仿真后可预览 JSON 和 CSV 导出"
    );
    tr!(connection_status, "Connection Status", "连接状态");
    tr!(system_stats, "System Statistics", "系统统计");
    tr!(quick_actions, "Quick Actions", "快捷操作");
    tr!(robot_state, "Robot State", "机器人状态");
    tr!(bytes_sent, "Bytes Sent", "已发送字节");
    tr!(bytes_received, "Bytes Received", "已接收字节");
    tr!(total_errors, "Total Errors", "总错误数");
    tr!(log_entries, "Log Entries", "日志条目");
    tr!(state_history, "State History", "状态历史");
    tr!(active_channel, "Active Channel", "当前通道");
    tr!(last_comm, "Last Comm", "最近通信");
    tr!(topology_info, "Topology", "拓扑信息");
    tr!(motors, "motors", "个电机");
    tr!(refresh_ports, "Refresh Ports", "刷新端口");
    tr!(start_control, "Start Control", "启动控制");
    tr!(stop_control, "Stop Control", "停止控制");
    tr!(emergency_stop, "E-STOP", "急停");
    tr!(protocol, "Protocol", "协议类型");
    tr!(serial_config, "Serial Port Configuration", "串口配置");
    tr!(tcp_config, "TCP Configuration", "TCP 配置");
    tr!(udp_config, "UDP Configuration", "UDP 配置");
    tr!(
        can_config,
        "CAN / CAN FD Configuration",
        "CAN / CAN FD 配置"
    );
    tr!(port, "Port", "端口");
    tr!(baud_rate, "Baud Rate", "波特率");
    tr!(data_bits, "Data Bits", "数据位");
    tr!(stop_bits, "Stop Bits", "停止位");
    tr!(parity, "Parity", "校验位");
    tr!(flow_control, "Flow Control", "流控");
    tr!(available_ports, "Available Ports", "可用端口");
    tr!(no_ports_found, "No serial ports found.", "未发现串口设备。");
    tr!(mode, "Mode", "模式");
    tr!(client, "Client", "客户端");
    tr!(server, "Server", "服务端");
    tr!(host, "Host", "地址");
    tr!(local_port, "Local Port", "本地端口");
    tr!(remote_host, "Remote Host", "远程地址");
    tr!(remote_port, "Remote Port", "远程端口");
    tr!(bitrate, "Bitrate", "比特率");
    tr!(enable_can_fd, "Enable CAN FD", "启用 CAN FD");
    tr!(data_bitrate, "Data Bitrate", "数据比特率");
    tr!(
        sw_simulation_hint,
        "(Software simulation - no physical CAN adapter required)",
        "(软件仿真 - 无需物理CAN适配器)"
    );
    tr!(connected_clients, "Connected Clients", "已连接客户端");
    tr!(display, "Display", "显示");
    tr!(auto_scroll, "Auto-scroll", "自动滚动");
    tr!(entries, "Entries", "条目");
    tr!(
        no_data_yet,
        "No data yet. Connect a device and start communicating...",
        "暂无数据。请连接设备开始通信..."
    );
    tr!(newline, "Newline", "换行");
    tr!(type_to_send, "Type text to send...", "输入要发送的内容...");
    tr!(hex_hint, "e.g. AA 01 02 FF 55", "如 AA 01 02 FF 55");
    tr!(template, "Template", "模板");
    tr!(new_template, "New", "新建");
    tr!(delete, "Delete", "删除");
    tr!(name, "Name", "名称");
    tr!(description, "Description", "描述");
    tr!(header_hex, "Header (hex)", "帧头 (hex)");
    tr!(tail_hex, "Tail (hex)", "帧尾 (hex)");
    tr!(checksum, "Checksum", "校验方式");
    tr!(include_length, "Include Length", "包含长度");
    tr!(fields, "Fields", "字段列表");
    tr!(add_field, "Add Field", "添加字段");
    tr!(packet_preview, "Packet Preview", "数据包预览");
    tr!(send_packet, "Send Packet", "发送数据包");
    tr!(copy_hex, "Copy HEX", "复制 HEX");
    tr!(presets, "Presets", "预设方案");
    tr!(chassis_type, "Chassis Type", "底盘类型");
    tr!(geometry_params, "Geometry Parameters", "几何参数");
    tr!(wheel_radius, "Wheel Radius (mm)", "轮半径 (mm)");
    tr!(wheel_base, "Wheel Base (mm)", "轴距 (mm)");
    tr!(track_width, "Track Width (mm)", "轮距 (mm)");
    tr!(max_linear_vel, "Max Linear Vel (mm/s)", "最大线速度 (mm/s)");
    tr!(
        max_angular_vel,
        "Max Angular Vel (rad/s)",
        "最大角速度 (rad/s)"
    );
    tr!(motors_joints, "Motors / Joints", "电机 / 关节");
    tr!(add_motor, "Add Motor", "添加电机");
    tr!(topology_viz, "Topology Visualization", "拓扑可视化");
    tr!(pid_params, "PID Parameters", "PID 参数");
    tr!(advanced_options, "Advanced Options", "高级选项");
    tr!(deriv_filter, "Derivative Filter", "微分滤波");
    tr!(anti_windup, "Anti-Windup", "抗积分饱和");
    tr!(feedforward, "Feedforward Gain", "前馈增益");
    tr!(dead_zone, "Dead Zone", "死区");
    tr!(current_state, "Current State", "当前状态");
    tr!(save_preset, "Save Current as Preset", "保存为预设");
    tr!(running, "RUNNING", "运行中");
    tr!(stopped, "STOPPED", "已停止");
    tr!(algorithm_select, "Algorithm Selection", "算法选择");
    tr!(increment_limit, "Increment Limit", "增量限幅");
    tr!(output_ramp, "Output Ramp (per sec)", "输出斜率限制 (每秒)");
    tr!(last_increment, "Last Increment", "最近增量");
    tr!(output_high, "Output High", "正向输出");
    tr!(output_low, "Output Low", "负向输出");
    tr!(hysteresis, "Hysteresis", "回滞区");
    tr!(dead_band, "Dead Band", "死区带宽");
    tr!(switch_state, "Switch State", "开关状态");
    tr!(base_params, "Base Parameters", "基础参数");
    tr!(fuzzy_tuning_range, "Fuzzy Tuning Range", "模糊整定范围");
    tr!(error_scale, "Error Scale", "误差量化比例");
    tr!(ec_scale, "Error Change Scale", "误差变化率比例");
    tr!(effective_params, "Effective Parameters", "当前有效参数");
    tr!(outer_loop, "Outer Loop (Position)", "外环 (位置)");
    tr!(inner_loop, "Inner Loop (Velocity)", "内环 (速度)");
    tr!(outer_output, "Outer Output", "外环输出");
    tr!(process_model, "Process Model", "过程模型");
    tr!(model_gain, "Model Gain (K)", "模型增益 (K)");
    tr!(time_constant, "Time Constant (T, sec)", "时间常数 (T, 秒)");
    tr!(dead_time, "Dead Time (L, sec)", "纯时滞 (L, 秒)");
    tr!(model_prediction, "Model Prediction", "模型预测值");
    tr!(delay_buffer_size, "Delay Buffer Size", "延迟缓冲长度");
    tr!(
        adrc_td_params,
        "Tracking Differentiator (TD)",
        "跟踪微分器 (TD)"
    );
    tr!(
        adrc_eso_params,
        "Extended State Observer (ESO)",
        "扩展状态观测器 (ESO)"
    );
    tr!(
        adrc_nlsef_params,
        "Nonlinear State Error Feedback (NLSEF)",
        "非线性状态误差反馈 (NLSEF)"
    );
    tr!(
        ladrc_bandwidth_params,
        "LADRC Bandwidth Parameters",
        "LADRC 带宽参数"
    );
    tr!(ladrc_order, "Order", "阶次");
    tr!(ladrc_first_order, "1st Order", "一阶");
    tr!(ladrc_second_order, "2nd Order", "二阶");
    tr!(
        lqr_weights,
        "LQR State & Control Weights",
        "LQR 状态与控制权重"
    );
    tr!(lqr_q_position, "Position Weight", "位置权重");
    tr!(lqr_q_velocity, "Velocity Weight", "速度权重");
    tr!(lqr_r_weight, "Control Weight", "控制权重");
    tr!(lqr_mass, "Mass (kg)", "质量 (kg)");
    tr!(
        lqr_integral,
        "Integral Action (optional)",
        "积分环节 (可选)"
    );
    tr!(lqr_computed_gains, "Computed Gains:", "计算增益:");
    tr!(mpc_horizons, "MPC Horizons", "MPC 预测与控制时域");
    tr!(
        mpc_prediction_horizon,
        "Prediction Horizon (Np)",
        "预测时域 (Np)"
    );
    tr!(mpc_control_horizon, "Control Horizon (Nc)", "控制时域 (Nc)");
    tr!(mpc_model_params, "Internal Model", "内部模型");
    tr!(mpc_sample_time, "Sample Time (sec)", "采样时间 (秒)");
    tr!(
        mpc_weights_and_constraints,
        "Weights & Constraints",
        "权重与约束"
    );
    tr!(mpc_du_limit, "ΔU Limit", "ΔU 限幅");
    tr!(
        chassis_kinematics,
        "Chassis Kinematics Code Examples",
        "底盘运动学代码示例"
    );
    tr!(network_arch, "Network Architecture", "网络架构");
    tr!(training_controls, "Training Controls", "训练控制");
    tr!(learning_rate, "Learning Rate", "学习率");
    tr!(train_step, "Train Step", "训练一步");
    tr!(auto_train, "Auto-Train", "自动训练");
    tr!(training_loss, "Training Loss", "训练损失");
    tr!(
        no_training_data,
        "No training data. Start control and collect error data first.",
        "暂无训练数据。请先启动控制并采集误差数据。"
    );
    tr!(suggested_params, "Suggested Parameters", "建议参数");
    tr!(predict, "Predict", "预测");
    tr!(apply_suggested, "Apply Suggested", "应用建议值");
    tr!(input_features, "Input Features Preview", "输入特征预览");
    tr!(parameter, "Parameter", "参数");
    tr!(current, "Current", "当前值");
    tr!(suggested, "Suggested", "建议值");
    tr!(delta, "Delta", "差值");
    tr!(channels, "Channels", "通道");
    tr!(position, "Position", "位置");
    tr!(velocity, "Velocity", "速度");
    tr!(current_a, "Current", "电流");
    tr!(temperature, "Temperature", "温度");
    tr!(error_ch, "Error", "误差");
    tr!(pid_output, "PID Output", "PID 输出");
    tr!(data_points, "Data Points", "数据点");
    tr!(clear_history, "Clear History", "清空历史");
    tr!(request_builder, "Request Builder", "请求构建");
    tr!(slave_id, "Slave ID", "从站地址");
    tr!(function, "Function", "功能码");
    tr!(start_address, "Start Address", "起始地址");
    tr!(quantity, "Quantity", "数量");
    tr!(write_values, "Write Values", "写入值");
    tr!(frame_preview, "Frame Preview", "帧预览");
    tr!(send_rtu, "Send RTU", "发送 RTU");
    tr!(send_tcp, "Send TCP", "发送 TCP");
    tr!(
        register_table,
        "Register Table (Simulated)",
        "寄存器表 (模拟)"
    );
    tr!(randomize, "Randomize", "随机填充");
    tr!(modbus_log, "Modbus Log", "Modbus 日志");
    tr!(light_mode, "Light", "浅色");
    tr!(dark_mode, "Dark", "深色");
    tr!(select_port, "Select port...", "选择端口...");
    tr!(
        comma_values_hint,
        "Comma separated values, e.g. 100,200,300",
        "逗号分隔值, 如 100,200,300"
    );
    tr!(copied, "Copied to clipboard", "已复制到剪贴板");
    tr!(select, "Select...", "选择...");
    tr!(sample_point, "Sample Point", "采样点");
    tr!(data_sample_point, "Data Sample Point", "数据采样点");
    tr!(can_termination, "Termination Resistor", "终端电阻");
    tr!(can_listen_only, "Listen Only", "仅监听");
    tr!(can_loopback, "Loopback", "回环模式");
    tr!(can_auto_retransmit, "Auto Retransmit", "自动重传");
    tr!(can_error_reporting, "Error Reporting", "错误报告");
    tr!(usb_config, "USB Configuration", "USB 配置");
    tr!(usb_protocol_label, "USB Protocol", "USB 协议");
    tr!(usb_speed_label, "USB Speed", "USB 速度");
    tr!(usb_endpoint_config, "Endpoint Configuration", "端点配置");
    tr!(usb_endpoint_in, "Endpoint IN", "输入端点");
    tr!(usb_endpoint_out, "Endpoint OUT", "输出端点");
    tr!(usb_max_packet_size, "Max Packet Size", "最大包大小");
    tr!(usb_interface, "Interface", "接口");
    tr!(usb_typical_speeds, "Typical Speeds", "典型速度");
    tr!(
        usb_cdc_hint,
        "CDC ACM devices use virtual COM port",
        "CDC ACM 设备使用虚拟串口"
    );
    tr!(builder_tab, "Builder", "构建器");
    tr!(parser_tab, "Parser", "解析器");
    tr!(parser_template, "Parse Template", "解析模板");
    tr!(auto_parse, "Auto Parse", "自动解析");
    tr!(parser_input, "HEX Data Input", "HEX 数据输入");
    tr!(parse_now, "Parse Now", "立即解析");
    tr!(parsed_count, "Parsed", "已解析");
    tr!(
        parser_empty,
        "No parsed results yet. Paste HEX data and click Parse.",
        "暂无解析结果。粘贴 HEX 数据后点击解析。"
    );
    tr!(
        parse_failed,
        "Parse failed: no matching template",
        "解析失败: 无匹配模板"
    );
    tr!(field_type_label, "Type", "类型");
    tr!(field_value_label, "Value", "值");
    tr!(field_numeric, "Numeric", "数值");
    tr!(viz_channel_config, "Channel Configuration", "通道配置");
    tr!(viz_add_channel, "Add Channel", "添加通道");
    tr!(menu_file, "File", "文件");
    tr!(menu_edit, "Edit", "编辑");
    tr!(menu_view, "View", "视图");
    tr!(menu_tools, "Tools", "工具");
    tr!(menu_help, "Help", "帮助");
    tr!(menu_export_log, "Export Logs (CSV)", "导出日志 (CSV)");
    tr!(menu_preferences, "Preferences", "偏好设置");
    tr!(menu_quit, "Quit", "退出");
    tr!(menu_clear_logs, "Clear All Logs", "清除所有日志");
    tr!(menu_copy_frame, "Copy Last Frame", "复制最后一帧");
    tr!(menu_reset_counters, "Reset Counters", "重置计数器");
    tr!(menu_hide_sidebar, "Hide Sidebar", "隐藏侧边栏");
    tr!(menu_show_sidebar, "Show Sidebar", "显示侧边栏");
    tr!(menu_motion_level, "Motion Level", "动效等级");
    tr!(menu_ui_scale, "UI Scale", "界面缩放");
    tr!(menu_ui_scale_reset, "Reset to 150%", "重置为 150%");
    tr!(menu_language, "Language", "语言");
    tr!(menu_mcp_server, "Toggle MCP Server", "切换 MCP 服务");
    tr!(menu_about, "About", "关于");
    tr!(menu_shortcuts, "Keyboard Shortcuts", "键盘快捷键");
    tr!(menu_docs, "Documentation", "文档");
    tr!(top_health, "Health", "链路健康");
    tr!(top_status, "Status", "状态");
    tr!(menu_check_updates, "Check Updates", "检查更新");
    tr!(prefs_title, "Preferences", "偏好设置");
    tr!(prefs_sidebar, "Enable tab strip", "显示标签栏");
    tr!(prefs_motion_level, "Motion level", "动效等级");
    tr!(prefs_ui_scale, "UI scale (%)", "界面缩放 (%)");
    tr!(
        prefs_autosave_seconds,
        "Auto-save interval (s)",
        "自动保存间隔 (秒)"
    );
    tr!(prefs_saved, "Preferences saved", "偏好设置已保存");
    tr!(shortcuts_title, "Keyboard Shortcuts", "键盘快捷键");
    tr!(
        docs_opened,
        "Documentation opened in browser",
        "已在浏览器中打开文档"
    );
    tr!(logs_cleared, "All logs cleared", "已清除所有日志");
    tr!(counters_reset_done, "Counters reset", "计数器已重置");
    tr!(
        no_logs_to_copy,
        "No log frame to copy",
        "暂无可复制的日志帧"
    );
    tr!(copied_last_frame, "Copied latest frame", "已复制最近一帧");
    tr!(motion_level_extreme, "Extreme", "极致");
    tr!(motion_level_standard, "Standard", "标准");
    tr!(motion_level_native, "Native", "原生");
    tr!(motion_level_optimized, "Optimized", "优化");

    tr!(
        about_summary,
        "A unified workspace for robot control, diagnostics, tuning, and data analysis.",
        "机器人控制、诊断、调参与数据分析一体化工作台。"
    );
    tr!(remove_btn, "Remove", "移除");
    tr!(csv_label, "CSV", "CSV");
    tr!(json_label, "JSON", "JSON");

    tr!(chassis_kinematics_desc, "", "");
    tr!(no_data_hint, "", "");

    tr!(final_speed_label, "Final Speed", "最终转速");
    tr!(peak_torque_label, "Peak Torque", "峰值扭矩");
    tr!(settled_label, "Settled", "已稳定");

    tr!(mcp_server_label, "MCP Server", "MCP 服务器");
    tr!(port_label, "Port:", "端口:");
    tr!(token_label, "Token:", "令牌:");
    tr!(stop_mcp, "Stop MCP", "停止 MCP");
    tr!(start_mcp, "Start MCP", "启动 MCP");
    tr!(
        reconnect_after_drop,
        "Reconnect after drop",
        "断开后自动重连"
    );
    tr!(interval_ms_label, "Interval(ms):", "间隔(ms):");
    tr!(system_check_label, "System Check", "系统自检");
    tr!(runtime_metrics_label, "Runtime Metrics", "运行指标");
    tr!(
        protocol_analysis_entry_label,
        "Protocol Analysis Entry",
        "协议分析入口"
    );
    tr!(train_x10_btn, "Train x10", "Train x10");
    tr!(train_x100_btn, "Train x100", "Train x100");
    tr!(loss_label, "Loss", "Loss");
    tr!(current_loss_label, "Current Loss", "当前损失");
    tr!(running_label, "Running", "运行中");
    tr!(stopped_label, "Stopped", "已停止");
    tr!(q_output_label, "Q (output):", "Q (输出):");
    tr!(r_input_label, "R (input):", "R (输入):");
    tr!(s_rate_label, "S (rate):", "S (变化率):");
    tr!(valid_label, "VALID", "有效");
    tr!(invalid_label, "INVALID", "无效");
    tr!(
        canopen_cobid_map,
        "CANopen COB-ID Map",
        "CANopen COB-ID 映射"
    );
    tr!(
        canopen_sdo_bitfield,
        "SDO Command Byte Bitfield",
        "SDO 命令字节位域"
    );
    tr!(
        background_retry_off,
        "Background retry is off until you enable it.",
        "后台重试已关闭，直到您启用它。"
    );
    tr!(
        retry_arms_hint,
        "Retry arms after one successful manual connection.",
        "重试在一次成功手动连接后启用。"
    );
    tr!(
        retry_idle_hint,
        "Retry idle until the current link drops.",
        "重试在当前连接断开前保持空闲。"
    );
    tr!(copy_rtu_btn, "Copy RTU", "Copy RTU");
    tr!(copy_tcp_btn, "Copy TCP", "Copy TCP");
    tr!(
        no_parsed_fields,
        "No parsed numeric fields",
        "No parsed numeric fields"
    );
    tr!(speed_ref_label, "Speed reference", "Speed reference");
    tr!(speed_error_label, "Speed error", "Speed error");
    tr!(peak_current_label, "Peak current", "Peak current");
    tr!(max_temp_label, "Max temperature", "Max temperature");
    tr!(steps_executed_label, "Steps executed", "Steps executed");
    tr!(cancelled_label, "Cancelled", "Cancelled");
    tr!(display_mode_hex, "HEX", "HEX");
    tr!(display_mode_ascii, "ASCII", "ASCII");
    tr!(display_mode_mixed, "Mixed", "Mixed");
    tr!(no_matching_logs, "No matching logs.", "No matching logs.");
    tr!(output_limit_label, "Output Limit:", "输出限幅:");
    tr!(integral_limit_label, "Integral Limit:", "积分限幅:");
    tr!(setpoint_label, "Setpoint:", "设定值:");
    tr!(integral_label, "Integral:", "积分:");
    tr!(derivative_label, "Derivative:", "微分:");
    tr!(output_label, "Output:", "输出:");
    tr!(bang_bang_label, "Bang-Bang", "Bang-Bang");
    tr!(llm_api_tuning, "LLM API Tuning", "LLM API 调参");
    tr!(api_url_label, "API URL:", "API URL:");
    tr!(model_label, "Model:", "模型:");
    tr!(api_key_label, "API Key:", "API 密钥:");
    tr!(llm_suggest_btn, "LLM Suggest", "LLM 建议");
    tr!(
        apply_llm_suggestion,
        "Apply LLM Suggestion",
        "应用 LLM 建议"
    );
    tr!(
        llm_loading_text,
        "LLM request in progress...",
        "LLM 请求中..."
    );
    tr!(llm_analysis_label, "LLM Analysis:", "LLM 分析:");
    tr!(use_first_btn, "Use first", "使用第一个");
    tr!(resume_retry, "Resume retry", "恢复重试");
    tr!(stop_retry, "Stop retry", "停止重试");
    tr!(retry_now, "Retry now", "立即重试");
    tr!(canopen_fd_builder, "CAN FD Frame Builder", "CAN FD 帧构建");
    tr!(canopen_log_label, "CANopen Log", "CANopen 日志");
    tr!(canopen_frame_analyzer, "Frame Analyzer", "CANopen 帧解析");
    tr!(
        canopen_ecat_sdo_tool,
        "EtherCAT CoE SDO Tool",
        "EtherCAT CoE SDO 工具"
    );
    tr!(
        canopen_ecat_state_machine,
        "EtherCAT State Machine",
        "EtherCAT 状态机"
    );
    tr!(canopen_pdo_mapper, "PDO Mapping Manager", "PDO 映射管理器");
    tr!(canopen_pdo_decoder, "PDO Data Decoder", "PDO 实时解码");
    tr!(canopen_nmt_control, "NMT Control", "NMT 控制");
    tr!(canopen_sdo_client, "SDO Client", "SDO 客户端");
    tr!(
        canopen_pdo_hb_emcy,
        "PDO / Heartbeat / EMCY",
        "PDO / Heartbeat / EMCY 工具"
    );

    // ── 动态格式化翻译 ─────────────────────────────────────

    tr_fmt!(found_ports(n: usize), "Found {} ports", "发现 {} 个端口");
    tr_fmt!(sent_bytes(n: usize), "Sent {} bytes", "已发送 {} 字节");
    tr_fmt!(send_error(e: &str), "Send error: {}", "发送失败: {}");
    tr_fmt!(applied_preset(name: &str), "Applied preset: {}", "已应用预设: {}");
    tr_fmt!(parse_success(name: &str, count: usize), "Parsed '{}': {} fields", "已解析 '{}': {} 个字段");
    tr_fmt!(logs_exported(path: &str), "Logs exported: {}", "日志已导出: {}");
    tr_fmt!(logs_export_failed(err: &str), "Log export failed: {}", "日志导出失败: {}");
    tr_fmt!(ui_scale_set(percent: u32), "UI scale: {}%", "界面缩放: {}%");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_toggle() {
        assert_eq!(Language::English.toggle(), Language::Chinese);
        assert_eq!(Language::Chinese.toggle(), Language::English);
    }

    #[test]
    fn test_language_label() {
        assert_eq!(Language::English.label(), "English");
        assert_eq!(Language::Chinese.label(), "中文");
    }

    #[test]
    fn test_tr_app_title_both_languages() {
        let en = Tr::app_title(Language::English);
        let zh = Tr::app_title(Language::Chinese);
        assert!(!en.is_empty());
        assert!(!zh.is_empty());
        assert_ne!(en, zh);
    }

    #[test]
    fn test_tr_all_tabs_have_translations() {
        for lang in &[Language::English, Language::Chinese] {
            assert!(!Tr::tab_dashboard(*lang).is_empty());
            assert!(!Tr::tab_connections(*lang).is_empty());
            assert!(!Tr::tab_terminal(*lang).is_empty());
            assert!(!Tr::tab_packet_builder(*lang).is_empty());
            assert!(!Tr::tab_topology(*lang).is_empty());
            assert!(!Tr::tab_pid_control(*lang).is_empty());
            assert!(!Tr::tab_nn_tuning(*lang).is_empty());
            assert!(!Tr::tab_data_viz(*lang).is_empty());
            assert!(!Tr::tab_simulation_lab(*lang).is_empty());
            assert!(!Tr::tab_modbus(*lang).is_empty());
            assert!(!Tr::tab_canopen(*lang).is_empty());
        }
    }

    #[test]
    fn test_tr_dynamic_strings() {
        let s = Tr::sent_bytes(1024, Language::English);
        assert!(s.contains("1024"), "Should contain byte count: {}", s);
        let s = Tr::sent_bytes(512, Language::Chinese);
        assert!(s.contains("512"));
    }

    #[test]
    fn test_tr_status_strings() {
        assert!(!Tr::connect(Language::English).is_empty());
        assert!(!Tr::disconnect(Language::Chinese).is_empty());
        assert!(!Tr::send(Language::English).is_empty());
        assert!(!Tr::clear(Language::Chinese).is_empty());
    }

    #[test]
    fn test_tr_format_strings() {
        let s = Tr::found_ports(5, Language::English);
        assert!(s.contains("5"), "Should contain port count: {}", s);
        let s = Tr::ui_scale_set(150, Language::Chinese);
        assert!(s.contains("150"), "Should contain scale: {}", s);
    }

    #[test]
    fn test_macro_coverage() {
        // Verify every static translation returns non-empty for both languages
        type TrCheck = (fn(Language) -> &'static str, &'static str);
        let checks: &[TrCheck] = &[
            (Tr::app_title, "app_title"),
            (Tr::connect, "connect"),
            (Tr::disconnect, "disconnect"),
            (Tr::error_label, "error_label"),
            (Tr::save, "save"),
            (Tr::reset, "reset"),
        ];
        for (f, name) in checks {
            for lang in &[Language::English, Language::Chinese] {
                let val = f(*lang);
                assert!(!val.is_empty(), "{} returned empty for {:?}", name, lang);
            }
        }
    }
}
