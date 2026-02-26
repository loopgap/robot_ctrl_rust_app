use crate::models::ConnectionStatus;
use anyhow::Result;
use chrono::Local;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

pub struct TcpService {
    stream: Option<TcpStream>,
    listener: Option<TcpListener>,
    pub status: ConnectionStatus,
    pub host: String,
    pub port: u16,
    pub is_server: bool,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error_count: u64,
    pub last_comm: String,
    pub connected_clients: Vec<String>,
}

impl Default for TcpService {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpService {
    pub fn new() -> Self {
        Self {
            stream: None,
            listener: None,
            status: ConnectionStatus::Disconnected,
            host: "127.0.0.1".into(),
            port: 8080,
            is_server: false,
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
            last_comm: "N/A".into(),
            connected_clients: Vec::new(),
        }
    }

    pub fn connect_client(&mut self) -> Result<()> {
        self.disconnect();
        self.status = ConnectionStatus::Connecting;
        let addr = format!("{}:{}", self.host, self.port);
        match TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?,
            Duration::from_secs(5),
        ) {
            Ok(stream) => {
                stream.set_nonblocking(true).ok();
                stream.set_read_timeout(Some(Duration::from_millis(1))).ok();
                self.stream = Some(stream);
                self.status = ConnectionStatus::Connected;
                self.is_server = false;
                log::info!("TCP connected to {}", addr);
                Ok(())
            }
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                Err(anyhow::anyhow!("TCP connect failed: {}", e))
            }
        }
    }

    pub fn start_server(&mut self) -> Result<()> {
        self.disconnect();
        self.status = ConnectionStatus::Connecting;
        let addr = format!("{}:{}", self.host, self.port);
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                listener.set_nonblocking(true).ok();
                self.listener = Some(listener);
                self.status = ConnectionStatus::Connected;
                self.is_server = true;
                log::info!("TCP server listening on {}", addr);
                Ok(())
            }
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                Err(anyhow::anyhow!("TCP server bind failed: {}", e))
            }
        }
    }

    /// 服务端：尝试接受新连接
    pub fn try_accept(&mut self) {
        if let Some(ref listener) = self.listener {
            if let Ok((stream, addr)) = listener.accept() {
                stream.set_nonblocking(true).ok();
                stream.set_read_timeout(Some(Duration::from_millis(1))).ok();
                let addr_str = addr.to_string();
                self.connected_clients.push(addr_str.clone());
                self.stream = Some(stream); // 简化：只保留最新连接
                log::info!("TCP client connected: {}", addr_str);
            }
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(ref stream) = self.stream {
            stream.shutdown(Shutdown::Both).ok();
        }
        self.stream = None;
        self.listener = None;
        self.status = ConnectionStatus::Disconnected;
        self.connected_clients.clear();
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.status.is_connected()
    }

    pub fn try_read(&mut self) -> Vec<u8> {
        if self.is_server {
            self.try_accept();
        }
        let stream = match self.stream.as_mut() {
            Some(s) => s,
            None => return Vec::new(),
        };
        let mut buf = [0u8; 4096];
        match stream.read(&mut buf) {
            Ok(0) => {
                // 连接关闭
                self.status = ConnectionStatus::Disconnected;
                Vec::new()
            }
            Ok(n) => {
                self.bytes_received += n as u64;
                self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                buf[..n].to_vec()
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Vec::new(),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Vec::new(),
            Err(e) => {
                log::error!("TCP read error: {}", e);
                self.error_count += 1;
                Vec::new()
            }
        }
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.write_all(data)?;
            stream.flush()?;
            self.bytes_sent += data.len() as u64;
            self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("TCP not connected"))
        }
    }

    pub fn reset_stats(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.error_count = 0;
    }
}
