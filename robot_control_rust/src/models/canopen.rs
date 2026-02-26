use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NmtCommand {
    StartRemoteNode = 0x01,
    StopRemoteNode = 0x02,
    EnterPreOperational = 0x80,
    ResetNode = 0x81,
    ResetCommunication = 0x82,
}

impl NmtCommand {
    pub fn all() -> &'static [NmtCommand] {
        &[
            Self::StartRemoteNode,
            Self::StopRemoteNode,
            Self::EnterPreOperational,
            Self::ResetNode,
            Self::ResetCommunication,
        ]
    }

    pub fn code(self) -> u8 {
        self as u8
    }
}

impl std::fmt::Display for NmtCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartRemoteNode => write!(f, "Start Remote Node (0x01)"),
            Self::StopRemoteNode => write!(f, "Stop Remote Node (0x02)"),
            Self::EnterPreOperational => write!(f, "Enter Pre-Operational (0x80)"),
            Self::ResetNode => write!(f, "Reset Node (0x81)"),
            Self::ResetCommunication => write!(f, "Reset Communication (0x82)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdoAction {
    UploadRequest,
    DownloadExpedited,
}

impl SdoAction {
    pub fn all() -> &'static [SdoAction] {
        &[Self::UploadRequest, Self::DownloadExpedited]
    }
}

impl std::fmt::Display for SdoAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UploadRequest => write!(f, "Upload Request (Read)"),
            Self::DownloadExpedited => write!(f, "Download Expedited (Write)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanopenFrame {
    pub cob_id: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CanopenSdoRequest {
    pub node_id: u8,
    pub action: SdoAction,
    pub index: u16,
    pub sub_index: u8,
    pub payload: Vec<u8>,
}

impl Default for CanopenSdoRequest {
    fn default() -> Self {
        Self {
            node_id: 1,
            action: SdoAction::UploadRequest,
            index: 0x1000,
            sub_index: 0,
            payload: Vec::new(),
        }
    }
}

impl CanopenSdoRequest {
    pub fn build(&self) -> CanopenFrame {
        let cob_id = 0x600 + self.node_id as u16;
        let mut data = vec![0u8; 8];

        match self.action {
            SdoAction::UploadRequest => {
                data[0] = 0x40;
            }
            SdoAction::DownloadExpedited => {
                let payload_len = self.payload.len().min(4);
                let n = (4 - payload_len) as u8;
                data[0] = 0x23 | (n << 2);
                for (i, b) in self.payload.iter().take(4).enumerate() {
                    data[4 + i] = *b;
                }
            }
        }

        let [idx_lo, idx_hi] = self.index.to_le_bytes();
        data[1] = idx_lo;
        data[2] = idx_hi;
        data[3] = self.sub_index;

        CanopenFrame { cob_id, data }
    }
}

pub fn build_nmt(node_id: u8, cmd: NmtCommand) -> CanopenFrame {
    CanopenFrame {
        cob_id: 0x000,
        data: vec![cmd.code(), node_id],
    }
}

pub fn build_heartbeat_producer_sdo(node_id: u8, producer_ms: u16) -> CanopenFrame {
    let req = CanopenSdoRequest {
        node_id,
        action: SdoAction::DownloadExpedited,
        index: 0x1017,
        sub_index: 0x00,
        payload: producer_ms.to_le_bytes().to_vec(),
    };
    req.build()
}

pub fn build_pdo(cob_id: u16, data: &[u8]) -> CanopenFrame {
    CanopenFrame {
        cob_id,
        data: data.iter().copied().take(8).collect(),
    }
}

pub fn decode_heartbeat_state(state: u8) -> &'static str {
    match state {
        0x00 => "Boot-up",
        0x04 => "Stopped",
        0x05 => "Operational",
        0x7F => "Pre-operational",
        _ => "Unknown",
    }
}

pub fn decode_emcy(data: &[u8]) -> Option<(u16, u8, String)> {
    if data.len() < 3 {
        return None;
    }
    let err = u16::from_le_bytes([data[0], data[1]]);
    let err_reg = data[2];
    let class = match err & 0xFF00 {
        0x1000 => "Generic Error",
        0x2000 => "Current",
        0x3000 => "Voltage",
        0x4000 => "Temperature",
        0x5000 => "Hardware",
        0x6000 => "Software",
        0x7000 => "Additional Modules",
        0x8000 => "Monitoring",
        0x9000 => "External Error",
        0xF000 => "Additional Functions",
        _ => "Manufacturer Specific",
    };
    Some((err, err_reg, class.to_string()))
}

pub fn canopen_id_role(cob_id: u16) -> &'static str {
    match cob_id {
        0x000 => "NMT",
        0x080 => "SYNC",
        0x081..=0x0FF => "EMCY",
        0x101..=0x17F => "TIME/Reserved",
        0x181..=0x1FF => "TPDO1",
        0x201..=0x27F => "RPDO1",
        0x281..=0x2FF => "TPDO2",
        0x301..=0x37F => "RPDO2",
        0x381..=0x3FF => "TPDO3",
        0x401..=0x47F => "RPDO3",
        0x481..=0x4FF => "TPDO4",
        0x501..=0x57F => "RPDO4",
        0x581..=0x5FF => "TSDO",
        0x601..=0x67F => "RSDO",
        0x701..=0x77F => "Heartbeat",
        _ => "Non-Standard",
    }
}

