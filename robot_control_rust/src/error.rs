//! 统一错误类型 — 使用 thiserror 派生所有 Display/Error 实现

use thiserror::Error;

/// 应用程序顶层错误枚举
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Serial port error: {0}")]
    Serial(String),

    #[cfg(feature = "hardware")]
    #[error("Serial port I/O error: {0}")]
    SerialPort(#[from] serialport::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "mcp")]
    #[error("MCP server error: {0}")]
    Mcp(String),

    #[cfg(feature = "llm")]
    #[error("LLM API error: {0}")]
    Llm(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("{0}")]
    Other(String),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Other(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Other(s.to_string())
    }
}

/// 通用结果类型别名
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display_serial() {
        let e = AppError::Serial("port busy".into());
        assert!(e.to_string().contains("Serial port error"));
        assert!(e.to_string().contains("port busy"));
    }

    #[test]
    fn test_display_network() {
        let e = AppError::Network("connection refused".into());
        assert!(e.to_string().contains("Network error"));
        assert!(e.to_string().contains("connection refused"));
    }

    #[test]
    fn test_display_config() {
        let e = AppError::Config("invalid baud rate".into());
        assert!(e.to_string().contains("Configuration error"));
        assert!(e.to_string().contains("invalid baud rate"));
    }

    #[test]
    fn test_display_parse() {
        let e = AppError::Parse("unexpected byte".into());
        assert!(e.to_string().contains("Parse error"));
        assert!(e.to_string().contains("unexpected byte"));
    }

    #[test]
    fn test_display_validation() {
        let e = AppError::Validation("out of range".into());
        assert!(e.to_string().contains("Validation error"));
        assert!(e.to_string().contains("out of range"));
    }

    #[test]
    fn test_display_timeout() {
        let e = AppError::Timeout(5000);
        assert_eq!(e.to_string(), "Timeout after 5000ms");
    }

    #[test]
    fn test_display_other() {
        let e = AppError::Other("unknown".into());
        assert_eq!(e.to_string(), "unknown");
    }
    #[test]
    fn test_from_string() {
        let e: AppError = "test error".to_string().into();
        match e {
            AppError::Other(s) => assert_eq!(s, "test error"),
            _ => panic!("expected Other variant"),
        }
    }

    #[test]
    fn test_from_str_ref() {
        let e: AppError = "str error".into();
        match e {
            AppError::Other(s) => assert_eq!(s, "str error"),
            _ => panic!("expected Other variant"),
        }
    }
    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out");
        let e: AppError = io_err.into();
        match e {
            AppError::Io(inner) => {
                assert_eq!(inner.kind(), std::io::ErrorKind::TimedOut);
            }
            _ => panic!("expected Io variant"),
        }
    }

    #[test]
    fn test_from_io_error_broken_pipe() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        let e: AppError = io_err.into();
        assert!(e.to_string().contains("I/O error"));
        assert!(e.to_string().contains("pipe broke"));
    }
    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let e: AppError = json_err.into();
        match e {
            AppError::Json(_) => {}
            _ => panic!("expected Json variant"),
        }
        assert!(e.to_string().contains("JSON error"));
    }
    #[test]
    fn test_debug_format() {
        let e = AppError::Network("test".into());
        let debug = format!("{:?}", e);
        assert!(debug.contains("Network"));
        assert!(debug.contains("test"));
    }
    #[test]
    fn test_app_result_ok() {
        fn make_ok() -> AppResult<i32> {
            Ok(42)
        }
        assert_eq!(make_ok().unwrap(), 42);
    }

    #[test]
    fn test_app_result_err() {
        fn make_err() -> AppResult<i32> {
            Err(AppError::Timeout(1000))
        }
        assert!(make_err().is_err());
        assert!(make_err().unwrap_err().to_string().contains("1000ms"));
    }
    #[test]
    fn test_io_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e: AppError = io_err.into();
        // std::error::Error::source should be available
        let source = std::error::Error::source(&e);
        assert!(source.is_some());
    }

    #[test]
    fn test_other_has_no_source() {
        let e = AppError::Other("no source".into());
        let source = std::error::Error::source(&e);
        assert!(source.is_none());
    }
    #[test]
    fn test_from_empty_string() {
        let e: AppError = "".to_string().into();
        assert_eq!(e.to_string(), "");
    }

    #[test]
    fn test_from_empty_str() {
        let e: AppError = "".into();
        assert_eq!(e.to_string(), "");
    }

    #[test]
    fn test_timeout_zero() {
        let e = AppError::Timeout(0);
        assert_eq!(e.to_string(), "Timeout after 0ms");
    }

    #[test]
    fn test_timeout_max() {
        let e = AppError::Timeout(u64::MAX);
        assert!(e.to_string().contains(&u64::MAX.to_string()));
    }

    #[test]
    fn test_variant_equality_via_display() {
        // Verify different variants produce different display prefixes
        let serial = AppError::Serial("x".into()).to_string();
        let network = AppError::Network("x".into()).to_string();
        let config = AppError::Config("x".into()).to_string();
        let parse = AppError::Parse("x".into()).to_string();
        let validation = AppError::Validation("x".into()).to_string();

        assert_ne!(serial, network);
        assert_ne!(network, config);
        assert_ne!(config, parse);
        assert_ne!(parse, validation);
    }
}
