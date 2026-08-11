pub mod animation;
pub mod connection_manager;
pub mod control_engine;
pub mod external_services;
pub mod log_manager;
pub mod protocol_hub;
pub mod simulation_lab;
pub mod visualization_store;

use connection_manager::ConnectionManager;
use control_engine::ControlEngine;
use external_services::ExternalServices;
use log_manager::LogManager;
use protocol_hub::ProtocolHub;
use simulation_lab::SimulationLabState;
use visualization_store::VisualizationStore;

use crate::i18n::Language;
use crate::models::*;
use crate::services::mcp_server;
use crate::services::*;
use robot_control_core::error::{AppError, AppResult};
use std::collections::VecDeque;
use std::fs::{metadata, OpenOptions};
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

// ──────────────────────────────────────────────────────────────────────
// 导航标签
// ──────────────────────────────────────────────────────────────────────

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
    SimulationLab,
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
            Self::SimulationLab => Tr::tab_simulation_lab(lang),
            Self::ModbusTools => Tr::tab_modbus(lang),
            Self::CanopenTools => Tr::tab_canopen(lang),
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
            Self::SimulationLab,
            Self::ModbusTools,
            Self::CanopenTools,
        ]
    }
}

// ──────────────────────────────────────────────────────────────────────
// 日志条目
// ──────────────────────────────────────────────────────────────────────

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
        let mut buf = String::with_capacity(self.data.len() * 3);
        self.format_data_to(&mut buf);
        buf
    }

    /// Write formatted data into an existing buffer, avoiding per-call allocation.
    pub fn format_data_to(&self, buf: &mut String) {
        use std::fmt::Write;
        match self.display_mode {
            DisplayMode::Hex => {
                for (i, b) in self.data.iter().enumerate() {
                    if i > 0 {
                        buf.push(' ');
                    }
                    let _ = write!(buf, "{:02X}", b);
                }
            }
            DisplayMode::Ascii => {
                buf.push_str(&String::from_utf8_lossy(&self.data));
            }
            DisplayMode::Mixed => {
                for (i, b) in self.data.iter().enumerate() {
                    if i > 0 {
                        buf.push(' ');
                    }
                    let _ = write!(buf, "{:02X}", b);
                }
                buf.push_str(" | ");
                for &b in &self.data {
                    if b.is_ascii_graphic() || b == b' ' {
                        buf.push(b as char);
                    } else {
                        buf.push('.');
                    }
                }
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// UI 状态
// ──────────────────────────────────────────────────────────────────────

pub struct UiState {
    // PID 鏂囨湰妗?
    pub kp_text: String,
    pub ki_text: String,
    pub kd_text: String,
    pub setpoint_text: String,
    pub output_limit_text: String,
    pub integral_limit_text: String,
    pub preset_name: String,
    pub preset_desc: String,

    // 缁堢
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
    pub nn_train_frame_counter: u64,

    // CAN advanced parameters
    pub can_bitrate_idx: usize,
    pub can_data_bitrate_idx: usize,
    pub can_sample_point_idx: usize,
    pub can_data_sample_point_idx: usize,
    pub can_sjw_idx: usize,
    pub can_data_sjw_idx: usize,

    // USB protocol settings
    pub usb_protocol_idx: usize,
    pub usb_speed_idx: usize,
    pub usb_vid_text: String,
    pub usb_pid_text: String,

    // Packet parser state
    pub parser_enabled: bool,
    pub parser_template_idx: usize,
    pub parser_auto_parse: bool,
    pub parser_hex_input: String,
    pub parser_last_auto_input: String,
    pub parser_last_auto_template_idx: usize,
    pub packet_builder_tab: usize, // 0=Builder, 1=Parser

    // Protocol analysis filters
    pub analysis_protocol_idx: usize,
    pub analysis_filter_tx: bool,
    pub analysis_filter_rx: bool,
    pub analysis_filter_info: bool,
    pub analysis_query: String,

    // Data visualization settings
    pub viz_add_channel_name: String,
    pub viz_add_source_idx: usize,
    pub viz_add_type_idx: usize,
    pub viz_source_type: usize, // 0=RobotState, 1=PacketField
    pub viz_pkt_template_idx: usize,

    // LLM settings
    pub llm_api_url: String,
    pub llm_api_key: String,
    pub llm_model_name: String,
    pub llm_temperature_text: String,
    pub llm_last_response: String,
    pub llm_loading: bool,

    // MCP server settings
    pub mcp_port_text: String,
    pub mcp_token_text: String,
    pub mcp_running: bool,

    // Sidebar
    pub sidebar_expanded: bool,

    // Motion level: 0=Extreme, 1=Standard, 2=Native, 3=Optimized
    pub motion_level_idx: usize,

    // UI scale percentage
    pub ui_scale_percent: u32,

    // Preference autosave interval in seconds
    pub prefs_autosave_interval_sec: u32,

    // Update checks
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
            send_text: String::new(),
            send_hex: false,
            auto_scroll: true,
            display_mode: DisplayMode::Hex,
            auto_newline: false,
            send_with_newline: true,
            newline_type: "\\r\\n".into(),
            repeat_send: false,
            repeat_interval_ms: 1000,
            auto_reconnect_enabled: false,
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
            serial_baud_idx: 12,
            tcp_host: "127.0.0.1".into(),
            tcp_port_text: "8080".into(),
            tcp_is_server: false,
            udp_local_port_text: "9000".into(),
            udp_remote_host: "127.0.0.1".into(),
            udp_remote_port_text: "9001".into(),
            nn_learning_rate_text: "0.01".into(),
            nn_auto_train: false,
            nn_train_frame_counter: 0,
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
            viz_add_channel_name: String::new(),
            viz_add_source_idx: 0,
            viz_add_type_idx: 0,
            viz_source_type: 0,
            viz_pkt_template_idx: 0,
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
            ui_scale_percent: 150,
            prefs_autosave_interval_sec: 3,
            update_channel: "stable-0.1".into(),
            update_manifest_url: String::new(),
            update_check_timeout_ms: 1500,
        }
    }
}

// Main application state

pub const MIN_UI_SCALE_PERCENT: u32 = 80;
pub const MAX_UI_SCALE_PERCENT: u32 = 250;
pub const DEFAULT_UI_SCALE_PERCENT: u32 = 150;
pub const UI_SCALE_STEP_PERCENT: i32 = 10;

pub struct AppState {
    // === 子模块 ===
    pub conn: ConnectionManager,
    pub control: ControlEngine,
    pub protocol: ProtocolHub,
    pub viz: VisualizationStore,
    pub log: LogManager,
    pub external: ExternalServices,
    pub simulation: SimulationLabState,

    // === 保留：纯 UI 状态 ===
    pub active_tab: ActiveTab,
    pub language: Language,
    pub ui: UiState,
    pub status_message: String,
    pub dark_mode: bool,
    pub build_version: &'static str,
    pub system_checks: Vec<SystemCheckItem>,
    pub metrics: AppMetrics,
    pub last_error_time: String,

    // === 保留：更新检查 ===
    pub update_latest_version: String,
    pub update_status_detail: String,
    pub update_available: bool,
    pub update_notes_url: String,
    pub update_last_checked_at: String,

    // === 保留：后台任务计时 ===
    last_background_tick_instant: Instant,
    error_burst_count: u32,
    last_error_burst_instant: Option<Instant>,
    resource_status: String,
    /// Set to true when data affecting resource_status changes.
    /// `refresh_resource_status` only rebuilds the string when this is true.
    pub(crate) resource_status_dirty: bool,
    platform_support_note: Option<String>,

    // === Theme ===
    pub theme: crate::views::ui_kit::AppTheme,
    pub high_contrast: bool,
    /// Timestamp when theme transition animation started (for fade overlay).
    pub theme_transition_start: Option<f64>,

    // === Animation System ===
    pub anim: crate::app::animation::AnimationManager,
}

const LOG_FILE_MAX_BYTES: u64 = 5 * 1024 * 1024;
const MAX_PENDING_LOG_LINES: usize = 10_000;
const PORT_SCAN_COOLDOWN_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy)]
pub struct PerformanceProfile {
    pub repaint_interval_ms: u64,
    pub io_poll_interval_ms: u64,
    pub background_task_interval_ms: u64,
    pub max_log_entries: usize,
    pub max_pending_log_lines: usize,
    pub max_state_history: usize,
    pub max_frame_entries: usize,
    pub max_chart_points: usize,
    pub max_protocol_log_entries: usize,
    pub max_parsed_packets: usize,
    pub error_burst_threshold: u32,
    pub reconnect_backoff_base_ms: u64,
    pub auto_downgrade_allowed: bool,
}

