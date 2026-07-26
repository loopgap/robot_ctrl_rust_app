//! 统一错误类型 — 使用 thiserror 派生
// The CLI path currently reports `BootEntryError` directly; suppress
// dead-code noise until ToolError is adopted there as well.
#![cfg_attr(not(feature = "gui"), allow(dead_code))]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Hash / encoding error: {0}")]
    Hash(String),

    #[error("Chrono parse error: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("{0}")]
    Other(String),
}

impl From<String> for ToolError {
    fn from(s: String) -> Self {
        ToolError::Other(s)
    }
}

impl From<&str> for ToolError {
    fn from(s: &str) -> Self {
        ToolError::Other(s.to_string())
    }
}

impl From<base64::DecodeError> for ToolError {
    fn from(e: base64::DecodeError) -> Self {
        ToolError::Hash(e.to_string())
    }
}

pub type ToolResult<T> = Result<T, ToolError>;
