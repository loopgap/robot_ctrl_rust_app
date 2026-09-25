use serde::{Deserialize, Serialize};

use super::DisplayMode;
use crate::i18n::Language;
use crate::models::{ConnectionType, SerialConfig};

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

pub fn parse_port(text: &str, label: &str) -> Result<u16, String> {
    let port: u16 = text
        .trim()
        .parse()
        .map_err(|_| format!("{} must be 1-65535", label))?;
    if port == 0 {
        return Err(format!("{} must be 1-65535", label));
    }
    Ok(port)
}

pub const DEFAULT_UPDATE_DOC_URL: &str =
    "https://github.com/loopgap/robot_ctrl_rust_app/blob/main/docs/src/README.md";
pub const DEFAULT_UPDATE_MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/example/robot_control_rust/main/update-manifest.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionTriplet {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct UpdateManifest {
    pub latest_version: String,
    pub channel: String,
    pub notes_url: String,
    pub min_supported_version: String,
}

pub fn parse_version_triplet(text: &str) -> Option<VersionTriplet> {
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

pub fn path_to_file_url(path: &std::path::Path) -> String {
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

pub fn resolve_local_help_url() -> Option<String> {
    use std::path::PathBuf;
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

pub fn valid_http_url(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Some(trimmed.to_string())
    } else {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserPreferences {
    pub schema_version: u32,
    pub language: Language,
    pub dark_mode: bool,
    #[serde(default)]
    pub high_contrast: bool,
    pub sidebar_expanded: bool,
    pub motion_level_idx: usize,
    pub active_tab_idx: usize,
    pub parser_auto_parse: bool,
    pub display_mode: DisplayMode,
    pub llm_api_url: String,
    pub llm_model_name: String,
    pub mcp_port_text: String,
    pub mcp_token_text: String,
    pub active_conn: ConnectionType,
    pub serial_config: SerialConfig,
    pub tcp_host: String,
    pub tcp_port_text: String,
    pub tcp_is_server: bool,
    pub udp_local_port_text: String,
    pub udp_remote_host: String,
    pub udp_remote_port_text: String,
    pub auto_newline: bool,
    pub auto_reconnect_enabled: bool,
    pub auto_reconnect_interval_ms: u32,
    pub quick_cmd_1: String,
    pub quick_cmd_2: String,
    pub quick_cmd_3: String,
    pub send_hex: bool,
    pub auto_scroll: bool,
    pub send_with_newline: bool,
    pub newline_type: String,
    pub repeat_send: bool,
    pub repeat_interval_ms: u32,
    pub can_id_text: String,
    pub can_data_text: String,
    pub can_extended: bool,
    pub can_fd: bool,
    pub can_bitrate_idx: usize,
    pub can_data_bitrate_idx: usize,
    pub can_sample_point_idx: usize,
    pub can_data_sample_point_idx: usize,
    pub can_sjw_idx: usize,
    pub can_data_sjw_idx: usize,
    pub usb_protocol_idx: usize,
    pub usb_speed_idx: usize,
    pub usb_vid_text: String,
    pub usb_pid_text: String,
    pub packet_template_idx: usize,
    pub parser_enabled: bool,
    pub parser_template_idx: usize,
    pub packet_builder_tab: usize,
    pub analysis_protocol_idx: usize,
    pub analysis_filter_tx: bool,
    pub analysis_filter_rx: bool,
    pub analysis_filter_info: bool,
    pub llm_temperature_text: String,
    pub ui_scale_percent: u32,
    pub prefs_autosave_interval_sec: u32,
    pub update_channel: String,
    pub update_manifest_url: String,
    pub update_check_timeout_ms: u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_triplet_parse_basic() {
        let v = parse_version_triplet("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn version_triplet_parse_v_prefix() {
        let v = parse_version_triplet("v10.20.30").unwrap();
        assert_eq!(v.major, 10);
    }

    #[test]
    fn version_triplet_parse_suffix() {
        let v = parse_version_triplet("1.0.0-beta").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn version_triplet_invalid() {
        assert!(parse_version_triplet("not-a-version").is_none());
        assert!(parse_version_triplet("").is_none());
        assert!(parse_version_triplet("1.2").is_none());
    }

    #[test]
    fn valid_http_url_accepts() {
        assert!(valid_http_url("https://example.com").is_some());
        assert!(valid_http_url("http://localhost").is_some());
    }

    #[test]
    fn valid_http_url_rejects() {
        assert!(valid_http_url("ftp://x").is_none());
        assert!(valid_http_url("").is_none());
        assert!(valid_http_url("not a url").is_none());
    }

    #[test]
    fn parse_port_valid() {
        assert_eq!(parse_port("8080", "port").unwrap(), 8080);
    }

    #[test]
    fn parse_port_zero() {
        assert!(parse_port("0", "port").is_err());
    }

    #[test]
    fn parse_port_overflow() {
        assert!(parse_port("99999", "port").is_err());
    }

    #[test]
    fn user_preferences_default_schema() {
        let p = UserPreferences::default();
        assert_eq!(p.schema_version, 2);
    }

    #[test]
    fn user_preferences_serde_roundtrip() {
        let p = UserPreferences::default();
        let json = serde_json::to_string(&p).unwrap();
        let restored: UserPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.schema_version, p.schema_version);
        assert_eq!(restored.dark_mode, p.dark_mode);
    }
}