pub fn object_dict_name(index: u16, sub_index: u8) -> &'static str {
    match (index, sub_index) {
        (0x1000, 0x00) => "Device Type",
        (0x1001, 0x00) => "Error Register",
        (0x1002, 0x00) => "Manufacturer Status Register",
        (0x1003, _) => "Pre-defined Error Field",
        (0x1005, 0x00) => "SYNC COB-ID",
        (0x1006, 0x00) => "Communication Cycle Period",
        (0x1007, 0x00) => "Synchronous Window Length",
        (0x1008, 0x00) => "Manufacturer Device Name",
        (0x1009, 0x00) => "Manufacturer Hardware Version",
        (0x100A, 0x00) => "Manufacturer Software Version",
        (0x100C, 0x00) => "Guard Time",
        (0x100D, 0x00) => "Life Time Factor",
        (0x1010, _) => "Store Parameters",
        (0x1011, _) => "Restore Default Parameters",
        (0x1014, 0x00) => "EMCY COB-ID",
        (0x1015, 0x00) => "EMCY Inhibit Time",
        (0x1016, _) => "Consumer Heartbeat Time",
        (0x1017, 0x00) => "Producer Heartbeat Time",
        (0x1018, 0x00) => "Identity Object (count)",
        (0x1018, 0x01) => "Vendor ID",
        (0x1018, 0x02) => "Product Code",
        (0x1018, 0x03) => "Revision Number",
        (0x1018, 0x04) => "Serial Number",
        (0x1400, _) => "RPDO1 Communication",
        (0x1401, _) => "RPDO2 Communication",
        (0x1402, _) => "RPDO3 Communication",
        (0x1403, _) => "RPDO4 Communication",
        (0x1600, _) => "RPDO1 Mapping",
        (0x1601, _) => "RPDO2 Mapping",
        (0x1602, _) => "RPDO3 Mapping",
        (0x1603, _) => "RPDO4 Mapping",
        (0x1800, 0x00) => "TPDO1 Communication (count)",
        (0x1800, 0x01) => "TPDO1 COB-ID",
        (0x1800, 0x02) => "TPDO1 Transmission Type",
        (0x1800, 0x03) => "TPDO1 Inhibit Time",
        (0x1800, 0x05) => "TPDO1 Event Timer",
        (0x1801, _) => "TPDO2 Communication",
        (0x1802, _) => "TPDO3 Communication",
        (0x1803, _) => "TPDO4 Communication",
        (0x1A00, 0x00) => "TPDO1 Mapping Entries",
        (0x1A00, _) => "TPDO1 Mapping Sub",
        (0x1A01, _) => "TPDO2 Mapping",
        (0x1A02, _) => "TPDO3 Mapping",
        (0x1A03, _) => "TPDO4 Mapping",
        (0x6000..=0x67FF, _) => "Device Profile Input",
        (0x6800..=0x6FFF, _) => "Device Profile Output",
        _ => "Custom Object",
    }
}

// ═══════════════════════════════════════════════════════════════
// PDO 映射与外部数据结构
// ═══════════════════════════════════════════════════════════════

/// PDO 方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdoDirection {
    Transmit,
    Receive,
}

impl std::fmt::Display for PdoDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transmit => write!(f, "TPDO"),
            Self::Receive => write!(f, "RPDO"),
        }
    }
}

/// PDO 字段数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdoDataType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
}

impl PdoDataType {
    pub fn all() -> &'static [PdoDataType] {
        &[
            Self::Bool,
            Self::U8,
            Self::I8,
            Self::U16,
            Self::I16,
            Self::U32,
            Self::I32,
            Self::F32,
        ]
    }

    pub fn bit_size(self) -> u8 {
        match self {
            Self::Bool => 1,
            Self::U8 | Self::I8 => 8,
            Self::U16 | Self::I16 => 16,
            Self::U32 | Self::I32 | Self::F32 => 32,
        }
    }

    pub fn byte_size(self) -> usize {
        self.bit_size().div_ceil(8) as usize
    }
}

impl std::fmt::Display for PdoDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// PDO 映射条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdoMappingEntry {
    pub name: String,
    pub index: u16,
    pub sub_index: u8,
    pub bit_length: u8,
    pub data_type: PdoDataType,
}

impl Default for PdoMappingEntry {
    fn default() -> Self {
        Self {
            name: "Signal".into(),
            index: 0x6000,
            sub_index: 0x01,
            bit_length: 16,
            data_type: PdoDataType::U16,
        }
    }
}

/// PDO 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdoConfig {
    pub name: String,
    pub direction: PdoDirection,
    pub cob_id: u16,
    pub node_id: u8,
    pub mappings: Vec<PdoMappingEntry>,
    pub enabled: bool,
}

impl Default for PdoConfig {
    fn default() -> Self {
        Self {
            name: "PDO1".into(),
            direction: PdoDirection::Transmit,
            cob_id: 0x181,
            node_id: 1,
            mappings: vec![PdoMappingEntry::default()],
            enabled: true,
        }
    }
}

impl PdoConfig {
    /// 计算映射总位数
    pub fn total_bits(&self) -> u32 {
        self.mappings.iter().map(|m| m.bit_length as u32).sum()
    }

    /// 计算映射总字节数
    pub fn total_bytes(&self) -> usize {
        self.total_bits().div_ceil(8) as usize
    }

