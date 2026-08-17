use crate::models::ConnectionStatus;
use anyhow::Result;
use chrono::Local;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

use super::connection_provider::ConnectionProvider;

/// Write timeout for TCP send operations (industrial: prevents UI freeze
/// if the peer stops ACKing during a network partition).
const TCP_WRITE_TIMEOUT: Duration = Duration::from_secs(3);

/// Bound for worker→main data channel.
/// Prevents unbounded memory growth if the UI thread falls behind.
const WORKER_TO_MAIN_BOUND: usize = 64;

/// Timeout for connect result channel (prevents UI freeze on unreachable host).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);

/// Bounded join timeout for worker thread shutdown.
const JOIN_TIMEOUT: Duration = Duration::from_secs(2);

pub struct TcpService {
    pub status: ConnectionStatus,
    pub host: String,
    pub port: u16,
    pub is_server: bool,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub error_count: u64,
    pub last_comm: String,
    pub connected_clients: Vec<String>,

    /// Shared client list between worker thread and main thread (server mode).
    /// The worker thread updates this when clients connect/disconnect.
    shared_clients: Option<Arc<Mutex<Vec<String>>>>,

    // Communication with the background thread
    tx: Option<mpsc::Sender<Vec<u8>>>,
    rx: Option<mpsc::Receiver<Vec<u8>>>,
    /// Pre-allocated scratch buffer for `try_read_raw` to avoid per-call heap allocation.
    rx_scratch: Vec<u8>,
    stop_flag: Arc<AtomicBool>,
    /// Set to true by the worker thread when it exits due to an I/O error.
    worker_errored: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl Default for TcpService {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpService {
    pub fn new() -> Self {
        Self {
            status: ConnectionStatus::Disconnected,
            host: "127.0.0.1".into(),
            port: 8080,
            is_server: false,
            bytes_sent: 0,
            bytes_received: 0,
            error_count: 0,
            last_comm: "N/A".into(),
            connected_clients: Vec::new(),
            shared_clients: None,
            tx: None,
            rx: None,
            rx_scratch: Vec::with_capacity(8192),
            stop_flag: Arc::new(AtomicBool::new(false)),
            worker_errored: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
        }
    }

    /// Spawns a background thread for the TCP client connection.
    /// The thread connects, then continuously reads/writes.
    /// Connection result is sent back via a bounded sync_channel.
    pub fn connect_client(&mut self) -> Result<()> {
        self.disconnect();
        self.status = ConnectionStatus::Connecting;

        let addr_str = format!("{}:{}", self.host, self.port);

        // Channel for main→worker data sends (unbounded: non-blocking for caller)
        let (tx_to_thread, rx_from_main) = mpsc::channel::<Vec<u8>>();
        // Bounded channel for worker→main received data
        let (tx_to_main, rx_from_thread) = mpsc::sync_channel::<Vec<u8>>(WORKER_TO_MAIN_BOUND);
        // Bounded sync_channel for connect result
        let (result_tx, result_rx) = mpsc::sync_channel::<Result<()>>(1);

        self.tx = Some(tx_to_thread);
        self.rx = Some(rx_from_thread);
        self.stop_flag = Arc::new(AtomicBool::new(false));
        self.worker_errored = Arc::new(AtomicBool::new(false));
        let stop_flag = self.stop_flag.clone();
        let worker_errored = self.worker_errored.clone();

        let host = self.host.clone();
        let port = self.port;

        let handle = thread::spawn(move || {
            // Phase 1: connect
            let addr = format!("{}:{}", host, port);
            let socket_addr: std::net::SocketAddr = match addr.parse() {
                Ok(a) => a,
                Err(e) => {
                    let _ = result_tx.send(Err(anyhow::anyhow!("Invalid address: {}", e)));
                    return;
                }
            };

            match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5)) {
                Ok(mut stream) => {
                    stream.set_nonblocking(true).ok();
                    stream
                        .set_read_timeout(Some(Duration::from_millis(10)))
                        .ok();
                    stream.set_write_timeout(Some(TCP_WRITE_TIMEOUT)).ok();
                    stream.set_nodelay(true).ok();

                    let _ = result_tx.send(Ok(()));

                    // Phase 2: read/write loop
                    Self::client_worker_loop(
                        &mut stream,
                        rx_from_main,
                        tx_to_main,
                        stop_flag,
                        worker_errored,
                    );
                }
                Err(e) => {
                    let _ = result_tx.send(Err(anyhow::anyhow!("TCP connect failed: {}", e)));
                }
            }
        });

        // Wait for connection result
        match result_rx.recv_timeout(CONNECT_TIMEOUT) {
            Ok(Ok(())) => {
                self.worker_handle = Some(handle);
                self.status = ConnectionStatus::Connected;
                self.is_server = false;
                info!("TCP connected to {}", addr_str);
                Ok(())
            }
            Ok(Err(e)) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                // Bounded join: the connect already failed, worker should exit quickly.
                let (jtx, jrx) = mpsc::channel();
                thread::spawn(move || {
                    let _ = handle.join();
                    let _ = jtx.send(());
                });
                let _ = jrx.recv_timeout(JOIN_TIMEOUT);
                Err(e)
            }
            Err(_) => {
                // Timeout — the worker thread may still be connecting.
                self.stop_flag.store(true, Ordering::Release);
                // Bounded join: prevent UI freeze if connect hangs.
                let (jtx, jrx) = mpsc::channel();
                thread::spawn(move || {
                    let _ = handle.join();
                    let _ = jtx.send(());
                });
                let _ = jrx.recv_timeout(JOIN_TIMEOUT);
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                Err(anyhow::anyhow!(
                    "TCP connect timed out after {:?}",
                    CONNECT_TIMEOUT
                ))
            }
        }
    }

    /// Spawns a background thread for the TCP server.
    /// Accepts connections and reads/writes data, forwarding to main via channel.
    pub fn start_server(&mut self) -> Result<()> {
        self.disconnect();
        self.status = ConnectionStatus::Connecting;
        let addr = format!("{}:{}", self.host, self.port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => {
                l.set_nonblocking(true).ok();
                l
            }
            Err(e) => {
                self.status = ConnectionStatus::Error;
                self.error_count += 1;
                return Err(anyhow::anyhow!("TCP server bind failed: {}", e));
            }
        };

        // Channel for main→worker data sends
        let (tx_to_thread, rx_from_main) = mpsc::channel::<Vec<u8>>();
        // Bounded channel for worker→main received data
        let (tx_to_main, rx_from_thread) = mpsc::sync_channel::<Vec<u8>>(WORKER_TO_MAIN_BOUND);

        self.tx = Some(tx_to_thread);
        self.rx = Some(rx_from_thread);
        self.stop_flag = Arc::new(AtomicBool::new(false));
        self.worker_errored = Arc::new(AtomicBool::new(false));
        let stop_flag = self.stop_flag.clone();
        let worker_errored = self.worker_errored.clone();

        // Shared list of connected client addresses, updated atomically by worker
        let shared_clients: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let clients_clone = shared_clients.clone();
        self.shared_clients = Some(shared_clients);

        let handle = thread::spawn(move || {
            Self::server_worker_loop(
                listener,
                rx_from_main,
                tx_to_main,
                stop_flag,
                worker_errored,
                clients_clone,
            );
        });

        self.worker_handle = Some(handle);
        self.status = ConnectionStatus::Connected;
        self.is_server = true;
        info!("TCP server listening on {}", addr);
        Ok(())
    }

    /// Keep `try_accept` for backward compatibility — now a no-op since the
    /// worker thread handles accept internally.
    pub fn try_accept(&mut self) {
        // No-op: accept is handled by the worker thread in server mode.
    }

    /// Client worker: read/write loop after connection is established.
    fn client_worker_loop(
        stream: &mut TcpStream,
        rx_from_main: mpsc::Receiver<Vec<u8>>,
        tx_to_main: mpsc::SyncSender<Vec<u8>>,
        stop_flag: Arc<AtomicBool>,
        worker_errored: Arc<AtomicBool>,
    ) {
        let mut buf = [0u8; 4096];
        while !stop_flag.load(Ordering::Acquire) {
            // Drain ALL pending write requests from the main thread
            while let Ok(data) = rx_from_main.try_recv() {
                if let Err(e) = stream.write_all(&data) {
                    error!("TCP write error: {}", e);
                    worker_errored.store(true, Ordering::Release);
                    return;
                }
                let _ = stream.flush();
            }

            // Non-blocking read
            match stream.read(&mut buf) {
                Ok(0) => {
                    // Peer closed connection (FIN). Exit loop to avoid
                    // CPU spin on repeated Ok(0).
                    info!("TCP client: peer closed connection");
                    worker_errored.store(true, Ordering::Release);
                    return;
                }
                Ok(n) => {
                    if tx_to_main.try_send(buf[..n].to_vec()).is_err() {
                        // Channel full or disconnected
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    error!("TCP read error: {}", e);
                    worker_errored.store(true, Ordering::Release);
                    return;
                }
            }

            thread::sleep(Duration::from_millis(1));
        }
        info!("TCP client worker thread exited");
    }

    /// Server worker: accept loop + read/write with active client.
    fn server_worker_loop(
        listener: TcpListener,
        rx_from_main: mpsc::Receiver<Vec<u8>>,
        tx_to_main: mpsc::SyncSender<Vec<u8>>,
        stop_flag: Arc<AtomicBool>,
        worker_errored: Arc<AtomicBool>,
        connected_clients: Arc<Mutex<Vec<String>>>,
    ) {
        let mut current_stream: Option<TcpStream> = None;
        let mut buf = [0u8; 4096];

        while !stop_flag.load(Ordering::Acquire) {
            // Poll for new connections (non-blocking listener)
            if let Ok((stream, addr)) = listener.accept() {
                stream.set_nonblocking(true).ok();
                stream
                    .set_read_timeout(Some(Duration::from_millis(10)))
                    .ok();
                stream.set_write_timeout(Some(TCP_WRITE_TIMEOUT)).ok();
                stream.set_nodelay(true).ok();
                let addr_str = addr.to_string();

                // Close previous client connection
                if let Some(ref old) = current_stream {
                    old.shutdown(Shutdown::Both).ok();
                }

                if let Ok(mut clients) = connected_clients.lock() {
                    clients.clear();
                    clients.push(addr_str.clone());
                }

                current_stream = Some(stream);
                warn!(
                    "TCP server: new client {} replaced previous connection",
                    addr_str
                );
                info!("TCP client connected: {}", addr_str);
            }

            // Read from current stream
            if let Some(ref mut stream) = current_stream {
                match stream.read(&mut buf) {
                    Ok(0) => {
                        // Peer closed connection (FIN).
                        info!("TCP server: client disconnected (peer closed)");
                        current_stream = None;
                        if let Ok(mut clients) = connected_clients.lock() {
                            clients.clear();
                        }
                    }
                    Ok(n) => if tx_to_main.try_send(buf[..n].to_vec()).is_err() {},
                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => {
                        // Client disconnected
                        current_stream = None;
                        if let Ok(mut clients) = connected_clients.lock() {
                            clients.clear();
                        }
                    }
                }
            }

            // Drain ALL pending write requests from the main thread
            let mut drop_count: u32 = 0;
            while let Ok(data) = rx_from_main.try_recv() {
                if let Some(ref mut stream) = current_stream {
                    match stream.write_all(&data) {
                        Ok(()) => {
                            let _ = stream.flush();
                        }
                        Err(ref e)
                            if e.kind() == std::io::ErrorKind::BrokenPipe
                                || e.kind() == std::io::ErrorKind::ConnectionReset
                                || e.kind() == std::io::ErrorKind::ConnectionAborted =>
                        {
                            // Client gone — drop it, keep server alive.
                            warn!("TCP server: write failed (client gone): {}", e);
                            current_stream = None;
                            if let Ok(mut clients) = connected_clients.lock() {
                                clients.clear();
                            }
                        }
                        Err(e) => {
                            error!("TCP server: write error: {}", e);
                            worker_errored.store(true, Ordering::Release);
                        }
                    }
                } else {
                    // No client connected — data is dropped. Log once per batch.
                    drop_count += 1;
                }
            }
            if drop_count > 0 {
                warn!(
                    "TCP server: dropped {} message(s) — no client connected",
                    drop_count
                );
            }

            thread::sleep(Duration::from_millis(1));
        }
        info!("TCP server worker thread exited");
    }
}

