use serde::{Deserialize, Serialize};
use std::fmt;

// ═══════════════════════════════════════════════════════════════
// 连接类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Serial,
    Tcp,
    Udp,
    Can,
    CanFd,
    ModbusRtu,
    ModbusTcp,
    Usb,
}

impl ConnectionType {
    pub fn all() -> &'static [ConnectionType] {
        &[
            Self::Serial,
            Self::Tcp,
            Self::Udp,
            Self::Can,
            Self::CanFd,
            Self::ModbusRtu,
            Self::ModbusTcp,
            Self::Usb,
        ]
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Serial => "📟",
            Self::Tcp => "🌐",
            Self::Udp => "📡",
            Self::Can => "🔧",
            Self::CanFd => "⚙",
            Self::ModbusRtu => "🏭",
            Self::ModbusTcp => "🏗",
            Self::Usb => "🔌",
        }
    }
}

impl fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial => write!(f, "Serial"),
            Self::Tcp => write!(f, "TCP"),
            Self::Udp => write!(f, "UDP"),
            Self::Can => write!(f, "CAN 2.0"),
            Self::CanFd => write!(f, "CAN FD"),
            Self::ModbusRtu => write!(f, "Modbus RTU"),
            Self::ModbusTcp => write!(f, "Modbus TCP"),
            Self::Usb => write!(f, "USB"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 连接状态
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Error)
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Connected => (46, 160, 67),
            Self::Connecting => (255, 165, 0),
            Self::Error => (218, 54, 51),
            Self::Disconnected => (128, 128, 128),
        }
    }
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting..."),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 串口配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
    pub parity: String,
    pub flow_control: String,
    pub timeout_ms: u64,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            port_name: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: "None".into(),
            flow_control: "None".into(),
            timeout_ms: 10,
        }
    }
}

impl SerialConfig {
    pub fn baud_rates() -> &'static [u32] {
        &[
            300, 600, 1200, 2400, 4800, 9600, 14400, 19200, 28800, 38400, 57600, 76800, 115200,
            230400, 460800, 576000, 921600, 1000000, 1500000, 2000000, 3000000,
        ]
    }

    pub fn parity_options() -> &'static [&'static str] {
        &["None", "Odd", "Even"]
    }

    pub fn data_bits_options() -> &'static [u8] {
        &[5, 6, 7, 8]
    }

    pub fn stop_bits_options() -> &'static [u8] {
        &[1, 2]
    }

    pub fn flow_control_options() -> &'static [&'static str] {
        &["None", "Hardware (RTS/CTS)", "Software (XON/XOFF)"]
    }
}

// ═══════════════════════════════════════════════════════════════
// TCP 配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
    pub is_server: bool,
    pub timeout_ms: u64,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            is_server: false,
            timeout_ms: 5000,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// UDP 配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpConfig {
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            local_host: "0.0.0.0".into(),
            local_port: 9000,
            remote_host: "127.0.0.1".into(),
            remote_port: 9001,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// CAN 配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanConfig {
    pub interface: String,
    pub bitrate: u32,
    pub fd_enabled: bool,
    pub data_bitrate: u32,
    pub sample_point: f32,      // 采样点 (0.0 ~ 1.0)
    pub data_sample_point: f32, // CAN FD 数据段采样点
    pub sjw: u8,                // 同步跳转宽度
    pub data_sjw: u8,           // CAN FD 数据段 SJW
    pub termination: bool,      // 终端电阻
    pub listen_only: bool,      // 只听模式
    pub loopback: bool,         // 回环模式
    pub auto_retransmit: bool,  // 自动重传
    pub error_reporting: bool,  // 错误报告
}

impl Default for CanConfig {
    fn default() -> Self {
        Self {
            interface: "vcan0".into(),
            bitrate: 500_000,
            fd_enabled: false,
            data_bitrate: 2_000_000,
            sample_point: 0.875,
            data_sample_point: 0.750,
            sjw: 1,
            data_sjw: 1,
            termination: true,
            listen_only: false,
            loopback: false,
            auto_retransmit: true,
            error_reporting: true,
        }
    }
}

