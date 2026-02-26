use serde::{Deserialize, Serialize};

/// 串口设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialDevice {
    /// 端口名称
    pub port_name: String,
    /// 波特率
    pub baud_rate: u32,
    /// 数据位
    pub data_bits: u8,
    /// 停止位
    pub stop_bits: u8,
    /// 奇偶校验
    pub parity: ParityMode,
    /// 超时 (毫秒)
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParityMode {
    None,
    Odd,
    Even,
}

impl Default for SerialDevice {
    fn default() -> Self {
        Self {
            port_name: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            stop_bits: 1,
            parity: ParityMode::None,
            timeout_ms: 1000,
        }
    }
}

impl SerialDevice {
    pub fn new(port_name: impl Into<String>) -> Self {
        Self {
            port_name: port_name.into(),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for ParityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParityMode::None => write!(f, "None"),
            ParityMode::Odd => write!(f, "Odd"),
            ParityMode::Even => write!(f, "Even"),
        }
    }
}