    /// 从映射构建 PDO 数据帧
    pub fn build_from_values(&self, values: &[f64]) -> CanopenFrame {
        let mut data = [0u8; 8];
        let mut bit_offset: usize = 0;

        for (i, mapping) in self.mappings.iter().enumerate() {
            let value = values.get(i).copied().unwrap_or(0.0);
            let byte_offset = bit_offset / 8;
            let bit_in_byte = bit_offset % 8;
            let bytes_needed = mapping.data_type.byte_size();

            if byte_offset + bytes_needed <= 8 && bit_in_byte == 0 {
                match mapping.data_type {
                    PdoDataType::Bool => {
                        if value != 0.0 {
                            data[byte_offset] |= 1 << (bit_offset % 8);
                        }
                    }
                    PdoDataType::U8 => data[byte_offset] = value as u8,
                    PdoDataType::I8 => data[byte_offset] = value as i8 as u8,
                    PdoDataType::U16 => {
                        let v = (value as u16).to_le_bytes();
                        data[byte_offset..byte_offset + 2].copy_from_slice(&v);
                    }
                    PdoDataType::I16 => {
                        let v = (value as i16).to_le_bytes();
                        data[byte_offset..byte_offset + 2].copy_from_slice(&v);
                    }
                    PdoDataType::U32 => {
                        let v = (value as u32).to_le_bytes();
                        data[byte_offset..byte_offset + 4].copy_from_slice(&v);
                    }
                    PdoDataType::I32 => {
                        let v = (value as i32).to_le_bytes();
                        data[byte_offset..byte_offset + 4].copy_from_slice(&v);
                    }
                    PdoDataType::F32 => {
                        let v = (value as f32).to_le_bytes();
                        data[byte_offset..byte_offset + 4].copy_from_slice(&v);
                    }
                }
            }
            bit_offset += mapping.bit_length as usize;
        }

        let actual_len = self.total_bytes().min(8);
        CanopenFrame {
            cob_id: self.cob_id,
            data: data[..actual_len].to_vec(),
        }
    }

