use anyhow::Result;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait ConnectionProvider {
    fn is_connected(&self) -> bool;
    fn disconnect(&mut self);
    fn try_read_raw(&mut self) -> Vec<u8>;
    fn send_data(&mut self, data: &[u8]) -> Result<()>;
    fn reset_stats(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_connection_provider_basic() {
        let mut mock = MockConnectionProvider::new();
        mock.expect_is_connected().return_const(true);
        assert!(mock.is_connected());
    }

    #[test]
    fn test_mock_connection_provider_send() {
        let mut mock = MockConnectionProvider::new();
        mock.expect_send_data()
            .withf(|data| data == b"test")
            .returning(|_| Ok(()));
        mock.send_data(b"test").unwrap();
    }

    #[test]
    fn test_mock_connection_provider_read() {
        let mut mock = MockConnectionProvider::new();
        mock.expect_try_read_raw().return_once(|| vec![0x01, 0x02]);
        assert_eq!(mock.try_read_raw(), vec![0x01, 0x02]);
    }

    #[test]
    fn test_mock_connection_provider_disconnect() {
        let mut mock = MockConnectionProvider::new();
        mock.expect_disconnect().times(1).returning(|| ());
        mock.disconnect();
    }

    #[test]
    fn test_mock_connection_provider_reset_stats() {
        let mut mock = MockConnectionProvider::new();
        mock.expect_reset_stats().times(1).returning(|| ());
        mock.reset_stats();
    }
}
