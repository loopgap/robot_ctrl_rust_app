use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

const MAX_CAN_FRAMES: usize = 10_000;

/// CAN 帧（软件模拟 - PC端无物理CAN接口，用于帧构建/解析/仿真）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFrame {
    pub id: u32,
    pub extended: bool, // 标准帧(11bit) / 扩展帧(29bit)
    pub fd: bool,       // CAN FD
    pub brs: bool,      // Bit Rate Switch (CAN FD)
    pub rtr: bool,      // Remote Transmission Request
    pub data: Vec<u8>,
    pub timestamp: DateTime<Local>,
    pub direction: FrameDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameDirection {
    Tx,
    Rx,
}

impl std::fmt::Display for FrameDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tx => write!(f, "TX"),
            Self::Rx => write!(f, "RX"),
        }
    }
}

impl Default for CanFrame {
    fn default() -> Self {
        Self {
            id: 0x100,
            extended: false,
            fd: false,
            brs: false,
            rtr: false,
            data: vec![0; 8],
            timestamp: Local::now(),
            direction: FrameDirection::Tx,
        }
    }
}

impl CanFrame {
    pub fn max_data_len(&self) -> usize {
        if self.fd {
            64
        } else {
            8
        }
    }

    pub fn dlc(&self) -> u8 {
        if self.fd {
            match self.data.len() {
                0..=8 => self.data.len() as u8,
                9..=12 => 12,
                13..=16 => 16,
                17..=20 => 20,
                21..=24 => 24,
                25..=32 => 32,
                33..=48 => 48,
                _ => 64,
            }
        } else {
            self.data.len().min(8) as u8
        }
    }

    pub fn id_str(&self) -> String {
        if self.extended {
            format!("0x{:08X}", self.id)
        } else {
            format!("0x{:03X}", self.id & 0x7FF)
        }
    }