    /// 将 PDO 数据按映射解码为数值
    pub fn decode_values(&self, data: &[u8]) -> Vec<(String, String, f64)> {
        let mut results = Vec::new();
        let mut bit_offset: usize = 0;

        for mapping in &self.mappings {
            let byte_offset = bit_offset / 8;
            let bytes_needed = mapping.data_type.byte_size();
            let raw_value: f64 = if byte_offset + bytes_needed <= data.len() {
                match mapping.data_type {
                    PdoDataType::Bool => {
                        let bit_in_byte = bit_offset % 8;
                        if (data[byte_offset] >> bit_in_byte) & 1 == 1 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    PdoDataType::U8 => data[byte_offset] as f64,
                    PdoDataType::I8 => data[byte_offset] as i8 as f64,
                    PdoDataType::U16 => {
                        u16::from_le_bytes([data[byte_offset], data[byte_offset + 1]]) as f64
                    }
                    PdoDataType::I16 => {
                        i16::from_le_bytes([data[byte_offset], data[byte_offset + 1]]) as f64
                    }
                    PdoDataType::U32 => {
                        let b = &data[byte_offset..byte_offset + 4];
                        u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64
                    }
                    PdoDataType::I32 => {
                        let b = &data[byte_offset..byte_offset + 4];
                        i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64
                    }
                    PdoDataType::F32 => {
                        let b = &data[byte_offset..byte_offset + 4];
                        f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64
                    }
                }
            } else {
                0.0
            };

            let display = match mapping.data_type {
                PdoDataType::Bool => if raw_value != 0.0 { "TRUE" } else { "FALSE" }.to_string(),
                PdoDataType::F32 => format!("{:.4}", raw_value),
                _ => format!("{}", raw_value as i64),
            };

            results.push((mapping.name.clone(), display, raw_value));
            bit_offset += mapping.bit_length as usize;
        }

        results
    }

    /// 导出为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// 从 JSON 导入
    pub fn from_json(json: &str) -> Option<PdoConfig> {
        serde_json::from_str(json).ok()
    }
}

/// 预定义的 PDO 配置集（工业常见）
pub fn preset_pdo_configs() -> Vec<PdoConfig> {
    vec![
        PdoConfig {
            name: "CiA 402 Statusword + Position".into(),
            direction: PdoDirection::Transmit,
            cob_id: 0x181,
            node_id: 1,
            mappings: vec![
                PdoMappingEntry {
                    name: "Statusword".into(),
                    index: 0x6041,
                    sub_index: 0,
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                },
                PdoMappingEntry {
                    name: "Position Actual".into(),
                    index: 0x6064,
                    sub_index: 0,
                    bit_length: 32,
                    data_type: PdoDataType::I32,
                },
            ],
            enabled: true,
        },
        PdoConfig {
            name: "CiA 402 Controlword + Target".into(),
            direction: PdoDirection::Receive,
            cob_id: 0x201,
            node_id: 1,
            mappings: vec![
                PdoMappingEntry {
                    name: "Controlword".into(),
                    index: 0x6040,
                    sub_index: 0,
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                },
                PdoMappingEntry {
                    name: "Target Position".into(),
                    index: 0x607A,
                    sub_index: 0,
                    bit_length: 32,
                    data_type: PdoDataType::I32,
                },
            ],
            enabled: true,
        },
        PdoConfig {
            name: "CiA 402 Velocity Mode".into(),
            direction: PdoDirection::Transmit,
            cob_id: 0x281,
            node_id: 1,
            mappings: vec![
                PdoMappingEntry {
                    name: "Statusword".into(),
                    index: 0x6041,
                    sub_index: 0,
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                },
                PdoMappingEntry {
                    name: "Velocity Actual".into(),
                    index: 0x606C,
                    sub_index: 0,
                    bit_length: 32,
                    data_type: PdoDataType::I32,
                },
            ],
            enabled: true,
        },
        PdoConfig {
            name: "CiA 401 Digital IO".into(),
            direction: PdoDirection::Transmit,
            cob_id: 0x181,
            node_id: 1,
            mappings: vec![
                PdoMappingEntry {
                    name: "DI Byte 0".into(),
                    index: 0x6000,
                    sub_index: 1,
                    bit_length: 8,
                    data_type: PdoDataType::U8,
                },
                PdoMappingEntry {
                    name: "DI Byte 1".into(),
                    index: 0x6000,
                    sub_index: 2,
                    bit_length: 8,
                    data_type: PdoDataType::U8,
                },
                PdoMappingEntry {
                    name: "AI Channel 0".into(),
                    index: 0x6401,
                    sub_index: 1,
                    bit_length: 16,
                    data_type: PdoDataType::I16,
                },
                PdoMappingEntry {
                    name: "AI Channel 1".into(),
                    index: 0x6401,
                    sub_index: 2,
                    bit_length: 16,
                    data_type: PdoDataType::I16,
                },
            ],
            enabled: true,
        },
    ]
}

/// CANopen 帧深度解析结果
#[derive(Debug, Clone)]
pub struct CanopenFrameAnalysis {
    pub cob_id: u16,
    pub node_id: u8,
    pub role: &'static str,
    pub fields: Vec<CanopenFieldInfo>,
    pub valid: bool,
    pub summary: String,
}

/// CANopen 帧字段信息
#[derive(Debug, Clone)]
pub struct CanopenFieldInfo {
    pub name: String,
    pub offset: usize,
    pub length: usize,
    pub raw_hex: String,
    pub decoded: String,
    pub color_idx: u8,
}

/// 深度解析 CANopen 帧
pub fn analyze_canopen_frame(cob_id: u16, data: &[u8]) -> CanopenFrameAnalysis {
    let role = canopen_id_role(cob_id);
    let node_id = if cob_id > 0 { (cob_id & 0x7F) as u8 } else { 0 };
    let mut fields = Vec::new();
    let mut valid = true;
    let summary;

    match role {
        "NMT" => {
            if data.len() >= 2 {
                let cmd_name = match data[0] {
                    0x01 => "Start",
                    0x02 => "Stop",
                    0x80 => "Pre-Op",
                    0x81 => "Reset Node",
                    0x82 => "Reset Comm",
                    _ => "Unknown",
                };
                fields.push(CanopenFieldInfo {
                    name: "NMT Command".into(),
                    offset: 0,
                    length: 1,
                    raw_hex: format!("{:02X}", data[0]),
                    decoded: cmd_name.into(),
                    color_idx: 0,
                });
                fields.push(CanopenFieldInfo {
                    name: "Node ID".into(),
                    offset: 1,
                    length: 1,
                    raw_hex: format!("{:02X}", data[1]),
                    decoded: format!("Node {}", data[1]),
                    color_idx: 1,
                });
                summary = format!("NMT {} → Node {}", cmd_name, data[1]);
            } else {
                valid = false;
                summary = "NMT: insufficient data".into();
            }
        }
        "TSDO" | "RSDO" => {
            if data.len() >= 4 {
                let cmd = data[0];
                let idx = u16::from_le_bytes([data[1], data[2]]);
                let sub = data[3];
                let ccs = cmd >> 5;
                let sdo_type = match (role, ccs) {
                    ("RSDO", 1) => "Download Init",
                    ("RSDO", 2) => "Upload Init",
                    ("TSDO", 2) => "Upload Response",
                    ("TSDO", 3) => "Download Response",
                    ("RSDO", 3) => "Download Segment",
                    ("TSDO", 0) => "Upload Segment",
                    (_, 4) => "Abort",
                    _ => "Unknown",
                };
                fields.push(CanopenFieldInfo {
                    name: "SDO Cmd".into(),
                    offset: 0,
                    length: 1,
                    raw_hex: format!("{:02X}", cmd),
                    decoded: format!("{} (ccs={})", sdo_type, ccs),
                    color_idx: 0,
                });
                fields.push(CanopenFieldInfo {
                    name: "Index".into(),
                    offset: 1,
                    length: 2,
                    raw_hex: format!("{:02X} {:02X}", data[1], data[2]),
                    decoded: format!("0x{:04X} [{}]", idx, object_dict_name(idx, sub)),
                    color_idx: 1,
                });
                fields.push(CanopenFieldInfo {
                    name: "SubIndex".into(),
                    offset: 3,
                    length: 1,
                    raw_hex: format!("{:02X}", sub),
                    decoded: format!("0x{:02X}", sub),
                    color_idx: 2,
                });
                if data.len() > 4 {
                    let payload_hex: Vec<String> =
                        data[4..].iter().map(|b| format!("{:02X}", b)).collect();
                    fields.push(CanopenFieldInfo {
                        name: "SDO Data".into(),
                        offset: 4,
                        length: data.len() - 4,
                        raw_hex: payload_hex.join(" "),
                        decoded: format!("{} bytes", data.len() - 4),
                        color_idx: 3,
                    });
                }
                summary = format!("{} {} 0x{:04X}:{:02X}", role, sdo_type, idx, sub);
            } else {
                valid = false;
                summary = format!("{}: insufficient SDO data", role);
            }
        }
        "EMCY" => {
            if let Some((err, reg, class)) = decode_emcy(data) {
                fields.push(CanopenFieldInfo {
                    name: "Error Code".into(),
                    offset: 0,
                    length: 2,
                    raw_hex: format!("{:02X} {:02X}", data[0], data[1]),
                    decoded: format!("0x{:04X} [{}]", err, class),
                    color_idx: 0,
                });
                fields.push(CanopenFieldInfo {
                    name: "Error Register".into(),
                    offset: 2,
                    length: 1,
                    raw_hex: format!("{:02X}", reg),
                    decoded: format!("0x{:02X}", reg),
                    color_idx: 1,
                });
                if data.len() > 3 {
                    let mfr: Vec<String> = data[3..].iter().map(|b| format!("{:02X}", b)).collect();
                    fields.push(CanopenFieldInfo {
                        name: "Manufacturer Data".into(),
                        offset: 3,
                        length: data.len() - 3,
                        raw_hex: mfr.join(" "),
                        decoded: format!("{} bytes", data.len() - 3),
                        color_idx: 2,
                    });
                }
                summary = format!("EMCY 0x{:04X} [{}]", err, class);
            } else {
                valid = false;
                summary = "EMCY: insufficient data".into();
            }
        }
        "Heartbeat" => {
            if let Some(&state_byte) = data.first() {
                let state_name = decode_heartbeat_state(state_byte);
                fields.push(CanopenFieldInfo {
                    name: "NMT State".into(),
                    offset: 0,
                    length: 1,
                    raw_hex: format!("{:02X}", state_byte),
                    decoded: format!("{} (0x{:02X})", state_name, state_byte),
                    color_idx: 0,
                });
                summary = format!("Heartbeat: Node {} = {}", node_id, state_name);
            } else {
                valid = false;
                summary = "Heartbeat: no data".into();
            }
        }
        s if s.contains("PDO") => {
            for (i, b) in data.iter().enumerate() {
                fields.push(CanopenFieldInfo {
                    name: format!("Byte {}", i),
                    offset: i,
                    length: 1,
                    raw_hex: format!("{:02X}", b),
                    decoded: format!("0x{:02X} ({})", b, b),
                    color_idx: (i % 4) as u8,
                });
            }
            summary = format!("{} Node {} [{} bytes]", role, node_id, data.len());
        }
        _ => {
            for (i, b) in data.iter().enumerate() {
                fields.push(CanopenFieldInfo {
                    name: format!("Byte {}", i),
                    offset: i,
                    length: 1,
                    raw_hex: format!("{:02X}", b),
                    decoded: format!("0x{:02X}", b),
                    color_idx: (i % 4) as u8,
                });
            }
            summary = format!("{} COB-ID=0x{:03X} [{} bytes]", role, cob_id, data.len());
        }
    }

    CanopenFrameAnalysis {
        cob_id,
        node_id,
        role,
        fields,
        valid,
        summary,
    }
}

// ═══════════════════════════════════════════════════════════════
// CAN / CAN FD 标准帧抽象
// ═══════════════════════════════════════════════════════════════

/// CAN 协议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanProtocolType {
    Standard,    // CAN 2.0A/B
    Fd,          // CAN FD
    EtherCatCoE, // EtherCAT CAN-over-EtherCAT
}

