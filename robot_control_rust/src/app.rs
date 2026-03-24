use crate::i18n::Language;
use crate::models::*;
use crate::services::*;
use std::fs::{metadata, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

// ═══════════════════════════════════════════════════════════════
// 导航标签
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard,
    Connections,
    SerialDebug,
    ProtocolAnalysis,
    PacketBuilder,
    Topology,
    PidControl,
    NnTuning,
    DataViz,
    ModbusTools,
    CanopenTools,
}

impl ActiveTab {
    pub fn label(&self, lang: Language) -> &'static str {
        use crate::i18n::Tr;
        match self {
            Self::Dashboard => Tr::tab_dashboard(lang),
            Self::Connections => Tr::tab_connections(lang),
            Self::SerialDebug => Tr::tab_terminal(lang),
            Self::ProtocolAnalysis => Tr::tab_protocol_analysis(lang),
            Self::PacketBuilder => Tr::tab_packet_builder(lang),
            Self::Topology => Tr::tab_topology(lang),
            Self::PidControl => Tr::tab_pid_control(lang),
            Self::NnTuning => Tr::tab_nn_tuning(lang),
            Self::DataViz => Tr::tab_data_viz(lang),
            Self::ModbusTools => Tr::tab_modbus(lang),
            Self::CanopenTools => Tr::tab_canopen(lang),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Dashboard => "DB",
            Self::Connections => "CN",
            Self::SerialDebug => "SR",
            Self::ProtocolAnalysis => "AN",
            Self::PacketBuilder => "PK",
            Self::Topology => "TP",
            Self::PidControl => "PD",
            Self::NnTuning => "NN",
            Self::DataViz => "DV",
            Self::ModbusTools => "MB",
            Self::CanopenTools => "CO",
        }
    }

    /// Category grouping for sidebar sections
    pub fn category(&self) -> &'static str {
        match self {
            Self::Dashboard => "OVERVIEW",
            Self::Connections | Self::SerialDebug => "COMM",
            Self::ProtocolAnalysis
            | Self::PacketBuilder
            | Self::ModbusTools
            | Self::CanopenTools => "PROTOCOL",
            Self::Topology | Self::PidControl | Self::NnTuning => "CONTROL",
            Self::DataViz => "ANALYSIS",
        }
    }

    pub fn all() -> &'static [ActiveTab] {
        &[
            Self::Dashboard,
            Self::Connections,
            Self::SerialDebug,
            Self::ProtocolAnalysis,
            Self::PacketBuilder,
            Self::Topology,
            Self::PidControl,
            Self::NnTuning,
            Self::DataViz,
            Self::ModbusTools,
            Self::CanopenTools,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════
// 日志条目
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub direction: LogDirection,
    pub data: Vec<u8>,
    pub display_mode: DisplayMode,
    pub channel: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogDirection {
    Tx,
    Rx,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DisplayMode {
    Hex,
    Ascii,
    Mixed,
}

impl LogEntry {
    pub fn format_data(&self) -> String {
        match self.display_mode {
            DisplayMode::Hex => self
                .data
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" "),
            DisplayMode::Ascii => String::from_utf8_lossy(&self.data).to_string(),
            DisplayMode::Mixed => {
                let hex = self
                    .data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                let ascii: String = self
                    .data
                    .iter()
                    .map(|&b| {
                        if b.is_ascii_graphic() || b == b' ' {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();
                format!("{} | {}", hex, ascii)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// UI 状态
// ═══════════════════════════════════════════════════════════════

pub struct UiState {
    // PID 文本框
    pub kp_text: String,
    pub ki_text: String,
    pub kd_text: String,
    pub setpoint_text: String,
    pub output_limit_text: String,
    pub integral_limit_text: String,
    pub preset_name: String,
    pub preset_desc: String,

    // 图表显示控制
    pub show_position: bool,
    pub show_velocity: bool,
    pub show_current: bool,
    pub show_temperature: bool,
    pub show_error: bool,
    pub show_pid_output: bool,

    // 终端
    pub send_text: String,
    pub send_hex: bool,
    pub auto_scroll: bool,
    pub display_mode: DisplayMode,
    pub auto_newline: bool,
    pub send_with_newline: bool,
    pub newline_type: String,
    pub repeat_send: bool,
    pub repeat_interval_ms: u32,
    pub auto_reconnect_enabled: bool,
    pub auto_reconnect_interval_ms: u32,
    pub quick_cmd_1: String,
    pub quick_cmd_2: String,
    pub quick_cmd_3: String,

    // Modbus
    pub modbus_slave_id_text: String,
    pub modbus_start_addr_text: String,
    pub modbus_quantity_text: String,
    pub modbus_write_values_text: String,
    pub modbus_fn_idx: usize,

    // CANopen
    pub canopen_node_id_text: String,
    pub canopen_nmt_cmd_idx: usize,
    pub canopen_sdo_action_idx: usize,
    pub canopen_index_text: String,
    pub canopen_subidx_text: String,
    pub canopen_payload_text: String,
    pub canopen_pdo_cobid_text: String,
    pub canopen_pdo_data_text: String,
    pub canopen_heartbeat_ms_text: String,
    pub canopen_decode_input: String,

    // CANopen PDO decode / analyze
    pub canopen_pdo_decode_hex: String,
    pub canopen_analyze_cobid_text: String,
    pub canopen_analyze_data_text: String,

    // Multi-protocol CAN
    pub canopen_protocol_idx: usize, // 0=CAN 2.0, 1=CAN FD, 2=EtherCAT CoE
    pub canopen_fd_data_text: String,
    pub canopen_ecat_write: bool,
    pub canopen_ecat_analyze_hex: String,

    // CAN
    pub can_id_text: String,
    pub can_data_text: String,
    pub can_extended: bool,
    pub can_fd: bool,

    // Packet Builder
    pub packet_template_idx: usize,

    // Connections
    pub conn_type_idx: usize,
    pub serial_port_search: String,
    pub serial_baud_idx: usize,

    // TCP/UDP
    pub tcp_host: String,
    pub tcp_port_text: String,
    pub tcp_is_server: bool,
    pub udp_local_port_text: String,
    pub udp_remote_host: String,
    pub udp_remote_port_text: String,

    // NN
    pub nn_learning_rate_text: String,
    pub nn_auto_train: bool,
    pub nn_train_interval: u32,

    // CAN 高级参数
    pub can_bitrate_idx: usize,
    pub can_data_bitrate_idx: usize,
    pub can_sample_point_idx: usize,
    pub can_data_sample_point_idx: usize,
    pub can_sjw_idx: usize,
    pub can_data_sjw_idx: usize,

    // USB 协议
    pub usb_protocol_idx: usize,
    pub usb_speed_idx: usize,
    pub usb_vid_text: String,
    pub usb_pid_text: String,

    // 数据包解析
    pub parser_enabled: bool,
    pub parser_template_idx: usize,
    pub parser_auto_parse: bool,
    pub parser_hex_input: String,
    pub parser_last_auto_input: String,
    pub parser_last_auto_template_idx: usize,
    pub packet_builder_tab: usize, // 0=Builder, 1=Parser

    // 协议分析页
    pub analysis_protocol_idx: usize,
    pub analysis_filter_tx: bool,
    pub analysis_filter_rx: bool,
    pub analysis_filter_info: bool,
    pub analysis_query: String,
    pub analysis_hex_input: String,

    // 数据可视化
    pub viz_add_channel_name: String,
    pub viz_add_source_idx: usize,
    pub viz_add_type_idx: usize,
    pub viz_source_type: usize, // 0=RobotState, 1=PacketField
    pub viz_pkt_template_idx: usize,
    pub viz_pkt_field_idx: usize,

    // LLM 配置
    pub llm_api_url: String,
    pub llm_api_key: String,
    pub llm_model_name: String,
    pub llm_temperature_text: String,
    pub llm_last_response: String,
    pub llm_loading: bool,

    // MCP 服务器
    pub mcp_port_text: String,
    pub mcp_token_text: String,
    pub mcp_running: bool,

    // 侧边栏
    pub sidebar_expanded: bool,

    // 动效层级：0=极致, 1=标准, 2=原生, 3=优化
    pub motion_level_idx: usize,

    // 偏好自动保存周期（秒）
    pub prefs_autosave_interval_sec: u32,

    // 更新检查
    pub update_channel: String,
    pub update_manifest_url: String,
    pub update_check_timeout_ms: u32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            kp_text: "1.000".into(),
            ki_text: "0.100".into(),
            kd_text: "0.010".into(),
            setpoint_text: "0.000".into(),
            output_limit_text: "100.0".into(),
            integral_limit_text: "100.0".into(),
            preset_name: String::new(),
            preset_desc: String::new(),
            show_position: true,
            show_velocity: true,
            show_current: true,
            show_temperature: true,
            show_error: true,
            show_pid_output: true,
            send_text: String::new(),
            send_hex: false,
            auto_scroll: true,
            display_mode: DisplayMode::Hex,
            auto_newline: false,
            send_with_newline: true,
            newline_type: "\\r\\n".into(),
            repeat_send: false,
            repeat_interval_ms: 1000,
            auto_reconnect_enabled: true,
            auto_reconnect_interval_ms: 2000,
            quick_cmd_1: "status".into(),
            quick_cmd_2: "help".into(),
            quick_cmd_3: "reboot".into(),
            modbus_slave_id_text: "1".into(),
            modbus_start_addr_text: "0".into(),
            modbus_quantity_text: "10".into(),
            modbus_write_values_text: String::new(),
            modbus_fn_idx: 2,
            canopen_node_id_text: "1".into(),
            canopen_nmt_cmd_idx: 0,
            canopen_sdo_action_idx: 0,
            canopen_index_text: "0x1000".into(),
            canopen_subidx_text: "0x00".into(),
            canopen_payload_text: "11 22 33 44".into(),
            canopen_pdo_cobid_text: "0x181".into(),
            canopen_pdo_data_text: "01 02 03 04 05 06 07 08".into(),
            canopen_heartbeat_ms_text: "1000".into(),
            canopen_decode_input: "80 00 01 00 00 00 00 00".into(),
            canopen_pdo_decode_hex: String::new(),
            canopen_analyze_cobid_text: "0x605".into(),
            canopen_analyze_data_text: "40 00 10 01 00 00 00 00".into(),
            canopen_protocol_idx: 0,
            canopen_fd_data_text: String::new(),
            canopen_ecat_write: false,
            canopen_ecat_analyze_hex: String::new(),
            can_id_text: "0x100".into(),
            can_data_text: "00 01 02 03 04 05 06 07".into(),
            can_extended: false,
            can_fd: false,
            packet_template_idx: 0,
            conn_type_idx: 0,
            serial_port_search: String::new(),
            serial_baud_idx: 12,
            tcp_host: "127.0.0.1".into(),
            tcp_port_text: "8080".into(),
            tcp_is_server: false,
            udp_local_port_text: "9000".into(),
            udp_remote_host: "127.0.0.1".into(),
            udp_remote_port_text: "9001".into(),
            nn_learning_rate_text: "0.01".into(),
            nn_auto_train: false,
            nn_train_interval: 100,
            can_bitrate_idx: 6,           // 500 kbps
            can_data_bitrate_idx: 2,      // 2 Mbps
            can_sample_point_idx: 4,      // 87.5%
            can_data_sample_point_idx: 0, // 75.0%
            can_sjw_idx: 0,
            can_data_sjw_idx: 0,
            usb_protocol_idx: 0,
            usb_speed_idx: 1,
            usb_vid_text: "0483".into(),
            usb_pid_text: "5740".into(),
            parser_enabled: false,
            parser_template_idx: 0,
            parser_auto_parse: true,
            parser_hex_input: String::new(),
            parser_last_auto_input: String::new(),
            parser_last_auto_template_idx: 0,
            packet_builder_tab: 0,
            analysis_protocol_idx: 0,
            analysis_filter_tx: true,
            analysis_filter_rx: true,
            analysis_filter_info: false,
            analysis_query: String::new(),
            analysis_hex_input: String::new(),
            viz_add_channel_name: String::new(),
            viz_add_source_idx: 0,
            viz_add_type_idx: 0,
            viz_source_type: 0,
            viz_pkt_template_idx: 0,
            viz_pkt_field_idx: 0,
            llm_api_url: "https://api.openai.com/v1/chat/completions".into(),
            llm_api_key: String::new(),
            llm_model_name: "gpt-4o-mini".into(),
            llm_temperature_text: "0.3".into(),
            llm_last_response: String::new(),
            llm_loading: false,
            mcp_port_text: "3000".into(),
            mcp_token_text: String::new(),
            mcp_running: false,
            sidebar_expanded: true,
            motion_level_idx: 2,
            prefs_autosave_interval_sec: 3,
            update_channel: "stable-0.1".into(),
            update_manifest_url: String::new(),
            update_check_timeout_ms: 1500,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 主应用状态
// ═══════════════════════════════════════════════════════════════

pub struct AppState {
    pub active_tab: ActiveTab,

    // 语言
    pub language: Language,

    // 通信服务
    pub serial: SerialService,
    pub tcp: TcpService,
    pub udp: UdpService,
    pub can: CanService,
    pub active_conn: ConnectionType,

    // 端口列表
    pub available_ports: Vec<String>,

    // 控制
    pub active_algorithm: ControlAlgorithmType,
    pub pid: PidController,
    pub incremental_pid: IncrementalPidController,
    pub bang_bang: BangBangController,
    pub fuzzy_pid: FuzzyPidController,
    pub cascade_pid: CascadePidController,
    pub smith_predictor: SmithPredictorController,
    pub adrc: AdrcController,
    pub ladrc: LadrcController,
    pub lqr: LqrController,
    pub mpc: MpcController,
    pub current_state: RobotState,
    pub state_history: Vec<RobotState>,
    pub is_running: bool,
    pub presets: Vec<Preset>,

    // 神经网络
    pub nn: NeuralNetwork,
    pub nn_suggested_kp: f64,
    pub nn_suggested_ki: f64,
    pub nn_suggested_kd: f64,

    // 拓扑
    pub topology: TopologyConfig,
    pub builtin_topologies: Vec<TopologyConfig>,

    // 日志
    pub log_entries: Vec<LogEntry>,

    // Packet Builder
    pub packet_templates: Vec<PacketTemplate>,

    // Modbus
    pub modbus_frame: ModbusFrame,
    pub modbus_registers: Vec<u16>,
    pub modbus_response_log: Vec<String>,

    // CANopen
    pub canopen_log: Vec<String>,

    // CANopen PDO Configs
    pub canopen_pdo_configs: Vec<PdoConfig>,

    // Packet Parser
    pub packet_parser: PacketParser,
    pub parsed_packets: Vec<ParsedPacket>,

    // Data Visualization
    pub data_channels: Vec<DataChannel>,
    pub channel_buffers: Vec<TimeSeriesBuffer>,

    // USB 配置
    pub usb_config: UsbConfig,

    // MCP server 状态
    pub mcp_server_handle: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub mcp_shared_state:
        std::sync::Arc<std::sync::Mutex<crate::services::mcp_server::McpSharedState>>,

    // UI
    pub ui: UiState,
    pub status_message: String,
    pub dark_mode: bool,
    pub build_version: &'static str,
    pub system_checks: Vec<SystemCheckItem>,
    pub metrics: AppMetrics,
    pub last_error_time: String,
    pub update_latest_version: String,
    pub update_status_detail: String,
    pub update_available: bool,
    pub update_notes_url: String,
    pub update_last_checked_at: String,
    pub channel_overflow_events: u64,
    can_dropped_frames_seen: u64,
    channel_overflow_notified: u64,
    last_rx_instant: Option<Instant>,
    reconnect_paused_by_user: bool,
    next_reconnect_at: Option<Instant>,
    last_mcp_sync_instant: Option<Instant>,
    port_scan_in_progress: bool,
    port_scan_rx: Option<std::sync::mpsc::Receiver<Vec<String>>>,
    pending_log_lines: Vec<String>,
    last_log_flush_instant: Instant,
    serial_connect_rx: Option<std::sync::mpsc::Receiver<Result<SerialService, String>>>,
    serial_connect_in_progress: bool,
    llm_result_rx: Option<
        std::sync::mpsc::Receiver<Result<crate::services::llm_service::SuggestedParams, String>>,
    >,
}

const MAX_HISTORY: usize = 2000;
const MAX_LOG: usize = 5000;
const LOG_FILE_MAX_BYTES: u64 = 5 * 1024 * 1024;
const DEFAULT_UPDATE_DOC_URL: &str =
    "https://github.com/search?q=robot_control_rust&type=repositories";
const DEFAULT_UPDATE_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/example/robot_control_rust/main/update-manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct VersionTriplet {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
#[derive(Default)]
struct UpdateManifest {
    latest_version: String,
    channel: String,
    notes_url: String,
    min_supported_version: String,
}

fn parse_version_triplet(text: &str) -> Option<VersionTriplet> {
    let normalized = text
        .trim()
        .trim_start_matches('v')
        .split('-')
        .next()
        .unwrap_or_default();
    let mut parts = normalized.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    Some(VersionTriplet {
        major,
        minor,
        patch,
    })
}

#[derive(Debug, Clone, Default)]
pub struct AppMetrics {
    pub connect_attempts: u64,
    pub connect_failures: u64,
    pub llm_requests: u64,
    pub llm_success: u64,
    pub llm_failures: u64,
    pub mcp_startups: u64,
}

#[derive(Debug, Clone)]
pub struct SystemCheckItem {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

fn parse_port(text: &str, label: &str) -> Result<u16, String> {
    let port: u16 = text
        .trim()
        .parse()
        .map_err(|_| format!("{} must be 1-65535", label))?;
    if port == 0 {
        return Err(format!("{} must be 1-65535", label));
    }
    Ok(port)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct UserPreferences {
    schema_version: u32,
    language: Language,
    dark_mode: bool,
    sidebar_expanded: bool,
    motion_level_idx: usize,
    active_tab_idx: usize,
    parser_auto_parse: bool,
    display_mode: DisplayMode,
    llm_api_url: String,
    llm_model_name: String,
    mcp_port_text: String,
    mcp_token_text: String,
    active_conn: ConnectionType,
    serial_config: SerialConfig,
    tcp_host: String,
    tcp_port_text: String,
    tcp_is_server: bool,
    udp_local_port_text: String,
    udp_remote_host: String,
    udp_remote_port_text: String,
    auto_newline: bool,
    auto_reconnect_enabled: bool,
    auto_reconnect_interval_ms: u32,
    quick_cmd_1: String,
    quick_cmd_2: String,
    quick_cmd_3: String,
    send_hex: bool,
    auto_scroll: bool,
    send_with_newline: bool,
    newline_type: String,
    repeat_send: bool,
    repeat_interval_ms: u32,
    can_id_text: String,
    can_data_text: String,
    can_extended: bool,
    can_fd: bool,
    can_bitrate_idx: usize,
    can_data_bitrate_idx: usize,
    can_sample_point_idx: usize,
    can_data_sample_point_idx: usize,
    can_sjw_idx: usize,
    can_data_sjw_idx: usize,
    usb_protocol_idx: usize,
    usb_speed_idx: usize,
    usb_vid_text: String,
    usb_pid_text: String,
    packet_template_idx: usize,
    parser_enabled: bool,
    parser_template_idx: usize,
    packet_builder_tab: usize,
    analysis_protocol_idx: usize,
    analysis_filter_tx: bool,
    analysis_filter_rx: bool,
    analysis_filter_info: bool,
    llm_temperature_text: String,
    prefs_autosave_interval_sec: u32,
    update_channel: String,
    update_manifest_url: String,
    update_check_timeout_ms: u32,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            schema_version: 2,
            language: Language::Chinese,
            dark_mode: true,
            sidebar_expanded: true,
            motion_level_idx: 2,
            active_tab_idx: 0,
            parser_auto_parse: true,
            display_mode: DisplayMode::Hex,
            llm_api_url: "https://api.openai.com/v1/chat/completions".into(),
            llm_model_name: "gpt-4o-mini".into(),
            mcp_port_text: "3000".into(),
            mcp_token_text: String::new(),
            active_conn: ConnectionType::Serial,
            serial_config: SerialConfig::default(),
            tcp_host: "127.0.0.1".into(),
            tcp_port_text: "8080".into(),
            tcp_is_server: false,
            udp_local_port_text: "9000".into(),
            udp_remote_host: "127.0.0.1".into(),
            udp_remote_port_text: "9001".into(),
            auto_newline: false,
            auto_reconnect_enabled: true,
            auto_reconnect_interval_ms: 2000,
            quick_cmd_1: "status".into(),
            quick_cmd_2: "help".into(),
            quick_cmd_3: "reboot".into(),
            send_hex: false,
            auto_scroll: true,
            send_with_newline: true,
            newline_type: "\\r\\n".into(),
            repeat_send: false,
            repeat_interval_ms: 1000,
            can_id_text: "0x123".into(),
            can_data_text: "01 02 03 04".into(),
            can_extended: false,
            can_fd: false,
            can_bitrate_idx: 5,
            can_data_bitrate_idx: 2,
            can_sample_point_idx: 2,
            can_data_sample_point_idx: 2,
            can_sjw_idx: 0,
            can_data_sjw_idx: 0,
            usb_protocol_idx: 0,
            usb_speed_idx: 2,
            usb_vid_text: "0x0483".into(),
            usb_pid_text: "0x5740".into(),
            packet_template_idx: 0,
            parser_enabled: false,
            parser_template_idx: 0,
            packet_builder_tab: 0,
            analysis_protocol_idx: 0,
            analysis_filter_tx: true,
            analysis_filter_rx: true,
            analysis_filter_info: false,
            llm_temperature_text: "0.7".into(),
            prefs_autosave_interval_sec: 3,
            update_channel: "stable-0.1".into(),
            update_manifest_url: String::new(),
            update_check_timeout_ms: 1500,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let pid = PidController::default();
        let ui = UiState {
            kp_text: format!("{:.3}", pid.kp),
            ki_text: format!("{:.3}", pid.ki),
            kd_text: format!("{:.3}", pid.kd),
            setpoint_text: format!("{:.3}", pid.setpoint),
            output_limit_text: format!("{:.1}", pid.output_limit),
            integral_limit_text: format!("{:.1}", pid.integral_limit),
            ..Default::default()
        };

        let mut s = Self {
            active_tab: ActiveTab::Dashboard,
            language: Language::Chinese,
            serial: SerialService::new(),
            tcp: TcpService::new(),
            udp: UdpService::new(),
            can: CanService::new(),
            active_conn: ConnectionType::Serial,
            available_ports: Vec::new(),
            active_algorithm: ControlAlgorithmType::ClassicPid,
            pid,
            incremental_pid: IncrementalPidController::default(),
            bang_bang: BangBangController::default(),
            fuzzy_pid: FuzzyPidController::default(),
            cascade_pid: CascadePidController::default(),
            smith_predictor: SmithPredictorController::default(),
            adrc: AdrcController::default(),
            ladrc: LadrcController::default(),
            lqr: LqrController::default(),
            mpc: MpcController::default(),
            current_state: RobotState::default(),
            state_history: Vec::new(),
            is_running: false,
            presets: Preset::defaults(),
            nn: NeuralNetwork::pid_tuner(),
            nn_suggested_kp: 1.0,
            nn_suggested_ki: 0.1,
            nn_suggested_kd: 0.01,
            topology: TopologyConfig::default(),
            builtin_topologies: TopologyConfig::builtin_configs(),
            log_entries: Vec::new(),
            packet_templates: PacketTemplate::builtin_templates(),
            modbus_frame: ModbusFrame::default(),
            modbus_registers: vec![0u16; 100],
            modbus_response_log: Vec::new(),
            canopen_log: Vec::new(),
            canopen_pdo_configs: Vec::new(),
            packet_parser: PacketParser::new(PacketTemplate::builtin_templates()),
            parsed_packets: Vec::new(),
            data_channels: DataChannel::default_channels(),
            channel_buffers: (0..6).map(|_| TimeSeriesBuffer::default()).collect(),
            usb_config: UsbConfig::default(),
            mcp_server_handle: None,
            mcp_shared_state: std::sync::Arc::new(std::sync::Mutex::new(
                crate::services::mcp_server::McpSharedState::default(),
            )),
            ui,
            status_message: "Ready".into(),
            dark_mode: true,
            build_version: env!("CARGO_PKG_VERSION"),
            system_checks: Vec::new(),
            metrics: AppMetrics::default(),
            last_error_time: "N/A".into(),
            update_latest_version: env!("CARGO_PKG_VERSION").into(),
            update_status_detail: "Update check not started".into(),
            update_available: false,
            update_notes_url: String::new(),
            update_last_checked_at: "N/A".into(),
            channel_overflow_events: 0,
            can_dropped_frames_seen: 0,
            channel_overflow_notified: 0,
            last_rx_instant: None,
            reconnect_paused_by_user: false,
            next_reconnect_at: None,
            last_mcp_sync_instant: None,
            port_scan_in_progress: false,
            port_scan_rx: None,
            pending_log_lines: Vec::new(),
            last_log_flush_instant: Instant::now(),
            serial_connect_rx: None,
            serial_connect_in_progress: false,
            llm_result_rx: None,
        };
        if let Ok(api_key) = std::env::var("LLM_API_KEY") {
            if !api_key.trim().is_empty() {
                s.ui.llm_api_key = api_key;
            }
        }
        if let Ok(mcp_token) = std::env::var("MCP_TOKEN") {
            if !mcp_token.trim().is_empty() {
                s.ui.mcp_token_text = mcp_token;
            }
        }
        s.load_user_preferences();
        s.refresh_ports();
        s.run_system_check();
        s
    }

    pub fn user_prefs_path() -> std::path::PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return std::path::PathBuf::from(appdata)
                    .join("robot_control_rust")
                    .join("preferences.json");
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return std::path::PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("robot_control_rust")
                    .join("preferences.json");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return std::path::PathBuf::from(home)
                .join(".config")
                .join("robot_control_rust")
                .join("preferences.json");
        }
        std::path::PathBuf::from("preferences.json")
    }

    fn log_file_path() -> PathBuf {
        let mut p = Self::user_prefs_path();
        p.pop();
        p.push("logs");
        p.push("app.log");
        p
    }

    fn rotate_log_if_needed(path: &PathBuf) {
        if let Ok(meta) = metadata(path) {
            if meta.len() > LOG_FILE_MAX_BYTES {
                let mut backup = path.clone();
                backup.set_extension("log.1");
                let _ = std::fs::rename(path, backup);
            }
        }
    }

    fn append_log(&mut self, entry: &LogEntry) {
        let line = format!(
            "{} [{}] [{}] {}",
            entry.timestamp,
            entry.channel,
            match entry.direction {
                LogDirection::Tx => "TX",
                LogDirection::Rx => "RX",
                LogDirection::Info => "INFO",
            },
            entry.format_data()
        );
        self.pending_log_lines.push(line);
        if self.pending_log_lines.len() > 10_000 {
            let overflow = self.pending_log_lines.len() - 10_000;
            self.pending_log_lines.drain(..overflow);
        }
    }

    pub fn flush_pending_logs(&mut self) {
        if self.pending_log_lines.is_empty() {
            return;
        }
        let now = Instant::now();
        if self.pending_log_lines.len() < 100
            && now.duration_since(self.last_log_flush_instant) < Duration::from_millis(800)
        {
            return;
        }

        let path = Self::log_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self::rotate_log_if_needed(&path);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            for line in self.pending_log_lines.drain(..) {
                let _ = writeln!(f, "{}", line);
            }
            self.last_log_flush_instant = now;
        }
    }

    pub fn report_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.last_error_time = chrono::Local::now().format("%H:%M:%S").to_string();
        self.status_message = message.clone();
        self.add_info_log(&format!("❌ {}", message));
        error!(target: "app", message = %self.status_message, "ui_error");
    }

    pub fn report_channel_overflow(&mut self, dropped: usize) {
        self.channel_overflow_events = self.channel_overflow_events.saturating_add(dropped as u64);
        warn!(target: "buffers", dropped = dropped, total = self.channel_overflow_events, "channel_overflow");
    }

    fn push_check(&mut self, name: &str, ok: bool, detail: impl Into<String>) {
        self.system_checks.push(SystemCheckItem {
            name: name.to_string(),
            ok,
            detail: detail.into(),
        });
    }

    pub fn run_system_check(&mut self) {
        self.system_checks.clear();

        let prefs_path = Self::user_prefs_path();
        let prefs_parent = prefs_path.parent().map(|p| p.to_path_buf());
        let prefs_ok = prefs_parent
            .as_ref()
            .map(|p| std::fs::create_dir_all(p).is_ok())
            .unwrap_or(false);
        self.push_check(
            "Preferences path",
            prefs_ok,
            prefs_path.display().to_string(),
        );

        let log_path = Self::log_file_path();
        let log_parent = log_path.parent().map(|p| p.to_path_buf());
        let log_ok = log_parent
            .as_ref()
            .map(|p| std::fs::create_dir_all(p).is_ok())
            .unwrap_or(false);
        self.push_check("Log path", log_ok, log_path.display().to_string());

        let mcp_port_ok = parse_port(&self.ui.mcp_port_text, "MCP port").is_ok();
        if mcp_port_ok {
            let port = self.ui.mcp_port_text.trim().parse::<u16>().unwrap_or(0);
            let bind_ok = if self.ui.mcp_running {
                true
            } else {
                TcpListener::bind(("127.0.0.1", port)).is_ok()
            };
            let detail = if self.ui.mcp_running {
                format!("127.0.0.1:{} (running)", port)
            } else {
                format!("127.0.0.1:{}", port)
            };
            self.push_check("MCP port available", bind_ok, detail);
        } else {
            self.push_check("MCP port available", false, "invalid mcp port");
        }

        let llm_ok = !self.ui.llm_api_url.trim().is_empty();
        self.push_check("LLM API URL", llm_ok, self.ui.llm_api_url.clone());

        let serial_ok = !self.available_ports.is_empty();
        self.push_check(
            "Serial ports",
            serial_ok,
            format!("{} ports detected", self.available_ports.len()),
        );

        info!(
            target: "self_check",
            checks = self.system_checks.len(),
            ok = self.system_checks.iter().filter(|c| c.ok).count(),
            "system_check_completed"
        );
    }

    pub fn update_doc_url(&self) -> String {
        if !self.update_notes_url.trim().is_empty() {
            return self.update_notes_url.trim().to_string();
        }
        if let Ok(url) = std::env::var("ROBOT_CONTROL_UPDATE_URL") {
            let trimmed = url.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                return trimmed.to_string();
            }
        }
        DEFAULT_UPDATE_DOC_URL.to_string()
    }

    pub fn update_manifest_url(&self) -> String {
        let configured = self.ui.update_manifest_url.trim();
        if configured.starts_with("http://") || configured.starts_with("https://") {
            return configured.to_string();
        }
        if let Ok(url) = std::env::var("ROBOT_CONTROL_UPDATE_MANIFEST_URL") {
            let trimmed = url.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                return trimmed.to_string();
            }
        }
        DEFAULT_UPDATE_MANIFEST_URL.to_string()
    }

    fn pre_1x_hint(current: VersionTriplet, latest: VersionTriplet) -> &'static str {
        if current.major == 0 && latest.major == 0 {
            if latest.minor > current.minor {
                "pre-1.0 minor upgrade (feature/breaking-ready)"
            } else if latest.patch > current.patch {
                "pre-1.0 patch upgrade (bugfix)"
            } else {
                "pre-1.0 same-version"
            }
        } else {
            "standard semver"
        }
    }

    fn fetch_update_manifest(&self, manifest_url: &str) -> Result<UpdateManifest, String> {
        let timeout = self.ui.update_check_timeout_ms.clamp(500, 10_000) as u64;
        let response = ureq::get(manifest_url)
            .timeout(Duration::from_millis(timeout))
            .call()
            .map_err(|e| e.to_string())?;
        let text = response.into_string().map_err(|e| e.to_string())?;
        serde_json::from_str::<UpdateManifest>(&text).map_err(|e| e.to_string())
    }

    fn evaluate_update_manifest(
        &self,
        manifest: &UpdateManifest,
    ) -> Result<(bool, String, String, String), String> {
        let latest_raw = manifest.latest_version.trim();
        if latest_raw.is_empty() {
            return Err("manifest.latest_version is empty".into());
        }
        let current = parse_version_triplet(self.build_version)
            .ok_or_else(|| format!("invalid current version: {}", self.build_version))?;
        let latest = parse_version_triplet(latest_raw)
            .ok_or_else(|| format!("invalid latest version in manifest: {}", latest_raw))?;

        let configured_channel = self.ui.update_channel.trim();
        let manifest_channel = manifest.channel.trim();
        if !configured_channel.is_empty()
            && configured_channel != "all"
            && !manifest_channel.is_empty()
            && configured_channel != manifest_channel
        {
            return Ok((
                false,
                format!(
                    "Channel mismatch (current={}, manifest={})",
                    configured_channel, manifest_channel
                ),
                self.update_doc_url(),
                latest_raw.to_string(),
            ));
        }

        if !manifest.min_supported_version.trim().is_empty() {
            if let Some(min_supported) = parse_version_triplet(&manifest.min_supported_version) {
                if current < min_supported {
                    let url = if manifest.notes_url.trim().is_empty() {
                        self.update_doc_url()
                    } else {
                        manifest.notes_url.trim().to_string()
                    };
                    return Ok((
                        true,
                        format!(
                            "Current version {} is below minimum supported {}",
                            self.build_version, manifest.min_supported_version
                        ),
                        url,
                        latest_raw.to_string(),
                    ));
                }
            }
        }

        let available = latest > current;
        let hint = Self::pre_1x_hint(current, latest);
        let detail = if available {
            format!(
                "Update available: {} -> {} ({})",
                self.build_version, latest_raw, hint
            )
        } else {
            format!("Already latest: {} ({})", self.build_version, hint)
        };
        let url = if manifest.notes_url.trim().is_empty() {
            self.update_doc_url()
        } else {
            manifest.notes_url.trim().to_string()
        };
        Ok((available, detail, url, latest_raw.to_string()))
    }

    pub fn update_status_summary(&self) -> String {
        format!(
            "Current {} | Latest {} | Channel {}",
            self.build_version, self.update_latest_version, self.ui.update_channel
        )
    }

    pub fn trigger_update_check(&mut self) -> String {
        self.update_last_checked_at = chrono::Local::now().format("%H:%M:%S").to_string();
        let manifest_url = self.update_manifest_url();
        let fallback_url = self.update_doc_url();

        match self.fetch_update_manifest(&manifest_url) {
            Ok(manifest) => match self.evaluate_update_manifest(&manifest) {
                Ok((available, detail, target_url, latest_version)) => {
                    self.update_available = available;
                    self.update_latest_version = latest_version;
                    self.update_status_detail = detail.clone();
                    self.update_notes_url = target_url.clone();
                    self.status_message = detail.clone();
                    self.add_info_log(&format!("ℹ {}", detail));
                    info!(
                        target: "app",
                        url = %target_url,
                        version = self.build_version,
                        latest = %self.update_latest_version,
                        available = self.update_available,
                        "update_check_completed"
                    );
                    target_url
                }
                Err(e) => {
                    self.update_available = false;
                    self.update_status_detail = format!("Update check failed to evaluate: {}", e);
                    self.status_message = self.update_status_detail.clone();
                    self.add_info_log(&format!("⚠ {}", self.update_status_detail));
                    warn!(target: "app", error = %e, "update_check_evaluate_failed");
                    fallback_url
                }
            },
            Err(e) => {
                self.update_available = false;
                self.update_status_detail = format!(
                    "Update check fallback to docs (manifest unavailable): {}",
                    e
                );
                self.status_message = self.update_status_detail.clone();
                self.add_info_log(&format!("⚠ {}", self.update_status_detail));
                warn!(target: "app", error = %e, url = %manifest_url, "update_manifest_fetch_failed");
                fallback_url
            }
        }
    }

    pub fn system_check_summary(&self) -> (usize, usize) {
        let total = self.system_checks.len();
        let ok = self.system_checks.iter().filter(|c| c.ok).count();
        (ok, total)
    }

    pub fn mcp_metrics_snapshot(&self) -> (u64, u64) {
        if let Ok(mcp) = self.mcp_shared_state.lock() {
            (mcp.request_count, mcp.unauthorized_count)
        } else {
            (0, 0)
        }
    }

    fn to_user_preferences(&self) -> UserPreferences {
        let active_tab_idx = ActiveTab::all()
            .iter()
            .position(|t| *t == self.active_tab)
            .unwrap_or(0);
        UserPreferences {
            schema_version: 2,
            language: self.language,
            dark_mode: self.dark_mode,
            sidebar_expanded: self.ui.sidebar_expanded,
            motion_level_idx: self.ui.motion_level_idx.min(3),
            active_tab_idx,
            parser_auto_parse: self.ui.parser_auto_parse,
            display_mode: self.ui.display_mode,
            llm_api_url: self.ui.llm_api_url.clone(),
            llm_model_name: self.ui.llm_model_name.clone(),
            mcp_port_text: self.ui.mcp_port_text.clone(),
            mcp_token_text: self.ui.mcp_token_text.clone(),
            active_conn: self.active_conn,
            serial_config: self.serial.config.clone(),
            tcp_host: self.ui.tcp_host.clone(),
            tcp_port_text: self.ui.tcp_port_text.clone(),
            tcp_is_server: self.ui.tcp_is_server,
            udp_local_port_text: self.ui.udp_local_port_text.clone(),
            udp_remote_host: self.ui.udp_remote_host.clone(),
            udp_remote_port_text: self.ui.udp_remote_port_text.clone(),
            auto_newline: self.ui.auto_newline,
            auto_reconnect_enabled: self.ui.auto_reconnect_enabled,
            auto_reconnect_interval_ms: self.ui.auto_reconnect_interval_ms,
            quick_cmd_1: self.ui.quick_cmd_1.clone(),
            quick_cmd_2: self.ui.quick_cmd_2.clone(),
            quick_cmd_3: self.ui.quick_cmd_3.clone(),
            send_hex: self.ui.send_hex,
            auto_scroll: self.ui.auto_scroll,
            send_with_newline: self.ui.send_with_newline,
            newline_type: self.ui.newline_type.clone(),
            repeat_send: self.ui.repeat_send,
            repeat_interval_ms: self.ui.repeat_interval_ms,
            can_id_text: self.ui.can_id_text.clone(),
            can_data_text: self.ui.can_data_text.clone(),
            can_extended: self.ui.can_extended,
            can_fd: self.ui.can_fd,
            can_bitrate_idx: self.ui.can_bitrate_idx,
            can_data_bitrate_idx: self.ui.can_data_bitrate_idx,
            can_sample_point_idx: self.ui.can_sample_point_idx,
            can_data_sample_point_idx: self.ui.can_data_sample_point_idx,
            can_sjw_idx: self.ui.can_sjw_idx,
            can_data_sjw_idx: self.ui.can_data_sjw_idx,
            usb_protocol_idx: self.ui.usb_protocol_idx,
            usb_speed_idx: self.ui.usb_speed_idx,
            usb_vid_text: self.ui.usb_vid_text.clone(),
            usb_pid_text: self.ui.usb_pid_text.clone(),
            packet_template_idx: self.ui.packet_template_idx,
            parser_enabled: self.ui.parser_enabled,
            parser_template_idx: self.ui.parser_template_idx,
            packet_builder_tab: self.ui.packet_builder_tab,
            analysis_protocol_idx: self.ui.analysis_protocol_idx,
            analysis_filter_tx: self.ui.analysis_filter_tx,
            analysis_filter_rx: self.ui.analysis_filter_rx,
            analysis_filter_info: self.ui.analysis_filter_info,
            llm_temperature_text: self.ui.llm_temperature_text.clone(),
            prefs_autosave_interval_sec: self.ui.prefs_autosave_interval_sec,
            update_channel: self.ui.update_channel.clone(),
            update_manifest_url: self.ui.update_manifest_url.clone(),
            update_check_timeout_ms: self.ui.update_check_timeout_ms,
        }
    }

    fn apply_user_preferences(&mut self, prefs: UserPreferences) {
        self.language = prefs.language;
        self.dark_mode = prefs.dark_mode;
        self.ui.sidebar_expanded = prefs.sidebar_expanded;
        self.ui.motion_level_idx = prefs.motion_level_idx.min(3);
        self.active_tab = *ActiveTab::all()
            .get(prefs.active_tab_idx)
            .unwrap_or(&ActiveTab::Dashboard);
        self.ui.parser_auto_parse = prefs.parser_auto_parse;
        self.ui.display_mode = prefs.display_mode;
        self.ui.llm_api_url = prefs.llm_api_url;
        self.ui.llm_model_name = prefs.llm_model_name;
        self.ui.mcp_port_text = prefs.mcp_port_text;
        self.ui.mcp_token_text = prefs.mcp_token_text;
        self.active_conn = prefs.active_conn;
        self.ui.conn_type_idx = ConnectionType::all()
            .iter()
            .position(|c| *c == self.active_conn)
            .unwrap_or(0);
        self.serial.config = prefs.serial_config;
        self.ui.tcp_host = prefs.tcp_host;
        self.ui.tcp_port_text = prefs.tcp_port_text;
        self.ui.tcp_is_server = prefs.tcp_is_server;
        self.ui.udp_local_port_text = prefs.udp_local_port_text;
        self.ui.udp_remote_host = prefs.udp_remote_host;
        self.ui.udp_remote_port_text = prefs.udp_remote_port_text;
        self.ui.auto_newline = prefs.auto_newline;
        self.ui.auto_reconnect_enabled = prefs.auto_reconnect_enabled;
        self.ui.auto_reconnect_interval_ms = prefs.auto_reconnect_interval_ms.clamp(500, 30000);
        self.ui.quick_cmd_1 = prefs.quick_cmd_1;
        self.ui.quick_cmd_2 = prefs.quick_cmd_2;
        self.ui.quick_cmd_3 = prefs.quick_cmd_3;
        self.ui.send_hex = prefs.send_hex;
        self.ui.auto_scroll = prefs.auto_scroll;
        self.ui.send_with_newline = prefs.send_with_newline;
        self.ui.newline_type = prefs.newline_type;
        self.ui.repeat_send = prefs.repeat_send;
        self.ui.repeat_interval_ms = prefs.repeat_interval_ms.clamp(50, 60_000);
        self.ui.can_id_text = prefs.can_id_text;
        self.ui.can_data_text = prefs.can_data_text;
        self.ui.can_extended = prefs.can_extended;
        self.ui.can_fd = prefs.can_fd;
        self.ui.can_bitrate_idx = prefs.can_bitrate_idx.min(8);
        self.ui.can_data_bitrate_idx = prefs.can_data_bitrate_idx.min(7);
        self.ui.can_sample_point_idx = prefs.can_sample_point_idx.min(5);
        self.ui.can_data_sample_point_idx = prefs.can_data_sample_point_idx.min(5);
        self.ui.can_sjw_idx = prefs.can_sjw_idx.min(3);
        self.ui.can_data_sjw_idx = prefs.can_data_sjw_idx.min(3);
        self.ui.usb_protocol_idx = prefs.usb_protocol_idx.min(11);
        self.ui.usb_speed_idx = prefs.usb_speed_idx.min(4);
        self.ui.usb_vid_text = prefs.usb_vid_text;
        self.ui.usb_pid_text = prefs.usb_pid_text;
        self.ui.packet_template_idx = prefs.packet_template_idx;
        self.ui.parser_enabled = prefs.parser_enabled;
        self.ui.parser_template_idx = prefs.parser_template_idx;
        self.ui.packet_builder_tab = prefs.packet_builder_tab.min(1);
        self.ui.analysis_protocol_idx = prefs.analysis_protocol_idx.min(7);
        self.ui.analysis_filter_tx = prefs.analysis_filter_tx;
        self.ui.analysis_filter_rx = prefs.analysis_filter_rx;
        self.ui.analysis_filter_info = prefs.analysis_filter_info;
        self.ui.llm_temperature_text = prefs.llm_temperature_text;
        self.ui.prefs_autosave_interval_sec = prefs.prefs_autosave_interval_sec.clamp(1, 300);
        self.ui.update_channel = prefs.update_channel;
        self.ui.update_manifest_url = prefs.update_manifest_url;
        self.ui.update_check_timeout_ms = prefs.update_check_timeout_ms.clamp(500, 10_000);
    }

    fn load_user_preferences_from_path(&mut self, path: &std::path::Path) {
        if let Ok(text) = std::fs::read_to_string(path) {
            match serde_json::from_str::<UserPreferences>(&text) {
                Ok(prefs) => {
                    self.apply_user_preferences(prefs);
                    self.status_message = "Preferences loaded".into();
                }
                Err(e) => {
                    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                    let mut backup = path.to_path_buf();
                    if let Some(ext) = path.extension() {
                        backup.set_extension(format!("{}.corrupt_{}", ext.to_string_lossy(), ts));
                    } else {
                        backup.set_extension(format!("corrupt_{}", ts));
                    }
                    let _ = std::fs::copy(path, &backup);
                    self.report_error(format!("Preferences parse failed: {}", e));
                }
            }
        }
    }

    pub fn load_user_preferences(&mut self) {
        let path = Self::user_prefs_path();
        self.load_user_preferences_from_path(&path);
    }

    fn save_user_preferences_to_path(&mut self, path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&self.to_user_preferences()) {
            Ok(text) => {
                let mut tmp_path = path.to_path_buf();
                tmp_path.set_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp_path, text) {
                    self.report_error(format!("Preferences save failed: {}", e));
                    return;
                }

                if path.exists() {
                    let mut bak_path = path.to_path_buf();
                    bak_path.set_extension("json.bak");
                    let _ = std::fs::copy(path, bak_path);
                    let _ = std::fs::remove_file(path);
                }

                if let Err(e) = std::fs::rename(&tmp_path, path) {
                    self.report_error(format!("Preferences commit failed: {}", e));
                    let _ = std::fs::remove_file(&tmp_path);
                }
            }
            Err(e) => {
                self.report_error(format!("Preferences serialize failed: {}", e));
            }
        }
    }

    pub fn save_user_preferences(&mut self) {
        let path = Self::user_prefs_path();
        self.save_user_preferences_to_path(&path);
    }

    pub fn save_user_preferences_as<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.save_user_preferences_to_path(path.as_ref());
    }

    pub fn load_user_preferences_from<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.load_user_preferences_from_path(path.as_ref());
    }

    pub fn reset_user_preferences(&mut self) {
        let defaults = UserPreferences::default();
        self.apply_user_preferences(defaults);
        self.status_message = "Preferences reset to defaults".into();
    }

    /// Shorthand for current language
    pub fn lang(&self) -> Language {
        self.language
    }

    pub fn refresh_ports(&mut self) {
        if self.port_scan_in_progress {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.port_scan_in_progress = true;
        self.port_scan_rx = Some(rx);
        self.status_message = "Scanning serial ports...".into();

        std::thread::spawn(move || {
            let ports = SerialService::scan_ports();
            let _ = tx.send(ports);
        });
    }

    fn apply_scanned_ports(&mut self, ports: Vec<String>) {
        let previous = self.serial.config.port_name.clone();
        self.available_ports = ports;

        if self.available_ports.is_empty() {
            self.serial.config.port_name.clear();
            self.status_message = "No serial ports found".into();
            return;
        }

        if !previous.is_empty() && self.available_ports.iter().any(|p| p == &previous) {
            self.serial.config.port_name = previous;
            self.status_message = format!(
                "Serial ports refreshed: {} detected",
                self.available_ports.len()
            );
            return;
        }

        self.serial.config.port_name = self.available_ports[0].clone();
        self.status_message = format!(
            "Serial port auto-selected: {}",
            self.serial.config.port_name
        );
    }

    pub fn total_bytes_sent(&self) -> u64 {
        self.serial.bytes_sent + self.tcp.bytes_sent + self.udp.bytes_sent
    }

    pub fn total_bytes_received(&self) -> u64 {
        self.serial.bytes_received + self.tcp.bytes_received + self.udp.bytes_received
    }

    pub fn total_errors(&self) -> u64 {
        self.serial.error_count + self.tcp.error_count + self.udp.error_count
    }

    pub fn is_any_connected(&self) -> bool {
        self.serial.is_connected()
            || self.tcp.is_connected()
            || self.udp.is_connected()
            || self.can.is_running
    }

    pub fn active_status(&self) -> ConnectionStatus {
        match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb => self.serial.status,
            ConnectionType::Tcp | ConnectionType::ModbusTcp => self.tcp.status,
            ConnectionType::Udp => self.udp.status,
            ConnectionType::Can | ConnectionType::CanFd => {
                if self.can.is_running {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                }
            }
            ConnectionType::ModbusRtu => self.serial.status,
        }
    }

    pub fn last_comm(&self) -> &str {
        match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb => &self.serial.last_comm,
            ConnectionType::Tcp | ConnectionType::ModbusTcp => &self.tcp.last_comm,
            ConnectionType::Udp => &self.udp.last_comm,
            ConnectionType::Can | ConnectionType::CanFd => {
                if self.can.frames.is_empty() {
                    "No CAN frame yet"
                } else {
                    "CAN bus active"
                }
            }
            ConnectionType::ModbusRtu => &self.serial.last_comm,
        }
    }

    pub fn link_health_text(&self) -> String {
        let status = self.active_status();
        if status == ConnectionStatus::Connecting {
            return "Connecting".into();
        }
        if status == ConnectionStatus::Error {
            return "Error".into();
        }

        if !status.is_connected() {
            if self.reconnect_paused_by_user {
                return "Offline (manual)".into();
            }
            return "Offline".into();
        }

        if let Some(last_rx) = self.last_rx_instant {
            let elapsed = last_rx.elapsed().as_secs_f32();
            if elapsed < 1.0 {
                "Live".into()
            } else if elapsed < 3.0 {
                format!("Good {:.1}s", elapsed)
            } else if elapsed < 10.0 {
                format!("Idle {:.1}s", elapsed)
            } else {
                format!("Stale {:.1}s", elapsed)
            }
        } else {
            "Connected (no RX yet)".into()
        }
    }

    pub fn maintain_connection(&mut self) {
        if !self.ui.auto_reconnect_enabled || self.reconnect_paused_by_user {
            self.next_reconnect_at = None;
            return;
        }

        if self.serial_connect_in_progress
            && matches!(
                self.active_conn,
                ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu
            )
        {
            return;
        }

        let supported = matches!(
            self.active_conn,
            ConnectionType::Serial
                | ConnectionType::Usb
                | ConnectionType::ModbusRtu
                | ConnectionType::Tcp
                | ConnectionType::ModbusTcp
                | ConnectionType::Udp
        );
        if !supported {
            return;
        }

        if self.active_status().is_connected() {
            self.next_reconnect_at = None;
            return;
        }

        let now = Instant::now();
        let interval_ms = self.ui.auto_reconnect_interval_ms.clamp(500, 30000) as u64;
        if self.next_reconnect_at.is_some_and(|next| next > now) {
            return;
        }

        self.next_reconnect_at = Some(now + Duration::from_millis(interval_ms));
        let _ = self.connect_active();
    }

    pub fn reconnect_paused(&self) -> bool {
        self.reconnect_paused_by_user
    }

    pub fn resume_auto_reconnect(&mut self) {
        self.reconnect_paused_by_user = false;
        self.next_reconnect_at = Some(Instant::now());
        self.status_message = "Auto reconnect resumed".into();
        self.add_info_log("Auto reconnect resumed");
    }

    // ─── 连接操作 ────────────────────────────────────────

    pub fn connect_active(&mut self) -> Result<(), String> {
        self.reconnect_paused_by_user = false;
        self.metrics.connect_attempts += 1;
        let result = match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                if self.serial.config.port_name.trim().is_empty() {
                    if self.port_scan_in_progress {
                        return Err("Serial port scan in progress, please wait...".into());
                    }
                    self.refresh_ports();
                    return Err(
                        "No serial port selected. Scanning started, please retry in a moment."
                            .into(),
                    );
                }

                if !self.available_ports.is_empty()
                    && !self
                        .available_ports
                        .iter()
                        .any(|p| p == &self.serial.config.port_name)
                {
                    self.refresh_ports();
                    return Err(
                        "Selected serial port is no longer available. Port scan started.".into(),
                    );
                }

                self.start_serial_connect_worker()
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => {
                self.tcp.host = self.ui.tcp_host.trim().to_string();
                if self.tcp.host.is_empty() {
                    return Err("TCP host required".into());
                }
                self.tcp.port = parse_port(&self.ui.tcp_port_text, "TCP port")?;
                self.tcp.is_server = self.ui.tcp_is_server;
                if self.ui.tcp_is_server {
                    self.tcp.start_server().map_err(|e| e.to_string())
                } else {
                    self.tcp.connect_client().map_err(|e| e.to_string())
                }
            }
            ConnectionType::Udp => {
                self.udp.local_port = parse_port(&self.ui.udp_local_port_text, "UDP local port")?;
                self.udp.remote_addr = self.ui.udp_remote_host.trim().to_string();
                self.udp.remote_port =
                    parse_port(&self.ui.udp_remote_port_text, "UDP remote port")?;
                if self.udp.remote_addr.is_empty() {
                    return Err("UDP remote host required".into());
                }
                self.udp.bind().map_err(|e| e.to_string())
            }
            ConnectionType::Can | ConnectionType::CanFd => {
                self.can.is_running = true;
                Ok(())
            }
        };

        let is_serial_mode = matches!(
            self.active_conn,
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu
        );

        match &result {
            Ok(_) if is_serial_mode => {
                self.status_message = format!("Connecting: {}", self.active_conn);
            }
            Ok(_) => {
                self.next_reconnect_at = None;
                info!(target: "connection", connection = %self.active_conn, "connect_success");
                self.add_info_log(&format!("Connected: {}", self.active_conn));
            }
            Err(e) => {
                self.metrics.connect_failures += 1;
                self.report_error(format!("Connect failed ({}): {}", self.active_conn, e));
            }
        }
        result
    }

    fn start_serial_connect_worker(&mut self) -> Result<(), String> {
        if self.serial_connect_in_progress {
            return Err("Serial connect already in progress".into());
        }

        let cfg = self.serial.config.clone();
        if cfg.port_name.trim().is_empty() {
            return Err("No serial port selected".into());
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.serial_connect_rx = Some(rx);
        self.serial_connect_in_progress = true;
        self.serial.status = ConnectionStatus::Connecting;

        thread::spawn(move || {
            let mut svc = SerialService::new();
            svc.config = cfg;
            let result = svc.connect().map(|_| svc).map_err(|e| e.to_string());
            let _ = tx.send(result);
        });

        Ok(())
    }

    pub fn disconnect_active(&mut self) {
        self.reconnect_paused_by_user = true;
        self.next_reconnect_at = None;
        self.serial_connect_in_progress = false;
        self.serial_connect_rx = None;
        match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                self.serial.disconnect()
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => self.tcp.disconnect(),
            ConnectionType::Udp => self.udp.close(),
            ConnectionType::Can | ConnectionType::CanFd => {
                self.can.is_running = false;
            }
        }
    }

    // ─── 发送数据 ────────────────────────────────────────

    pub fn send_data(&mut self, data: &[u8]) -> Result<(), String> {
        let result = match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                self.serial.send_data(data).map_err(|e| e.to_string())
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => {
                self.tcp.send_data(data).map_err(|e| e.to_string())
            }
            ConnectionType::Udp => self.udp.send_default(data).map_err(|e| e.to_string()),
            _ => Err("Channel not supported".into()),
        };

        if result.is_ok() {
            self.add_log(LogDirection::Tx, data, &self.active_conn.to_string());
        } else if let Err(e) = &result {
            self.report_error(format!("Send failed ({}): {}", self.active_conn, e));
        }
        result
    }

    // ─── 轮询数据 ────────────────────────────────────────

    pub fn poll_data(&mut self) {
        if self.serial.is_connected() {
            let raw = self.serial.try_read_raw();
            if !raw.is_empty() {
                self.last_rx_instant = Some(Instant::now());
                self.add_log(LogDirection::Rx, &raw, "Serial");
                self.serial.push_rx_data(&raw);
            }
        }

        if self.tcp.is_connected() {
            let data = self.tcp.try_read();
            if !data.is_empty() {
                self.last_rx_instant = Some(Instant::now());
                self.add_log(LogDirection::Rx, &data, "TCP");
            }
        }

        if self.udp.is_connected() {
            let data = self.udp.try_read();
            if !data.is_empty() {
                self.last_rx_instant = Some(Instant::now());
                self.add_log(LogDirection::Rx, &data, "UDP");
            }
        }

        if self.serial.is_connected() {
            while let Some(state) = self.serial.try_parse_state_from_buffer() {
                let mut s = state;
                if self.is_running {
                    let output = self.compute_active_algorithm(s.position, s.velocity);
                    s.pid_output = output;
                    s.error = self.get_active_setpoint() - s.position;
                    let _ = self.serial.send_position_control(output);
                }
                self.current_state = s.clone();
                self.state_history.push(s);
                if self.state_history.len() > MAX_HISTORY {
                    self.state_history
                        .drain(..self.state_history.len() - MAX_HISTORY);
                }
            }
        }

        self.sync_mcp_state();
    }

    fn add_log(&mut self, dir: LogDirection, data: &[u8], channel: &str) {
        let entry = LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction: dir,
            data: data.to_vec(),
            display_mode: self.ui.display_mode,
            channel: channel.into(),
        };
        self.log_entries.push(entry.clone());
        if self.log_entries.len() > MAX_LOG {
            self.log_entries.drain(..self.log_entries.len() - MAX_LOG);
        }
        self.append_log(&entry);
    }

    pub fn add_info_log(&mut self, msg: &str) {
        let entry = LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction: LogDirection::Info,
            data: msg.as_bytes().to_vec(),
            display_mode: DisplayMode::Ascii,
            channel: "System".into(),
        };
        self.log_entries.push(entry.clone());
        if self.log_entries.len() > MAX_LOG {
            self.log_entries.drain(..self.log_entries.len() - MAX_LOG);
        }
        self.append_log(&entry);
    }

    // ─── 控制操作 ────────────────────────────────────────

    pub fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
        if self.is_running {
            self.reset_active_algorithm();
            self.status_message = "Control started".into();
        } else {
            self.status_message = "Control stopped".into();
        }
    }

    pub fn emergency_stop(&mut self) {
        self.is_running = false;
        if self.serial.is_connected() {
            let _ = self.serial.send_emergency_stop();
        }
        self.status_message = "EMERGENCY STOP!".into();
        self.add_info_log("\u{26A0} Emergency Stop activated!");
    }

    // ─── 控制算法分发 ────────────────────────────────────

    /// 调用当前激活的控制算法进行计算
    pub fn compute_active_algorithm(&mut self, position: f64, velocity: f64) -> f64 {
        match self.active_algorithm {
            ControlAlgorithmType::ClassicPid => self.pid.compute(position),
            ControlAlgorithmType::IncrementalPid => self.incremental_pid.compute(position),
            ControlAlgorithmType::BangBang => self.bang_bang.compute(position),
            ControlAlgorithmType::FuzzyPid => self.fuzzy_pid.compute(position),
            ControlAlgorithmType::CascadePid => self.cascade_pid.compute(position, velocity),
            ControlAlgorithmType::SmithPredictor => self.smith_predictor.compute(position),
            ControlAlgorithmType::Adrc => self.adrc.compute(position),
            ControlAlgorithmType::Ladrc => self.ladrc.compute(position),
            ControlAlgorithmType::Lqr => self.lqr.compute(position),
            ControlAlgorithmType::Mpc => self.mpc.compute(position),
        }
    }

    /// 获取当前算法的设定值
    pub fn get_active_setpoint(&self) -> f64 {
        match self.active_algorithm {
            ControlAlgorithmType::ClassicPid => self.pid.setpoint,
            ControlAlgorithmType::IncrementalPid => self.incremental_pid.setpoint,
            ControlAlgorithmType::BangBang => self.bang_bang.setpoint,
            ControlAlgorithmType::FuzzyPid => self.fuzzy_pid.setpoint,
            ControlAlgorithmType::CascadePid => self.cascade_pid.setpoint,
            ControlAlgorithmType::SmithPredictor => self.smith_predictor.setpoint,
            ControlAlgorithmType::Adrc => self.adrc.setpoint,
            ControlAlgorithmType::Ladrc => self.ladrc.setpoint,
            ControlAlgorithmType::Lqr => self.lqr.setpoint,
            ControlAlgorithmType::Mpc => self.mpc.setpoint,
        }
    }

    /// 重置当前激活的控制算法
    pub fn reset_active_algorithm(&mut self) {
        match self.active_algorithm {
            ControlAlgorithmType::ClassicPid => self.pid.reset(),
            ControlAlgorithmType::IncrementalPid => self.incremental_pid.reset(),
            ControlAlgorithmType::BangBang => self.bang_bang.reset(),
            ControlAlgorithmType::FuzzyPid => self.fuzzy_pid.reset(),
            ControlAlgorithmType::CascadePid => self.cascade_pid.reset(),
            ControlAlgorithmType::SmithPredictor => self.smith_predictor.reset(),
            ControlAlgorithmType::Adrc => self.adrc.reset(),
            ControlAlgorithmType::Ladrc => self.ladrc.reset(),
            ControlAlgorithmType::Lqr => self.lqr.reset(),
            ControlAlgorithmType::Mpc => self.mpc.reset(),
        }
    }

    /// 获取当前算法的输出值
    pub fn get_active_output(&self) -> f64 {
        match self.active_algorithm {
            ControlAlgorithmType::ClassicPid => self.pid.output,
            ControlAlgorithmType::IncrementalPid => self.incremental_pid.output,
            ControlAlgorithmType::BangBang => self.bang_bang.output,
            ControlAlgorithmType::FuzzyPid => self.fuzzy_pid.output,
            ControlAlgorithmType::CascadePid => self.cascade_pid.output,
            ControlAlgorithmType::SmithPredictor => self.smith_predictor.output,
            ControlAlgorithmType::Adrc => self.adrc.output,
            ControlAlgorithmType::Ladrc => self.ladrc.output,
            ControlAlgorithmType::Lqr => self.lqr.output,
            ControlAlgorithmType::Mpc => self.mpc.output,
        }
    }

    // ─── NN 调参 ──────────────────────────────────────────

    pub fn nn_suggest_params(&mut self) {
        let errors: Vec<f64> = self.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 10 {
            return;
        }
        let features = NeuralNetwork::extract_features(&errors);
        let output = self.nn.forward(&features);
        self.nn_suggested_kp = output[0] * 5.0;
        self.nn_suggested_ki = output[1] * 2.0;
        self.nn_suggested_kd = output[2] * 1.0;
    }

    pub fn nn_train_step(&mut self) {
        let errors: Vec<f64> = self.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 20 {
            return;
        }
        let features = NeuralNetwork::extract_features(&errors);
        let performance =
            1.0 / (1.0 + errors.iter().map(|e| e.abs()).sum::<f64>() / errors.len() as f64);
        let target = vec![
            (self.pid.kp / 5.0).clamp(0.0, 1.0) * performance,
            (self.pid.ki / 2.0).clamp(0.0, 1.0) * performance,
            (self.pid.kd / 1.0).clamp(0.0, 1.0) * performance,
        ];
        let loss = self.nn.train_step(&features, &target);
        self.status_message = format!(
            "NN Training - Loss: {:.6}, Epoch: {}",
            loss, self.nn.training_epochs
        );
    }

    pub fn apply_nn_params(&mut self) {
        self.pid.kp = self.nn_suggested_kp;
        self.pid.ki = self.nn_suggested_ki;
        self.pid.kd = self.nn_suggested_kd;
        self.ui.kp_text = format!("{:.3}", self.pid.kp);
        self.ui.ki_text = format!("{:.3}", self.pid.ki);
        self.ui.kd_text = format!("{:.3}", self.pid.kd);
        self.status_message = "Applied NN suggested parameters".into();
    }

    // ─── 解析数据联动可视化 ──────────────────────────────

    /// 将解析出的数据包字段推送到对应的可视化通道缓冲区
    pub fn feed_parsed_to_channels(&mut self, parsed: &ParsedPacket) {
        let mut dropped_total = 0usize;
        for (i, ch) in self.data_channels.iter().enumerate() {
            if !ch.enabled {
                continue;
            }
            if let DataSource::PacketField {
                ref template_name,
                ref field_name,
            } = ch.source
            {
                if *template_name == parsed.template_name {
                    if let Some(val) = parsed.field_value(field_name) {
                        while self.channel_buffers.len() <= i {
                            self.channel_buffers.push(TimeSeriesBuffer::default());
                        }
                        let dropped = self.channel_buffers[i].push_with_overflow(val);
                        if dropped > 0 {
                            dropped_total += dropped;
                        }
                    }
                }
            }
        }
        if dropped_total > 0 {
            self.report_channel_overflow(dropped_total);
        }
    }

    /// 从解析结果的字段快速创建可视化通道
    pub fn add_channel_from_parsed_field(
        &mut self,
        template_name: &str,
        field_name: &str,
        viz_type: crate::models::VizType,
    ) {
        if let Some((idx, _)) = self.data_channels.iter().enumerate().find(|(_, ch)| {
            matches!(
                &ch.source,
                DataSource::PacketField { template_name: t, field_name: f }
                    if t == template_name && f == field_name
            )
        }) {
            self.data_channels[idx].enabled = true;
            self.status_message = format!(
                "Packet field channel already exists: {}/{}",
                template_name, field_name
            );
            return;
        }

        let colors = [
            [65, 155, 255],
            [255, 165, 0],
            [255, 100, 100],
            [255, 100, 255],
            [255, 50, 50],
            [100, 255, 100],
            [200, 200, 50],
            [100, 200, 200],
            [180, 100, 255],
            [255, 200, 100],
            [100, 200, 100],
            [200, 150, 80],
        ];
        let c = colors[self.data_channels.len() % colors.len()];
        let name = format!("{}/{}", template_name, field_name);
        let ch = DataChannel::new(
            &name,
            DataSource::PacketField {
                template_name: template_name.into(),
                field_name: field_name.into(),
            },
            viz_type,
            c,
        );
        self.data_channels.push(ch);
        self.channel_buffers.push(TimeSeriesBuffer::default());

        // 回填已有的解析结果
        let buf_idx = self.channel_buffers.len() - 1;
        let mut dropped_total = 0usize;
        for pkt in &self.parsed_packets {
            if pkt.template_name == template_name {
                if let Some(val) = pkt.field_value(field_name) {
                    let dropped = self.channel_buffers[buf_idx].push_with_overflow(val);
                    if dropped > 0 {
                        dropped_total += dropped;
                    }
                }
            }
        }
        if dropped_total > 0 {
            self.report_channel_overflow(dropped_total);
        }
    }

    /// 获取已解析数据包中所有可用的 (template_name, field_name) 对
    pub fn available_packet_fields(&self) -> Vec<(String, String)> {
        let mut fields = Vec::new();
        for pkt in &self.parsed_packets {
            for f in &pkt.fields {
                if f.value_f64.is_some() {
                    let pair = (pkt.template_name.clone(), f.name.clone());
                    if !fields.contains(&pair) {
                        fields.push(pair);
                    }
                }
            }
        }
        fields
    }

    // ─── LLM 智能调参 ────────────────────────────────────

    /// 使用 LLM API 获取调参建议
    pub fn llm_suggest_params(&mut self) {
        use crate::services::llm_service::LlmService;
        if self.ui.llm_loading {
            self.status_message = "LLM request is already running".into();
            return;
        }
        let errors: Vec<f64> = self.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 10 {
            self.report_error("Need at least 10 data points for LLM analysis");
            return;
        }

        let api_key = if self.ui.llm_api_key.trim().is_empty() {
            std::env::var("LLM_API_KEY").unwrap_or_default()
        } else {
            self.ui.llm_api_key.clone()
        };

        if api_key.trim().is_empty() {
            self.report_error("LLM API key is empty (and LLM_API_KEY env not set)");
            return;
        }

        let api_url = self.ui.llm_api_url.clone();
        let model = self.ui.llm_model_name.clone();
        let current_params = crate::services::llm_service::PidParams {
            kp: self.pid.kp,
            ki: self.pid.ki,
            kd: self.pid.kd,
            setpoint: self.pid.setpoint,
        };

        self.ui.llm_loading = true;
        self.metrics.llm_requests += 1;
        self.ui.llm_last_response = "Requesting LLM...".into();
        self.status_message = "LLM request started".into();
        info!(target: "llm", model = %model, api_url = %api_url, "llm_request_started");

        let (tx, rx) = std::sync::mpsc::channel();
        self.llm_result_rx = Some(rx);

        std::thread::spawn(move || {
            let llm = LlmService::new(api_url, api_key, model);
            let result = llm.suggest_pid_params(&current_params, &errors);
            let _ = tx.send(result);
        });
    }

    pub fn poll_background_tasks(&mut self) {
        self.flush_pending_logs();

        if let Some(rx) = self.serial_connect_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.serial_connect_in_progress = false;
                    match result {
                        Ok(serial) => {
                            self.serial = serial;
                            self.next_reconnect_at = None;
                            info!(target: "connection", connection = %self.active_conn, "connect_success");
                            self.add_info_log(&format!("Connected: {}", self.active_conn));
                        }
                        Err(e) => {
                            self.serial.status = ConnectionStatus::Error;
                            self.metrics.connect_failures += 1;
                            self.report_error(format!(
                                "Connect failed ({}): {}",
                                self.active_conn, e
                            ));
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    self.serial_connect_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.serial_connect_in_progress = false;
                    self.serial.status = ConnectionStatus::Error;
                    self.report_error("Serial connect worker disconnected unexpectedly");
                }
            }
        }

        if let Some(rx) = self.port_scan_rx.take() {
            match rx.try_recv() {
                Ok(ports) => {
                    self.port_scan_in_progress = false;
                    self.apply_scanned_ports(ports);
                }
                Err(TryRecvError::Empty) => {
                    self.port_scan_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.port_scan_in_progress = false;
                    self.report_error("Serial port scan worker disconnected unexpectedly");
                }
            }
        }

        if let Some(rx) = self.llm_result_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.ui.llm_loading = false;
                    match result {
                        Ok(suggested) => {
                            self.metrics.llm_success += 1;
                            self.nn_suggested_kp = suggested.kp;
                            self.nn_suggested_ki = suggested.ki;
                            self.nn_suggested_kd = suggested.kd;
                            self.ui.llm_last_response = suggested.reasoning.clone();
                            self.status_message = format!(
                                "LLM suggested: Kp={:.4} Ki={:.4} Kd={:.4}",
                                suggested.kp, suggested.ki, suggested.kd
                            );
                            self.add_info_log("LLM suggestion completed");
                            info!(target: "llm", "llm_request_success");
                        }
                        Err(e) => {
                            self.metrics.llm_failures += 1;
                            self.ui.llm_last_response = format!("Error: {}", e);
                            self.report_error(format!("LLM error: {}", e));
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    self.llm_result_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.ui.llm_loading = false;
                    self.metrics.llm_failures += 1;
                    self.report_error("LLM worker disconnected unexpectedly");
                }
            }
        }
    }

    // ─── MCP 服务器 ─────────────────────────────────────

    pub fn start_mcp_server(&mut self) -> Result<(), String> {
        if self.ui.mcp_running {
            return Ok(());
        }
        let port = parse_port(&self.ui.mcp_port_text, "MCP port")?;
        let token = self.ui.mcp_token_text.trim().to_string();
        let token = if token.is_empty() { None } else { Some(token) };

        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        crate::services::mcp_server::McpServer::start(
            self.mcp_shared_state.clone(),
            port,
            token,
            running.clone(),
        )?;

        self.mcp_server_handle = Some(running);
        self.ui.mcp_running = true;
        self.metrics.mcp_startups += 1;
        self.status_message = format!("MCP server started on 0.0.0.0:{}", port);
        self.add_info_log(&self.status_message.clone());
        info!(target: "mcp", port = port, "mcp_server_started");
        Ok(())
    }

    pub fn stop_mcp_server(&mut self) {
        if let Some(running) = self.mcp_server_handle.take() {
            crate::services::mcp_server::McpServer::stop(running);
        }
        self.ui.mcp_running = false;
        self.status_message = "MCP server stopped".into();
        self.add_info_log("MCP server stopped");
        info!(target: "mcp", "mcp_server_stopped");
    }

    pub fn toggle_mcp_server(&mut self) {
        if self.ui.mcp_running {
            self.stop_mcp_server();
        } else {
            match self.start_mcp_server() {
                Ok(()) => {}
                Err(e) => {
                    self.status_message = format!("MCP start failed: {}", e);
                }
            }
        }
    }

    pub fn reset_counters(&mut self) {
        self.serial.bytes_sent = 0;
        self.serial.bytes_received = 0;
        self.serial.error_count = 0;
        self.tcp.bytes_sent = 0;
        self.tcp.bytes_received = 0;
        self.tcp.error_count = 0;
        self.udp.bytes_sent = 0;
        self.udp.bytes_received = 0;
        self.udp.error_count = 0;
    }

    pub fn sync_mcp_state(&mut self) {
        if self.can.dropped_frames > self.can_dropped_frames_seen {
            self.can_dropped_frames_seen = self.can.dropped_frames;
            self.report_error(format!(
                "CAN frame buffer reached limit; dropped {} frames",
                self.can.dropped_frames
            ));
        }

        if self.channel_overflow_events > self.channel_overflow_notified {
            self.channel_overflow_notified = self.channel_overflow_events;
            self.report_error(format!(
                "Data channel buffers reached limit; dropped {} points",
                self.channel_overflow_events
            ));
        }

        if !self.ui.mcp_running {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_mcp_sync_instant {
            if now.duration_since(last) < Duration::from_millis(120) {
                return;
            }
        }
        self.last_mcp_sync_instant = Some(now);

        if let Ok(mut mcp) = self.mcp_shared_state.lock() {
            mcp.kp = self.pid.kp;
            mcp.ki = self.pid.ki;
            mcp.kd = self.pid.kd;
            mcp.setpoint = self.pid.setpoint;
            mcp.current_state = self.current_state.clone();
            mcp.state_history = self
                .state_history
                .iter()
                .rev()
                .take(500)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            mcp.parsed_packets = self
                .parsed_packets
                .iter()
                .rev()
                .take(200)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            mcp.suggested_kp = self.nn_suggested_kp;
            mcp.suggested_ki = self.nn_suggested_ki;
            mcp.suggested_kd = self.nn_suggested_kd;
            mcp.status = self.status_message.clone();
        }
    }

    // ─── 图表数据 ────────────────────────────────────────

    pub fn chart_data(&self, f: impl Fn(&RobotState) -> f64) -> Vec<[f64; 2]> {
        let start = self.state_history.len().saturating_sub(200);
        self.state_history[start..]
            .iter()
            .enumerate()
            .map(|(i, s)| [i as f64, f(s)])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_port, parse_version_triplet, AppState, LOG_FILE_MAX_BYTES};
    use std::fs;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}", name, nanos))
    }

    #[test]
    fn test_parse_port_valid() {
        assert_eq!(parse_port("8080", "TCP port").unwrap(), 8080);
    }

    #[test]
    fn test_parse_port_invalid() {
        assert!(parse_port("0", "TCP port").is_err());
        assert!(parse_port("70000", "TCP port").is_err());
        assert!(parse_port("abc", "TCP port").is_err());
    }

    #[test]
    fn test_parse_version_triplet_supports_v_prefix_and_suffix() {
        let v = parse_version_triplet("v0.1.7-beta.1").expect("version parsed");
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 7);
        assert!(parse_version_triplet("0.1").is_none());
    }

    #[test]
    fn test_preferences_roundtrip_custom_path() {
        let path = unique_temp_file("prefs_roundtrip.json");

        let mut s1 = AppState::new();
        s1.ui.tcp_host = "192.168.1.10".into();
        s1.ui.tcp_port_text = "12345".into();
        s1.ui.mcp_token_text = "token-abc".into();
        s1.active_conn = crate::models::ConnectionType::Tcp;
        s1.save_user_preferences_to_path(&path);

        let mut s2 = AppState::new();
        s2.load_user_preferences_from_path(&path);

        assert_eq!(s2.ui.tcp_host, "192.168.1.10");
        assert_eq!(s2.ui.tcp_port_text, "12345");
        assert_eq!(s2.ui.mcp_token_text, "token-abc");
        assert_eq!(s2.active_conn, crate::models::ConnectionType::Tcp);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_preferences_roundtrip_extended_fields() {
        let path = unique_temp_file("prefs_extended_roundtrip.json");

        let mut s1 = AppState::new();
        s1.ui.motion_level_idx = 3;
        s1.ui.usb_protocol_idx = 9;
        s1.ui.usb_speed_idx = 4;
        s1.ui.can_bitrate_idx = 8;
        s1.ui.parser_auto_parse = false;
        s1.ui.analysis_protocol_idx = 7;
        s1.ui.analysis_filter_info = true;
        s1.ui.prefs_autosave_interval_sec = 9;
        s1.ui.update_channel = "preview-0.1".into();
        s1.ui.update_manifest_url = "https://example.com/manifest.json".into();
        s1.ui.update_check_timeout_ms = 2600;
        s1.save_user_preferences_as(&path);

        let mut s2 = AppState::new();
        s2.load_user_preferences_from(&path);

        assert_eq!(s2.ui.motion_level_idx, 3);
        assert_eq!(s2.ui.usb_protocol_idx, 9);
        assert_eq!(s2.ui.usb_speed_idx, 4);
        assert_eq!(s2.ui.can_bitrate_idx, 8);
        assert!(!s2.ui.parser_auto_parse);
        assert_eq!(s2.ui.analysis_protocol_idx, 7);
        assert!(s2.ui.analysis_filter_info);
        assert_eq!(s2.ui.prefs_autosave_interval_sec, 9);
        assert_eq!(s2.ui.update_channel, "preview-0.1");
        assert_eq!(
            s2.ui.update_manifest_url,
            "https://example.com/manifest.json"
        );
        assert_eq!(s2.ui.update_check_timeout_ms, 2600);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_reset_user_preferences_restores_defaults() {
        let mut s = AppState::new();
        s.dark_mode = false;
        s.ui.sidebar_expanded = false;
        s.ui.motion_level_idx = 3;
        s.ui.tcp_host = "10.0.0.8".into();

        s.reset_user_preferences();

        assert!(s.dark_mode);
        assert!(s.ui.sidebar_expanded);
        assert_eq!(s.ui.motion_level_idx, 2);
        assert_eq!(s.ui.tcp_host, "127.0.0.1");
    }

    #[test]
    fn test_can_status_reflects_running_state() {
        let mut s = AppState::new();
        s.active_conn = crate::models::ConnectionType::Can;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Disconnected
        );
        s.can.is_running = true;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connected
        );
        assert!(s.is_any_connected());
    }

    #[test]
    fn test_connect_disconnect_can_channel() {
        let mut s = AppState::new();
        s.active_conn = crate::models::ConnectionType::CanFd;
        s.connect_active().unwrap();
        assert!(s.can.is_running);
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connected
        );
        s.disconnect_active();
        assert!(!s.can.is_running);
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Disconnected
        );
    }

    #[test]
    fn test_mcp_system_check_when_running_uses_running_state() {
        let mut s = AppState::new();
        s.ui.mcp_port_text = "3000".into();
        s.ui.mcp_running = true;
        s.run_system_check();
        let check = s
            .system_checks
            .iter()
            .find(|c| c.name == "MCP port available")
            .expect("MCP check exists");
        assert!(check.ok);
        assert!(check.detail.contains("running"));
    }

    #[test]
    fn test_serial_status_mapping_and_link_health() {
        let mut s = AppState::new();
        s.active_conn = crate::models::ConnectionType::Serial;

        s.serial.status = crate::models::ConnectionStatus::Connecting;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connecting
        );
        assert_eq!(s.link_health_text(), "Connecting");

        s.serial.status = crate::models::ConnectionStatus::Error;
        assert_eq!(s.active_status(), crate::models::ConnectionStatus::Error);
        assert_eq!(s.link_health_text(), "Error");

        s.serial.status = crate::models::ConnectionStatus::Disconnected;
        s.reconnect_paused_by_user = true;
        assert_eq!(s.link_health_text(), "Offline (manual)");
    }

    #[test]
    fn test_serial_is_any_connected_reflects_serial_service() {
        let mut s = AppState::new();
        s.serial.status = crate::models::ConnectionStatus::Disconnected;
        assert!(!s.is_any_connected());

        s.serial.status = crate::models::ConnectionStatus::Connected;
        assert!(!s.is_any_connected());

        s.serial.status = crate::models::ConnectionStatus::Connected;
        s.active_conn = crate::models::ConnectionType::Serial;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connected
        );
    }

    #[test]
    fn test_serial_auto_reconnect_throttled_by_interval() {
        let mut s = AppState::new();
        s.active_conn = crate::models::ConnectionType::Serial;
        s.ui.auto_reconnect_enabled = true;
        s.ui.auto_reconnect_interval_ms = 3000;
        s.reconnect_paused_by_user = false;
        s.serial.config.port_name.clear();
        s.port_scan_in_progress = false;

        let before_attempts = s.metrics.connect_attempts;
        s.maintain_connection();
        let after_first = s.metrics.connect_attempts;
        assert_eq!(after_first, before_attempts + 1);
        assert!(s.next_reconnect_at.is_some());

        s.maintain_connection();
        let after_second = s.metrics.connect_attempts;
        assert_eq!(after_second, after_first);
    }

    #[test]
    fn test_serial_auto_reconnect_respects_manual_pause() {
        let mut s = AppState::new();
        s.active_conn = crate::models::ConnectionType::Serial;
        s.ui.auto_reconnect_enabled = true;
        s.reconnect_paused_by_user = true;
        s.next_reconnect_at = Some(std::time::Instant::now());

        let before_attempts = s.metrics.connect_attempts;
        s.maintain_connection();

        assert_eq!(s.metrics.connect_attempts, before_attempts);
        assert!(s.next_reconnect_at.is_none());
    }

    #[test]
    fn test_rotate_log_if_needed_creates_backup() {
        let path = unique_temp_file("app.log");
        let mut f = fs::File::create(&path).unwrap();
        let bytes = vec![b'x'; (LOG_FILE_MAX_BYTES as usize) + 1];
        f.write_all(&bytes).unwrap();

        AppState::rotate_log_if_needed(&path);

        let mut backup = path.clone();
        backup.set_extension("log.1");
        assert!(backup.exists());

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(backup);
    }
}
