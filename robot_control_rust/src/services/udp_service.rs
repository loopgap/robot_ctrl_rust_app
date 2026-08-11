use crate::models::ConnectionStatus;
use anyhow::Result;
use chrono::Local;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

use super::connection_provider::ConnectionProvider;

/// Bound for worker→main data channel.
const WORKER_TO_MAIN_BOUND: usize = 64;

/// Bounded join timeout for worker thread shutdown.
const JOIN_TIMEOUT: Duration = Duration::from_secs(2);

/// Message sent from main thread to the UDP worker thread.
enum UdpWorkerMsg {
    /// Send to the default remote address.
    SendDefault(Vec<u8>),
    /// Send to a specific address ("host:port").
    SendTo(Vec<u8>, String),
}

/// Received data plus the source address (stored as String for Send-ability).
struct UdpRecvData {
    data: Vec<u8>,
    from_addr: String,
}

pub struct UdpService {
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

    // Communication with the background thread
    tx: Option<mpsc::Sender<UdpWorkerMsg>>,
    rx: Option<mpsc::Receiver<UdpRecvData>>,
    /// Pre-allocated scratch buffer for `try_read_raw` to avoid per-call heap allocation.
    rx_scratch: Vec<u8>,
    stop_flag: Arc<AtomicBool>,
    /// Set to true by the worker thread when it exits due to an I/O error.
    worker_errored: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl Default for UdpService {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpService {
    pub fn new() -> Self {
        Self {
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
            tx: None,
            rx: None,
            rx_scratch: Vec::with_capacity(8192),
            stop_flag: Arc::new(AtomicBool::new(false)),
            worker_errored: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
        }
    }

    /// Binds the UDP socket and spawns a background worker thread for I/O.
    pub fn bind(&mut self) -> Result<()> {
        self.close();
        self.status = ConnectionStatus::Connecting;

        let addr = format!("{}:{}", self.local_addr, self.local_port);
        let remote_addr = format!("{}:{}", self.remote_addr, self.remote_port);

        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => {
                s.set_nonblocking(true).ok();
                s
            }
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                return Err(anyhow::anyhow!("UDP bind failed: {}", e));
            }
        };

        // Channel for main→worker send commands
        let (tx_to_thread, rx_from_main) = mpsc::channel::<UdpWorkerMsg>();
        // Bounded channel for worker→main received data
        let (tx_to_main, rx_from_thread) = mpsc::sync_channel::<UdpRecvData>(WORKER_TO_MAIN_BOUND);

        self.tx = Some(tx_to_thread);
        self.rx = Some(rx_from_thread);
        self.stop_flag = Arc::new(AtomicBool::new(false));
        self.worker_errored = Arc::new(AtomicBool::new(false));
        let stop_flag = self.stop_flag.clone();
        let worker_errored = self.worker_errored.clone();

        let handle = thread::spawn(move || {
            Self::worker_loop(
                socket,
                rx_from_main,
                tx_to_main,
                stop_flag,
                worker_errored,
                &remote_addr,
            );
        });

        self.worker_handle = Some(handle);
        self.status = ConnectionStatus::Connected;
        info!("UDP bound to {}", addr);
        Ok(())
    }

