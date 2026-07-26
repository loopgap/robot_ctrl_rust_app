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
