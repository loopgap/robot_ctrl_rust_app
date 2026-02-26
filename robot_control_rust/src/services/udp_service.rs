use crate::models::ConnectionStatus;
use anyhow::Result;
use chrono::Local;
use std::net::UdpSocket;

pub struct UdpService {
    socket: Option<UdpSocket>,
    pub status: ConnectionStatus,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error_count: u64,
    pub last_comm: String,
    pub last_recv_from: String,
}

impl Default for UdpService {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpService {
    pub fn new() -> Self {
        Self {
            socket: None,
            status: ConnectionStatus::Disconnected,
            local_addr: "0.0.0.0".into(),
            local_port: 9000,
            remote_addr: "127.0.0.1".into(),
            remote_port: 9001,
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
            last_comm: "N/A".into(),
            last_recv_from: String::new(),
        }
    }

    pub fn bind(&mut self) -> Result<()> {
        self.close();
        let addr = format!("{}:{}", self.local_addr, self.local_port);
        match UdpSocket::bind(&addr) {
            Ok(socket) => {
                socket.set_nonblocking(true)?;
                self.socket = Some(socket);
                self.status = ConnectionStatus::Connected;
                log::info!("UDP bound to {}", addr);
                Ok(())
            }
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                Err(anyhow::anyhow!("UDP bind failed: {}", e))
            }
        }
    }

    pub fn close(&mut self) {
        self.socket = None;
        self.status = ConnectionStatus::Disconnected;
    }

    pub fn is_connected(&self) -> bool {
        self.socket.is_some() && self.status.is_connected()
    }

    pub fn try_read(&mut self) -> Vec<u8> {
        let socket = match self.socket.as_ref() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let mut buf = [0u8; 65535];
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                self.bytes_received += n as u64;
                self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                self.last_recv_from = addr.to_string();
                buf[..n].to_vec()
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Vec::new(),
            Err(e) => {
                log::error!("UDP read error: {}", e);
                self.error_count += 1;
                Vec::new()
            }
        }
    }

    pub fn send_to(&mut self, data: &[u8], addr: &str) -> Result<()> {
        if let Some(ref socket) = self.socket {
            let n = socket.send_to(data, addr)?;
            self.bytes_sent += n as u64;
            self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("UDP socket not bound"))
        }
    }

    pub fn send_default(&mut self, data: &[u8]) -> Result<()> {
        let addr = format!("{}:{}", self.remote_addr, self.remote_port);
        self.send_to(data, &addr)
    }

    pub fn reset_stats(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.error_count = 0;
    }
}
