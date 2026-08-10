use crate::models::{ConnectionStatus, RobotState, SerialConfig};
use anyhow::Result;
use chrono::Local;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use super::connection_provider::ConnectionProvider;

const PACKET_HEADER: u8 = 0xAA;
const PACKET_TAIL: u8 = 0x55;

/// Maximum rx_buffer capacity before oldest data is discarded.
/// Protects against unbounded memory growth from noise / baud-rate mismatch.
const RX_BUFFER_MAX: usize = 64 * 1024; // 64 KiB

/// Maximum allowed payload length in a single packet.
/// Frames with `length` field above this are discarded immediately.
const PACKET_PAYLOAD_MAX: usize = 200;

/// 串口通信服务，管理串口连接的生命周期
///
/// 使用后台线程进行串口读写，通过 mpsc channel 与主线程通信。
/// 支持自动重连、字节统计、错误计数。
pub struct SerialService {
    pub status: ConnectionStatus,
    pub config: SerialConfig,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error_count: u64,
    pub last_comm: String,
    rx_buffer: Vec<u8>,

    // Communication with the background thread
    tx: Option<mpsc::Sender<Vec<u8>>>,
    rx: Option<mpsc::Receiver<Vec<u8>>>,
    stop_flag: Arc<AtomicBool>,
    /// Set to true by the worker thread when it exits due to an I/O error.
    /// The main thread can check this to detect unexpected disconnects.
    worker_errored: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl Default for SerialService {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialService {
    pub fn new() -> Self {
        Self {
            status: ConnectionStatus::Disconnected,
            config: SerialConfig::default(),
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
            last_comm: "N/A".into(),
            rx_buffer: Vec::with_capacity(4096),
            tx: None,
            rx: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            worker_errored: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
        }
    }

    pub fn scan_ports() -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            for _ in 0..3 {
                if let Ok(ports) = serialport::available_ports() {
                    if !ports.is_empty() {
                        return ports.into_iter().map(|p| p.port_name).collect();
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        }

        serialport::available_ports()
            .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
            .unwrap_or_default()
    }

    pub fn get_port_info(port_name: &str) -> String {
        serialport::available_ports()
            .ok()
            .and_then(|ports| {
                ports
                    .into_iter()
                    .find(|p| p.port_name == port_name)
                    .map(|p| format!("{:?}", p.port_type))
            })
            .unwrap_or_else(|| "Unknown".into())
    }

    pub fn connect(&mut self) -> Result<()> {
        self.disconnect();
        self.status = ConnectionStatus::Connecting;

        let parity = match self.config.parity.as_str() {
            "Odd" => Parity::Odd,
            "Even" => Parity::Even,
            _ => Parity::None,
        };
        let data_bits = match self.config.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        };
        let stop_bits = match self.config.stop_bits {
            2 => StopBits::Two,
            _ => StopBits::One,
        };
        let flow = match self.config.flow_control.as_str() {
            "Hardware (RTS/CTS)" => FlowControl::Hardware,
            "Software (XON/XOFF)" => FlowControl::Software,
            _ => FlowControl::None,
        };

        let port_builder = serialport::new(&self.config.port_name, self.config.baud_rate)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .data_bits(data_bits)
            .stop_bits(stop_bits)
            .parity(parity)
            .flow_control(flow);

        let mut port_result = Err(anyhow::anyhow!("Init error"));
        // Retry on all platforms — embedded Linux (Raspberry Pi, etc.) may also
        // need a second attempt after udev settles.
        let retries = 2;

        for attempt in 1..=retries {
            port_result = port_builder
                .clone()
                .open()
                .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e));
            if port_result.is_ok() {
                break;
            }
            if attempt < retries {
                warn!("Retry {} to open port {}", attempt, self.config.port_name);
                thread::sleep(Duration::from_millis(500));
            }
        }

        let mut port = match port_result {
            Ok(p) => p,
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                return Err(e);
            }
        };

