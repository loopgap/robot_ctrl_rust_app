use crate::models::*;
use crate::services::*;
use std::sync::mpsc;
use std::time::Instant;

/// 管理所有连接服务（串口、TCP、UDP、CAN）的核心结构体
///
/// 负责连接生命周期管理、自动重连、数据收发统计。
/// 通过 `active_conn` 字段跟踪当前活跃的连接类型。
pub struct ConnectionManager {
    pub serial: SerialService,
    pub tcp: TcpService,
    pub udp: UdpService,
    pub can: CanService,
    pub active_conn: ConnectionType,
    pub available_ports: Vec<String>,
    pub usb_config: UsbConfig,
    pub reconnect_armed: bool,
    pub reconnect_paused_by_user: bool,
    pub next_reconnect_at: Option<Instant>,
    pub last_rx_instant: Option<Instant>,

    pub port_scan_rx: Option<mpsc::Receiver<Vec<String>>>,
    pub serial_connect_rx: Option<mpsc::Receiver<Result<SerialService, String>>>,
    pub serial_connect_in_progress: bool,
    pub port_scan_in_progress: bool,
    pub last_io_poll_instant: Instant,
    pub last_connection_check_instant: Instant,
    pub last_log_flush_instant: Instant,
    pub last_port_scan_request_at: Option<Instant>,
    pub pending_log_lines: Vec<String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            serial: SerialService::new(),
            tcp: TcpService::new(),
            udp: UdpService::new(),
            can: CanService::new(),
            active_conn: ConnectionType::Serial,
            available_ports: Vec::new(),
            usb_config: UsbConfig::default(),
            reconnect_armed: false,
            reconnect_paused_by_user: false,
            next_reconnect_at: None,
            last_rx_instant: None,