impl PerformanceProfile {
    pub fn for_motion_level(idx: usize) -> Self {
        match idx {
            0 => Self {
                repaint_interval_ms: 8,
                io_poll_interval_ms: 8,
                background_task_interval_ms: 120,
                max_log_entries: 8_000,
                max_pending_log_lines: 10_000,
                max_state_history: 4_000,
                max_frame_entries: 10_000,
                max_chart_points: 4_000,
                max_protocol_log_entries: 320,
                max_parsed_packets: 320,
                error_burst_threshold: 6,
                reconnect_backoff_base_ms: 1_000,
                auto_downgrade_allowed: true,
            },
            1 => Self {
                repaint_interval_ms: 16,
                io_poll_interval_ms: 16,
                background_task_interval_ms: 180,
                max_log_entries: 4_000,
                max_pending_log_lines: 6_000,
                max_state_history: 2_000,
                max_frame_entries: 6_000,
                max_chart_points: 2_000,
                max_protocol_log_entries: 200,
                max_parsed_packets: 200,
                error_burst_threshold: 8,
                reconnect_backoff_base_ms: 1_500,
                auto_downgrade_allowed: false,
            },
            2 => Self {
                repaint_interval_ms: 33,
                io_poll_interval_ms: 33,
                background_task_interval_ms: 250,
                max_log_entries: 2_000,
                max_pending_log_lines: 3_000,
                max_state_history: 1_200,
                max_frame_entries: 3_000,
                max_chart_points: 1_000,
                max_protocol_log_entries: 120,
                max_parsed_packets: 120,
                error_burst_threshold: 10,
                reconnect_backoff_base_ms: 2_500,
                auto_downgrade_allowed: false,
            },
            _ => Self {
                repaint_interval_ms: 66,
                io_poll_interval_ms: 66,
                background_task_interval_ms: 400,
                max_log_entries: 1_000,
                max_pending_log_lines: 1_500,
                max_state_history: 600,
                max_frame_entries: 1_000,
                max_chart_points: 400,
                max_protocol_log_entries: 80,
                max_parsed_packets: 80,
                error_burst_threshold: 12,
                reconnect_backoff_base_ms: 4_000,
                auto_downgrade_allowed: false,
            },
        }
    }
}
const DEFAULT_UPDATE_DOC_URL: &str =
    "https://github.com/loopgap/robot_ctrl_rust_app/blob/main/docs/src/README.md";
const DEFAULT_UPDATE_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/example/robot_control_rust/main/update-manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct VersionTriplet {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(default)]
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

fn path_to_file_url(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut raw = canonical.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = raw.strip_prefix("//?/") {
        raw = stripped.to_string();
    }
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    let escaped = raw
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('?', "%3F");
    format!("file://{escaped}")
}

fn resolve_local_help_url() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("help_index.html"));
            candidates.push(exe_dir.join("help").join("index.html"));
            candidates.push(exe_dir.join("docs").join("index.html"));
            for ancestor in exe_dir.ancestors().take(4) {
                candidates.push(ancestor.join("help_index.html"));
                candidates.push(ancestor.join("docs").join("help").join("index.html"));
                candidates.push(ancestor.join("docs").join("index.html"));
                candidates.push(ancestor.join("docs").join("book").join("index.html"));
                candidates.push(ancestor.join("docs").join("site").join("index.html"));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("/usr/share/rust-tools-suite/help_index.html"));
        candidates.push(PathBuf::from("/usr/share/rust-tools-suite/docs/index.html"));
        candidates.push(PathBuf::from(
            "/usr/share/rust-tools-suite/docs/book/index.html",
        ));
        candidates.push(PathBuf::from(
            "/usr/share/doc/rust-tools-suite/help_index.html",
        ));
        candidates.push(PathBuf::from(
            "/usr/share/doc/rust-tools-suite/docs/index.html",
        ));
        candidates.push(PathBuf::from(
            "/usr/share/doc/rust-tools-suite/docs/book/index.html",
        ));
    }

    candidates.push(PathBuf::from("help_index.html"));
    candidates.push(PathBuf::from("docs").join("help").join("index.html"));
    candidates.push(PathBuf::from("docs").join("index.html"));
    candidates.push(PathBuf::from("docs").join("book").join("index.html"));
    candidates.push(PathBuf::from("docs").join("site").join("index.html"));

    candidates
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path_to_file_url(&path))
}

fn valid_http_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Some(trimmed.to_string())
    } else {
        None
    }
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
    #[serde(default)]
    high_contrast: bool,
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
    ui_scale_percent: u32,
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
            high_contrast: false,
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
            auto_reconnect_enabled: false,
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
            ui_scale_percent: 150,
            prefs_autosave_interval_sec: 3,
            update_channel: "stable-0.1".into(),
            update_manifest_url: String::new(),
            update_check_timeout_ms: 1500,
        }
    }
}