        let (tx_to_thread, rx_from_main) = mpsc::channel::<Vec<u8>>();
        // Bounded channel (64 slots) for worker→main data transfer.
        // Prevents unbounded memory growth if the UI thread falls behind
        // (industrial: protects against data burst under high baud rate).
        const WORKER_TO_MAIN_BOUND: usize = 64;
        let (tx_to_main, rx_from_thread) = mpsc::sync_channel::<Vec<u8>>(WORKER_TO_MAIN_BOUND);

        self.tx = Some(tx_to_thread);
        self.rx = Some(rx_from_thread);
        self.stop_flag = Arc::new(AtomicBool::new(false));
        self.worker_errored = Arc::new(AtomicBool::new(false));
        let stop_flag = self.stop_flag.clone();
        let worker_errored = self.worker_errored.clone();

        let port_name = self.config.port_name.clone();

        let handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            'worker: while !stop_flag.load(Ordering::Acquire) {
                // Drain ALL pending write requests from the main thread
                while let Ok(data) = rx_from_main.try_recv() {
                    if let Err(e) = port.write_all(&data) {
                        error!("Serial write error on {}: {}", port_name, e);
                        worker_errored.store(true, Ordering::Release);
                        break 'worker;
                    }
                    let _ = port.flush();
                }