impl ConnectionProvider for TcpService {
    fn is_connected(&self) -> bool {
        if self.worker_errored.load(Ordering::Acquire) {
            return false;
        }
        self.tx.is_some() && self.status.is_connected()
    }

    fn disconnect(&mut self) {
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
                        "TCP worker thread did not exit within {:?}; detaching",
                        JOIN_TIMEOUT
                    );
                }
            }
            let elapsed = join_start.elapsed();
            if elapsed > Duration::from_millis(100) {
                info!(
                    "TCP worker join took {:.1}s (timeout={:?})",
                    elapsed.as_secs_f32(),
                    JOIN_TIMEOUT
                );
            }
        }
        // Drain residual messages from the channel to prevent memory leak
        if let Some(rx) = self.rx.take() {
            while rx.try_recv().is_ok() {}
        }
        self.tx = None;
        self.status = if had_error {
            ConnectionStatus::HardwareFault
        } else {
            ConnectionStatus::Disconnected
        };
        self.connected_clients.clear();
        self.shared_clients = None;
    }

    fn try_read_raw(&mut self) -> Vec<u8> {
        // Detect worker thread I/O failure early so callers see the right status.
        if self.worker_errored.load(Ordering::Acquire)
            && self.status != ConnectionStatus::HardwareFault
        {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
        }

        // Sync connected_clients from the worker thread (server mode).
        if let Some(ref shared) = self.shared_clients {
            if let Ok(clients) = shared.lock() {
                self.connected_clients = clients.clone();
            }
        }

        // Reuse pre-allocated scratch buffer to avoid per-call heap allocation.
        self.rx_scratch.clear();
        if let Some(rx) = &self.rx {
            while let Ok(data) = rx.try_recv() {
                self.rx_scratch.extend_from_slice(&data);
            }
        }

        if !self.rx_scratch.is_empty() {
            self.bytes_received += self.rx_scratch.len() as u64;
            self.last_comm = Local::now().format("%H:%M:%S%.3f").to_string();
        }

        std::mem::take(&mut self.rx_scratch)
    }

    fn send_data(&mut self, data: &[u8]) -> Result<()> {
        // Fast-fail if the worker thread already exited due to I/O error.
        if self.worker_errored.load(Ordering::Acquire) {
            self.status = ConnectionStatus::HardwareFault;
            self.error_count += 1;
            return Err(anyhow::anyhow!(
                "TCP worker exited due to I/O error (device fault)"
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
                    self.status = ConnectionStatus::Error;
                    self.error_count += 1;
                    Err(anyhow::anyhow!("Background thread disconnected"))
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

    #[test]
    fn test_send_data_fails_fast_when_worker_errored() {
        let mut service = TcpService::default();
        service.worker_errored.store(true, Ordering::Release);
        let result = service.send_data(b"data");
        assert!(result.is_err());
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
    }

    #[test]
    fn test_try_read_raw_sets_hardware_fault_on_worker_error() {
        let mut service = TcpService::default();
        service.worker_errored.store(true, Ordering::Release);
        service.status = ConnectionStatus::Connected;
        let data = service.try_read_raw();
        assert!(data.is_empty());
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
        assert_eq!(service.error_count, 1);
    }

    #[test]
    fn test_disconnect_sets_hardware_fault_when_worker_errored() {
        let mut service = TcpService::default();
        service.worker_errored.store(true, Ordering::Release);
        service.disconnect();
        assert_eq!(service.status, ConnectionStatus::HardwareFault);
    }
}
