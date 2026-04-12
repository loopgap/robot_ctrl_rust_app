use serde::{Deserialize, Serialize};
use std::fmt;

/// 连接状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl ConnectionStatus {
    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    /// 是否已断开（包括错误状态）
    pub fn is_disconnected(&self) -> bool {
        matches!(
            self,
            ConnectionStatus::Disconnected | ConnectionStatus::Error
        )
    }

    /// 获取状态对应的颜色 (R, G, B, A)
    pub fn color(&self) -> (u8, u8, u8, u8) {
        match self {
            ConnectionStatus::Connected => (0, 200, 0, 50),
            ConnectionStatus::Connecting => (255, 165, 0, 50),
            ConnectionStatus::Error => (255, 0, 0, 50),
            ConnectionStatus::Disconnected => (128, 128, 128, 50),
        }
    }
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Connecting => write!(f, "Connecting"),
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Error => write!(f, "Error"),
        }
    }
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}