impl CanProtocolType {
    pub fn all() -> &'static [CanProtocolType] {
        &[Self::Standard, Self::Fd, Self::EtherCatCoE]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "CAN 2.0",
            Self::Fd => "CAN FD",
            Self::EtherCatCoE => "EtherCAT CoE",
        }
    }
}

impl std::fmt::Display for CanProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// 通用 CAN / CAN FD 帧结构
#[derive(Debug, Clone)]
pub struct CanStdFrame {
    pub can_id: u32,
    pub data: Vec<u8>,
    pub is_extended: bool, // 29-bit ID
    pub is_fd: bool,       // CAN FD
    pub brs: bool,         // Bit Rate Switch (FD only)
    pub esi: bool,         // Error State Indicator (FD only)
}

impl CanStdFrame {
    /// 构建标准 CAN 2.0 帧
    pub fn new(id: u32, data: &[u8], extended: bool) -> Self {
        Self {
            can_id: if extended {
                id & 0x1FFF_FFFF
            } else {
                id & 0x7FF
            },
            data: data.iter().copied().take(8).collect(),
            is_extended: extended,
            is_fd: false,
            brs: false,
            esi: false,
        }
    }

    /// 构建 CAN FD 帧（最大 64 字节）
    pub fn new_fd(id: u32, data: &[u8], extended: bool) -> Self {
        Self {
            can_id: if extended {
                id & 0x1FFF_FFFF
            } else {
                id & 0x7FF
            },
            data: data.iter().copied().take(64).collect(),
            is_extended: extended,
            is_fd: true,
            brs: true,
            esi: false,
        }
    }

    pub fn dlc(&self) -> usize {
        self.data.len()
    }