                // Read from serial port to send to main thread
                match port.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        // try_send: drop data rather than block if main thread is slow.
                        // Industrial: prevents worker thread stall under burst traffic.
                        if tx_to_main.try_send(buf[..n].to_vec()).is_err() {
                            // Channel full or disconnected — drop this chunk silently.
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        error!("Serial read error on {}: {}", port_name, e);
                        worker_errored.store(true, Ordering::Release);
                        break 'worker;
                    }
                }

                thread::sleep(Duration::from_millis(1));
            }
            info!("Serial thread for {} exited", port_name);
        });

        self.worker_handle = Some(handle);
        self.status = ConnectionStatus::Connected;
        info!("Connected to {}", self.config.port_name);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        let had_error = self.worker_errored.load(Ordering::Acquire);
        self.stop_flag.store(true, Ordering::Release);
        if let Some(handle) = self.worker_handle.take() {
            // Bounded join: wait up to 2 seconds for the worker thread to exit.
            // Industrial: prevents UI freeze if the serial port driver hangs
            // (e.g., USB unplug during an in-flight read on some OS/driver combos).
            //
            // std::thread::JoinHandle has no join_timeout; we use a helper thread
            // to implement a bounded wait.
            let join_start = Instant::now();
            let join_timeout = Duration::from_secs(2);
            let (done_tx, done_rx) = mpsc::channel();
            thread::spawn(move || {
                let _ = handle.join();
                let _ = done_tx.send(());
            });
            match done_rx.recv_timeout(join_timeout) {
                Ok(()) => { /* thread exited cleanly */ }
                Err(_) => {
                    warn!(
                        "Serial worker thread did not exit within {:?}; \
                         detaching (port may be hung)",
                        join_timeout
                    );
                }
            }
            let elapsed = join_start.elapsed();
            if elapsed > Duration::from_millis(100) {
                info!(
                    "Serial worker join took {:.1}s (timeout={:?})",
                    elapsed.as_secs_f32(),
                    join_timeout
                );
            }
        }
        // Drain residual messages from the channel to prevent memory leak
        // if the worker accumulated data before shutdown.
        if let Some(rx) = self.rx.take() {
            while rx.try_recv().is_ok() {}
        }
        self.tx = None;
        // Distinguish I/O-failure disconnect from clean user-initiated disconnect.
        // IEC 61784: HardwareFault represents device-level failures
        // (USB unplug, cable break, serial port vanished).
        self.status = if had_error {
            ConnectionStatus::HardwareFault
        } else {
            ConnectionStatus::Disconnected
        };
        self.rx_buffer.clear();
    }

    pub fn push_rx_data(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        // Guard against unbounded growth: if buffer would exceed max,
        // discard oldest bytes first (preserving partial packet search).
        if self.rx_buffer.len() + data.len() > RX_BUFFER_MAX {
            let excess = self.rx_buffer.len() + data.len() - RX_BUFFER_MAX;
            self.rx_buffer.drain(..excess.min(self.rx_buffer.len()));
        }
        self.rx_buffer.extend_from_slice(data);
    }

    pub fn try_parse_state_from_buffer(&mut self) -> Option<RobotState> {
        self.try_parse_packet()
    }

    pub fn try_read_state(&mut self) -> Option<RobotState> {
        let data = self.try_read_raw();
        if data.is_empty() {
            return None;
        }
        self.push_rx_data(&data);
        self.try_parse_packet()
    }

    fn try_parse_packet(&mut self) -> Option<RobotState> {
        let header_pos = self.rx_buffer.iter().position(|&b| b == PACKET_HEADER)?;
        if header_pos > 0 {
            self.rx_buffer.drain(..header_pos);
        }
        if self.rx_buffer.len() < 5 {
            return None;
        }

        let length = self.rx_buffer[2] as usize;
        // Reject impossibly large payloads early to avoid holding garbage data.
        if length > PACKET_PAYLOAD_MAX {
            self.rx_buffer.drain(..1);
            return None;
        }
        let total = 3 + length + 2;
        if self.rx_buffer.len() < total {
            return None;
        }
        if self.rx_buffer[total - 1] != PACKET_TAIL {
            self.rx_buffer.drain(..1);
            return None;
        }

        let payload = &self.rx_buffer[3..3 + length];
        let checksum = self.rx_buffer[total - 2];
        let calc: u8 = self.rx_buffer[1..3 + length]
            .iter()
            .fold(0u8, |a, &b| a.wrapping_add(b));
        if checksum != calc {
            self.rx_buffer.drain(..1);
            self.error_count += 1;
            return None;
        }

        let state = if payload.len() >= 16 {
            let pos = f32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as f64;
            let vel = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]) as f64;
            let cur = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]) as f64;
            let temp =
                f32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]) as f64;
            Some(RobotState::new(pos, vel, cur, temp))
        } else {
            None
        };

        self.rx_buffer.drain(..total);
        state
    }

    pub fn try_read_raw(&mut self) -> Vec<u8> {
        // Detect worker thread I/O failure early so callers see the right status.
        if self.worker_errored.load(Ordering::Acquire)
            && self.status != ConnectionStatus::HardwareFault
        {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
        }

        let mut all_data = Vec::new();
        if let Some(rx) = &self.rx {
            while let Ok(data) = rx.try_recv() {
                all_data.extend_from_slice(&data);
            }
        }

        if !all_data.is_empty() {
            self.bytes_received += all_data.len() as u64;
            self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
        }

        all_data
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        // Fast-fail if the worker thread already exited due to I/O error.
        // Prevents spurious SendError log spam after a device-level failure.
        if self.worker_errored.load(Ordering::Acquire) {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
            return Err(anyhow::anyhow!(
                "Serial worker exited due to I/O error (device fault)"
            ));
        }
        if let Some(tx) = &self.tx {
            match tx.send(data.to_vec()) {
                Ok(_) => {
                    self.bytes_sent += data.len() as u64;
                    self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                    Ok(())
                }
                Err(_) => {
                    // Channel disconnected means the worker thread exited.
                    self.status = ConnectionStatus::Error;
                    self.error_count += 1;
                    Err(anyhow::anyhow!("Background thread disconnected"))
                }
            }
        } else {
            Err(anyhow::anyhow!("Port not open"))
        }
    }

    pub fn send_string(&mut self, s: &str) -> Result<()> {
        self.send_data(s.as_bytes())
    }

    pub fn encode_packet(cmd: u8, payload: &[u8]) -> Vec<u8> {
        let mut pkt = Vec::with_capacity(payload.len() + 5);
        pkt.push(PACKET_HEADER);
        pkt.push(cmd);
        pkt.push(payload.len() as u8);
        pkt.extend_from_slice(payload);
        let checksum: u8 = pkt[1..].iter().fold(0u8, |a, &b| a.wrapping_add(b));
        pkt.push(checksum);
        pkt.push(PACKET_TAIL);
        pkt
    }

    pub fn send_position_control(&mut self, pos: f64) -> Result<()> {
        let bytes = (pos as f32).to_le_bytes();
        let pkt = Self::encode_packet(0x01, &bytes);
        self.send_data(&pkt)
    }

    pub fn send_emergency_stop(&mut self) -> Result<()> {
        let pkt = Self::encode_packet(0xFF, &[0x01]);
        self.send_data(&pkt)
    }

    pub fn reset_stats(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.error_count = 0;
    }
}

