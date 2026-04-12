//! Connection abstraction traits

use crate::error::Result;

/// Trait for all connection providers (Serial, TCP, UDP, CAN, etc.)
pub trait ConnectionProvider: Send + Sync {
    /// Check if the connection is currently active
    fn is_connected(&self) -> bool;

    /// Disconnect from the remote endpoint
    fn disconnect(&mut self);

    /// Try to read raw bytes from the connection
    fn try_read_raw(&mut self) -> Vec<u8>;

    /// Send raw bytes to the remote endpoint
    fn send_data(&mut self, data: &[u8]) -> Result<()>;

    /// Reset connection statistics
    fn reset_stats(&mut self);

    /// Get connection name for display
    fn name(&self) -> &str;
}