impl CanConfig {
    /// Standard CAN 2.0 arbitration baud rates
    pub fn standard_bitrates() -> &'static [(u32, &'static str)] {
        &[
            (10_000, "10 kbps"),
            (20_000, "20 kbps"),
            (50_000, "50 kbps"),
            (100_000, "100 kbps"),
            (125_000, "125 kbps"),
            (250_000, "250 kbps"),
            (500_000, "500 kbps"),
            (800_000, "800 kbps"),
            (1_000_000, "1 Mbps"),
        ]
    }

    /// CAN FD data segment baud rates
    pub fn fd_data_bitrates() -> &'static [(u32, &'static str)] {
        &[
            (500_000, "500 kbps"),
            (1_000_000, "1 Mbps"),
            (2_000_000, "2 Mbps"),
            (4_000_000, "4 Mbps"),
            (5_000_000, "5 Mbps"),
            (8_000_000, "8 Mbps"),
            (10_000_000, "10 Mbps"),
            (12_000_000, "12 Mbps"),
        ]
    }

    /// Standard sample point options
    pub fn sample_point_options() -> &'static [(f32, &'static str)] {
        &[
            (0.750, "75.0%"),
            (0.800, "80.0%"),
            (0.833, "83.3%"),
            (0.857, "85.7%"),
            (0.875, "87.5%"),
        ]
    }

    /// SJW options
    pub fn sjw_options() -> &'static [u8] {
        &[1, 2, 3, 4]
    }
}

// ═══════════════════════════════════════════════════════════════
// USB 协议类型
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbProtocol {
    CdcAcm,  // 通信设备类 - 虚拟串口
    Hid,     // 人机接口设备
    Msc,     // 大容量存储
    Midi,    // MIDI 音频设备
    Audio,   // USB Audio
    Video,   // USB Video Class (UVC)
    CdcEcm,  // 以太网控制模型
    CdcNcm,  // 网络控制模型
    Dfu,     // 设备固件升级
    Vendor,  // 厂商自定义
    Printer, // 打印机
    Hub,     // USB Hub
}

impl UsbProtocol {
    pub fn all() -> &'static [UsbProtocol] {
        &[
            Self::CdcAcm,
            Self::Hid,
            Self::Msc,
            Self::Midi,
            Self::Audio,
            Self::Video,
            Self::CdcEcm,
            Self::CdcNcm,
            Self::Dfu,
            Self::Vendor,
            Self::Printer,
            Self::Hub,
        ]
    }

    pub fn class_code(&self) -> u8 {
        match self {
            Self::CdcAcm | Self::CdcEcm | Self::CdcNcm => 0x02,
            Self::Hid => 0x03,
            Self::Msc => 0x08,
            Self::Midi | Self::Audio => 0x01,
            Self::Video => 0x0E,
            Self::Dfu => 0xFE,
            Self::Vendor => 0xFF,
            Self::Printer => 0x07,
            Self::Hub => 0x09,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::CdcAcm => "CDC ACM - Virtual COM Port / AT Modem",
            Self::Hid => "HID - Keyboard / Mouse / Gamepad / Custom",
            Self::Msc => "MSC - Mass Storage (USB Flash Drive)",
            Self::Midi => "MIDI - Musical Instrument",
            Self::Audio => "UAC - USB Audio Device",
            Self::Video => "UVC - USB Video Camera",
            Self::CdcEcm => "CDC ECM - Ethernet over USB",
            Self::CdcNcm => "CDC NCM - Network Control Model",
            Self::Dfu => "DFU - Device Firmware Upgrade",
            Self::Vendor => "Vendor Specific (Custom Protocol)",
            Self::Printer => "Printer Class",
            Self::Hub => "USB Hub Device",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::CdcAcm => "\u{1F4DF}",                // 📟
            Self::Hid => "\u{2328}",                    // ⌨
            Self::Msc => "\u{1F4BE}",                   // 💾
            Self::Midi => "\u{1F3B5}",                  // 🎵
            Self::Audio => "\u{1F50A}",                 // 🔊
            Self::Video => "\u{1F4F7}",                 // 📷
            Self::CdcEcm | Self::CdcNcm => "\u{1F310}", // 🌐
            Self::Dfu => "\u{2B06}",                    // ⬆
            Self::Vendor => "\u{1F527}",                // 🔧
            Self::Printer => "\u{1F5A8}",               // 🖨
            Self::Hub => "\u{1F500}",                   // 🔀
        }
    }

    /// Common USB speeds for this protocol
    pub fn typical_speeds(&self) -> &[&str] {
        match self {
            Self::CdcAcm => &["Full Speed (12 Mbps)", "High Speed (480 Mbps)"],
            Self::Hid => &["Low Speed (1.5 Mbps)", "Full Speed (12 Mbps)"],
            Self::Msc => &[
                "Full Speed (12 Mbps)",
                "High Speed (480 Mbps)",
                "SuperSpeed (5 Gbps)",
            ],
            Self::Video => &["High Speed (480 Mbps)", "SuperSpeed (5 Gbps)"],
            Self::Audio => &["Full Speed (12 Mbps)", "High Speed (480 Mbps)"],
            _ => &["Full Speed (12 Mbps)", "High Speed (480 Mbps)"],
        }
    }
}