impl ConnectionProvider for SerialService {
    fn is_connected(&self) -> bool {
        // Detect unexpected worker thread exit (I/O error)
        if self.worker_errored.load(Ordering::Acquire) {
            return false;
        }
        self.tx.is_some() && self.status.is_connected()
    }

    fn disconnect(&mut self) {
        self.disconnect()
    }

    fn try_read_raw(&mut self) -> Vec<u8> {
        self.try_read_raw()
    }

    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.send_data(data)
    }

    fn reset_stats(&mut self) {
        self.reset_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_service_default() {
        let service = SerialService::default();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
        assert_eq!(service.bytes_sent, 0);
        assert_eq!(service.bytes_received, 0);
        assert_eq!(service.error_count, 0);
        assert!(!service.is_connected());
    }

    #[test]
    fn test_disconnect_when_not_connected() {
        let mut service = SerialService::default();
        service.disconnect();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_reset_stats() {
        let mut service = SerialService {
            bytes_sent: 100,
            bytes_received: 200,
            error_count: 5,
            ..Default::default()
        };
        service.reset_stats();
        assert_eq!(service.bytes_sent, 0);
        assert_eq!(service.bytes_received, 0);
        assert_eq!(service.error_count, 0);
    }

    #[test]
    fn test_send_data_when_disconnected_returns_error() {
        let mut service = SerialService::default();
        assert!(service.send_data(b"ping").is_err());
        assert_eq!(service.status, ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_connect_with_empty_port_returns_error() {
        let mut service = SerialService::default();
        service.config.port_name.clear();
        assert!(service.connect().is_err());
        assert_eq!(service.status, ConnectionStatus::Error);
        service.disconnect();
    }

    #[test]
    fn test_encode_packet() {
        let pkt = SerialService::encode_packet(0x01, &[0x02, 0x03]);
        assert_eq!(pkt[0], 0xAA);
        assert_eq!(pkt[1], 0x01);
        assert_eq!(pkt[2], 2);
        assert_eq!(pkt[3], 0x02);
        assert_eq!(pkt[4], 0x03);
        assert_eq!(pkt[6], 0x55);
    }

    #[test]
    fn test_send_data_fails_fast_when_worker_errored() {
        let mut service = SerialService::default();
        service.worker_errored.store(true, Ordering::Release);
        // Even though tx is None, the worker_errored check should fire first.
        let result = service.send_data(b"data");
        assert!(result.is_err());
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
    }

    #[test]
    fn test_try_read_raw_sets_hardware_fault_on_worker_error() {
        let mut service = SerialService::default();
        service.worker_errored.store(true, Ordering::Release);
        service.status = ConnectionStatus::Connected; // pretend we were connected
        let data = service.try_read_raw();
        assert!(data.is_empty()); // no rx channel in default
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
        assert_eq!(service.error_count, 1);
    }

    #[test]
    fn test_disconnect_sets_hardware_fault_when_worker_errored() {
        let mut service = SerialService::default();
        service.worker_errored.store(true, Ordering::Release);
        service.disconnect();
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
    }

    #[test]
    fn test_disconnect_sets_disconnected_when_no_error() {
        let mut service = SerialService::default();
        // worker_errored is false by default
        service.disconnect();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
    }
}