    /// CAN FD DLC 编码（返回 0~15 的 DLC 码）
    pub fn fd_dlc_code(&self) -> Option<u8> {
        if !self.is_fd {
            return None;
        }
        Some(fd_len_to_dlc(self.data.len()))
    }

    /// 转为 CanopenFrame（若兼容 11-bit ID + <=8 bytes）
    pub fn to_canopen_frame(&self) -> Option<CanopenFrame> {
        if self.is_extended || self.data.len() > 8 {
            return None;
        }
        Some(CanopenFrame {
            cob_id: self.can_id as u16,
            data: self.data.clone(),
        })
    }
}

/// CAN FD DLC → 实际长度 映射
pub fn fd_dlc_to_len(dlc: u8) -> usize {
    match dlc {
        0..=8 => dlc as usize,
        9 => 12,
        10 => 16,
        11 => 20,
        12 => 24,
        13 => 32,
        14 => 48,
        15 => 64,
        _ => 64,
    }
}

/// 实际长度 → CAN FD DLC 映射
pub fn fd_len_to_dlc(len: usize) -> u8 {
    match len {
        0..=8 => len as u8,
        9..=12 => 9,
        13..=16 => 10,
        17..=20 => 11,
        21..=24 => 12,
        25..=32 => 13,
        33..=48 => 14,
        _ => 15,
    }
}

/// CAN FD 合法载荷长度
pub fn is_fd_valid_len(len: usize) -> bool {
    matches!(len, 0..=8 | 12 | 16 | 20 | 24 | 32 | 48 | 64)
}

// ═══════════════════════════════════════════════════════════════
// EtherCAT CoE (CAN-over-EtherCAT) 支持
// ═══════════════════════════════════════════════════════════════

/// EtherCAT 状态机状态
pub fn ecat_state_name(state: u8) -> &'static str {
    match state {
        1 => "Init",
        2 => "Pre-Operational",
        3 => "Bootstrap",
        4 => "Safe-Operational",
        8 => "Operational",
        _ => "Unknown",
    }
}

/// EtherCAT CoE SDO 请求
#[derive(Debug, Clone)]
pub struct EcatCoeSdoRequest {
    pub slave_addr: u16,
    pub index: u16,
    pub sub_index: u8,
    pub data: Vec<u8>,
    pub is_write: bool,
}

/// EtherCAT CoE 帧（简化模型）
#[derive(Debug, Clone)]
pub struct EcatCoeFrame {
    pub mailbox_header: Vec<u8>, // 6 bytes: length(2) + address(2) + channel/priority + type
    pub coe_data: Vec<u8>,       // CoE SDO data
    pub summary: String,
}

impl EcatCoeSdoRequest {
    /// 构建 CoE SDO 请求帧
    pub fn build_coe_frame(&self) -> EcatCoeFrame {
        // Mailbox Header (6 bytes)
        let sdo_data_len = if self.is_write {
            6 + self.data.len()
        } else {
            6
        };
        let mbx_len = (2 + sdo_data_len) as u16; // CoE header(2) + SDO
        let mut header = Vec::with_capacity(6);
        header.extend_from_slice(&mbx_len.to_le_bytes()); // Length
        header.extend_from_slice(&self.slave_addr.to_le_bytes()); // Address
        header.push(0x00); // Channel/Priority
        header.push(0x03); // Mailbox type = CoE (0x03)

        // CoE Header (2 bytes): number=0, service=SDO request
        let mut coe = Vec::new();
        let coe_type: u16 = 0x02 << 12; // CoE SDO service type
        coe.extend_from_slice(&coe_type.to_le_bytes());

        // SDO Header
        let cmd = if self.is_write {
            let n = (4usize.saturating_sub(self.data.len())) as u8;
            0x23u8 | (n << 2) // Download expedited
        } else {
            0x40u8 // Upload request
        };
        coe.push(cmd);
        let [idx_lo, idx_hi] = self.index.to_le_bytes();
        coe.push(idx_lo);
        coe.push(idx_hi);
        coe.push(self.sub_index);
        if self.is_write {
            for &b in self.data.iter().take(4) {
                coe.push(b);
            }
            // Pad to 4 bytes
            coe.resize(coe.len() + (4 - self.data.len().min(4)), 0);
        } else {
            coe.extend_from_slice(&[0, 0, 0, 0]);
        }

        let action = if self.is_write { "Write" } else { "Read" };
        let summary = format!(
            "CoE SDO {} Slave={} Idx=0x{:04X}:{:02X}",
            action, self.slave_addr, self.index, self.sub_index
        );

        EcatCoeFrame {
            mailbox_header: header,
            coe_data: coe,
            summary,
        }
    }
}

/// EtherCAT CoE 帧分析结果
#[derive(Debug, Clone)]
pub struct EcatCoeAnalysis {
    pub fields: Vec<CanopenFieldInfo>,
    pub summary: String,
    pub valid: bool,
}