impl AppState {
    // ──────────────────────────────────────────────────────────────────────
    // 初始化与配置
    // ──────────────────────────────────────────────────────────────────────
    pub fn new() -> Self {
        let control = ControlEngine::new();
        let pid = control.algorithms[0]
            .as_any()
            .downcast_ref::<PidController>()
            .cloned()
            .unwrap_or_default();
        let ui = UiState {
            kp_text: format!("{:.3}", pid.kp),
            ki_text: format!("{:.3}", pid.ki),
            kd_text: format!("{:.3}", pid.kd),
            setpoint_text: format!("{:.3}", pid.setpoint),
            output_limit_text: format!("{:.1}", pid.output_limit),
            integral_limit_text: format!("{:.1}", pid.integral_limit),
            ..Default::default()
        };

        let mut conn = ConnectionManager::new();
        conn.available_ports = Vec::new();

        let mut s = Self {
            conn,
            control,
            protocol: ProtocolHub::new(),
            viz: VisualizationStore::new(),
            log: LogManager::new(),
            external: ExternalServices::new(),
            simulation: SimulationLabState::new(),
            active_tab: ActiveTab::Dashboard,
            language: Language::Chinese,
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
            resource_status: "Balanced".into(),
            resource_status_dirty: true,
            platform_support_note: None,
            last_background_tick_instant: Instant::now(),
            error_burst_count: 0,
            last_error_burst_instant: None,
            theme: crate::views::ui_kit::AppTheme::dark(),
            high_contrast: false,
            theme_transition_start: None,
            anim: crate::app::animation::AnimationManager::new(),
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
        s.apply_performance_profile();
        s.platform_support_note = s.detect_platform_support_note();
        s.refresh_resource_status();
        s.run_system_check();
        s
    }

    pub fn performance_profile(&self) -> PerformanceProfile {
        PerformanceProfile::for_motion_level(self.ui.motion_level_idx)
    }

    pub fn repaint_interval_ms(&self) -> u64 {
        self.performance_profile().repaint_interval_ms
    }

    fn trim_vec<T>(items: &mut Vec<T>, max_len: usize) {
        if items.len() > max_len {
            items.drain(..items.len() - max_len);
        }
    }

    fn trim_vec_deque<T>(items: &mut VecDeque<T>, max_len: usize) {
        if items.len() > max_len {
            items.drain(..items.len() - max_len);
        }
    }

    pub fn apply_performance_profile(&mut self) {
        let profile = self.performance_profile();
        Self::trim_vec_deque(&mut self.log.log_entries, profile.max_log_entries.max(1));
        Self::trim_vec(
            &mut self.conn.pending_log_lines,
            profile.max_pending_log_lines.max(1),
        );
        Self::trim_vec(
            &mut self.control.state_history,
            profile.max_state_history.max(1),
        );
        Self::trim_vec(
            &mut self.protocol.parsed_packets,
            profile.max_parsed_packets.max(1),
        );
        Self::trim_vec(
            &mut self.protocol.modbus_response_log,
            profile.max_protocol_log_entries.max(1),
        );
        Self::trim_vec(
            &mut self.protocol.canopen_log,
            profile.max_protocol_log_entries.max(1),
        );
        self.conn
            .can
            .set_max_frames(profile.max_frame_entries.max(1));
        for buffer in &mut self.viz.channel_buffers {
            buffer.set_max_points(profile.max_chart_points.max(1));
        }
        self.resource_status_dirty = true;
        self.refresh_resource_status();
    }

    pub fn refresh_resource_status(&mut self) {
        if !self.resource_status_dirty {
            return;
        }
        self.resource_status_dirty = false;
        let profile = self.performance_profile();
        use std::fmt::Write;
        let mut s = String::with_capacity(80);
        let _ = write!(
            s,
            "{}ms | logs {}/{} | state {}/{} | can {}/{}",
            profile.repaint_interval_ms,
            self.log.log_entries.len(),
            profile.max_log_entries,
            self.control.state_history.len(),
            profile.max_state_history,
            self.conn.can.frames.len(),
            self.conn.can.max_frame_capacity(),
        );
        self.resource_status = s;
    }

    fn detect_platform_support_note(&self) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            return Some("Supported baseline: Windows 8+".into());
        }
        #[cfg(target_os = "linux")]
        {
            return Some("Supported baseline: Ubuntu 20.04+".into());
        }
        #[allow(unreachable_code)]
        Some("Supported baseline: Windows 8+ / Ubuntu 20.04+".into())
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
        use std::fmt::Write;
        let mut line = String::with_capacity(64 + entry.data.len() * 3);
        let _ = write!(
            line,
            "{} [{}] [{}] ",
            entry.timestamp,
            entry.channel,
            match entry.direction {
                LogDirection::Tx => "TX",
                LogDirection::Rx => "RX",
                LogDirection::Info => "INFO",
            },
        );
        entry.format_data_to(&mut line);
        self.conn.pending_log_lines.push(line);
        let max_pending = self
            .performance_profile()
            .max_pending_log_lines
            .max(MAX_PENDING_LOG_LINES / 8);
        Self::trim_vec(&mut self.conn.pending_log_lines, max_pending);
    }

    pub fn flush_pending_logs(&mut self) {
        if self.conn.pending_log_lines.is_empty() {
            return;
        }
        let now = Instant::now();
        let min_flush_ms = self
            .performance_profile()
            .background_task_interval_ms
            .max(120);
        if self.conn.pending_log_lines.len() < 100
            && now.duration_since(self.conn.last_log_flush_instant)
                < Duration::from_millis(min_flush_ms)
        {
            return;
        }

        let path = Self::log_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        Self::rotate_log_if_needed(&path);
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
            for line in self.conn.pending_log_lines.drain(..) {
                let _ = writeln!(f, "{}", line);
            }
            self.conn.last_log_flush_instant = now;
        }
    }

    pub fn report_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        let now = Instant::now();
        let profile = self.performance_profile();
        if self
            .last_error_burst_instant
            .is_some_and(|last| now.duration_since(last) <= Duration::from_secs(5))
        {
            self.error_burst_count = self.error_burst_count.saturating_add(1);
        } else {
            self.error_burst_count = 1;
        }
        self.last_error_burst_instant = Some(now);
        self.last_error_time = chrono::Local::now().format("%H:%M:%S").to_string();
        self.status_message = message.clone();
        self.resource_status_dirty = true;
        self.refresh_resource_status();
        if self.error_burst_count >= profile.error_burst_threshold {
            self.status_message = format!("{} | burst {}", message, self.error_burst_count);
        }
        self.add_info_log(&format!("Error: {}", message));
        error!(target: "app", message = %self.status_message, "ui_error");
    }

    pub fn report_channel_overflow(&mut self, dropped: usize) {
        self.viz.channel_overflow_events = self
            .viz
            .channel_overflow_events
            .saturating_add(dropped as u64);
        warn!(target: "buffers", dropped = dropped, total = self.viz.channel_overflow_events, "channel_overflow");
    }

    fn push_check(&mut self, name: &str, ok: bool, detail: impl Into<String>) {
        self.system_checks.push(SystemCheckItem {
            name: name.to_string(),
            ok,
            detail: detail.into(),
        });
    }

    // ──────────────────────────────────────────────────────────────────────
    // 错误报告与系统检查
    // ──────────────────────────────────────────────────────────────────────
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

        let serial_ok = !self.conn.available_ports.is_empty();
        self.push_check(
            "Serial ports",
            serial_ok,
            format!("{} ports detected", self.conn.available_ports.len()),
        );

        info!(
            target: "self_check",
            checks = self.system_checks.len(),
            ok = self.system_checks.iter().filter(|c| c.ok).count(),
            "system_check_completed"
        );
    }

    pub fn documentation_url(&self) -> String {
        if let Some(url) = resolve_local_help_url() {
            return url;
        }
        if let Ok(url) = std::env::var("ROBOT_CONTROL_UPDATE_URL") {
            if let Some(valid) = valid_http_url(&url) {
                return valid;
            }
        }
        DEFAULT_UPDATE_DOC_URL.to_string()
    }

    pub fn update_doc_url(&self) -> String {
        if !self.update_notes_url.trim().is_empty() {
            return self.update_notes_url.trim().to_string();
        }
        if let Ok(url) = std::env::var("ROBOT_CONTROL_UPDATE_URL") {
            if let Some(valid) = valid_http_url(&url) {
                return valid;
            }
        }
        self.documentation_url()
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
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_millis(timeout)))
            .build();
        let agent: ureq::Agent = config.into();
        let mut response = agent
            .get(manifest_url)
            .call()
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|e| format!("Failed to read HTTP response body: {e}"))?;
        serde_json::from_str::<UpdateManifest>(&text).map_err(|e| format!("JSON parse failed: {e}"))
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
                    self.add_info_log(&format!("Update: {}", detail));
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
                    self.add_info_log(&format!("Update warning: {}", self.update_status_detail));
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
                self.add_info_log(&format!("Update warning: {}", self.update_status_detail));
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
        match self.external.mcp_shared_state.try_lock() {
            Ok(s) => (s.request_count, s.unauthorized_count),
            Err(_) => (0, 0),
        }
    }

    // ──────────────────────────────────────────────────────────────────────
    // 用户偏好设置
    // ──────────────────────────────────────────────────────────────────────
    fn to_user_preferences(&self) -> UserPreferences {
        let active_tab_idx = ActiveTab::all()
            .iter()
            .position(|t| *t == self.active_tab)
            .unwrap_or(0);
        UserPreferences {
            schema_version: 2,
            language: self.language,
            dark_mode: self.dark_mode,
            high_contrast: self.high_contrast,
            sidebar_expanded: self.ui.sidebar_expanded,
            motion_level_idx: self.ui.motion_level_idx.min(3),
            active_tab_idx,
            parser_auto_parse: self.ui.parser_auto_parse,
            display_mode: self.ui.display_mode,
            llm_api_url: self.ui.llm_api_url.clone(),
            llm_model_name: self.ui.llm_model_name.clone(),
            mcp_port_text: self.ui.mcp_port_text.clone(),
            mcp_token_text: self.ui.mcp_token_text.clone(),
            active_conn: self.conn.active_conn,
            serial_config: self.conn.serial.config.clone(),
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
            ui_scale_percent: self.ui.ui_scale_percent.clamp(100, 220),
            prefs_autosave_interval_sec: self.ui.prefs_autosave_interval_sec,
            update_channel: self.ui.update_channel.clone(),
            update_manifest_url: self.ui.update_manifest_url.clone(),
            update_check_timeout_ms: self.ui.update_check_timeout_ms,
        }
    }

    fn apply_user_preferences(&mut self, prefs: UserPreferences) {
        self.language = prefs.language;
        self.dark_mode = prefs.dark_mode;
        self.high_contrast = prefs.high_contrast;
        self.rebuild_theme();
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
        self.conn.active_conn = prefs.active_conn;
        self.ui.conn_type_idx = ConnectionType::all()
            .iter()
            .position(|c| *c == self.conn.active_conn)
            .unwrap_or(0);
        self.conn.serial.config = prefs.serial_config;
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
        self.ui.ui_scale_percent = prefs.ui_scale_percent.clamp(100, 220);
        self.ui.prefs_autosave_interval_sec = prefs.prefs_autosave_interval_sec.clamp(1, 300);
        self.ui.update_channel = prefs.update_channel;
        self.ui.update_manifest_url = prefs.update_manifest_url;
        self.ui.update_check_timeout_ms = prefs.update_check_timeout_ms.clamp(500, 10_000);
    }

    /// Rebuild `theme` from current `dark_mode` and `high_contrast` settings.
    pub fn rebuild_theme(&mut self) {
        let base = if self.dark_mode {
            crate::views::ui_kit::AppTheme::dark()
        } else {
            crate::views::ui_kit::AppTheme::light()
        };
        self.theme = if self.high_contrast {
            base.high_contrast()
        } else {
            base
        };
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

    pub fn preferences_snapshot(&self) -> AppResult<(PathBuf, String)> {
        let text = serde_json::to_string_pretty(&self.to_user_preferences())
            .map_err(|e| format!("Preferences serialize failed: {}", e))?;
        Ok((Self::user_prefs_path(), text))
    }

    pub fn write_preferences_snapshot(path: &Path, text: &str) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Preferences create dir failed: {}", e))?;
        }

        let mut tmp_path = path.to_path_buf();
        tmp_path.set_extension("json.tmp");
        std::fs::write(&tmp_path, text).map_err(|e| format!("Preferences save failed: {}", e))?;

        if path.exists() {
            let mut bak_path = path.to_path_buf();
            bak_path.set_extension("json.bak");
            let _ = std::fs::copy(path, &bak_path);
            let _ = std::fs::remove_file(path);
        }

        std::fs::rename(&tmp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            format!("Preferences commit failed: {}", e)
        })?;
        Ok(())
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

    fn csv_escape(value: &str) -> String {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    }

    pub fn export_logs_csv(&self) -> AppResult<std::path::PathBuf> {
        let mut export_dir = Self::user_prefs_path();
        export_dir.pop();
        export_dir.push("exports");
        std::fs::create_dir_all(&export_dir)
            .map_err(|e| format!("create export dir failed: {}", e))?;

        let file_name = format!("logs_{}.csv", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let file_path = export_dir.join(file_name);

        let mut csv = String::from("timestamp,channel,direction,display_mode,data\n");
        for entry in &self.log.log_entries {
            let direction = match entry.direction {
                LogDirection::Tx => "TX",
                LogDirection::Rx => "RX",
                LogDirection::Info => "INFO",
            };
            let display_mode = match entry.display_mode {
                DisplayMode::Hex => "HEX",
                DisplayMode::Ascii => "ASCII",
                DisplayMode::Mixed => "MIXED",
            };

            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                Self::csv_escape(&entry.timestamp),
                Self::csv_escape(&entry.channel),
                Self::csv_escape(direction),
                Self::csv_escape(display_mode),
                Self::csv_escape(&entry.format_data())
            ));
        }

        std::fs::write(&file_path, csv).map_err(|e| format!("write export file failed: {}", e))?;
        Ok(file_path)
    }

    // ──────────────────────────────────────────────────────────────────────
    // 连接管理与通信
    // ──────────────────────────────────────────────────────────────────────
    pub fn refresh_ports(&mut self) {
        if self.conn.port_scan_in_progress {
            return;
        }
        let now = Instant::now();
        if let Some(last) = self.conn.last_port_scan_request_at {
            if now.duration_since(last) < Duration::from_millis(PORT_SCAN_COOLDOWN_MS) {
                self.status_message = format!(
                    "Port scan cooling down. Retry in {:.1}s",
                    (PORT_SCAN_COOLDOWN_MS as f32 - now.duration_since(last).as_millis() as f32)
                        / 1000.0
                );
                return;
            }
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.conn.last_port_scan_request_at = Some(now);
        self.conn.port_scan_in_progress = true;
        self.conn.port_scan_rx = Some(rx);
        self.status_message = "Scanning serial ports...".into();

        std::thread::spawn(move || {
            let ports = SerialService::scan_ports();
            let _ = tx.send(ports);
        });
    }

    fn apply_scanned_ports(&mut self, ports: Vec<String>) {
        let previous = self.conn.serial.config.port_name.clone();
        self.conn.available_ports = ports;

        if self.conn.available_ports.is_empty() {
            self.conn.serial.config.port_name.clear();
            self.status_message = "No serial ports found".into();
            return;
        }

        if !previous.is_empty() && self.conn.available_ports.iter().any(|p| p == &previous) {
            self.conn.serial.config.port_name = previous;
            self.status_message = format!(
                "Serial ports refreshed: {} detected",
                self.conn.available_ports.len()
            );
            return;
        }

        self.conn.serial.config.port_name = self.conn.available_ports[0].clone();
        self.status_message = format!(
            "Serial port auto-selected: {}",
            self.conn.serial.config.port_name
        );
    }

    pub fn total_bytes_sent(&self) -> u64 {
        self.conn.total_bytes_sent()
    }

    pub fn total_bytes_received(&self) -> u64 {
        self.conn.total_bytes_received()
    }

    pub fn total_errors(&self) -> u64 {
        self.conn.total_errors()
    }

    pub fn is_any_connected(&self) -> bool {
        self.conn.is_any_connected()
    }

    pub fn active_status(&self) -> ConnectionStatus {
        self.conn.active_status()
    }

    pub fn last_comm(&self) -> &str {
        self.conn.last_comm()
    }

    pub fn link_health_text(&self) -> String {
        self.conn.link_health_text()
    }

    pub fn reconnect_paused(&self) -> bool {
        self.conn.reconnect_paused()
    }

    pub fn reconnect_armed(&self) -> bool {
        self.conn.reconnect_armed()
    }

    pub fn reconnect_countdown_text(&self) -> Option<String> {
        Some(self.conn.reconnect_countdown_text())
    }

    fn clear_reconnect_schedule(&mut self) {
        self.conn.clear_reconnect_schedule();
    }

    fn arm_auto_reconnect(&mut self) {
        self.conn.arm_auto_reconnect();
    }

    pub fn pause_auto_reconnect(&mut self) {
        self.conn.pause_auto_reconnect();
        self.status_message = "Reconnect paused".into();
        self.add_info_log("Reconnect paused");
    }

    pub fn resume_auto_reconnect(&mut self) {
        if !self.conn.reconnect_armed() {
            self.status_message = "Reconnect is idle until a manual connection succeeds".into();
            return;
        }
        self.conn.resume_auto_reconnect();
        self.status_message = "Auto reconnect resumed".into();
        self.add_info_log("Auto reconnect resumed");
    }

    pub fn set_active_connection(&mut self, conn: ConnectionType) {
        if self.conn.active_conn == conn {
            return;
        }
        self.conn.set_active_connection(conn);
        self.status_message =
            "Connection target changed. Refresh ports and connect when ready.".into();
    }

    pub fn maintain_connection(&mut self) {
        let profile = self.performance_profile();
        let now = Instant::now();

        if !self.ui.auto_reconnect_enabled
            || self.conn.reconnect_paused()
            || !self.conn.reconnect_armed()
        {
            self.clear_reconnect_schedule();
            return;
        }

        self.conn.last_connection_check_instant = now;

        if self.conn.serial_connect_in_progress
            && matches!(
                self.conn.active_conn,
                ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu
            )
        {
            return;
        }

        let supported = matches!(
            self.conn.active_conn,
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
            self.clear_reconnect_schedule();
            return;
        }

        if matches!(
            self.conn.active_conn,
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu
        ) && self.conn.serial.config.port_name.trim().is_empty()
        {
            self.clear_reconnect_schedule();
            self.status_message =
                "Reconnect waiting for a selected serial port. Click Refresh ports.".into();
            return;
        }

        if self.conn.next_reconnect_at.is_some_and(|next| next > now) {
            return;
        }

        // Exponential backoff: base interval × 2^attempts, capped at 60 seconds.
        // After 10 consecutive failures, enter idle-pause to save CPU.
        const MAX_RECONNECT_ATTEMPTS: u32 = 10;
        const BACKOFF_CAP_MS: u64 = 60_000;

        if self.conn.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            self.add_info_log(&format!(
                "Auto-reconnect paused after {} consecutive failures. \
                 Disarm and re-arm to retry.",
                MAX_RECONNECT_ATTEMPTS,
            ));
            self.clear_reconnect_schedule();
            self.status_message = "Reconnect exhausted. Toggle auto-reconnect to retry.".into();
            return;
        }

        let mut interval_ms = (self.ui.auto_reconnect_interval_ms as u64).clamp(500, 30_000);
        interval_ms = interval_ms.max(profile.reconnect_backoff_base_ms.max(500));
        // Exponential backoff multiplier
        interval_ms = interval_ms
            .saturating_mul(1u64 << self.conn.reconnect_attempts.min(10))
            .min(BACKOFF_CAP_MS);

        if self.error_burst_count >= profile.error_burst_threshold {
            interval_ms = interval_ms.saturating_mul(2).min(BACKOFF_CAP_MS);
            if profile.auto_downgrade_allowed && self.ui.motion_level_idx == 0 {
                self.ui.motion_level_idx = 1;
                self.apply_performance_profile();
                self.add_info_log(
                    "Performance downgraded to Standard after repeated connection errors",
                );
            }
        }

        self.conn.reconnect_attempts += 1;
        self.conn.next_reconnect_at = Some(now + Duration::from_millis(interval_ms));
        if let Err(e) = self.connect_active() {
            self.add_info_log(&format!("Auto-reconnect failed: {e}"));
        }
    }

    pub fn connect_active(&mut self) -> AppResult<()> {
        self.conn.reconnect_paused_by_user = false;
        self.metrics.connect_attempts += 1;
        let result = match self.conn.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                if self.conn.serial.config.port_name.trim().is_empty() {
                    return Err(
                        "No serial port selected. Click Refresh ports, choose one, then connect."
                            .into(),
                    );
                }

                if !self.conn.available_ports.is_empty()
                    && !self
                        .conn
                        .available_ports
                        .iter()
                        .any(|p| p == &self.conn.serial.config.port_name)
                {
                    return Err(
                        "Selected serial port is no longer available. Click Refresh ports.".into(),
                    );
                }

                self.start_serial_connect_worker()
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => {
                self.conn.tcp.host = self.ui.tcp_host.trim().to_string();
                if self.conn.tcp.host.is_empty() {
                    return Err("TCP host required".into());
                }
                self.conn.tcp.port = parse_port(&self.ui.tcp_port_text, "TCP port")?;
                self.conn.tcp.is_server = self.ui.tcp_is_server;
                if self.ui.tcp_is_server {
                    self.conn
                        .tcp
                        .start_server()
                        .map_err(|e| format!("TCP server start failed: {e}"))
                } else {
                    self.conn
                        .tcp
                        .connect_client()
                        .map_err(|e| format!("TCP client connect failed: {e}"))
                }
            }
            ConnectionType::Udp => {
                self.conn.udp.local_port =
                    parse_port(&self.ui.udp_local_port_text, "UDP local port")?;
                self.conn.udp.remote_addr = self.ui.udp_remote_host.trim().to_string();
                self.conn.udp.remote_port =
                    parse_port(&self.ui.udp_remote_port_text, "UDP remote port")?;
                if self.conn.udp.remote_addr.is_empty() {
                    return Err("UDP remote host required".into());
                }
                self.conn
                    .udp
                    .bind()
                    .map_err(|e| format!("UDP bind failed: {e}"))
            }
            ConnectionType::Can | ConnectionType::CanFd => {
                self.conn.can.is_running = true;
                Ok(())
            }
        }
        .map_err(AppError::Other);

        let is_serial_mode = matches!(
            self.conn.active_conn,
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu
        );

        match &result {
            Ok(_) if is_serial_mode => {
                self.status_message = format!("Connecting: {}", self.conn.active_conn);
            }
            Ok(_) => {
                self.arm_auto_reconnect();
                self.conn.reconnect_attempts = 0; // Reset backoff on success
                info!(target: "connection", connection = %self.conn.active_conn, "connect_success");
                self.add_info_log(&format!("Connected: {}", self.conn.active_conn));
            }
            Err(e) => {
                self.metrics.connect_failures += 1;
                self.report_error(format!("Connect failed ({}): {}", self.conn.active_conn, e));
            }
        }
        result
    }

    fn start_serial_connect_worker(&mut self) -> Result<(), String> {
        if self.conn.serial_connect_in_progress {
            return Err("Serial connect already in progress".into());
        }

        let cfg = self.conn.serial.config.clone();
        if cfg.port_name.trim().is_empty() {
            return Err("No serial port selected".into());
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.conn.serial_connect_rx = Some(rx);
        self.conn.serial_connect_in_progress = true;
        self.conn.serial.status = ConnectionStatus::Connecting;

        thread::spawn(move || {
            let mut svc = SerialService::new();
            svc.config = cfg;
            let result = svc
                .connect()
                .map(|_| svc)
                .map_err(|e| format!("Serial port open failed: {e}"));
            let _ = tx.send(result);
        });

        Ok(())
    }

    pub fn disconnect_active(&mut self) {
        self.conn.reconnect_armed = false;
        self.conn.reconnect_paused_by_user = true;
        self.conn.clear_reconnect_schedule();
        self.conn.serial_connect_in_progress = false;
        self.conn.serial_connect_rx = None;
        match self.conn.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                self.conn.serial.disconnect()
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => self.conn.tcp.disconnect(),
            ConnectionType::Udp => self.conn.udp.close(),
            ConnectionType::Can | ConnectionType::CanFd => {
                self.conn.can.is_running = false;
            }
        }
    }

    pub fn send_data(&mut self, data: &[u8]) -> AppResult<()> {
        let result = match self.conn.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => self
                .conn
                .serial
                .send_data(data)
                .map_err(|e| format!("Serial send failed: {e}")),
            ConnectionType::Tcp | ConnectionType::ModbusTcp => self
                .conn
                .tcp
                .send_data(data)
                .map_err(|e| format!("TCP send failed: {e}")),
            ConnectionType::Udp => self
                .conn
                .udp
                .send_default(data)
                .map_err(|e| format!("UDP send failed: {e}")),
            _ => Err("Channel not supported".into()),
        }
        .map_err(AppError::Other);

        if result.is_ok() {
            self.add_log(LogDirection::Tx, data, &self.conn.active_conn.to_string());
        } else if let Err(e) = &result {
            self.report_error(format!("Send failed ({}): {}", self.conn.active_conn, e));
        }
        result
    }

    pub fn poll_data(&mut self) {
        let profile = self.performance_profile();
        let now = Instant::now();
        if now.duration_since(self.conn.last_io_poll_instant)
            < Duration::from_millis(profile.io_poll_interval_ms)
        {
            return;
        }
        self.conn.last_io_poll_instant = now;

        // Collect raw data from all connected interfaces for packet parsing
        let mut all_raw: Vec<u8> = Vec::new();

        if self.conn.serial.is_connected() {
            let raw = self.conn.serial.try_read_raw();
            if !raw.is_empty() {
                self.conn.last_rx_instant = Some(now);
                self.add_log(LogDirection::Rx, &raw, "Serial");
                self.conn.serial.push_rx_data(&raw);
                all_raw.extend_from_slice(&raw);
            }
        }

        if self.conn.tcp.is_connected() {
            let data = self.conn.tcp.try_read_raw();
            if !data.is_empty() {
                self.conn.last_rx_instant = Some(now);
                self.add_log(LogDirection::Rx, &data, "TCP");
                all_raw.extend_from_slice(&data);
            }
        }

        if self.conn.udp.is_connected() {
            let data = self.conn.udp.try_read_raw();
            if !data.is_empty() {
                self.conn.last_rx_instant = Some(now);
                self.add_log(LogDirection::Rx, &data, "UDP");
                all_raw.extend_from_slice(&data);
            }
        }

        // Try to parse raw data with registered packet templates
        if !all_raw.is_empty() && !self.protocol.packet_templates.is_empty() {
            // Ensure parser templates are in sync
            if self.protocol.packet_parser.template_count() != self.protocol.packet_templates.len()
            {
                self.protocol.sync_packet_parser();
            }
            if let Some(parsed) = self.protocol.packet_parser.try_parse(&all_raw) {
                self.feed_parsed_to_channels(&parsed);
                self.protocol.parsed_packets.push(parsed);
                Self::trim_vec(&mut self.protocol.parsed_packets, 200);
            }
        }

        if self.conn.serial.is_connected() {
            while let Some(state) = self.conn.serial.try_parse_state_from_buffer() {
                let mut s = state;
                if self.control.is_running {
                    let output = self.compute_active_algorithm(s.position, s.velocity);
                    s.pid_output = output;
                    s.error = self.get_active_setpoint() - s.position;
                    if let Err(e) = self.conn.serial.send_position_control(output) {
                        self.report_error(format!("Position control send failed: {e}"));
                    }
                }
                self.control.current_state = s.clone();
                self.control.state_history.push(s);
                Self::trim_vec(
                    &mut self.control.state_history,
                    profile.max_state_history.max(1),
                );
            }
        }

        self.sync_mcp_state();
        self.resource_status_dirty = true;
        self.refresh_resource_status();
    }
    fn add_log(&mut self, dir: LogDirection, data: &[u8], channel: &str) {
        let msg = String::from_utf8_lossy(data);
        self.log
            .add_log_with_display_mode(dir, &msg, self.ui.display_mode, channel);
        self.append_log(&LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction: dir,
            data: data.to_vec(),
            display_mode: self.ui.display_mode,
            channel: channel.into(),
        });
        self.resource_status_dirty = true;
        self.refresh_resource_status();
    }

    pub fn add_info_log(&mut self, msg: &str) {
        self.log
            .add_log_with_display_mode(LogDirection::Info, msg, DisplayMode::Ascii, "System");
        self.append_log(&LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
            direction: LogDirection::Info,
            data: msg.as_bytes().to_vec(),
            display_mode: DisplayMode::Ascii,
            channel: "System".into(),
        });
        self.resource_status_dirty = true;
        self.refresh_resource_status();
    }

    // Control actions

    pub fn toggle_running(&mut self) {
        self.control.toggle_running();
        if self.control.is_running {
            self.reset_active_algorithm();
            self.status_message = "Control started".into();
        } else {
            self.status_message = "Control stopped".into();
        }
    }

    pub fn emergency_stop(&mut self) {
        self.control.is_running = false;
        let send_result = if self.conn.serial.is_connected() {
            self.conn.serial.send_emergency_stop()
        } else {
            Ok(())
        };
        match send_result {
            Ok(()) => {
                self.status_message = "EMERGENCY STOP!".into();
                self.add_info_log("Warning: Emergency Stop activated!");
            }
            Err(e) => {
                self.status_message = "EMERGENCY STOP \u{2014} send failed!".into();
                self.report_error(format!(
                    "Emergency stop command failed to send: {e}. Motor may still be running!"
                ));
            }
        }
    }

    // Control algorithm dispatch

    /// Compute the output with the active control algorithm.
    pub fn compute_active_algorithm(&mut self, position: f64, velocity: f64) -> f64 {
        self.control.compute_dual(position, velocity)
    }

    pub fn get_active_setpoint(&self) -> f64 {
        self.control.setpoint()
    }

    pub fn reset_active_algorithm(&mut self) {
        self.control.reset_active();
    }

    // Neural-network tuning

    pub fn nn_suggest_params(&mut self) {
        let errors: Vec<f64> = self.control.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 10 {
            return;
        }
        let features = NeuralNetwork::extract_features(&errors);
        let output = self.control.nn.forward(&features);
        self.control.nn_suggested_kp = output[0] * 5.0;
        self.control.nn_suggested_ki = output[1] * 2.0;
        self.control.nn_suggested_kd = output[2] * 1.0;
    }

    pub fn nn_train_step(&mut self) {
        let errors: Vec<f64> = self.control.state_history.iter().map(|s| s.error).collect();
        if errors.len() < 20 {
            return;
        }
        let features = NeuralNetwork::extract_features(&errors);
        let performance =
            1.0 / (1.0 + errors.iter().map(|e| e.abs()).sum::<f64>() / errors.len() as f64);
        let pid = self.control.pid();
        let target = vec![
            (pid.kp / 5.0).clamp(0.0, 1.0) * performance,
            (pid.ki / 2.0).clamp(0.0, 1.0) * performance,
            (pid.kd / 1.0).clamp(0.0, 1.0) * performance,
        ];
        let loss = self.control.nn.train_step(&features, &target);
        self.status_message = format!(
            "NN Training - Loss: {:.6}, Epoch: {}",
            loss, self.control.nn.training_epochs
        );
    }

    pub fn apply_nn_params(&mut self) {
        let kp = self.control.nn_suggested_kp;
        let ki = self.control.nn_suggested_ki;
        let kd = self.control.nn_suggested_kd;
        let pid = self.control.pid_mut();
        pid.kp = kp;
        pid.ki = ki;
        pid.kd = kd;
        self.ui.kp_text = format!("{:.3}", kp);
        self.ui.ki_text = format!("{:.3}", ki);
        self.ui.kd_text = format!("{:.3}", kd);
        self.status_message = "Applied NN suggested parameters".into();
    }

    // Parsed packet data feeds visualization channels

    /// Push parsed packet fields into matching visualization channel buffers.
    pub fn feed_parsed_to_channels(&mut self, parsed: &ParsedPacket) {
        let mut dropped_total = 0usize;
        for (i, ch) in self.viz.data_channels.iter().enumerate() {
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
                        while self.viz.channel_buffers.len() <= i {
                            self.viz.channel_buffers.push(TimeSeriesBuffer::default());
                        }
                        let dropped = self.viz.channel_buffers[i].push_with_overflow(val);
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

    /// Create a visualization channel from a parsed packet field.
    pub fn add_channel_from_parsed_field(
        &mut self,
        template_name: &str,
        field_name: &str,
        viz_type: crate::models::VizType,
    ) {
        if let Some((idx, _)) = self.viz.data_channels.iter().enumerate().find(|(_, ch)| {
            matches!(
                &ch.source,
                DataSource::PacketField { template_name: t, field_name: f }
                    if t == template_name && f == field_name
            )
        }) {
            self.viz.data_channels[idx].enabled = true;
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
        let c = colors[self.viz.data_channels.len() % colors.len()];
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
        self.viz.data_channels.push(ch);
        self.viz.channel_buffers.push(TimeSeriesBuffer::default());

        // Backfill existing parsed packet values.
        let buf_idx = self.viz.channel_buffers.len() - 1;
        let mut dropped_total = 0usize;
        for pkt in &self.protocol.parsed_packets {
            if pkt.template_name == template_name {
                if let Some(val) = pkt.field_value(field_name) {
                    let dropped = self.viz.channel_buffers[buf_idx].push_with_overflow(val);
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

    /// Return all numeric parsed packet fields as (template_name, field_name).
    pub fn available_packet_fields(&self) -> Vec<(String, String)> {
        let mut fields = Vec::new();
        for pkt in &self.protocol.parsed_packets {
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

    // LLM-assisted tuning

    /// Request PID tuning suggestions from the configured LLM API.
    pub fn llm_suggest_params(&mut self) {
        use crate::services::llm_service::LlmService;
        if self.ui.llm_loading {
            self.status_message = "LLM request is already running".into();
            return;
        }
        let errors: Vec<f64> = self.control.state_history.iter().map(|s| s.error).collect();
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
        let current_params = {
            let pid = self.control.pid();
            crate::services::llm_service::PidParams {
                kp: pid.kp,
                ki: pid.ki,
                kd: pid.kd,
                setpoint: pid.setpoint,
            }
        };

        self.ui.llm_loading = true;
        self.metrics.llm_requests += 1;
        self.ui.llm_last_response = "Requesting LLM...".into();
        self.status_message = "LLM request started".into();
        info!(target: "llm", model = %model, api_url = %api_url, "llm_request_started");

        let (tx, rx) = std::sync::mpsc::channel();
        self.external.llm_result_rx = Some(rx);

        std::thread::spawn(move || {
            let llm = LlmService::new(api_url, api_key, model);
            let result = llm
                .suggest_pid_params(&current_params, &errors)
                .map_err(|e| format!("LLM service error: {e}"));
            let _ = tx.send(result);
        });
    }

    pub fn poll_background_tasks(&mut self) {
        let now = Instant::now();
        let min_tick_ms = self
            .performance_profile()
            .background_task_interval_ms
            .max(120);
        if now.duration_since(self.last_background_tick_instant)
            < Duration::from_millis(min_tick_ms)
        {
            self.flush_pending_logs();
            return;
        }
        self.last_background_tick_instant = now;

        self.flush_pending_logs();

        if let Some(rx) = self.conn.serial_connect_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.conn.serial_connect_in_progress = false;
                    match result {
                        Ok(serial) => {
                            self.conn.serial = serial;
                            self.arm_auto_reconnect();
                            info!(target: "connection", connection = %self.conn.active_conn, "connect_success");
                            self.add_info_log(&format!("Connected: {}", self.conn.active_conn));
                        }
                        Err(e) => {
                            self.conn.serial.status = ConnectionStatus::Error;
                            self.metrics.connect_failures += 1;
                            self.report_error(format!(
                                "Connect failed ({}): {}",
                                self.conn.active_conn, e
                            ));
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    self.conn.serial_connect_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.conn.serial_connect_in_progress = false;
                    self.conn.serial.status = ConnectionStatus::Error;
                    self.report_error("Serial connect worker disconnected unexpectedly");
                }
            }
        }

        if let Some(rx) = self.conn.port_scan_rx.take() {
            match rx.try_recv() {
                Ok(ports) => {
                    self.conn.port_scan_in_progress = false;
                    self.apply_scanned_ports(ports);
                }
                Err(TryRecvError::Empty) => {
                    self.conn.port_scan_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.conn.port_scan_in_progress = false;
                    self.report_error("Serial port scan worker disconnected unexpectedly");
                }
            }
        }

        if let Some(rx) = self.external.llm_result_rx.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.ui.llm_loading = false;
                    match result {
                        Ok(suggested) => {
                            self.metrics.llm_success += 1;
                            self.control.nn_suggested_kp = suggested.kp;
                            self.control.nn_suggested_ki = suggested.ki;
                            self.control.nn_suggested_kd = suggested.kd;
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
                    self.external.llm_result_rx = Some(rx);
                }
                Err(TryRecvError::Disconnected) => {
                    self.ui.llm_loading = false;
                    self.metrics.llm_failures += 1;
                    self.report_error("LLM worker disconnected unexpectedly");
                }
            }
        }

        self.apply_performance_profile();
        self.resource_status_dirty = true;
        self.refresh_resource_status();
    }

    pub fn start_mcp_server(&mut self) {
        let state = self.external.mcp_shared_state.clone();

        // 在 tokio 运行时中启动 MCP 服务器
        let handle = tokio::spawn(async move {
            if let Err(e) = mcp_server::start_mcp_server(state).await {
                tracing::error!("MCP server error: {:?}", e);
            }
        });

        self.external.mcp_server_handle = Some(handle);
        self.ui.mcp_running = true;
        self.metrics.mcp_startups += 1;
        self.status_message = "MCP server started".into();
    }

    pub fn stop_mcp_server(&mut self) {
        if let Some(handle) = self.external.mcp_server_handle.take() {
            handle.abort();
        }
        self.ui.mcp_running = false;
        self.status_message = "MCP server stopped".into();
    }

    pub fn toggle_mcp_server(&mut self) {
        if self.ui.mcp_running {
            self.stop_mcp_server();
        } else {
            self.start_mcp_server();
        }
    }

    pub fn reset_counters(&mut self) {
        self.conn.reset_counters();
    }

    pub fn sync_mcp_state(&mut self) {
        // 使用 try_lock 避免阻塞 GUI 线程
        let mut s = match self.external.mcp_shared_state.try_lock() {
            Ok(s) => s,
            Err(_) => return, // 如果锁被占用，跳过本次同步
        };

        // 同步 PID 参数
        let pid = self.control.pid();
        s.kp = pid.kp;
        s.ki = pid.ki;
        s.kd = pid.kd;
        s.setpoint = pid.setpoint;

        // 同步当前状态
        s.current_state = self.control.current_state.clone();

        // 同步状态历史（最近 500 条）
        let history = &self.control.state_history;
        let start = if history.len() > 500 {
            history.len() - 500
        } else {
            0
        };
        s.state_history = history[start..].to_vec();

        // 同步解析的数据包（最近 200 条）
        // ... 类似逻辑

        // 同步 AI 建议参数
        s.suggested_kp = self.control.nn_suggested_kp;
        s.suggested_ki = self.control.nn_suggested_ki;
        s.suggested_kd = self.control.nn_suggested_kd;
        s.status = self.status_message.clone();
    }

    // Chart data
}

#[cfg(test)]
mod tests {
    use super::{parse_port, parse_version_triplet, valid_http_url, AppState, LOG_FILE_MAX_BYTES};
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
        s1.conn.active_conn = crate::models::ConnectionType::Tcp;
        s1.save_user_preferences_to_path(&path);

        let mut s2 = AppState::new();
        s2.load_user_preferences_from_path(&path);

        assert_eq!(s2.ui.tcp_host, "192.168.1.10");
        assert_eq!(s2.ui.tcp_port_text, "12345");
        assert_eq!(s2.ui.mcp_token_text, "token-abc");
        assert_eq!(s2.conn.active_conn, crate::models::ConnectionType::Tcp);

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
        s1.ui.ui_scale_percent = 125;
        s1.ui.prefs_autosave_interval_sec = 9;
        s1.ui.update_channel = "preview-0.1".into();
        s1.ui.update_manifest_url = "https://example.com/manifest.json".into();
        s1.ui.update_check_timeout_ms = 2600;
        s1.save_user_preferences_to_path(&path);

        let mut s2 = AppState::new();
        s2.load_user_preferences_from_path(&path);

        assert_eq!(s2.ui.motion_level_idx, 3);
        assert_eq!(s2.ui.usb_protocol_idx, 9);
        assert_eq!(s2.ui.usb_speed_idx, 4);
        assert_eq!(s2.ui.can_bitrate_idx, 8);
        assert!(!s2.ui.parser_auto_parse);
        assert_eq!(s2.ui.analysis_protocol_idx, 7);
        assert!(s2.ui.analysis_filter_info);
        assert_eq!(s2.ui.ui_scale_percent, 125);
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
        assert_eq!(s.ui.ui_scale_percent, 150);
        assert_eq!(s.ui.tcp_host, "127.0.0.1");
    }

    #[test]
    fn test_can_status_reflects_running_state() {
        let mut s = AppState::new();
        s.conn.active_conn = crate::models::ConnectionType::Can;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Disconnected
        );
        s.conn.can.is_running = true;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connected
        );
        assert!(s.is_any_connected());
    }

    #[test]
    fn test_connect_disconnect_can_channel() {
        let mut s = AppState::new();
        s.conn.active_conn = crate::models::ConnectionType::CanFd;
        s.connect_active().unwrap();
        assert!(s.conn.can.is_running);
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connected
        );
        s.disconnect_active();
        assert!(!s.conn.can.is_running);
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
        s.conn.active_conn = crate::models::ConnectionType::Serial;

        s.conn.serial.status = crate::models::ConnectionStatus::Connecting;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connecting
        );
        assert_eq!(s.link_health_text(), "Connecting");

        s.conn.serial.status = crate::models::ConnectionStatus::Error;
        assert_eq!(s.active_status(), crate::models::ConnectionStatus::Error);
        assert_eq!(s.link_health_text(), "Error");

        s.conn.serial.status = crate::models::ConnectionStatus::Disconnected;
        s.conn.pause_auto_reconnect();
        assert_eq!(s.link_health_text(), "Offline (manual)");
    }

    #[test]
    fn test_serial_is_any_connected_reflects_serial_service() {
        let mut s = AppState::new();
        s.conn.serial.status = crate::models::ConnectionStatus::Disconnected;
        assert!(!s.is_any_connected());

        s.conn.serial.status = crate::models::ConnectionStatus::Connected;
        assert!(!s.is_any_connected());

        s.conn.serial.status = crate::models::ConnectionStatus::Connected;
        s.conn.active_conn = crate::models::ConnectionType::Serial;
        assert_eq!(
            s.active_status(),
            crate::models::ConnectionStatus::Connected
        );
    }

    #[test]
    fn test_serial_auto_reconnect_throttled_by_interval() {
        let mut s = AppState::new();
        s.conn.active_conn = crate::models::ConnectionType::Serial;
        s.ui.auto_reconnect_enabled = true;
        s.ui.auto_reconnect_interval_ms = 3000;
        s.conn.arm_auto_reconnect();
        s.conn.resume_auto_reconnect();
        s.conn.serial.config.port_name = "COM1".into();
        s.conn.port_scan_in_progress = false;

        let before_attempts = s.metrics.connect_attempts;
        s.maintain_connection();
        let after_first = s.metrics.connect_attempts;
        assert_eq!(after_first, before_attempts + 1);
        assert!(s.conn.next_reconnect_at.is_some());

        s.maintain_connection();
        let after_second = s.metrics.connect_attempts;
        assert_eq!(after_second, after_first);
    }

    #[test]
    fn test_serial_auto_reconnect_respects_manual_pause() {
        let mut s = AppState::new();
        s.conn.active_conn = crate::models::ConnectionType::Serial;
        s.ui.auto_reconnect_enabled = true;
        s.conn.pause_auto_reconnect();
        s.conn.next_reconnect_at = Some(std::time::Instant::now());

        let before_attempts = s.metrics.connect_attempts;
        s.maintain_connection();

        assert_eq!(s.metrics.connect_attempts, before_attempts);
        assert!(s.conn.next_reconnect_at.is_none());
    }

    #[test]
    fn test_serial_auto_reconnect_requires_prior_success() {
        let mut s = AppState::new();
        s.conn.active_conn = crate::models::ConnectionType::Serial;
        s.ui.auto_reconnect_enabled = true;
        s.conn.resume_auto_reconnect();
        s.conn.serial.config.port_name = "COM1".into();

        let before_attempts = s.metrics.connect_attempts;
        s.maintain_connection();
        assert_eq!(s.metrics.connect_attempts, before_attempts);
        assert!(s.conn.next_reconnect_at.is_none());

        s.conn.arm_auto_reconnect();
        s.maintain_connection();
        assert_eq!(s.metrics.connect_attempts, before_attempts + 1);
    }

    #[test]
    fn test_connect_active_requires_manual_port_refresh() {
        let mut s = AppState::new();
        s.conn.active_conn = crate::models::ConnectionType::Serial;
        s.conn.serial.config.port_name.clear();
        s.conn.port_scan_in_progress = false;

        let err = s.connect_active().unwrap_err();
        assert!(err.to_string().contains("Refresh ports"));
        assert!(!s.conn.port_scan_in_progress);
    }

    #[test]
    fn test_manual_disconnect_disarms_reconnect() {
        let mut s = AppState::new();
        s.conn.arm_auto_reconnect();
        s.conn.resume_auto_reconnect();
        s.conn.next_reconnect_at = Some(std::time::Instant::now());

        s.disconnect_active();

        assert!(!s.conn.reconnect_armed());
        assert!(s.conn.reconnect_paused());
        assert!(s.conn.next_reconnect_at.is_none());
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

    #[test]
    fn test_valid_http_url_valid() {
        assert!(valid_http_url("https://example.com").is_some());
        assert!(valid_http_url("http://localhost:3000").is_some());
    }

    #[test]
    fn test_valid_http_url_invalid() {
        assert!(valid_http_url("not-a-url").is_none());
        assert!(valid_http_url("ftp://example.com").is_none());
    }

    #[test]
    fn test_valid_http_url_with_whitespace() {
        assert!(valid_http_url("  https://example.com  ").is_some());
    }

    #[test]
    fn test_csv_escape_simple() {
        assert_eq!(AppState::csv_escape("hello"), "\"hello\"");
    }

    #[test]
    fn test_csv_escape_with_comma() {
        assert_eq!(AppState::csv_escape("a,b"), "\"a,b\"");
    }

    #[test]
    fn test_csv_escape_with_quotes() {
        assert_eq!(AppState::csv_escape("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn test_trim_vec_within_limit() {
        let mut v = vec![1, 2, 3];
        AppState::trim_vec(&mut v, 5);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_trim_vec_exceeds_limit() {
        let mut v = vec![1, 2, 3, 4, 5, 6];
        AppState::trim_vec(&mut v, 3);
        assert_eq!(v, vec![4, 5, 6]);
    }

    #[test]
    fn test_trim_vec_exact_limit() {
        let mut v = vec![1, 2, 3];
        AppState::trim_vec(&mut v, 3);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_trim_vec_one_over() {
        let mut v = vec![1, 2, 3, 4];
        AppState::trim_vec(&mut v, 3);
        assert_eq!(v, vec![2, 3, 4]);
    }

    #[test]
    fn test_parse_version_triplet_with_v_prefix() {
        let result = parse_version_triplet("v1.2.3");
        assert!(result.is_some());
        let v = result.unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_version_triplet_with_suffix() {
        let result = parse_version_triplet("1.2.3-beta.1");
        assert!(result.is_some());
        let v = result.unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_version_triplet_invalid() {
        let result = parse_version_triplet("abc");
        assert!(result.is_none());
    }
}
