use crate::models::{ConnectionStatus, RobotState, SerialConfig};
use anyhow::Result;
use chrono::Local;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::Read;
use std::time::Duration;

const PACKET_HEADER: u8 = 0xAA;
const PACKET_TAIL: u8 = 0x55;

pub struct SerialService {
    port: Option<Box<dyn SerialPort>>,
    pub status: ConnectionStatus,
    pub config: SerialConfig,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error_count: u64,
    pub last_comm: String,
    rx_buffer: Vec<u8>,
}

impl Default for SerialService {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialService {
    pub fn new() -> Self {
        Self {
            port: None,
            status: ConnectionStatus::Disconnected,
            config: SerialConfig::default(),
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
            last_comm: "N/A".into(),
            rx_buffer: Vec::with_capacity(4096),
        }
    }

    pub fn scan_ports() -> Vec<String> {
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

        let port = serialport::new(&self.config.port_name, self.config.baud_rate)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .data_bits(data_bits)
            .stop_bits(stop_bits)
            .parity(parity)
            .flow_control(flow)
            .open();

        match port {
            Ok(p) => {
                self.port = Some(p);
                self.status = ConnectionStatus::Connected;
                log::info!("Connected to {}", self.config.port_name);
                Ok(())
            }
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                Err(anyhow::anyhow!("Failed to connect: {}", e))
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.port = None;
        self.status = ConnectionStatus::Disconnected;
        self.rx_buffer.clear();
    }

    pub fn is_connected(&self) -> bool {
        self.port.is_some() && self.status.is_connected()
    }

    /// 非阻塞读取 - 尝试读取可用数据
    pub fn try_read_raw(&mut self) -> Vec<u8> {
        let port = match self.port.as_mut() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let mut buf = [0u8; 1024];
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                self.bytes_received += n as u64;
                self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                buf[..n].to_vec()
            }
            Ok(_) => Vec::new(),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Vec::new(),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Vec::new(),
            Err(e) => {
                log::error!("Serial read error: {}", e);
                self.error_count += 1;
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::BrokenPipe
                        | std::io::ErrorKind::NotConnected
                        | std::io::ErrorKind::UnexpectedEof
                ) {
                    self.port = None;
                    self.status = ConnectionStatus::Error;
                }
                Vec::new()
            }
        }
    }

    pub fn push_rx_data(&mut self, data: &[u8]) {
        if !data.is_empty() {
            self.rx_buffer.extend_from_slice(data);
        }
    }

    pub fn try_parse_state_from_buffer(&mut self) -> Option<RobotState> {
        self.try_parse_packet()
    }

    /// 尝试读取并解析为 RobotState
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

    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut port) = self.port {
            use std::io::Write;
            if let Err(e) = port.write_all(data) {
                self.error_count += 1;
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::BrokenPipe
                        | std::io::ErrorKind::NotConnected
                        | std::io::ErrorKind::UnexpectedEof
                ) {
                    self.port = None;
                    self.status = ConnectionStatus::Error;
                }
                return Err(e.into());
            }

            if let Err(e) = port.flush() {
                self.error_count += 1;
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::BrokenPipe
                        | std::io::ErrorKind::NotConnected
                        | std::io::ErrorKind::UnexpectedEof
                ) {
                    self.port = None;
                    self.status = ConnectionStatus::Error;
                }
                return Err(e.into());
            }

            self.bytes_sent += data.len() as u64;
            self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
            Ok(())
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