/// 分析 EtherCAT CoE 帧（从 mailbox data 开始）
pub fn analyze_ecat_coe_frame(data: &[u8]) -> EcatCoeAnalysis {
    let mut fields = Vec::new();
    let summary;
    let valid;

    if data.len() >= 6 {
        // Mailbox Header
        let mbx_len = u16::from_le_bytes([data[0], data[1]]);
        let mbx_addr = u16::from_le_bytes([data[2], data[3]]);
        let mbx_type = data[5] & 0x0F;

        fields.push(CanopenFieldInfo {
            name: "MBX Length".into(),
            offset: 0,
            length: 2,
            raw_hex: format!("{:02X} {:02X}", data[0], data[1]),
            decoded: format!("{} bytes", mbx_len),
            color_idx: 0,
        });
        fields.push(CanopenFieldInfo {
            name: "MBX Address".into(),
            offset: 2,
            length: 2,
            raw_hex: format!("{:02X} {:02X}", data[2], data[3]),
            decoded: format!("Slave {}", mbx_addr),
            color_idx: 1,
        });
        fields.push(CanopenFieldInfo {
            name: "MBX Type".into(),
            offset: 5,
            length: 1,
            raw_hex: format!("{:02X}", data[5]),
            decoded: match mbx_type {
                0x01 => "ERR".into(),
                0x02 => "AoE".into(),
                0x03 => "CoE".into(),
                0x04 => "FoE".into(),
                0x05 => "SoE".into(),
                _ => format!("Type {}", mbx_type),
            },
            color_idx: 2,
        });

        // CoE SDO Data (starts at offset 6)
        if mbx_type == 0x03 && data.len() >= 12 {
            let cmd = data[8];
            let ccs = cmd >> 5;
            let idx = u16::from_le_bytes([data[9], data[10]]);
            let sub = data[11];
            let sdo_type = match ccs {
                1 => "Download Init",
                2 => "Upload Init",
                3 => "Download Seg",
                4 => "Abort",
                _ => "Unknown",
            };
            fields.push(CanopenFieldInfo {
                name: "CoE SDO Cmd".into(),
                offset: 8,
                length: 1,
                raw_hex: format!("{:02X}", cmd),
                decoded: format!("{} (ccs={})", sdo_type, ccs),
                color_idx: 3,
            });
            fields.push(CanopenFieldInfo {
                name: "OD Index".into(),
                offset: 9,
                length: 2,
                raw_hex: format!("{:02X} {:02X}", data[9], data[10]),
                decoded: format!("0x{:04X} [{}]", idx, object_dict_name(idx, sub)),
                color_idx: 4,
            });
            fields.push(CanopenFieldInfo {
                name: "SubIndex".into(),
                offset: 11,
                length: 1,
                raw_hex: format!("{:02X}", sub),
                decoded: format!("0x{:02X}", sub),
                color_idx: 0,
            });
            summary = format!(
                "CoE SDO {} 0x{:04X}:{:02X} → Slave {}",
                sdo_type, idx, sub, mbx_addr
            );
            valid = true;
        } else {
            summary = format!(
                "Mailbox Type={} Slave={} Len={}",
                mbx_type, mbx_addr, mbx_len
            );
            valid = data.len() >= 6;
        }
    } else {
        summary = "EtherCAT: insufficient data".into();
        valid = false;
    }

    EcatCoeAnalysis {
        fields,
        summary,
        valid,
    }
}

// ═══════════════════════════════════════════════════════════════
// 多协议帧联合体
// ═══════════════════════════════════════════════════════════════

/// 统一多协议帧
#[derive(Debug, Clone)]
pub struct MultiProtocolFrame {
    pub protocol: CanProtocolType,
    pub frame: CanStdFrame,
    pub ecat_coe: Option<EcatCoeFrame>,
}

impl MultiProtocolFrame {
    /// 创建 CANopen 标准 CAN 帧
    pub fn canopen_nmt(node_id: u8, cmd: NmtCommand) -> Self {
        let co_frame = build_nmt(node_id, cmd);
        Self {
            protocol: CanProtocolType::Standard,
            frame: CanStdFrame::new(co_frame.cob_id as u32, &co_frame.data, false),
            ecat_coe: None,
        }
    }

    /// CAN FD PDO（支持 >8 字节载荷）
    pub fn can_fd_pdo(cob_id: u16, data: &[u8]) -> Self {
        Self {
            protocol: CanProtocolType::Fd,
            frame: CanStdFrame::new_fd(cob_id as u32, data, false),
            ecat_coe: None,
        }
    }

    /// EtherCAT CoE SDO
    pub fn ecat_coe_sdo(slave: u16, index: u16, sub_index: u8, data: &[u8], write: bool) -> Self {
        let req = EcatCoeSdoRequest {
            slave_addr: slave,
            index,
            sub_index,
            data: data.to_vec(),
            is_write: write,
        };
        let coe = req.build_coe_frame();
        Self {
            protocol: CanProtocolType::EtherCatCoE,
            frame: CanStdFrame::new(0, &[], false), // EtherCAT does not use CAN ID
            ecat_coe: Some(coe),
        }
    }

