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
pub struct Tr;

impl Tr {
    // ─── 通用 ─────────────────────────────────────────────

    pub fn app_title(lang: Language) -> &'static str {
        match lang {
            Language::English => "Robot Control Suite",
            Language::Chinese => "机器人控制调试套件",
        }
    }

    pub fn connect(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connect",
            Language::Chinese => "连接",
        }
    }

    pub fn disconnect(lang: Language) -> &'static str {
        match lang {
            Language::English => "Disconnect",
            Language::Chinese => "断开",
        }
    }

    pub fn send(lang: Language) -> &'static str {
        match lang {
            Language::English => "Send",
            Language::Chinese => "发送",
        }
    }

    pub fn clear(lang: Language) -> &'static str {
        match lang {
            Language::English => "Clear",
            Language::Chinese => "清空",
        }
    }

    pub fn refresh(lang: Language) -> &'static str {
        match lang {
            Language::English => "Refresh",
            Language::Chinese => "刷新",
        }
    }

    pub fn save(lang: Language) -> &'static str {
        match lang {
            Language::English => "Save",
            Language::Chinese => "保存",
        }
    }

    pub fn apply(lang: Language) -> &'static str {
        match lang {
            Language::English => "Apply",
            Language::Chinese => "应用",
        }
    }

    pub fn reset(lang: Language) -> &'static str {
        match lang {
            Language::English => "Reset",
            Language::Chinese => "重置",
        }
    }

    pub fn start(lang: Language) -> &'static str {
        match lang {
            Language::English => "Start",
            Language::Chinese => "启动",
        }
    }

    pub fn stop(lang: Language) -> &'static str {
        match lang {
            Language::English => "Stop",
            Language::Chinese => "停止",
        }
    }

    pub fn ready(lang: Language) -> &'static str {
        match lang {
            Language::English => "Ready",
            Language::Chinese => "就绪",
        }
    }

    pub fn error_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "Error",
            Language::Chinese => "错误",
        }
    }

    pub fn connected(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connected",
            Language::Chinese => "已连接",
        }
    }

    pub fn disconnected(lang: Language) -> &'static str {
        match lang {
            Language::English => "Disconnected",
            Language::Chinese => "已断开",
        }
    }

    // ─── 导航标签 ──────────────────────────────────────────

    pub fn tab_dashboard(lang: Language) -> &'static str {
        match lang {
            Language::English => "Dashboard",
            Language::Chinese => "仪表盘",
        }
    }

    pub fn tab_connections(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connections",
            Language::Chinese => "连接管理",
        }
    }

    pub fn tab_terminal(lang: Language) -> &'static str {
        match lang {
            Language::English => "Terminal",
            Language::Chinese => "终端调试",
        }
    }

    pub fn tab_protocol_analysis(lang: Language) -> &'static str {
        match lang {
            Language::English => "Protocol Analysis",
            Language::Chinese => "协议分析",
        }
    }

    pub fn tab_packet_builder(lang: Language) -> &'static str {
        match lang {
            Language::English => "Packet Builder",
            Language::Chinese => "协议组包",
        }
    }

    pub fn tab_topology(lang: Language) -> &'static str {
        match lang {
            Language::English => "Topology",
            Language::Chinese => "机器人拓扑",
        }
    }

    pub fn tab_pid_control(lang: Language) -> &'static str {
        match lang {
            Language::English => "Control Algorithms",
            Language::Chinese => "控制算法",
        }
    }

    pub fn tab_nn_tuning(lang: Language) -> &'static str {
        match lang {
            Language::English => "NN Auto-Tune",
            Language::Chinese => "神经网络调参",
        }
    }

    pub fn tab_data_viz(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Viz",
            Language::Chinese => "数据可视化",
        }
    }

    pub fn tab_modbus(lang: Language) -> &'static str {
        match lang {
            Language::English => "Modbus Tools",
            Language::Chinese => "Modbus 工具",
        }
    }

    pub fn tab_canopen(lang: Language) -> &'static str {
        match lang {
            Language::English => "CANopen Tools",
            Language::Chinese => "CANopen 工具",
        }
    }

    // ─── Dashboard ────────────────────────────────────────

    pub fn connection_status(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connection Status",
            Language::Chinese => "连接状态",
        }
    }

    pub fn system_stats(lang: Language) -> &'static str {
        match lang {
            Language::English => "System Statistics",
            Language::Chinese => "系统统计",
        }
    }

    pub fn quick_actions(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quick Actions",
            Language::Chinese => "快捷操作",
        }
    }

    pub fn robot_state(lang: Language) -> &'static str {
        match lang {
            Language::English => "Robot State",
            Language::Chinese => "机器人状态",
        }
    }

    pub fn bytes_sent(lang: Language) -> &'static str {
        match lang {
            Language::English => "Bytes Sent",
            Language::Chinese => "已发送字节",
        }
    }

    pub fn bytes_received(lang: Language) -> &'static str {
        match lang {
            Language::English => "Bytes Received",
            Language::Chinese => "已接收字节",
        }
    }

    pub fn total_errors(lang: Language) -> &'static str {
        match lang {
            Language::English => "Total Errors",
            Language::Chinese => "总错误数",
        }
    }

    pub fn log_entries(lang: Language) -> &'static str {
        match lang {
            Language::English => "Log Entries",
            Language::Chinese => "日志条目",
        }
    }

    pub fn state_history(lang: Language) -> &'static str {
        match lang {
            Language::English => "State History",
            Language::Chinese => "状态历史",
        }
    }

    pub fn active_channel(lang: Language) -> &'static str {
        match lang {
            Language::English => "Active Channel",
            Language::Chinese => "当前通道",
        }
    }

    pub fn last_comm(lang: Language) -> &'static str {
        match lang {
            Language::English => "Last Comm",
            Language::Chinese => "最近通信",
        }
    }

    pub fn topology_info(lang: Language) -> &'static str {
        match lang {
            Language::English => "Topology",
            Language::Chinese => "拓扑信息",
        }
    }

    pub fn motors(lang: Language) -> &'static str {
        match lang {
            Language::English => "motors",
            Language::Chinese => "个电机",
        }
    }

    pub fn refresh_ports(lang: Language) -> &'static str {
        match lang {
            Language::English => "Refresh Ports",
            Language::Chinese => "刷新端口",
        }
    }

    pub fn start_control(lang: Language) -> &'static str {
        match lang {
            Language::English => "Start Control",
            Language::Chinese => "启动控制",
        }
    }

    pub fn stop_control(lang: Language) -> &'static str {
        match lang {
            Language::English => "Stop Control",
            Language::Chinese => "停止控制",
        }
    }

    pub fn emergency_stop(lang: Language) -> &'static str {
        match lang {
            Language::English => "E-STOP",
            Language::Chinese => "急停",
        }
    }

    // ─── Connections ──────────────────────────────────────

    pub fn protocol(lang: Language) -> &'static str {
        match lang {
            Language::English => "Protocol",
            Language::Chinese => "协议类型",
        }
    }

    pub fn serial_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "Serial Port Configuration",
            Language::Chinese => "串口配置",
        }
    }

    pub fn tcp_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "TCP Configuration",
            Language::Chinese => "TCP 配置",
        }
    }

    pub fn udp_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "UDP Configuration",
            Language::Chinese => "UDP 配置",
        }
    }

    pub fn can_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "CAN / CAN FD Configuration",
            Language::Chinese => "CAN / CAN FD 配置",
        }
    }

    pub fn port(lang: Language) -> &'static str {
        match lang {
            Language::English => "Port",
            Language::Chinese => "端口",
        }
    }

    pub fn baud_rate(lang: Language) -> &'static str {
        match lang {
            Language::English => "Baud Rate",
            Language::Chinese => "波特率",
        }
    }

    pub fn data_bits(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Bits",
            Language::Chinese => "数据位",
        }
    }

    pub fn stop_bits(lang: Language) -> &'static str {
        match lang {
            Language::English => "Stop Bits",
            Language::Chinese => "停止位",
        }
    }

    pub fn parity(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parity",
            Language::Chinese => "校验位",
        }
    }

    pub fn flow_control(lang: Language) -> &'static str {
        match lang {
            Language::English => "Flow Control",
            Language::Chinese => "流控",
        }
    }

    pub fn available_ports(lang: Language) -> &'static str {
        match lang {
            Language::English => "Available Ports",
            Language::Chinese => "可用端口",
        }
    }

    pub fn no_ports_found(lang: Language) -> &'static str {
        match lang {
            Language::English => "No serial ports found.",
            Language::Chinese => "未发现串口设备。",
        }
    }

    pub fn mode(lang: Language) -> &'static str {
        match lang {
            Language::English => "Mode",
            Language::Chinese => "模式",
        }
    }

    pub fn client(lang: Language) -> &'static str {
        match lang {
            Language::English => "Client",
            Language::Chinese => "客户端",
        }
    }

    pub fn server(lang: Language) -> &'static str {
        match lang {
            Language::English => "Server",
            Language::Chinese => "服务端",
        }
    }

    pub fn host(lang: Language) -> &'static str {
        match lang {
            Language::English => "Host",
            Language::Chinese => "地址",
        }
    }

    pub fn local_port(lang: Language) -> &'static str {
        match lang {
            Language::English => "Local Port",
            Language::Chinese => "本地端口",
        }
    }

    pub fn remote_host(lang: Language) -> &'static str {
        match lang {
            Language::English => "Remote Host",
            Language::Chinese => "远程地址",
        }
    }

    pub fn remote_port(lang: Language) -> &'static str {
        match lang {
            Language::English => "Remote Port",
            Language::Chinese => "远程端口",
        }
    }

    pub fn bitrate(lang: Language) -> &'static str {
        match lang {
            Language::English => "Bitrate",
            Language::Chinese => "比特率",
        }
    }

    pub fn enable_can_fd(lang: Language) -> &'static str {
        match lang {
            Language::English => "Enable CAN FD",
            Language::Chinese => "启用 CAN FD",
        }
    }

    pub fn data_bitrate(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Bitrate",
            Language::Chinese => "数据比特率",
        }
    }

    pub fn sw_simulation_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => "(Software simulation - no physical CAN adapter required)",
            Language::Chinese => "(软件仿真 - 无需物理CAN适配器)",
        }
    }

    pub fn connected_clients(lang: Language) -> &'static str {
        match lang {
            Language::English => "Connected Clients",
            Language::Chinese => "已连接客户端",
        }
    }

    // ─── Terminal ──────────────────────────────────────────

    pub fn display(lang: Language) -> &'static str {
        match lang {
            Language::English => "Display",
            Language::Chinese => "显示",
        }
    }

    pub fn auto_scroll(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto-scroll",
            Language::Chinese => "自动滚动",
        }
    }

    pub fn entries(lang: Language) -> &'static str {
        match lang {
            Language::English => "Entries",
            Language::Chinese => "条目",
        }
    }

    pub fn no_data_yet(lang: Language) -> &'static str {
        match lang {
            Language::English => "No data yet. Connect a device and start communicating...",
            Language::Chinese => "暂无数据。请连接设备开始通信...",
        }
    }

    pub fn newline(lang: Language) -> &'static str {
        match lang {
            Language::English => "Newline",
            Language::Chinese => "换行",
        }
    }

    pub fn type_to_send(lang: Language) -> &'static str {
        match lang {
            Language::English => "Type text to send...",
            Language::Chinese => "输入要发送的内容...",
        }
    }

    pub fn hex_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => "e.g. AA 01 02 FF 55",
            Language::Chinese => "如 AA 01 02 FF 55",
        }
    }

    // ─── Packet Builder ───────────────────────────────────

    pub fn template(lang: Language) -> &'static str {
        match lang {
            Language::English => "Template",
            Language::Chinese => "模板",
        }
    }

    pub fn new_template(lang: Language) -> &'static str {
        match lang {
            Language::English => "New",
            Language::Chinese => "新建",
        }
    }

    pub fn delete(lang: Language) -> &'static str {
        match lang {
            Language::English => "Delete",
            Language::Chinese => "删除",
        }
    }

    pub fn name(lang: Language) -> &'static str {
        match lang {
            Language::English => "Name",
            Language::Chinese => "名称",
        }
    }

    pub fn description(lang: Language) -> &'static str {
        match lang {
            Language::English => "Description",
            Language::Chinese => "描述",
        }
    }

    pub fn header_hex(lang: Language) -> &'static str {
        match lang {
            Language::English => "Header (hex)",
            Language::Chinese => "帧头 (hex)",
        }
    }

    pub fn tail_hex(lang: Language) -> &'static str {
        match lang {
            Language::English => "Tail (hex)",
            Language::Chinese => "帧尾 (hex)",
        }
    }

    pub fn checksum(lang: Language) -> &'static str {
        match lang {
            Language::English => "Checksum",
            Language::Chinese => "校验方式",
        }
    }

    pub fn include_length(lang: Language) -> &'static str {
        match lang {
            Language::English => "Include Length",
            Language::Chinese => "包含长度",
        }
    }

    pub fn fields(lang: Language) -> &'static str {
        match lang {
            Language::English => "Fields",
            Language::Chinese => "字段列表",
        }
    }

    pub fn add_field(lang: Language) -> &'static str {
        match lang {
            Language::English => "Add Field",
            Language::Chinese => "添加字段",
        }
    }

    pub fn packet_preview(lang: Language) -> &'static str {
        match lang {
            Language::English => "Packet Preview",
            Language::Chinese => "数据包预览",
        }
    }

    pub fn send_packet(lang: Language) -> &'static str {
        match lang {
            Language::English => "Send Packet",
            Language::Chinese => "发送数据包",
        }
    }

    pub fn copy_hex(lang: Language) -> &'static str {
        match lang {
            Language::English => "Copy HEX",
            Language::Chinese => "复制 HEX",
        }
    }

    // ─── Topology ─────────────────────────────────────────

    pub fn presets(lang: Language) -> &'static str {
        match lang {
            Language::English => "Presets",
            Language::Chinese => "预设方案",
        }
    }

    pub fn chassis_type(lang: Language) -> &'static str {
        match lang {
            Language::English => "Chassis Type",
            Language::Chinese => "底盘类型",
        }
    }

    pub fn geometry_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Geometry Parameters",
            Language::Chinese => "几何参数",
        }
    }

    pub fn wheel_radius(lang: Language) -> &'static str {
        match lang {
            Language::English => "Wheel Radius (mm)",
            Language::Chinese => "轮半径 (mm)",
        }
    }

    pub fn wheel_base(lang: Language) -> &'static str {
        match lang {
            Language::English => "Wheel Base (mm)",
            Language::Chinese => "轴距 (mm)",
        }
    }

    pub fn track_width(lang: Language) -> &'static str {
        match lang {
            Language::English => "Track Width (mm)",
            Language::Chinese => "轮距 (mm)",
        }
    }

    pub fn max_linear_vel(lang: Language) -> &'static str {
        match lang {
            Language::English => "Max Linear Vel (mm/s)",
            Language::Chinese => "最大线速度 (mm/s)",
        }
    }

    pub fn max_angular_vel(lang: Language) -> &'static str {
        match lang {
            Language::English => "Max Angular Vel (rad/s)",
            Language::Chinese => "最大角速度 (rad/s)",
        }
    }

    pub fn motors_joints(lang: Language) -> &'static str {
        match lang {
            Language::English => "Motors / Joints",
            Language::Chinese => "电机 / 关节",
        }
    }

    pub fn add_motor(lang: Language) -> &'static str {
        match lang {
            Language::English => "Add Motor",
            Language::Chinese => "添加电机",
        }
    }

    pub fn topology_viz(lang: Language) -> &'static str {
        match lang {
            Language::English => "Topology Visualization",
            Language::Chinese => "拓扑可视化",
        }
    }

    // ─── PID Control ──────────────────────────────────────

    pub fn pid_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "PID Parameters",
            Language::Chinese => "PID 参数",
        }
    }

    pub fn advanced_options(lang: Language) -> &'static str {
        match lang {
            Language::English => "Advanced Options",
            Language::Chinese => "高级选项",
        }
    }

    pub fn deriv_filter(lang: Language) -> &'static str {
        match lang {
            Language::English => "Derivative Filter",
            Language::Chinese => "微分滤波",
        }
    }

    pub fn anti_windup(lang: Language) -> &'static str {
        match lang {
            Language::English => "Anti-Windup",
            Language::Chinese => "抗积分饱和",
        }
    }

    pub fn feedforward(lang: Language) -> &'static str {
        match lang {
            Language::English => "Feedforward Gain",
            Language::Chinese => "前馈增益",
        }
    }

    pub fn dead_zone(lang: Language) -> &'static str {
        match lang {
            Language::English => "Dead Zone",
            Language::Chinese => "死区",
        }
    }

    pub fn current_state(lang: Language) -> &'static str {
        match lang {
            Language::English => "Current State",
            Language::Chinese => "当前状态",
        }
    }

    pub fn save_preset(lang: Language) -> &'static str {
        match lang {
            Language::English => "Save Current as Preset",
            Language::Chinese => "保存为预设",
        }
    }

    pub fn running(lang: Language) -> &'static str {
        match lang {
            Language::English => "RUNNING",
            Language::Chinese => "运行中",
        }
    }

    pub fn stopped(lang: Language) -> &'static str {
        match lang {
            Language::English => "STOPPED",
            Language::Chinese => "已停止",
        }
    }

    pub fn control_active(lang: Language) -> &'static str {
        match lang {
            Language::English => "Control Active",
            Language::Chinese => "控制运行中",
        }
    }

    // ─── 控制算法选择 ──────────────────────────────────────

    pub fn control_algorithm(lang: Language) -> &'static str {
        match lang {
            Language::English => "Control Algorithm",
            Language::Chinese => "控制算法",
        }
    }

    pub fn algorithm_select(lang: Language) -> &'static str {
        match lang {
            Language::English => "Algorithm Selection",
            Language::Chinese => "算法选择",
        }
    }

    pub fn algorithm_description(lang: Language) -> &'static str {
        match lang {
            Language::English => "Algorithm Description",
            Language::Chinese => "算法描述",
        }
    }

    // ── 增量式 PID ──

    pub fn increment_limit(lang: Language) -> &'static str {
        match lang {
            Language::English => "Increment Limit",
            Language::Chinese => "增量限幅",
        }
    }

    pub fn output_ramp(lang: Language) -> &'static str {
        match lang {
            Language::English => "Output Ramp (per sec)",
            Language::Chinese => "输出斜率限制 (每秒)",
        }
    }

    pub fn last_increment(lang: Language) -> &'static str {
        match lang {
            Language::English => "Last Increment",
            Language::Chinese => "最近增量",
        }
    }

    // ── Bang-Bang ──

    pub fn output_high(lang: Language) -> &'static str {
        match lang {
            Language::English => "Output High",
            Language::Chinese => "正向输出",
        }
    }

    pub fn output_low(lang: Language) -> &'static str {
        match lang {
            Language::English => "Output Low",
            Language::Chinese => "负向输出",
        }
    }

    pub fn hysteresis(lang: Language) -> &'static str {
        match lang {
            Language::English => "Hysteresis",
            Language::Chinese => "回滞区",
        }
    }

    pub fn dead_band(lang: Language) -> &'static str {
        match lang {
            Language::English => "Dead Band",
            Language::Chinese => "死区带宽",
        }
    }

    pub fn switch_state(lang: Language) -> &'static str {
        match lang {
            Language::English => "Switch State",
            Language::Chinese => "开关状态",
        }
    }

    // ── 模糊 PID ──

    pub fn base_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Base Parameters",
            Language::Chinese => "基础参数",
        }
    }

    pub fn fuzzy_tuning_range(lang: Language) -> &'static str {
        match lang {
            Language::English => "Fuzzy Tuning Range",
            Language::Chinese => "模糊整定范围",
        }
    }

    pub fn error_scale(lang: Language) -> &'static str {
        match lang {
            Language::English => "Error Scale",
            Language::Chinese => "误差量化比例",
        }
    }

    pub fn ec_scale(lang: Language) -> &'static str {
        match lang {
            Language::English => "Error Change Scale",
            Language::Chinese => "误差变化率比例",
        }
    }

    pub fn effective_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Effective Parameters",
            Language::Chinese => "当前有效参数",
        }
    }

    // ── 串级 PID ──

    pub fn outer_loop(lang: Language) -> &'static str {
        match lang {
            Language::English => "Outer Loop (Position)",
            Language::Chinese => "外环 (位置)",
        }
    }

    pub fn inner_loop(lang: Language) -> &'static str {
        match lang {
            Language::English => "Inner Loop (Velocity)",
            Language::Chinese => "内环 (速度)",
        }
    }

    pub fn outer_output(lang: Language) -> &'static str {
        match lang {
            Language::English => "Outer Output",
            Language::Chinese => "外环输出",
        }
    }

    // ── Smith 预估 ──

    pub fn process_model(lang: Language) -> &'static str {
        match lang {
            Language::English => "Process Model",
            Language::Chinese => "过程模型",
        }
    }

    pub fn model_gain(lang: Language) -> &'static str {
        match lang {
            Language::English => "Model Gain (K)",
            Language::Chinese => "模型增益 (K)",
        }
    }

    pub fn time_constant(lang: Language) -> &'static str {
        match lang {
            Language::English => "Time Constant (T, sec)",
            Language::Chinese => "时间常数 (T, 秒)",
        }
    }

    pub fn dead_time(lang: Language) -> &'static str {
        match lang {
            Language::English => "Dead Time (L, sec)",
            Language::Chinese => "纯时滞 (L, 秒)",
        }
    }

    pub fn model_prediction(lang: Language) -> &'static str {
        match lang {
            Language::English => "Model Prediction",
            Language::Chinese => "模型预测值",
        }
    }

    pub fn delay_buffer_size(lang: Language) -> &'static str {
        match lang {
            Language::English => "Delay Buffer Size",
            Language::Chinese => "延迟缓冲长度",
        }
    }

    // ─── ADRC ─────────────────────────────────────────────

    pub fn adrc_td_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Tracking Differentiator (TD)",
            Language::Chinese => "跟踪微分器 (TD)",
        }
    }

    pub fn adrc_eso_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Extended State Observer (ESO)",
            Language::Chinese => "扩展状态观测器 (ESO)",
        }
    }

    pub fn adrc_nlsef_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Nonlinear State Error Feedback (NLSEF)",
            Language::Chinese => "非线性状态误差反馈 (NLSEF)",
        }
    }

    // ─── LADRC ────────────────────────────────────────────

    pub fn ladrc_bandwidth_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "LADRC Bandwidth Parameters",
            Language::Chinese => "LADRC 带宽参数",
        }
    }

    pub fn ladrc_order(lang: Language) -> &'static str {
        match lang {
            Language::English => "Order",
            Language::Chinese => "阶次",
        }
    }

    pub fn ladrc_first_order(lang: Language) -> &'static str {
        match lang {
            Language::English => "1st Order",
            Language::Chinese => "一阶",
        }
    }

    pub fn ladrc_second_order(lang: Language) -> &'static str {
        match lang {
            Language::English => "2nd Order",
            Language::Chinese => "二阶",
        }
    }

    // ─── LQR ──────────────────────────────────────────────

    pub fn lqr_weights(lang: Language) -> &'static str {
        match lang {
            Language::English => "LQR State & Control Weights",
            Language::Chinese => "LQR 状态与控制权重",
        }
    }

    pub fn lqr_q_position(lang: Language) -> &'static str {
        match lang {
            Language::English => "Position Weight",
            Language::Chinese => "位置权重",
        }
    }

    pub fn lqr_q_velocity(lang: Language) -> &'static str {
        match lang {
            Language::English => "Velocity Weight",
            Language::Chinese => "速度权重",
        }
    }

    pub fn lqr_r_weight(lang: Language) -> &'static str {
        match lang {
            Language::English => "Control Weight",
            Language::Chinese => "控制权重",
        }
    }

    pub fn lqr_mass(lang: Language) -> &'static str {
        match lang {
            Language::English => "Mass (kg)",
            Language::Chinese => "质量 (kg)",
        }
    }

    pub fn lqr_integral(lang: Language) -> &'static str {
        match lang {
            Language::English => "Integral Action (optional)",
            Language::Chinese => "积分环节 (可选)",
        }
    }

    pub fn lqr_computed_gains(lang: Language) -> &'static str {
        match lang {
            Language::English => "Computed Gains:",
            Language::Chinese => "计算增益:",
        }
    }

    // ─── MPC ──────────────────────────────────────────────

    pub fn mpc_horizons(lang: Language) -> &'static str {
        match lang {
            Language::English => "MPC Horizons",
            Language::Chinese => "MPC 预测与控制时域",
        }
    }

    pub fn mpc_prediction_horizon(lang: Language) -> &'static str {
        match lang {
            Language::English => "Prediction Horizon (Np)",
            Language::Chinese => "预测时域 (Np)",
        }
    }

    pub fn mpc_control_horizon(lang: Language) -> &'static str {
        match lang {
            Language::English => "Control Horizon (Nc)",
            Language::Chinese => "控制时域 (Nc)",
        }
    }

    pub fn mpc_model_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Internal Model",
            Language::Chinese => "内部模型",
        }
    }

    pub fn mpc_sample_time(lang: Language) -> &'static str {
        match lang {
            Language::English => "Sample Time (sec)",
            Language::Chinese => "采样时间 (秒)",
        }
    }

    pub fn mpc_weights_and_constraints(lang: Language) -> &'static str {
        match lang {
            Language::English => "Weights & Constraints",
            Language::Chinese => "权重与约束",
        }
    }

    pub fn mpc_du_limit(lang: Language) -> &'static str {
        match lang {
            Language::English => "ΔU Limit",
            Language::Chinese => "ΔU 限幅",
        }
    }

    // ─── 底盘运动学 ────────────────────────────────────────

    pub fn chassis_kinematics(lang: Language) -> &'static str {
        match lang {
            Language::English => "Chassis Kinematics Code Examples",
            Language::Chinese => "底盘运动学代码示例",
        }
    }

    pub fn chassis_kinematics_desc(lang: Language) -> &'static str {
        match lang {
            Language::English => {
                "Reference code templates for common robot chassis forward/inverse kinematics"
            }
            Language::Chinese => "常见机器人底盘正/逆运动学参考代码模板",
        }
    }

    // ─── NN Tuning ────────────────────────────────────────

    pub fn network_arch(lang: Language) -> &'static str {
        match lang {
            Language::English => "Network Architecture",
            Language::Chinese => "网络架构",
        }
    }

    pub fn training_controls(lang: Language) -> &'static str {
        match lang {
            Language::English => "Training Controls",
            Language::Chinese => "训练控制",
        }
    }

    pub fn learning_rate(lang: Language) -> &'static str {
        match lang {
            Language::English => "Learning Rate",
            Language::Chinese => "学习率",
        }
    }

    pub fn train_step(lang: Language) -> &'static str {
        match lang {
            Language::English => "Train Step",
            Language::Chinese => "训练一步",
        }
    }

    pub fn auto_train(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto-Train",
            Language::Chinese => "自动训练",
        }
    }

    pub fn training_loss(lang: Language) -> &'static str {
        match lang {
            Language::English => "Training Loss",
            Language::Chinese => "训练损失",
        }
    }

    pub fn no_training_data(lang: Language) -> &'static str {
        match lang {
            Language::English => "No training data. Start control and collect error data first.",
            Language::Chinese => "暂无训练数据。请先启动控制并采集误差数据。",
        }
    }

    pub fn suggested_params(lang: Language) -> &'static str {
        match lang {
            Language::English => "Suggested Parameters",
            Language::Chinese => "建议参数",
        }
    }

    pub fn predict(lang: Language) -> &'static str {
        match lang {
            Language::English => "Predict",
            Language::Chinese => "预测",
        }
    }

    pub fn apply_suggested(lang: Language) -> &'static str {
        match lang {
            Language::English => "Apply Suggested",
            Language::Chinese => "应用建议值",
        }
    }

    pub fn input_features(lang: Language) -> &'static str {
        match lang {
            Language::English => "Input Features Preview",
            Language::Chinese => "输入特征预览",
        }
    }

    pub fn parameter(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parameter",
            Language::Chinese => "参数",
        }
    }

    pub fn current(lang: Language) -> &'static str {
        match lang {
            Language::English => "Current",
            Language::Chinese => "当前值",
        }
    }

    pub fn suggested(lang: Language) -> &'static str {
        match lang {
            Language::English => "Suggested",
            Language::Chinese => "建议值",
        }
    }

    pub fn delta(lang: Language) -> &'static str {
        match lang {
            Language::English => "Delta",
            Language::Chinese => "差值",
        }
    }

    // ─── Data Viz ──────────────────────────────────────────

    pub fn channels(lang: Language) -> &'static str {
        match lang {
            Language::English => "Channels",
            Language::Chinese => "通道",
        }
    }

    pub fn position(lang: Language) -> &'static str {
        match lang {
            Language::English => "Position",
            Language::Chinese => "位置",
        }
    }

    pub fn velocity(lang: Language) -> &'static str {
        match lang {
            Language::English => "Velocity",
            Language::Chinese => "速度",
        }
    }

    pub fn current_a(lang: Language) -> &'static str {
        match lang {
            Language::English => "Current",
            Language::Chinese => "电流",
        }
    }

    pub fn temperature(lang: Language) -> &'static str {
        match lang {
            Language::English => "Temperature",
            Language::Chinese => "温度",
        }
    }

    pub fn error_ch(lang: Language) -> &'static str {
        match lang {
            Language::English => "Error",
            Language::Chinese => "误差",
        }
    }

    pub fn pid_output(lang: Language) -> &'static str {
        match lang {
            Language::English => "PID Output",
            Language::Chinese => "PID 输出",
        }
    }

    pub fn data_points(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Points",
            Language::Chinese => "数据点",
        }
    }

    pub fn clear_history(lang: Language) -> &'static str {
        match lang {
            Language::English => "Clear History",
            Language::Chinese => "清空历史",
        }
    }

    pub fn no_data_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => {
                "No data to display. Connect a device and start control to see real-time charts."
            }
            Language::Chinese => "无数据可显示。请连接设备并启动控制以查看实时图表。",
        }
    }

    // ─── Modbus ────────────────────────────────────────────

    pub fn request_builder(lang: Language) -> &'static str {
        match lang {
            Language::English => "Request Builder",
            Language::Chinese => "请求构建",
        }
    }

    pub fn slave_id(lang: Language) -> &'static str {
        match lang {
            Language::English => "Slave ID",
            Language::Chinese => "从站地址",
        }
    }

    pub fn function(lang: Language) -> &'static str {
        match lang {
            Language::English => "Function",
            Language::Chinese => "功能码",
        }
    }

    pub fn start_address(lang: Language) -> &'static str {
        match lang {
            Language::English => "Start Address",
            Language::Chinese => "起始地址",
        }
    }

    pub fn quantity(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quantity",
            Language::Chinese => "数量",
        }
    }

    pub fn write_values(lang: Language) -> &'static str {
        match lang {
            Language::English => "Write Values",
            Language::Chinese => "写入值",
        }
    }

    pub fn frame_preview(lang: Language) -> &'static str {
        match lang {
            Language::English => "Frame Preview",
            Language::Chinese => "帧预览",
        }
    }

    pub fn send_rtu(lang: Language) -> &'static str {
        match lang {
            Language::English => "Send RTU",
            Language::Chinese => "发送 RTU",
        }
    }

    pub fn send_tcp(lang: Language) -> &'static str {
        match lang {
            Language::English => "Send TCP",
            Language::Chinese => "发送 TCP",
        }
    }

    pub fn register_table(lang: Language) -> &'static str {
        match lang {
            Language::English => "Register Table (Simulated)",
            Language::Chinese => "寄存器表 (模拟)",
        }
    }

    pub fn randomize(lang: Language) -> &'static str {
        match lang {
            Language::English => "Randomize",
            Language::Chinese => "随机填充",
        }
    }

    pub fn modbus_log(lang: Language) -> &'static str {
        match lang {
            Language::English => "Modbus Log",
            Language::Chinese => "Modbus 日志",
        }
    }

    // ─── 状态栏 ────────────────────────────────────────────

    pub fn found_ports(n: usize, lang: Language) -> String {
        match lang {
            Language::English => format!("Found {} ports", n),
            Language::Chinese => format!("发现 {} 个端口", n),
        }
    }

    pub fn sent_bytes(n: usize, lang: Language) -> String {
        match lang {
            Language::English => format!("Sent {} bytes", n),
            Language::Chinese => format!("已发送 {} 字节", n),
        }
    }

    pub fn send_error(e: &str, lang: Language) -> String {
        match lang {
            Language::English => format!("Send error: {}", e),
            Language::Chinese => format!("发送失败: {}", e),
        }
    }

    pub fn applied_preset(name: &str, lang: Language) -> String {
        match lang {
            Language::English => format!("Applied preset: {}", name),
            Language::Chinese => format!("已应用预设: {}", name),
        }
    }

    pub fn light_mode(lang: Language) -> &'static str {
        match lang {
            Language::English => "Light",
            Language::Chinese => "浅色",
        }
    }

    pub fn dark_mode(lang: Language) -> &'static str {
        match lang {
            Language::English => "Dark",
            Language::Chinese => "深色",
        }
    }

    pub fn select_port(lang: Language) -> &'static str {
        match lang {
            Language::English => "Select port...",
            Language::Chinese => "选择端口...",
        }
    }

    pub fn comma_values_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => "Comma separated values, e.g. 100,200,300",
            Language::Chinese => "逗号分隔值, 如 100,200,300",
        }
    }

    pub fn reset_pid(lang: Language) -> &'static str {
        match lang {
            Language::English => "Reset PID State",
            Language::Chinese => "重置 PID 状态",
        }
    }

    pub fn copied(lang: Language) -> &'static str {
        match lang {
            Language::English => "Copied to clipboard",
            Language::Chinese => "已复制到剪贴板",
        }
    }

    pub fn select(lang: Language) -> &'static str {
        match lang {
            Language::English => "Select...",
            Language::Chinese => "选择...",
        }
    }

    // ─── CAN 高级参数 ──────────────────────────────────────

    pub fn sample_point(lang: Language) -> &'static str {
        match lang {
            Language::English => "Sample Point",
            Language::Chinese => "采样点",
        }
    }

    pub fn data_sample_point(lang: Language) -> &'static str {
        match lang {
            Language::English => "Data Sample Point",
            Language::Chinese => "数据采样点",
        }
    }

    pub fn can_termination(lang: Language) -> &'static str {
        match lang {
            Language::English => "Termination Resistor",
            Language::Chinese => "终端电阻",
        }
    }

    pub fn can_listen_only(lang: Language) -> &'static str {
        match lang {
            Language::English => "Listen Only",
            Language::Chinese => "仅监听",
        }
    }

    pub fn can_loopback(lang: Language) -> &'static str {
        match lang {
            Language::English => "Loopback",
            Language::Chinese => "回环模式",
        }
    }

    pub fn can_auto_retransmit(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto Retransmit",
            Language::Chinese => "自动重传",
        }
    }

    pub fn can_error_reporting(lang: Language) -> &'static str {
        match lang {
            Language::English => "Error Reporting",
            Language::Chinese => "错误报告",
        }
    }

    // ─── USB 协议 ──────────────────────────────────────────

    pub fn usb_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "USB Configuration",
            Language::Chinese => "USB 配置",
        }
    }

    pub fn usb_protocol_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "USB Protocol",
            Language::Chinese => "USB 协议",
        }
    }

    pub fn usb_speed_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "USB Speed",
            Language::Chinese => "USB 速度",
        }
    }

    pub fn usb_endpoint_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "Endpoint Configuration",
            Language::Chinese => "端点配置",
        }
    }

    pub fn usb_endpoint_in(lang: Language) -> &'static str {
        match lang {
            Language::English => "Endpoint IN",
            Language::Chinese => "输入端点",
        }
    }

    pub fn usb_endpoint_out(lang: Language) -> &'static str {
        match lang {
            Language::English => "Endpoint OUT",
            Language::Chinese => "输出端点",
        }
    }

    pub fn usb_max_packet_size(lang: Language) -> &'static str {
        match lang {
            Language::English => "Max Packet Size",
            Language::Chinese => "最大包大小",
        }
    }

    pub fn usb_interface(lang: Language) -> &'static str {
        match lang {
            Language::English => "Interface",
            Language::Chinese => "接口",
        }
    }

    pub fn usb_typical_speeds(lang: Language) -> &'static str {
        match lang {
            Language::English => "Typical Speeds",
            Language::Chinese => "典型速度",
        }
    }

    pub fn usb_cdc_hint(lang: Language) -> &'static str {
        match lang {
            Language::English => "CDC ACM devices use virtual COM port",
            Language::Chinese => "CDC ACM 设备使用虚拟串口",
        }
    }

    // ─── Packet Builder / Parser ───────────────────────────

    pub fn builder_tab(lang: Language) -> &'static str {
        match lang {
            Language::English => "Builder",
            Language::Chinese => "构建器",
        }
    }

    pub fn parser_tab(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parser",
            Language::Chinese => "解析器",
        }
    }

    pub fn parser_template(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parse Template",
            Language::Chinese => "解析模板",
        }
    }

    pub fn auto_parse(lang: Language) -> &'static str {
        match lang {
            Language::English => "Auto Parse",
            Language::Chinese => "自动解析",
        }
    }

    pub fn parser_input(lang: Language) -> &'static str {
        match lang {
            Language::English => "HEX Data Input",
            Language::Chinese => "HEX 数据输入",
        }
    }

    pub fn parse_now(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parse Now",
            Language::Chinese => "立即解析",
        }
    }

    pub fn parsed_count(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parsed",
            Language::Chinese => "已解析",
        }
    }

    pub fn parser_empty(lang: Language) -> &'static str {
        match lang {
            Language::English => "No parsed results yet. Paste HEX data and click Parse.",
            Language::Chinese => "暂无解析结果。粘贴 HEX 数据后点击解析。",
        }
    }

    pub fn parse_success(name: &str, count: usize, lang: Language) -> String {
        match lang {
            Language::English => format!("Parsed '{}': {} fields", name, count),
            Language::Chinese => format!("已解析 '{}': {} 个字段", name, count),
        }
    }

    pub fn parse_failed(lang: Language) -> &'static str {
        match lang {
            Language::English => "Parse failed: no matching template",
            Language::Chinese => "解析失败: 无匹配模板",
        }
    }

    pub fn field_type_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "Type",
            Language::Chinese => "类型",
        }
    }

    pub fn field_value_label(lang: Language) -> &'static str {
        match lang {
            Language::English => "Value",
            Language::Chinese => "值",
        }
    }

    pub fn field_numeric(lang: Language) -> &'static str {
        match lang {
            Language::English => "Numeric",
            Language::Chinese => "数值",
        }
    }

    // ─── 数据可视化 ────────────────────────────────────────

    pub fn viz_channel_config(lang: Language) -> &'static str {
        match lang {
            Language::English => "Channel Configuration",
            Language::Chinese => "通道配置",
        }
    }

    pub fn viz_add_channel(lang: Language) -> &'static str {
        match lang {
            Language::English => "Add Channel",
            Language::Chinese => "添加通道",
        }
    }

    // ─── 企业级菜单栏 / Enterprise Menu Bar ──────────────

    pub fn menu_file(lang: Language) -> &'static str {
        match lang {
            Language::English => "File",
            Language::Chinese => "文件",
        }
    }
    pub fn menu_edit(lang: Language) -> &'static str {
        match lang {
            Language::English => "Edit",
            Language::Chinese => "编辑",
        }
    }
    pub fn menu_view(lang: Language) -> &'static str {
        match lang {
            Language::English => "View",
            Language::Chinese => "视图",
        }
    }
    pub fn menu_tools(lang: Language) -> &'static str {
        match lang {
            Language::English => "Tools",
            Language::Chinese => "工具",
        }
    }
    pub fn menu_help(lang: Language) -> &'static str {
        match lang {
            Language::English => "Help",
            Language::Chinese => "帮助",
        }
    }
    pub fn menu_export_log(lang: Language) -> &'static str {
        match lang {
            Language::English => "Export Logs (CSV)",
            Language::Chinese => "导出日志 (CSV)",
        }
    }
    pub fn menu_import_preset(lang: Language) -> &'static str {
        match lang {
            Language::English => "Import Preset...",
            Language::Chinese => "导入预设...",
        }
    }
    pub fn menu_preferences(lang: Language) -> &'static str {
        match lang {
            Language::English => "Preferences",
            Language::Chinese => "偏好设置",
        }
    }
    pub fn menu_quit(lang: Language) -> &'static str {
        match lang {
            Language::English => "Quit",
            Language::Chinese => "退出",
        }
    }
    pub fn menu_clear_logs(lang: Language) -> &'static str {
        match lang {
            Language::English => "Clear All Logs",
            Language::Chinese => "清除所有日志",
        }
    }
    pub fn menu_copy_frame(lang: Language) -> &'static str {
        match lang {
            Language::English => "Copy Last Frame",
            Language::Chinese => "复制最后一帧",
        }
    }
    pub fn menu_reset_counters(lang: Language) -> &'static str {
        match lang {
            Language::English => "Reset Counters",
            Language::Chinese => "重置计数器",
        }
    }
    pub fn menu_hide_sidebar(lang: Language) -> &'static str {
        match lang {
            Language::English => "Hide Sidebar",
            Language::Chinese => "隐藏侧边栏",
        }
    }
    pub fn menu_show_sidebar(lang: Language) -> &'static str {
        match lang {
            Language::English => "Show Sidebar",
            Language::Chinese => "显示侧边栏",
        }
    }
    pub fn menu_motion_level(lang: Language) -> &'static str {
        match lang {
            Language::English => "Motion Level",
            Language::Chinese => "动效等级",
        }
    }
    pub fn menu_language(lang: Language) -> &'static str {
        match lang {
            Language::English => "Language",
            Language::Chinese => "语言",
        }
    }
    pub fn menu_mcp_server(lang: Language) -> &'static str {
        match lang {
            Language::English => "Toggle MCP Server",
            Language::Chinese => "切换 MCP 服务",
        }
    }
    pub fn menu_about(lang: Language) -> &'static str {
        match lang {
            Language::English => "About",
            Language::Chinese => "关于",
        }
    }
    pub fn menu_shortcuts(lang: Language) -> &'static str {
        match lang {
            Language::English => "Keyboard Shortcuts",
            Language::Chinese => "键盘快捷键",
        }
    }
    pub fn menu_docs(lang: Language) -> &'static str {
        match lang {
            Language::English => "Documentation",
            Language::Chinese => "文档",
        }
    }
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
            assert!(!Tr::tab_modbus(*lang).is_empty());
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
}