            port_scan_rx: None,
            serial_connect_rx: None,
            serial_connect_in_progress: false,
            port_scan_in_progress: false,
            last_io_poll_instant: Instant::now(),
            last_connection_check_instant: Instant::now(),
            last_log_flush_instant: Instant::now(),
            last_port_scan_request_at: None,
            pending_log_lines: Vec::new(),
        }
    }

    pub fn set_active_connection(&mut self, conn: ConnectionType) {
        self.active_conn = conn;
    }

    pub fn is_any_connected(&self) -> bool {
        self.serial.is_connected()
            || self.tcp.is_connected()
            || self.udp.is_connected()
            || self.can.is_running
    }

    pub fn active_status(&self) -> ConnectionStatus {
        match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                self.serial.status
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => self.tcp.status,
            ConnectionType::Udp => self.udp.status,
            ConnectionType::Can | ConnectionType::CanFd => {
                if self.can.is_running {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                }
            }
        }
    }

    pub fn last_comm(&self) -> &str {
        match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                &self.serial.last_comm
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => &self.tcp.last_comm,
            ConnectionType::Udp => &self.udp.last_comm,
            ConnectionType::Can | ConnectionType::CanFd => {
                if self.can.frames.is_empty() {
                    "No CAN frame yet"
                } else {
                    "CAN bus active"
                }
            }
        }
    }

    pub fn link_health_text(&self) -> String {
        let status = self.active_status();
        match status {
            ConnectionStatus::Connected => {
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
            ConnectionStatus::Connecting => "Connecting".into(),
            ConnectionStatus::Error => "Error".into(),
            ConnectionStatus::Disconnected => {
                if self.reconnect_paused_by_user {
                    "Offline (manual)".into()
                } else {
                    "Offline".into()
                }
            }
        }
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

    pub fn send_data(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self.active_conn {
            ConnectionType::Serial | ConnectionType::Usb | ConnectionType::ModbusRtu => {
                self.serial.send_data(data)?;
            }
            ConnectionType::Tcp | ConnectionType::ModbusTcp => {
                self.tcp.send_data(data)?;
            }
            ConnectionType::Udp => {
                self.udp.send_default(data)?;
            }
            _ => {
                return Err(anyhow::anyhow!("No active connection"));
            }
        }
        Ok(())
    }

    pub fn arm_auto_reconnect(&mut self) {
        self.reconnect_armed = true;
    }

    pub fn pause_auto_reconnect(&mut self) {
        self.reconnect_paused_by_user = true;
    }

    pub fn resume_auto_reconnect(&mut self) {
        self.reconnect_paused_by_user = false;
    }

    pub fn reconnect_armed(&self) -> bool {
        self.reconnect_armed
    }

    pub fn reconnect_paused(&self) -> bool {
        self.reconnect_paused_by_user
    }

    pub fn clear_reconnect_schedule(&mut self) {
        self.reconnect_armed = false;
        self.next_reconnect_at = None;
    }

    pub fn reconnect_countdown_text(&self) -> String {
        if !self.reconnect_armed {
            return "N/A".to_string();
        }
        match self.next_reconnect_at {
            Some(next) => {
                let now = Instant::now();
                if next > now {
                    format!("{:.1}s", (next - now).as_secs_f32())
                } else {
                    "Due now".to_string()
                }
            }
            None => "N/A".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default_state() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.active_conn, ConnectionType::Serial);
        assert!(!cm.is_any_connected());
        assert!(!cm.reconnect_armed());
        assert!(cm.available_ports.is_empty());
    }

    #[test]
    fn test_active_status_disconnected() {
        let cm = ConnectionManager::new();
        assert!(matches!(cm.active_status(), ConnectionStatus::Disconnected));
    }

    #[test]
    fn test_set_active_connection() {
        let mut cm = ConnectionManager::new();
        cm.set_active_connection(ConnectionType::Tcp);
        assert_eq!(cm.active_conn, ConnectionType::Tcp);
    }

    #[test]
    fn test_total_bytes_start_at_zero() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.total_bytes_sent(), 0);
        assert_eq!(cm.total_bytes_received(), 0);
        assert_eq!(cm.total_errors(), 0);
    }

    #[test]
    fn test_reset_counters() {
        let mut cm = ConnectionManager::new();
        cm.serial.bytes_sent = 100;
        cm.reset_counters();
        assert_eq!(cm.total_bytes_sent(), 0);
    }

    #[test]
    fn test_reconnect_lifecycle() {
        let mut cm = ConnectionManager::new();
        assert!(!cm.reconnect_armed());
        cm.arm_auto_reconnect();
        assert!(cm.reconnect_armed());
        cm.pause_auto_reconnect();
        assert!(cm.reconnect_paused());
        cm.resume_auto_reconnect();
        assert!(!cm.reconnect_paused());
        cm.clear_reconnect_schedule();
        assert!(!cm.reconnect_armed());
    }

    #[test]
    fn test_send_data_when_disconnected() {
        let mut cm = ConnectionManager::new();
        assert!(cm.send_data(&[0x01]).is_err());
    }

    #[test]
    fn test_link_health_text() {
        let cm = ConnectionManager::new();
        let text = cm.link_health_text();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_reconnect_countdown_not_armed() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.reconnect_countdown_text(), "N/A");
    }

    #[test]
    fn test_total_bytes_sent_received() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.total_bytes_sent(), 0);
        assert_eq!(cm.total_bytes_received(), 0);
    }

    #[test]
    fn test_total_errors() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.total_errors(), 0);
    }

    #[test]
    fn test_link_health_text_disconnected() {
        let cm = ConnectionManager::new();
        let text = cm.link_health_text();
        assert!(!text.is_empty());
    }

    #[test]
    fn test_reconnect_lifecycle_detailed() {
        let mut cm = ConnectionManager::new();
        assert!(!cm.reconnect_armed);
        cm.arm_auto_reconnect();
        assert!(cm.reconnect_armed);
        cm.clear_reconnect_schedule();
        assert!(!cm.reconnect_armed);
    }

    #[test]
    fn test_pause_resume_reconnect() {
        let mut cm = ConnectionManager::new();
        cm.arm_auto_reconnect();
        cm.pause_auto_reconnect();
        assert!(cm.reconnect_paused_by_user);
        cm.resume_auto_reconnect();
        assert!(!cm.reconnect_paused_by_user);
    }

    #[test]
    fn test_active_conn_default() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.active_conn, crate::models::ConnectionType::Serial);
    }
}