    /// 合并发送数据（序列化为字节流）
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.protocol {
            CanProtocolType::Standard | CanProtocolType::Fd => {
                let mut out = Vec::new();
                out.extend_from_slice(&self.frame.can_id.to_le_bytes());
                out.push(self.frame.dlc() as u8);
                out.extend_from_slice(&self.frame.data);
                out
            }
            CanProtocolType::EtherCatCoE => {
                if let Some(ref coe) = self.ecat_coe {
                    let mut out = coe.mailbox_header.clone();
                    out.extend_from_slice(&coe.coe_data);
                    out
                } else {
                    Vec::new()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_nmt() {
        let f = build_nmt(1, NmtCommand::StartRemoteNode);
        assert_eq!(f.cob_id, 0x000);
        assert_eq!(f.data, vec![0x01, 0x01]);
    }

    #[test]
    fn test_build_sdo_upload() {
        let r = CanopenSdoRequest {
            node_id: 5,
            action: SdoAction::UploadRequest,
            index: 0x1018,
            sub_index: 1,
            payload: vec![],
        };
        let f = r.build();
        assert_eq!(f.cob_id, 0x605);
        assert_eq!(f.data[0], 0x40);
        assert_eq!(f.data[1], 0x18);
        assert_eq!(f.data[2], 0x10);
        assert_eq!(f.data[3], 0x01);
    }

    #[test]
    fn test_build_sdo_download_expedited_2bytes() {
        let r = CanopenSdoRequest {
            node_id: 1,
            action: SdoAction::DownloadExpedited,
            index: 0x1017,
            sub_index: 0,
            payload: vec![0xE8, 0x03],
        };
        let f = r.build();
        assert_eq!(f.data[0], 0x2B);
        assert_eq!(f.data[4], 0xE8);
        assert_eq!(f.data[5], 0x03);
    }

    #[test]
    fn test_decode_heartbeat_state() {
        assert_eq!(decode_heartbeat_state(0x05), "Operational");
        assert_eq!(decode_heartbeat_state(0x7F), "Pre-operational");
    }

    #[test]
    fn test_decode_emcy() {
        let out = decode_emcy(&[0x00, 0x10, 0x01, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(out.0, 0x1000);
        assert_eq!(out.1, 0x01);
    }

    #[test]
    fn test_canopen_id_role() {
        assert_eq!(canopen_id_role(0x000), "NMT");
        assert_eq!(canopen_id_role(0x605), "RSDO");
        assert_eq!(canopen_id_role(0x705), "Heartbeat");
    }

    // ── CAN / CAN FD 标准帧测试 ──
    #[test]
    fn test_can_std_frame_build() {
        let f = CanStdFrame::new(0x123, &[0xAA, 0xBB], false);
        assert_eq!(f.can_id, 0x123);
        assert_eq!(f.data.len(), 2);
        assert!(!f.is_extended);
        assert!(!f.is_fd);
        assert_eq!(f.dlc(), 2);
    }

    #[test]
    fn test_can_fd_frame_build() {
        let payload = vec![0u8; 24];
        let f = CanStdFrame::new_fd(0x1ABCDEF, &payload, true);
        assert!(f.is_extended);
        assert!(f.is_fd);
        assert_eq!(f.data.len(), 24);
        assert_eq!(f.dlc(), 24);
        assert_eq!(f.fd_dlc_code(), Some(12));
    }

    #[test]
    fn test_can_fd_dlc_mapping() {
        assert_eq!(fd_dlc_to_len(8), 8);
        assert_eq!(fd_dlc_to_len(9), 12);
        assert_eq!(fd_dlc_to_len(15), 64);
        assert_eq!(fd_dlc_to_len(7), 7);
    }

    // ── EtherCAT CoE 测试 ──
    #[test]
    fn test_ecat_coe_sdo_build() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 1,
            index: 0x6040,
            sub_index: 0,
            data: vec![0x06, 0x00],
            is_write: true,
        };
        let frame = sdo.build_coe_frame();
        assert!(!frame.mailbox_header.is_empty());
        assert!(frame.mailbox_header.len() >= 6);
        assert!(!frame.coe_data.is_empty());
    }

    #[test]
    fn test_ecat_state_name() {
        assert_eq!(ecat_state_name(1), "Init");
        assert_eq!(ecat_state_name(2), "Pre-Operational");
        assert_eq!(ecat_state_name(4), "Safe-Operational");
        assert_eq!(ecat_state_name(8), "Operational");
        assert_eq!(ecat_state_name(0xFF), "Unknown");
    }

    #[test]
    fn test_analyze_ecat_coe_frame() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 1,
            index: 0x6040,
            sub_index: 0,
            data: vec![0x06, 0x00],
            is_write: true,
        };
        let frame = sdo.build_coe_frame();
        let mut combined = frame.mailbox_header.clone();
        combined.extend_from_slice(&frame.coe_data);
        let analysis = analyze_ecat_coe_frame(&combined);
        assert!(analysis.fields.len() >= 2);
        assert!(!analysis.summary.is_empty());
    }

    #[test]
    fn test_can_protocol_type_display() {
        assert_eq!(CanProtocolType::Standard.label(), "CAN 2.0");
        assert_eq!(CanProtocolType::Fd.label(), "CAN FD");
        assert_eq!(CanProtocolType::EtherCatCoE.label(), "EtherCAT CoE");
    }

    #[test]
    fn test_multi_protocol_frame_builder() {
        // Standard CAN -> CANopen NMT
        let f = MultiProtocolFrame::canopen_nmt(1, NmtCommand::StartRemoteNode);
        assert_eq!(f.protocol, CanProtocolType::Standard);
        assert!(!f.frame.is_fd);

        // CAN FD PDO
        let payload = vec![0u8; 16];
        let f2 = MultiProtocolFrame::can_fd_pdo(0x181, &payload);
        assert_eq!(f2.protocol, CanProtocolType::Fd);
        assert!(f2.frame.is_fd);
        assert_eq!(f2.frame.data.len(), 16);
    }
}