impl fmt::Display for UsbProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CdcAcm => write!(f, "CDC ACM (Serial)"),
            Self::Hid => write!(f, "HID"),
            Self::Msc => write!(f, "Mass Storage"),
            Self::Midi => write!(f, "MIDI"),
            Self::Audio => write!(f, "Audio (UAC)"),
            Self::Video => write!(f, "Video (UVC)"),
            Self::CdcEcm => write!(f, "CDC ECM (Ethernet)"),
            Self::CdcNcm => write!(f, "CDC NCM (Network)"),
            Self::Dfu => write!(f, "DFU"),
            Self::Vendor => write!(f, "Vendor Specific"),
            Self::Printer => write!(f, "Printer"),
            Self::Hub => write!(f, "Hub"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// USB 配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbConfig {
    pub protocol: UsbProtocol,
    pub vid: u16,
    pub pid: u16,
    pub speed: UsbSpeed,
    pub interface_num: u8,
    pub endpoint_in: u8,
    pub endpoint_out: u8,
    pub max_packet_size: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbSpeed {
    LowSpeed,
    FullSpeed,
    HighSpeed,
    SuperSpeed,
    SuperSpeedPlus,
}

impl UsbSpeed {
    pub fn all() -> &'static [UsbSpeed] {
        &[
            Self::LowSpeed,
            Self::FullSpeed,
            Self::HighSpeed,
            Self::SuperSpeed,
            Self::SuperSpeedPlus,
        ]
    }

    pub fn bandwidth(&self) -> &str {
        match self {
            Self::LowSpeed => "1.5 Mbps",
            Self::FullSpeed => "12 Mbps",
            Self::HighSpeed => "480 Mbps",
            Self::SuperSpeed => "5 Gbps",
            Self::SuperSpeedPlus => "10+ Gbps",
        }
    }
}

impl fmt::Display for UsbSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LowSpeed => write!(f, "Low Speed (1.5 Mbps)"),
            Self::FullSpeed => write!(f, "Full Speed (12 Mbps)"),
            Self::HighSpeed => write!(f, "High Speed (480 Mbps)"),
            Self::SuperSpeed => write!(f, "SuperSpeed (5 Gbps)"),
            Self::SuperSpeedPlus => write!(f, "SuperSpeed+ (10 Gbps)"),
        }
    }
}