    /// Core worker loop: reads from socket, sends to main; writes from channel.
    fn worker_loop(
        socket: UdpSocket,
        rx_from_main: mpsc::Receiver<UdpWorkerMsg>,
        tx_to_main: mpsc::SyncSender<UdpRecvData>,
        stop_flag: Arc<AtomicBool>,
        worker_errored: Arc<AtomicBool>,
        remote_addr: &str,
    ) {
        let remote = remote_addr.to_string();
        let mut buf = [0u8; 65535];

        while !stop_flag.load(Ordering::Acquire) {
            // Drain ALL pending send requests from the main thread
            while let Ok(msg) = rx_from_main.try_recv() {
                let result = match &msg {
                    UdpWorkerMsg::SendDefault(data) => socket.send_to(data, &remote),
                    UdpWorkerMsg::SendTo(data, addr) => socket.send_to(data, addr),
                };
                if let Err(e) = result {
                    error!("UDP send error: {}", e);
                    worker_errored.store(true, Ordering::Release);
                    return;
                }
            }

            // Non-blocking read
            match socket.recv_from(&mut buf) {
                Ok((n, from)) => {
                    let recv = UdpRecvData {
                        data: buf[..n].to_vec(),
                        from_addr: from.to_string(),
                    };
                    if tx_to_main.try_send(recv).is_err() {
                        // Channel full or disconnected
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    error!("UDP read error: {}", e);
                    worker_errored.store(true, Ordering::Release);
                    return;
                }
            }

            thread::sleep(Duration::from_millis(1));
        }
        info!("UDP worker thread exited");
    }

    pub fn close(&mut self) {
        let had_error = self.worker_errored.load(Ordering::Acquire);
        self.stop_flag.store(true, Ordering::Release);
        if let Some(handle) = self.worker_handle.take() {
            let join_start = std::time::Instant::now();
            let (done_tx, done_rx) = mpsc::channel();
            thread::spawn(move || {
                let _ = handle.join();
                let _ = done_tx.send(());
            });
            match done_rx.recv_timeout(JOIN_TIMEOUT) {
                Ok(()) => {}
                Err(_) => {
                    warn!(
                        "UDP worker thread did not exit within {:?}; detaching",
                        JOIN_TIMEOUT
                    );
                }
            }
            let elapsed = join_start.elapsed();
            if elapsed > Duration::from_millis(100) {
                info!(
                    "UDP worker join took {:.1}s (timeout={:?})",
                    elapsed.as_secs_f32(),
                    JOIN_TIMEOUT
                );
            }
        }
        // Drain residual messages from the channel
        if let Some(rx) = self.rx.take() {
            while rx.try_recv().is_ok() {}
        }
        self.tx = None;
        self.status = if had_error {
            ConnectionStatus::HardwareFault
        } else {
            ConnectionStatus::Disconnected
        };
    }

    pub fn send_to(&mut self, data: &[u8], addr: &str) -> Result<()> {
        if self.worker_errored.load(Ordering::Acquire) {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
            return Err(anyhow::anyhow!(
                "UDP worker exited due to I/O error (device fault)"
            ));
        }
        if let Some(tx) = &self.tx {
            match tx.send(UdpWorkerMsg::SendTo(data.to_vec(), addr.to_string())) {
                Ok(_) => {
                    self.bytes_sent += data.len() as u64;
                    self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                    Ok(())
                }
                Err(_) => {
                    self.status = ConnectionStatus::Error;
                    self.error_count += 1;
                    Err(anyhow::anyhow!("Background thread disconnected"))
                }
            }
        } else {
            Err(anyhow::anyhow!("UDP socket not bound"))
        }
    }

    pub fn send_default(&mut self, data: &[u8]) -> Result<()> {
        if self.worker_errored.load(Ordering::Acquire) {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
            return Err(anyhow::anyhow!(
                "UDP worker exited due to I/O error (device fault)"
            ));
        }
        if let Some(tx) = &self.tx {
            match tx.send(UdpWorkerMsg::SendDefault(data.to_vec())) {
                Ok(_) => {
                    self.bytes_sent += data.len() as u64;
                    self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
                    Ok(())
                }
                Err(_) => {
                    self.status = ConnectionStatus::Error;
                    self.error_count += 1;
                    Err(anyhow::anyhow!("Background thread disconnected"))
                }
            }
        } else {
            Err(anyhow::anyhow!("UDP socket not bound"))
        }
    }
}

impl ConnectionProvider for UdpService {
    fn is_connected(&self) -> bool {
        if self.worker_errored.load(Ordering::Acquire) {
            return false;
        }
        self.tx.is_some() && self.status.is_connected()
    }

    fn disconnect(&mut self) {
        self.close();
    }

    fn try_read_raw(&mut self) -> Vec<u8> {
        // Detect worker thread I/O failure early so callers see the right status.
        if self.worker_errored.load(Ordering::Acquire)
            && self.status != ConnectionStatus::HardwareFault
        {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
        }

        // Reuse pre-allocated scratch buffer to avoid per-call heap allocation.
        self.rx_scratch.clear();
        let mut has_recv = false;
        if let Some(rx) = &self.rx {
            while let Ok(recv) = rx.try_recv() {
                if !has_recv {
                    // Record source address from first received packet
                    self.last_recv_from = recv.from_addr;
                    has_recv = true;
                }
                self.rx_scratch.extend_from_slice(&recv.data);
            }
        }

        if !self.rx_scratch.is_empty() {
            self.bytes_received += self.rx_scratch.len() as u64;
            self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
        }

        std::mem::take(&mut self.rx_scratch)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.send_default(data)
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
    fn test_udp_service_default() {
        let service = UdpService::default();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
        assert_eq!(service.local_addr, "0.0.0.0");
        assert_eq!(service.local_port, 9000);
        assert_eq!(service.remote_addr, "127.0.0.1");
        assert_eq!(service.remote_port, 9001);
        assert!(!service.is_connected());
    }

    #[test]
    fn test_close_when_not_bound() {
        let mut service = UdpService::default();
        service.close();
        assert_eq!(service.status, ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_reset_stats() {
        let mut service = UdpService {
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
        let mut service = UdpService::default();
        assert!(service.send_default(b"ping").is_err());
        assert_eq!(service.status, ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_send_data_fails_fast_when_worker_errored() {
        let mut service = UdpService::default();
        service.worker_errored.store(true, Ordering::Release);
        let result = service.send_default(b"data");
        assert!(result.is_err());
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
    }

    #[test]
    fn test_try_read_raw_sets_hardware_fault_on_worker_error() {
        let mut service = UdpService::default();
        service.worker_errored.store(true, Ordering::Release);
        service.status = ConnectionStatus::Connected;
        let data = service.try_read_raw();
        assert!(data.is_empty());
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
        assert_eq!(service.error_count, 1);
    }

    #[test]
    fn test_disconnect_sets_hardware_fault_when_worker_errored() {
        let mut service = UdpService::default();
        service.worker_errored.store(true, Ordering::Release);
        service.disconnect();
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
    }
}
