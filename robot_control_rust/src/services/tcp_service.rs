use crate::models::ConnectionStatus;
use anyhow::Result;
use chrono::Local;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;
use tracing::{error, info, warn};

use super::connection_provider::ConnectionProvider;

/// Write timeout for TCP send operations (industrial: prevents UI freeze
/// if the peer stops ACKing during a network partition).
const TCP_WRITE_TIMEOUT: Duration = Duration::from_secs(3);

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
                // Industrial: write timeout prevents UI freeze on network partition.
                stream.set_write_timeout(Some(TCP_WRITE_TIMEOUT)).ok();
                // Industrial: disable Nagle for low-latency Modbus TCP / CANopen.
                stream.set_nodelay(true).ok();
                self.stream = Some(stream);
                self.status = ConnectionStatus::Connected;
                self.is_server = false;
                info!("TCP connected to {}", addr);
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
                info!("TCP server listening on {}", addr);
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
                // Industrial: write timeout prevents UI freeze on network partition.
                stream.set_write_timeout(Some(TCP_WRITE_TIMEOUT)).ok();
                // Industrial: disable Nagle for low-latency protocol responses.
                stream.set_nodelay(true).ok();
                let addr_str = addr.to_string();
                // Close previous client connection if any (industrial: prevent
                // stale stream leak when new client connects).
                if let Some(ref old) = self.stream {
                    old.shutdown(Shutdown::Both).ok();
                }
                self.connected_clients.push(addr_str.clone());
                self.stream = Some(stream);
                warn!(
                    "TCP server: new client {} replaced previous connection",
                    addr_str
                );
                info!("TCP client connected: {}", addr_str);
            }
        }
    }
}

impl ConnectionProvider for TcpService {
    fn is_connected(&self) -> bool {
        self.stream.is_some() && self.status.is_connected()
    }

    fn disconnect(&mut self) {
        if let Some(ref stream) = self.stream {
            stream.shutdown(Shutdown::Both).ok();
        }
        self.stream = None;
        self.listener = None;
        self.status = ConnectionStatus::Disconnected;
        self.connected_clients.clear();
    }

    fn try_read_raw(&mut self) -> Vec<u8> {
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
                error!("TCP read error: {}", e);
                self.error_count += 1;
                Vec::new()
            }
        }
    }

    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        if let Some(ref mut stream) = self.stream {
            match stream.write_all(data) {
                Ok(()) => {
                    stream.flush().ok(); // best-effort after successful write_all
                    self.bytes_sent += data.len() as u64;
                    self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                    Ok(())
                }
                Err(e) => {
                    // Industrial: BrokenPipe/ConnectionReset → mark Disconnected
                    // so reconnect logic can trigger, rather than leaving stale
                    // Connected status that prevents recovery.
                    use std::io::ErrorKind;
                    match e.kind() {
                        ErrorKind::BrokenPipe
                        | ErrorKind::ConnectionReset
                        | ErrorKind::ConnectionAborted => {
                            warn!("TCP send: peer gone ({}), marking disconnected", e);
                            self.status = ConnectionStatus::Disconnected;
                        }
                        ErrorKind::TimedOut | ErrorKind::WouldBlock => {
                            // Write timeout under non-blocking / SO_SNDTIMEO
                            warn!("TCP send timeout: {}", e);
                            self.status = ConnectionStatus::Error;
                        }
                        _ => {
                            self.status = ConnectionStatus::Error;
                        }
                    }
                    self.error_count += 1;
                    Err(anyhow::anyhow!("TCP send failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("TCP not connected"))
        }
    }

    fn reset_stats(&mut self) {
        self.bytes_sent = 0;
        self.bytes_received = 0;
        self.error_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_service_default() {
        let service = TcpService::default();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
        assert_eq!(service.bytes_sent, 0);
        assert_eq!(service.bytes_received, 0);
        assert_eq!(service.host, "127.0.0.1");
        assert_eq!(service.port, 8080);
        assert!(!service.is_server);
        assert!(!service.is_connected());
    }

    #[test]
    fn test_disconnect_when_not_connected() {
        let mut service = TcpService::default();
        service.disconnect();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
        assert!(service.connected_clients.is_empty());
    }

    #[test]
    fn test_reset_stats() {
        let mut service = TcpService {
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
        let mut service = TcpService::default();
        assert!(service.send_data(b"ping").is_err());
        assert_eq!(service.status, ConnectionStatus::Disconnected);
    }
}