    pub fn data_hex(&self) -> String {
        self.data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 序列化帧为字节（简化协议）
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let flags: u8 = (self.extended as u8)
            | ((self.fd as u8) << 1)
            | ((self.brs as u8) << 2)
            | ((self.rtr as u8) << 3);
        bytes.push(flags);
        bytes.extend_from_slice(&self.id.to_le_bytes());
        bytes.push(self.data.len() as u8);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

/// CAN 服务（帧记录器 + 仿真）
pub struct CanService {
    pub frames: Vec<CanFrame>,
    pub dropped_frames: u64,
    pub filters: Vec<CanFilter>,
    pub is_running: bool,
    pub bitrate: u32,
    pub fd_enabled: bool,
    pub data_bitrate: u32,
    pub frame_count_tx: u64,
    pub frame_count_rx: u64,
    pub bus_load: f32,
    // 高级配置
    pub config_sample_point: f32,
    pub config_data_sample_point: f32,
    pub config_sjw: u8,
    pub config_data_sjw: u8,
    pub config_termination: bool,
    pub config_listen_only: bool,
    pub config_loopback: bool,
    pub config_auto_retransmit: bool,
    pub config_error_reporting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFilter {
    pub id: u32,
    pub mask: u32,
    pub enabled: bool,
    pub name: String,
}

impl Default for CanService {
    fn default() -> Self {
        Self::new()
    }
}

impl CanService {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            dropped_frames: 0,
            filters: Vec::new(),
            is_running: false,
            bitrate: 500_000,
            fd_enabled: false,
            data_bitrate: 2_000_000,
            frame_count_tx: 0,
            frame_count_rx: 0,
            bus_load: 0.0,
            config_sample_point: 0.875,
            config_data_sample_point: 0.750,
            config_sjw: 1,
            config_data_sjw: 1,
            config_termination: true,
            config_listen_only: false,
            config_loopback: false,
            config_auto_retransmit: true,
            config_error_reporting: true,
        }
    }

    pub fn max_frame_capacity(&self) -> usize {
        MAX_CAN_FRAMES
    }

    pub fn set_max_frames(&mut self, max_frames: usize) -> usize {
        let max_frames = max_frames.max(64).min(MAX_CAN_FRAMES);
        if self.frames.len() > max_frames {
            let overflow = self.frames.len() - max_frames;
            self.frames.drain(..overflow);
            self.dropped_frames += overflow as u64;
            return overflow;
        }
        0
    }

    pub fn add_frame(&mut self, frame: CanFrame) {
        match frame.direction {
            FrameDirection::Tx => self.frame_count_tx += 1,
            FrameDirection::Rx => self.frame_count_rx += 1,
        }
        self.frames.push(frame);
        // 限制历史大小
        if self.frames.len() > MAX_CAN_FRAMES {
            let overflow = self.frames.len() - MAX_CAN_FRAMES;
            self.frames.drain(..overflow);
            self.dropped_frames += overflow as u64;
        }
    }

    pub fn filtered_frames(&self) -> Vec<&CanFrame> {
        if self.filters.is_empty() || self.filters.iter().all(|f| !f.enabled) {
            return self.frames.iter().collect();
        }
        self.frames
            .iter()
            .filter(|frame| {
                self.filters.iter().any(|filter| {
                    filter.enabled && (frame.id & filter.mask) == (filter.id & filter.mask)
                })
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
        self.dropped_frames = 0;
        self.frame_count_tx = 0;
        self.frame_count_rx = 0;
    }

    /// 模拟发送帧（记录到日志）
    pub fn send_frame(&mut self, mut frame: CanFrame) {
        frame.direction = FrameDirection::Tx;
        frame.timestamp = Local::now();
        self.add_frame(frame);
    }

    /// 模拟接收帧（用于测试）
    pub fn simulate_rx(&mut self, id: u32, data: &[u8]) {
        let frame = CanFrame {
            id,
            data: data.to_vec(),
            direction: FrameDirection::Rx,
            timestamp: Local::now(),
            ..Default::default()
        };
        self.add_frame(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_frame_default() {
        let frame = CanFrame::default();
        assert_eq!(frame.id, 0x100);
        assert!(!frame.extended);
        assert!(!frame.fd);
        assert_eq!(frame.data.len(), 8);
    }

    #[test]
    fn test_can_frame_dlc_standard() {
        let frame = CanFrame {
            data: vec![0; 8],
            ..Default::default()
        };
        assert_eq!(frame.dlc(), 8);
    }

    #[test]
    fn test_can_frame_dlc_fd() {
        let frame = CanFrame {
            data: vec![0; 48],
            fd: true,
            ..Default::default()
        };
        assert_eq!(frame.dlc(), 48);
        let frame64 = CanFrame {
            data: vec![0; 64],
            fd: true,
            ..Default::default()
        };
        assert_eq!(frame64.dlc(), 64);
    }

    #[test]
    fn test_can_frame_max_data_len() {
        let std = CanFrame::default();
        assert_eq!(std.max_data_len(), 8);
        let fd = CanFrame {
            fd: true,
            ..Default::default()
        };
        assert_eq!(fd.max_data_len(), 64);
    }

    #[test]
    fn test_can_frame_id_str() {
        let std = CanFrame {
            id: 0x123,
            ..Default::default()
        };
        assert_eq!(std.id_str(), "0x123");
        let ext = CanFrame {
            id: 0x12345678,
            extended: true,
            ..Default::default()
        };
        assert_eq!(ext.id_str(), "0x12345678");
    }

    #[test]
    fn test_can_frame_id_str_standard_mask() {
        // 标准帧 ID 只有 11 bit, 应该 mask 到 0x7FF
        let std = CanFrame {
            id: 0xFFF,
            extended: false,
            ..Default::default()
        };
        assert_eq!(std.id_str(), "0x7FF"); // masked
    }

    #[test]
    fn test_can_frame_data_hex() {
        let f = CanFrame {
            data: vec![0xAA, 0x55, 0x00],
            ..Default::default()
        };
        assert_eq!(f.data_hex(), "AA 55 00");
    }

    #[test]
    fn test_can_frame_to_bytes() {
        let f = CanFrame {
            id: 0x100,
            data: vec![1, 2, 3],
            ..Default::default()
        };
        let bytes = f.to_bytes();
        assert_eq!(bytes[0], 0x00); // flags: extended=0, fd=0, brs=0, rtr=0
                                    // id is LE 4 bytes
        assert_eq!(&bytes[1..5], &0x100u32.to_le_bytes());
        assert_eq!(bytes[5], 3); // data length
        assert_eq!(&bytes[6..], &[1, 2, 3]);
    }

    #[test]
    fn test_can_service_new() {
        let svc = CanService::new();
        assert!(!svc.is_running);
        assert_eq!(svc.bitrate, 500_000);
        assert!(svc.frames.is_empty());
        assert_eq!(svc.dropped_frames, 0);
    }

    #[test]
    fn test_can_service_send_frame() {
        let mut svc = CanService::new();
        svc.send_frame(CanFrame::default());
        assert_eq!(svc.frame_count_tx, 1);
        assert_eq!(svc.frame_count_rx, 0);
        assert_eq!(svc.frames.len(), 1);
        assert_eq!(svc.frames[0].direction, FrameDirection::Tx);
    }

    #[test]
    fn test_can_service_simulate_rx() {
        let mut svc = CanService::new();
        svc.simulate_rx(0x200, &[0x01, 0x02]);
        assert_eq!(svc.frame_count_rx, 1);
        assert_eq!(svc.frames[0].id, 0x200);
        assert_eq!(svc.frames[0].direction, FrameDirection::Rx);
    }

    #[test]
    fn test_can_service_clear() {
        let mut svc = CanService::new();
        svc.send_frame(CanFrame::default());
        svc.simulate_rx(0x100, &[0]);
        svc.clear();
        assert!(svc.frames.is_empty());
        assert_eq!(svc.frame_count_tx, 0);
        assert_eq!(svc.frame_count_rx, 0);
    }

    #[test]
    fn test_can_service_frame_limit() {
        let mut svc = CanService::new();
        for i in 0..10500 {
            svc.simulate_rx(i as u32, &[0]);
        }
        assert!(svc.frames.len() <= MAX_CAN_FRAMES);
        assert_eq!(svc.dropped_frames, 500);
    }

    #[test]
    fn test_can_filter() {
        let mut svc = CanService::new();
        svc.filters.push(CanFilter {
            id: 0x100,
            mask: 0x7FF,
            enabled: true,
            name: "Test".into(),
        });
        svc.simulate_rx(0x100, &[1]);
        svc.simulate_rx(0x200, &[2]);
        let filtered = svc.filtered_frames();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 0x100);
    }

    #[test]
    fn test_can_filter_disabled() {
        let mut svc = CanService::new();
        svc.filters.push(CanFilter {
            id: 0x100,
            mask: 0x7FF,
            enabled: false,
            name: "Off".into(),
        });
        svc.simulate_rx(0x100, &[1]);
        svc.simulate_rx(0x200, &[2]);
        let filtered = svc.filtered_frames();
        // 所有filter都disabled时，返回全部帧
        assert_eq!(filtered.len(), 2);
    }
}
