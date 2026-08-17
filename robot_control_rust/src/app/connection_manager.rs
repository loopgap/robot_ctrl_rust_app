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
    /// Number of consecutive reconnect attempts since last successful connection.
    pub reconnect_attempts: u32,
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
            reconnect_attempts: 0,
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
            ConnectionStatus::HardwareFault => "HW Fault".into(),
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
        self.reconnect_attempts = 0;
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
    use std::time::Duration;

    #[test]
    fn test_new_default_state() {
        let cm = ConnectionManager::new();
        assert_eq!(cm.active_conn, ConnectionType::Serial);
        assert!(!cm.is_any_connected());
        assert!(!cm.reconnect_armed());
        assert!(cm.available_ports.is_empty());
        assert!(!cm.reconnect_paused_by_user);
        assert!(cm.next_reconnect_at.is_none());
        assert!(cm.last_rx_instant.is_none());
        assert_eq!(cm.reconnect_attempts, 0);
        assert!(!cm.serial_connect_in_progress);
        assert!(!cm.port_scan_in_progress);
    }

    #[test]
    fn test_active_status_all_connection_types() {
        let mut cm = ConnectionManager::new();

        // Serial types → serial.status
        cm.set_active_connection(ConnectionType::Serial);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);
        cm.set_active_connection(ConnectionType::Usb);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);
        cm.set_active_connection(ConnectionType::ModbusRtu);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);

        // TCP types → tcp.status
        cm.set_active_connection(ConnectionType::Tcp);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);
        cm.set_active_connection(ConnectionType::ModbusTcp);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);

        // UDP → udp.status
        cm.set_active_connection(ConnectionType::Udp);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);

        // CAN types → based on is_running
        cm.set_active_connection(ConnectionType::Can);
        assert_eq!(cm.active_status(), ConnectionStatus::Disconnected);
        cm.can.is_running = true;
        assert_eq!(cm.active_status(), ConnectionStatus::Connected);
        cm.set_active_connection(ConnectionType::CanFd);
        assert_eq!(cm.active_status(), ConnectionStatus::Connected);
    }

    #[test]
    fn test_link_health_text_all_statuses() {
        let mut cm = ConnectionManager::new();

        // Disconnected (default)
        assert_eq!(cm.link_health_text(), "Offline");
        cm.reconnect_paused_by_user = true;
        assert_eq!(cm.link_health_text(), "Offline (manual)");
        cm.reconnect_paused_by_user = false;

        // Connecting
        cm.serial.status = ConnectionStatus::Connecting;
        assert_eq!(cm.link_health_text(), "Connecting");

        // Error
        cm.serial.status = ConnectionStatus::Error;
        assert_eq!(cm.link_health_text(), "Error");

        // HardwareFault
        cm.serial.status = ConnectionStatus::HardwareFault;
        assert_eq!(cm.link_health_text(), "HW Fault");

        // Connected with no RX yet
        cm.serial.status = ConnectionStatus::Connected;
        cm.last_rx_instant = None;
        assert_eq!(cm.link_health_text(), "Connected (no RX yet)");
    }

    #[test]
    fn test_link_health_text_connected_with_elapsed() {
        let mut cm = ConnectionManager::new();
        cm.serial.status = ConnectionStatus::Connected;

        // Live (< 1s)
        cm.last_rx_instant = Some(Instant::now());
        let text = cm.link_health_text();
        assert_eq!(text, "Live");

        // Stale (> 10s)
        cm.last_rx_instant = Some(Instant::now() - Duration::from_secs(15));
        let text = cm.link_health_text();
        assert!(text.starts_with("Stale"), "Expected 'Stale', got: {}", text);
    }

    #[test]
    fn test_reset_counters_all_services() {
        let mut cm = ConnectionManager::new();
        cm.serial.bytes_sent = 100;
        cm.serial.bytes_received = 200;
        cm.serial.error_count = 5;
        cm.tcp.bytes_sent = 300;
        cm.tcp.bytes_received = 400;
        cm.tcp.error_count = 10;
        cm.udp.bytes_sent = 500;
        cm.udp.bytes_received = 600;
        cm.udp.error_count = 15;
        cm.reset_counters();
        assert_eq!(cm.total_bytes_sent(), 0);
        assert_eq!(cm.total_bytes_received(), 0);
        assert_eq!(cm.total_errors(), 0);
    }

    #[test]
    fn test_total_bytes_aggregates_all_services() {
        let mut cm = ConnectionManager::new();
        cm.serial.bytes_sent = 100;
        cm.tcp.bytes_sent = 200;
        cm.udp.bytes_sent = 300;
        assert_eq!(cm.total_bytes_sent(), 600);

        cm.serial.bytes_received = 10;
        cm.tcp.bytes_received = 20;
        cm.udp.bytes_received = 30;
        assert_eq!(cm.total_bytes_received(), 60);

        cm.serial.error_count = 1;
        cm.tcp.error_count = 2;
        cm.udp.error_count = 3;
        assert_eq!(cm.total_errors(), 6);
    }

    #[test]
    fn test_reconnect_lifecycle_complete() {
        let mut cm = ConnectionManager::new();
        assert!(!cm.reconnect_armed());
        assert!(!cm.reconnect_paused());

        cm.arm_auto_reconnect();
        assert!(cm.reconnect_armed());
        assert!(!cm.reconnect_paused());

        cm.pause_auto_reconnect();
        assert!(cm.reconnect_armed());
        assert!(cm.reconnect_paused());

        cm.resume_auto_reconnect();
        assert!(cm.reconnect_armed());
        assert!(!cm.reconnect_paused());

        cm.reconnect_attempts = 5;
        cm.next_reconnect_at = Some(Instant::now() + Duration::from_secs(10));
        cm.clear_reconnect_schedule();
        assert!(!cm.reconnect_armed());
        assert!(cm.next_reconnect_at.is_none());
        assert_eq!(cm.reconnect_attempts, 0);
    }

    #[test]
    fn test_send_data_routes_to_correct_service() {
        let mut cm = ConnectionManager::new();

        // Serial (disconnected → error)
        cm.set_active_connection(ConnectionType::Serial);
        assert!(cm.send_data(&[0x01]).is_err());

        // TCP (disconnected → error)
        cm.set_active_connection(ConnectionType::Tcp);
        assert!(cm.send_data(&[0x01]).is_err());

        // UDP (disconnected → error)
        cm.set_active_connection(ConnectionType::Udp);
        assert!(cm.send_data(&[0x01]).is_err());

        // CAN → error (no active connection)
        cm.set_active_connection(ConnectionType::Can);
        assert!(cm.send_data(&[0x01]).is_err());
    }

    #[test]
    fn test_reconnect_countdown_armed_with_timer() {
        let mut cm = ConnectionManager::new();
        cm.arm_auto_reconnect();
        cm.next_reconnect_at = Some(Instant::now() + Duration::from_secs(5));
        let text = cm.reconnect_countdown_text();
        // Should show a countdown like "4.Xs" or "5.Xs"
        assert!(
            text.ends_with('s'),
            "Expected countdown text, got: {}",
            text
        );
        assert!(text != "N/A");
    }

    #[test]
    fn test_reconnect_countdown_due_now() {
        let mut cm = ConnectionManager::new();
        cm.arm_auto_reconnect();
        cm.next_reconnect_at = Some(Instant::now() - Duration::from_secs(1));
        let text = cm.reconnect_countdown_text();
        assert_eq!(text, "Due now");
    }

    #[test]
    fn test_last_comm_all_types() {
        let mut cm = ConnectionManager::new();

        cm.set_active_connection(ConnectionType::Serial);
        assert_eq!(cm.last_comm(), "N/A");

        cm.set_active_connection(ConnectionType::Tcp);
        assert_eq!(cm.last_comm(), "N/A");

        cm.set_active_connection(ConnectionType::Udp);
        assert_eq!(cm.last_comm(), "N/A");

        cm.set_active_connection(ConnectionType::Can);
        assert_eq!(cm.last_comm(), "No CAN frame yet");
    }

    #[test]
    fn test_is_any_connected_reflects_service_state() {
        let mut cm = ConnectionManager::new();
        assert!(!cm.is_any_connected());

        // CAN running counts as connected
        cm.can.is_running = true;
        assert!(cm.is_any_connected());
        cm.can.is_running = false;
        assert!(!cm.is_any_connected());
    }
}
