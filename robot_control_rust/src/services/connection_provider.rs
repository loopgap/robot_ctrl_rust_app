use anyhow::Result;

pub trait ConnectionProvider {
    fn is_connected(&self) -> bool;
    fn disconnect(&mut self);
    fn try_read_raw(&mut self) -> Vec<u8>;
    fn send_data(&mut self, data: &[u8]) -> Result<()>;
    fn reset_stats(&mut self);
}