impl Default for UsbConfig {
    fn default() -> Self {
        Self {
            protocol: UsbProtocol::CdcAcm,
            vid: 0x0483, // STMicroelectronics VID
            pid: 0x5740, // Virtual COM Port PID
            speed: UsbSpeed::FullSpeed,
            interface_num: 0,
            endpoint_in: 0x81,
            endpoint_out: 0x01,
            max_packet_size: 64,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Modbus 配置
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusConfig {
    pub slave_id: u8,
    pub is_tcp: bool,
    // RTU 串口设置复用 SerialConfig
    // TCP 网络设置
    pub tcp_host: String,
    pub tcp_port: u16,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            slave_id: 1,
            is_tcp: false,
            tcp_host: "127.0.0.1".into(),
            tcp_port: 502,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_type_all() {
        assert_eq!(ConnectionType::all().len(), 8);
    }

    #[test]
    fn test_connection_type_display() {
        assert_eq!(format!("{}", ConnectionType::Serial), "Serial");
        assert_eq!(format!("{}", ConnectionType::Tcp), "TCP");
        assert_eq!(format!("{}", ConnectionType::ModbusTcp), "Modbus TCP");
    }

    #[test]
    fn test_connection_status_default() {
        let s = ConnectionStatus::default();
        assert_eq!(s, ConnectionStatus::Disconnected);
        assert!(s.is_disconnected());
        assert!(!s.is_connected());
    }

    #[test]
    fn test_connection_status_colors() {
        let (r, g, b) = ConnectionStatus::Connected.color_rgb();
        assert_eq!((r, g, b), (46, 160, 67)); // green
        let (r, g, b) = ConnectionStatus::Error.color_rgb();
        assert_eq!((r, g, b), (218, 54, 51)); // red
    }

    #[test]
    fn test_serial_config_default() {
        let cfg = SerialConfig::default();
        assert_eq!(cfg.baud_rate, 115200);
        assert_eq!(cfg.data_bits, 8);
        assert_eq!(cfg.stop_bits, 1);
        assert_eq!(cfg.parity, "None");
        assert_eq!(cfg.flow_control, "None");
        assert_eq!(cfg.timeout_ms, 10);
    }

    #[test]
    fn test_serial_config_baud_rates() {
        let rates = SerialConfig::baud_rates();
        assert!(rates.contains(&9600));
        assert!(rates.contains(&115200));
        assert!(rates.contains(&921600));
    }

    #[test]
    fn test_serial_config_options() {
        assert!(SerialConfig::parity_options().contains(&"None"));
        assert!(SerialConfig::parity_options().contains(&"Odd"));
        assert!(SerialConfig::data_bits_options().contains(&8));
        assert!(SerialConfig::stop_bits_options().contains(&1));
    }

    #[test]
    fn test_tcp_config_default() {
        let cfg = TcpConfig::default();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 8080);
        assert!(!cfg.is_server);
    }

    #[test]
    fn test_udp_config_default() {
        let cfg = UdpConfig::default();
        assert_eq!(cfg.local_host, "0.0.0.0");
        assert_eq!(cfg.local_port, 9000);
        assert_eq!(cfg.remote_port, 9001);
    }

    #[test]
    fn test_connection_type_icons_non_empty() {
        for ct in ConnectionType::all() {
            assert!(!ct.icon().is_empty(), "{:?} should have an icon", ct);
        }
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(
            format!("{}", ConnectionStatus::Disconnected),
            "Disconnected"
        );
        assert_eq!(format!("{}", ConnectionStatus::Connected), "Connected");
        assert_eq!(format!("{}", ConnectionStatus::Connecting), "Connecting...");
        assert_eq!(format!("{}", ConnectionStatus::Error), "Error");
    }

    // ─── CAN 配置 ──────────────────────
    #[test]
    fn test_can_config_default() {
        let cfg = CanConfig::default();
        assert_eq!(cfg.bitrate, 500_000);
        assert_eq!(cfg.data_bitrate, 2_000_000);
        assert!(!cfg.fd_enabled);
        assert!((cfg.sample_point - 0.875).abs() < 0.001);
        assert!(cfg.termination);
        assert!(cfg.auto_retransmit);
    }

    #[test]
    fn test_can_standard_bitrates() {
        let rates = CanConfig::standard_bitrates();
        assert!(rates.len() >= 9);
        assert!(rates.iter().any(|(r, _)| *r == 125_000));
        assert!(rates.iter().any(|(r, _)| *r == 500_000));
        assert!(rates.iter().any(|(r, _)| *r == 1_000_000));
    }

    #[test]
    fn test_can_fd_data_bitrates() {
        let rates = CanConfig::fd_data_bitrates();
        assert!(rates.len() >= 8);
        assert!(rates.iter().any(|(r, _)| *r == 2_000_000));
        assert!(rates.iter().any(|(r, _)| *r == 5_000_000));
        assert!(rates.iter().any(|(r, _)| *r == 8_000_000));
    }

    #[test]
    fn test_can_sample_point_options() {
        let opts = CanConfig::sample_point_options();
        assert!(opts.len() >= 5);
        for &(sp, _) in opts {
            assert!((0.5..=1.0).contains(&sp));
        }
    }

    #[test]
    fn test_can_sjw_options() {
        let opts = CanConfig::sjw_options();
        assert!(opts.contains(&1));
        assert!(opts.contains(&4));
    }

    // ─── USB 协议 ──────────────────────
    #[test]
    fn test_usb_protocol_all() {
        assert_eq!(UsbProtocol::all().len(), 12);
    }

    #[test]
    fn test_usb_protocol_display() {
        assert!(format!("{}", UsbProtocol::CdcAcm).contains("CDC"));
        assert!(format!("{}", UsbProtocol::Hid).contains("HID"));
        assert!(format!("{}", UsbProtocol::Msc).contains("Mass Storage"));
    }

    #[test]
    fn test_usb_protocol_class_codes() {
        assert_eq!(UsbProtocol::CdcAcm.class_code(), 0x02);
        assert_eq!(UsbProtocol::Hid.class_code(), 0x03);
        assert_eq!(UsbProtocol::Msc.class_code(), 0x08);
        assert_eq!(UsbProtocol::Vendor.class_code(), 0xFF);
    }

    #[test]
    fn test_usb_protocol_descriptions_non_empty() {
        for p in UsbProtocol::all() {
            assert!(
                !p.description().is_empty(),
                "{:?} should have description",
                p
            );
        }
    }

    #[test]
    fn test_usb_protocol_icons_non_empty() {
        for p in UsbProtocol::all() {
            assert!(!p.icon().is_empty(), "{:?} should have icon", p);
        }
    }

    #[test]
    fn test_usb_speeds() {
        assert_eq!(UsbSpeed::all().len(), 5);
        assert_eq!(UsbSpeed::FullSpeed.bandwidth(), "12 Mbps");
        assert_eq!(UsbSpeed::HighSpeed.bandwidth(), "480 Mbps");
    }

    #[test]
    fn test_usb_config_default() {
        let cfg = UsbConfig::default();
        assert_eq!(cfg.protocol, UsbProtocol::CdcAcm);
        assert_eq!(cfg.vid, 0x0483);
        assert_eq!(cfg.speed, UsbSpeed::FullSpeed);
        assert_eq!(cfg.max_packet_size, 64);
    }

    #[test]
    fn test_usb_typical_speeds() {
        for p in UsbProtocol::all() {
            assert!(!p.typical_speeds().is_empty(), "{:?} should have speeds", p);
        }
    }
}
