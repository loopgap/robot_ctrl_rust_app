use serde::{Deserialize, Serialize};

/// CANopen NMT state per CiA 301 §7.2.1.
///
/// A node transitions through these states after power-up or reset:
/// - **Initializing**: self-configuration phase (vendor-specific).
/// - **PreOperational**: NMT slave, PDOs not active, SDOs active.
/// - **Operational**: full communication (PDOs + SDOs active).
/// - **Stopped**: only NMT and Boot-Up frames processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NmtState {
    #[default]
    Initializing,
    PreOperational,
    Operational,
    Stopped,
}

impl NmtState {
    /// Decode NMT heartbeat protocol byte (per CiA 301 §7.2.8).
    pub fn from_heartbeat_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(Self::Initializing),
            0x04 => Some(Self::Stopped),
            0x05 => Some(Self::Operational),
            0x7F => Some(Self::PreOperational),
            _ => None,
        }
    }

    pub fn heartbeat_code(self) -> u8 {
        match self {
            Self::Initializing => 0x00,
            Self::Stopped => 0x04,
            Self::Operational => 0x05,
            Self::PreOperational => 0x7F,
        }
    }
}

impl std::fmt::Display for NmtState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "Initializing"),
            Self::PreOperational => write!(f, "Pre-Operational"),
            Self::Operational => write!(f, "Operational"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

/// CANopen Error Register bit field per CiA 301 §7.5.1.
///
/// Bit 0 = generic error, Bit 1 = current, Bit 2 = voltage,
/// Bit 3 = temperature, Bit 4 = communication error, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EmcyErrorRegister {
    pub bits: u8,
}

impl EmcyErrorRegister {
    pub fn generic_error(&self) -> bool {
        self.bits & 0x01 != 0
    }
    pub fn current_error(&self) -> bool {
        self.bits & 0x02 != 0
    }
    pub fn voltage_error(&self) -> bool {
        self.bits & 0x04 != 0
    }
    pub fn temperature_error(&self) -> bool {
        self.bits & 0x08 != 0
    }
    pub fn communication_error(&self) -> bool {
        self.bits & 0x10 != 0
    }
    pub fn device_profile_error(&self) -> bool {
        self.bits & 0x20 != 0
    }
    pub fn manufacturer_error(&self) -> bool {
        self.bits & 0x80 != 0
    }

    pub fn describe(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.generic_error() {
            v.push("Generic Error");
        }
        if self.current_error() {
            v.push("Current Error");
        }
        if self.voltage_error() {
            v.push("Voltage Error");
        }
        if self.temperature_error() {
            v.push("Temperature Error");
        }
        if self.communication_error() {
            v.push("Communication Error");
        }
        if self.device_profile_error() {
            v.push("Device Profile Error");
        }
        if self.manufacturer_error() {
            v.push("Manufacturer Error");
        }
        if v.is_empty() {
            v.push("No Error");
        }
        v
    }
}

/// CANopen EMCY (Emergency) error code classification per CiA 301 §7.5.1.
///
/// Error codes 0x1000–0xFFFF are grouped into classes:
/// 0x1000 Generic, 0x2000 Current, 0x3000 Voltage,
/// 0x4000 Temperature, 0x5000 Device HW, 0x6000 Device SW,
/// 0x7000 Monitoring, 0x8000 External, 0xF000 Additional HW,
/// 0xFF00 Device-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmcyErrorClass {
    GenericError,
    Current,
    Voltage,
    Temperature,
    DeviceHardware,
    DeviceSoftware,
    Monitoring,
    External,
    AdditionalHardware,
    DeviceSpecific,
    Unknown(u16),
}

impl EmcyErrorClass {
    pub fn from_code(code: u16) -> Self {
        match code {
            0x1000..=0x1FFF => Self::GenericError,
            0x2000..=0x2FFF => Self::Current,
            0x3000..=0x3FFF => Self::Voltage,
            0x4000..=0x4FFF => Self::Temperature,
            0x5000..=0x5FFF => Self::DeviceHardware,
            0x6000..=0x6FFF => Self::DeviceSoftware,
            0x7000..=0x7FFF => Self::Monitoring,
            0x8000..=0x8FFF => Self::External,
            0xF000..=0xF0FF => Self::AdditionalHardware,
            0xFF00..=0xFFFF => Self::DeviceSpecific,
            other => Self::Unknown(other),
        }
    }

    /// Decode known EMCY error codes to human-readable description.
    pub fn describe_code(code: u16) -> &'static str {
        match code {
            0x1000 => "Error Reset / No Error",
            0x1001 => "Generic Error",
            0x1002 => "Current (at output) cannot be eliminated",
            0x1003 => "Voltage cannot be eliminated",
            0x1004 => "Temperature error",
            0x1005 => "Device hardware error",
            0x1006 => "Device software error",
            0x1007 => "Monitoring error (watchdog)",
            0x2110 => "CAN overrun (objects lost)",
            0x2130 => "PDO not processed due to length error",
            0x2200 => "RxPDO length exceeded",
            0x3100 => "Input voltage too high",
            0x3110 => "Input voltage too low",
            0x3200 => "Output voltage too high",
            0x3210 => "Output voltage too low",
            0x4210 => "Ambient temperature too high",
            0x4310 => "Device temperature too high",
            0x5100 => "Power supply fault",
            0x6100 => "Software reset (watchdog)",
            0x6110 => "Internal software error",
            0x6200 => "User parameter error",
            0x7100 => "Sensor fault",
            0x7200 => "Speed/position limit exceeded",
            0x8100 => "CAN bus off",
            0x8110 => "CRC error on CAN message",
            0x8120 => "Protocol error (FTM/ATM)",
            0x8130 => "PDO not served (length mismatch)",
            0x8200 => "Sync error (too many devices or bus disturbance)",
            0xFF01 => "Manufacturer-specific: motor stall detected",
            _ => "Unknown EMCY error code",
        }
    }
}

impl std::fmt::Display for EmcyErrorClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenericError => write!(f, "Generic Error"),
            Self::Current => write!(f, "Current"),
            Self::Voltage => write!(f, "Voltage"),
            Self::Temperature => write!(f, "Temperature"),
            Self::DeviceHardware => write!(f, "Device Hardware"),
            Self::DeviceSoftware => write!(f, "Device Software"),
            Self::Monitoring => write!(f, "Monitoring"),
            Self::External => write!(f, "External"),
            Self::AdditionalHardware => write!(f, "Additional Hardware"),
            Self::DeviceSpecific => write!(f, "Device-Specific"),
            Self::Unknown(c) => write!(f, "Unknown(0x{:04X})", c),
        }
    }
}

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

// PDO 映射与外部数据结构

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
        node_id,
        role,
        fields,
        valid,
        summary,
    }
}

// CAN / CAN FD 标准帧抽象

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
    pub brs: bool,         // Bit Rate Switch (FD only)
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
            brs: false,
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
            brs: true,
        }
    }

    pub fn dlc(&self) -> usize {
        self.data.len()
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

// EtherCAT CoE (CAN-over-EtherCAT) 支持

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

// 多协议帧联合体

/// 统一多协议帧
#[derive(Debug, Clone)]
pub struct MultiProtocolFrame {
    pub protocol: CanProtocolType,
    pub frame: CanStdFrame,
    pub ecat_coe: Option<EcatCoeFrame>,
}

impl MultiProtocolFrame {
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

    #[test]
    fn test_can_std_frame_build() {
        let f = CanStdFrame::new(0x123, &[0xAA, 0xBB], false);
        assert_eq!(f.can_id, 0x123);
        assert_eq!(f.data.len(), 2);
        assert!(!f.is_extended);
        assert_eq!(f.dlc(), 2);
    }

    #[test]
    fn test_can_fd_frame_build() {
        let payload = vec![0u8; 24];
        let f = CanStdFrame::new_fd(0x1ABCDEF, &payload, true);
        assert!(f.is_extended);
        assert_eq!(f.data.len(), 24);
        assert_eq!(f.dlc(), 24);
    }

    #[test]
    fn test_can_fd_dlc_mapping() {
        assert_eq!(fd_dlc_to_len(8), 8);
        assert_eq!(fd_dlc_to_len(9), 12);
        assert_eq!(fd_dlc_to_len(15), 64);
        assert_eq!(fd_dlc_to_len(7), 7);
    }

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
        // CAN FD PDO
        let payload = vec![0u8; 16];
        let f2 = MultiProtocolFrame::can_fd_pdo(0x181, &payload);
        assert_eq!(f2.protocol, CanProtocolType::Fd);
        assert_eq!(f2.frame.data.len(), 16);
    }
    #[test]
    fn decode_heartbeat_all_states() {
        assert_eq!(decode_heartbeat_state(0x00), "Boot-up");
        assert_eq!(decode_heartbeat_state(0x04), "Stopped");
        assert_eq!(decode_heartbeat_state(0x05), "Operational");
        assert_eq!(decode_heartbeat_state(0x7F), "Pre-operational");
        assert_eq!(decode_heartbeat_state(0x01), "Unknown");
        assert_eq!(decode_heartbeat_state(0xFF), "Unknown");
    }
    #[test]
    fn decode_emcy_too_short_returns_none() {
        assert!(decode_emcy(&[]).is_none());
        assert!(decode_emcy(&[0x00]).is_none());
        assert!(decode_emcy(&[0x00, 0x01]).is_none());
    }

    #[test]
    fn decode_emcy_all_error_classes() {
        let cases: &[(u16, &str)] = &[
            (0x1000, "Generic Error"),
            (0x20FF, "Current"),
            (0x3000, "Voltage"),
            (0x4000, "Temperature"),
            (0x5000, "Hardware"),
            (0x6000, "Software"),
            (0x7000, "Additional Modules"),
            (0x8000, "Monitoring"),
            (0x9000, "External Error"),
            (0xF000, "Additional Functions"),
            (0xABCD, "Manufacturer Specific"),
        ];
        for &(err_code, expected_class) in cases {
            let bytes = err_code.to_le_bytes();
            let data = [bytes[0], bytes[1], 0x01, 0, 0, 0, 0, 0];
            let out = decode_emcy(&data).unwrap();
            assert_eq!(out.0, err_code);
            assert_eq!(out.2, expected_class, "err=0x{:04X}", err_code);
        }
    }

    #[test]
    fn decode_emcy_error_register_preserved() {
        let data = [0x00, 0x20, 0xFF, 0, 0, 0, 0, 0];
        let out = decode_emcy(&data).unwrap();
        assert_eq!(out.1, 0xFF);
    }
    #[test]
    fn canopen_id_role_all_pdo_ranges() {
        assert_eq!(canopen_id_role(0x181), "TPDO1");
        assert_eq!(canopen_id_role(0x1FF), "TPDO1");
        assert_eq!(canopen_id_role(0x201), "RPDO1");
        assert_eq!(canopen_id_role(0x281), "TPDO2");
        assert_eq!(canopen_id_role(0x301), "RPDO2");
        assert_eq!(canopen_id_role(0x381), "TPDO3");
        assert_eq!(canopen_id_role(0x401), "RPDO3");
        assert_eq!(canopen_id_role(0x481), "TPDO4");
        assert_eq!(canopen_id_role(0x501), "RPDO4");
    }

    #[test]
    fn canopen_id_role_sdo_and_heartbeat() {
        assert_eq!(canopen_id_role(0x581), "TSDO");
        assert_eq!(canopen_id_role(0x5FF), "TSDO");
        assert_eq!(canopen_id_role(0x601), "RSDO");
        assert_eq!(canopen_id_role(0x67F), "RSDO");
        assert_eq!(canopen_id_role(0x701), "Heartbeat");
        assert_eq!(canopen_id_role(0x77F), "Heartbeat");
    }

    #[test]
    fn canopen_id_role_special_ids() {
        assert_eq!(canopen_id_role(0x000), "NMT");
        assert_eq!(canopen_id_role(0x080), "SYNC");
        assert_eq!(canopen_id_role(0x081), "EMCY");
        assert_eq!(canopen_id_role(0x0FF), "EMCY");
        assert_eq!(canopen_id_role(0x101), "TIME/Reserved");
    }

    #[test]
    fn canopen_id_role_non_standard() {
        assert_eq!(canopen_id_role(0x780), "Non-Standard");
        assert_eq!(canopen_id_role(0xFFF), "Non-Standard");
    }
    #[test]
    fn object_dict_name_communication_objects() {
        assert_eq!(object_dict_name(0x1000, 0), "Device Type");
        assert_eq!(object_dict_name(0x1001, 0), "Error Register");
        assert_eq!(object_dict_name(0x1005, 0), "SYNC COB-ID");
        assert_eq!(object_dict_name(0x1017, 0), "Producer Heartbeat Time");
        assert_eq!(object_dict_name(0x1018, 1), "Vendor ID");
        assert_eq!(object_dict_name(0x1018, 4), "Serial Number");
    }

    #[test]
    fn object_dict_name_pdo_objects() {
        assert_eq!(object_dict_name(0x1400, 0), "RPDO1 Communication");
        assert_eq!(object_dict_name(0x1600, 0), "RPDO1 Mapping");
        assert_eq!(object_dict_name(0x1800, 1), "TPDO1 COB-ID");
        assert_eq!(object_dict_name(0x1A00, 0), "TPDO1 Mapping Entries");
    }

    #[test]
    fn object_dict_name_device_profile() {
        assert_eq!(object_dict_name(0x6000, 0), "Device Profile Input");
        assert_eq!(object_dict_name(0x6800, 0), "Device Profile Output");
        assert_eq!(object_dict_name(0xFFFF, 0), "Custom Object");
    }
    #[test]
    fn pdo_data_type_bit_sizes() {
        assert_eq!(PdoDataType::Bool.bit_size(), 1);
        assert_eq!(PdoDataType::U8.bit_size(), 8);
        assert_eq!(PdoDataType::I8.bit_size(), 8);
        assert_eq!(PdoDataType::U16.bit_size(), 16);
        assert_eq!(PdoDataType::I16.bit_size(), 16);
        assert_eq!(PdoDataType::U32.bit_size(), 32);
        assert_eq!(PdoDataType::I32.bit_size(), 32);
        assert_eq!(PdoDataType::F32.bit_size(), 32);
    }

    #[test]
    fn pdo_data_type_byte_sizes() {
        assert_eq!(PdoDataType::Bool.byte_size(), 1); // ceil(1/8)
        assert_eq!(PdoDataType::U8.byte_size(), 1);
        assert_eq!(PdoDataType::U16.byte_size(), 2);
        assert_eq!(PdoDataType::U32.byte_size(), 4);
        assert_eq!(PdoDataType::F32.byte_size(), 4);
    }

    #[test]
    fn pdo_data_type_display() {
        assert_eq!(format!("{}", PdoDataType::Bool), "Bool");
        assert_eq!(format!("{}", PdoDataType::U16), "U16");
        assert_eq!(format!("{}", PdoDataType::F32), "F32");
    }
    #[test]
    fn pdo_direction_display() {
        assert_eq!(format!("{}", PdoDirection::Transmit), "TPDO");
        assert_eq!(format!("{}", PdoDirection::Receive), "RPDO");
    }

    #[test]
    fn pdo_direction_not_equal() {
        assert_ne!(PdoDirection::Transmit, PdoDirection::Receive);
    }
    #[test]
    fn pdo_mapping_entry_default() {
        let entry = PdoMappingEntry::default();
        assert_eq!(entry.name, "Signal");
        assert_eq!(entry.index, 0x6000);
        assert_eq!(entry.sub_index, 0x01);
        assert_eq!(entry.bit_length, 16);
        assert_eq!(entry.data_type, PdoDataType::U16);
    }
    #[test]
    fn pdo_config_default() {
        let cfg = PdoConfig::default();
        assert_eq!(cfg.name, "PDO1");
        assert_eq!(cfg.direction, PdoDirection::Transmit);
        assert_eq!(cfg.cob_id, 0x181);
        assert_eq!(cfg.node_id, 1);
        assert!(cfg.enabled);
        assert_eq!(cfg.mappings.len(), 1);
    }

    #[test]
    fn pdo_config_total_bits_single_mapping() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                bit_length: 16,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(cfg.total_bits(), 16);
        assert_eq!(cfg.total_bytes(), 2);
    }

    #[test]
    fn pdo_config_total_bits_multiple_mappings() {
        let cfg = PdoConfig {
            mappings: vec![
                PdoMappingEntry {
                    bit_length: 8,
                    ..Default::default()
                },
                PdoMappingEntry {
                    bit_length: 16,
                    ..Default::default()
                },
                PdoMappingEntry {
                    bit_length: 1,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        assert_eq!(cfg.total_bits(), 25);
        assert_eq!(cfg.total_bytes(), 4); // ceil(25/8)
    }

    #[test]
    fn pdo_config_build_u16_values() {
        let cfg = PdoConfig {
            mappings: vec![
                PdoMappingEntry {
                    name: "Speed".into(),
                    index: 0x6000,
                    sub_index: 1,
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                },
                PdoMappingEntry {
                    name: "Torque".into(),
                    index: 0x6000,
                    sub_index: 2,
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                },
            ],
            ..Default::default()
        };
        let frame = cfg.build_from_values(&[1000.0, 2000.0]);
        // First U16 (1000 = 0x03E8) at offset 0
        assert_eq!(frame.data[0], 0xE8);
        assert_eq!(frame.data[1], 0x03);
        // Second U16 (2000 = 0x07D0) at offset 2
        assert_eq!(frame.data[2], 0xD0);
        assert_eq!(frame.data[3], 0x07);
    }

    #[test]
    fn pdo_config_build_missing_values_default_zero() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                bit_length: 16,
                data_type: PdoDataType::U16,
                ..Default::default()
            }],
            ..Default::default()
        };
        let frame = cfg.build_from_values(&[]);
        assert_eq!(frame.data[0], 0);
        assert_eq!(frame.data[1], 0);
    }
    #[test]
    fn fd_dlc_to_len_standard_range() {
        for dlc in 0u8..=8 {
            assert_eq!(fd_dlc_to_len(dlc), dlc as usize);
        }
    }

    #[test]
    fn fd_dlc_to_len_extended_range() {
        assert_eq!(fd_dlc_to_len(9), 12);
        assert_eq!(fd_dlc_to_len(10), 16);
        assert_eq!(fd_dlc_to_len(11), 20);
        assert_eq!(fd_dlc_to_len(12), 24);
        assert_eq!(fd_dlc_to_len(13), 32);
        assert_eq!(fd_dlc_to_len(14), 48);
        assert_eq!(fd_dlc_to_len(15), 64);
    }
    #[test]
    fn ecat_state_name_all_known() {
        assert_eq!(ecat_state_name(1), "Init");
        assert_eq!(ecat_state_name(2), "Pre-Operational");
        assert_eq!(ecat_state_name(3), "Bootstrap");
        assert_eq!(ecat_state_name(4), "Safe-Operational");
        assert_eq!(ecat_state_name(8), "Operational");
    }

    #[test]
    fn ecat_state_name_unknown_values() {
        assert_eq!(ecat_state_name(0), "Unknown");
        assert_eq!(ecat_state_name(255), "Unknown");
    }
    #[test]
    fn can_protocol_type_all_labels() {
        assert_eq!(CanProtocolType::Standard.label(), "CAN 2.0");
        assert_eq!(CanProtocolType::Fd.label(), "CAN FD");
        assert_eq!(CanProtocolType::EtherCatCoE.label(), "EtherCAT CoE");
    }

    #[test]
    fn can_protocol_type_display() {
        assert_eq!(format!("{}", CanProtocolType::Standard), "CAN 2.0");
        assert_eq!(format!("{}", CanProtocolType::Fd), "CAN FD");
    }
    #[test]
    fn nmt_command_all_variants_compile() {
        let _ = NmtCommand::StartRemoteNode;
        let _ = NmtCommand::StopRemoteNode;
        let _ = NmtCommand::EnterPreOperational;
        let _ = NmtCommand::ResetNode;
        let _ = NmtCommand::ResetCommunication;
    }

    #[test]
    fn build_nmt_different_commands() {
        let f1 = build_nmt(1, NmtCommand::StartRemoteNode);
        assert_eq!(f1.data[0], 0x01);

        let f2 = build_nmt(2, NmtCommand::StopRemoteNode);
        assert_eq!(f2.data[0], 0x02);
        assert_eq!(f2.data[1], 0x02);
    }
    #[test]
    fn sdo_download_4byte_expedited() {
        let r = CanopenSdoRequest {
            node_id: 1,
            action: SdoAction::DownloadExpedited,
            index: 0x6040,
            sub_index: 0,
            payload: vec![0x06, 0x00, 0x00, 0x00],
        };
        let f = r.build();
        // 4-byte expedited: ccs=1, e=1, s=1, n=0 → 0x23
        assert_eq!(f.data[0], 0x23);
    }

    #[test]
    fn sdo_upload_request_node_id_affects_cob_id() {
        let r1 = CanopenSdoRequest {
            node_id: 1,
            action: SdoAction::UploadRequest,
            index: 0x1000,
            sub_index: 0,
            payload: vec![],
        };
        let r2 = CanopenSdoRequest {
            node_id: 127,
            action: SdoAction::UploadRequest,
            index: 0x1000,
            sub_index: 0,
            payload: vec![],
        };
        assert_eq!(r1.build().cob_id, 0x601);
        assert_eq!(r2.build().cob_id, 0x67F);
    }
    #[test]
    fn multi_protocol_frame_to_bytes_standard() {
        let f = MultiProtocolFrame::can_fd_pdo(0x181, &[0xAA; 8]);
        let bytes = f.to_bytes();
        assert!(!bytes.is_empty());
        // First 4 bytes: can_id (le)
        let id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(id, 0x181);
    }
    #[test]
    fn can_std_frame_extended_id() {
        let f = CanStdFrame::new(0x1FFFFFF, &[], true);
        assert!(f.is_extended);
        assert_eq!(f.can_id, 0x1FFFFFF);
        assert_eq!(f.dlc(), 0);
    }

    #[test]
    fn can_std_frame_empty_data() {
        let f = CanStdFrame::new(0x100, &[], false);
        assert_eq!(f.data.len(), 0);
        assert_eq!(f.dlc(), 0);
    }

    #[test]
    fn can_std_frame_max_data_8_bytes() {
        let f = CanStdFrame::new(0x100, &[0xFF; 8], false);
        assert_eq!(f.data.len(), 8);
        assert_eq!(f.dlc(), 8);
    }

    #[test]
    fn can_fd_frame_max_64_bytes() {
        let f = CanStdFrame::new_fd(0x100, &[0xAA; 64], false);
        assert_eq!(f.data.len(), 64);
        assert_eq!(f.dlc(), 64);
    }
    #[test]
    fn analyze_ecat_coe_frame_short_data() {
        let analysis = analyze_ecat_coe_frame(&[0x01, 0x02]);
        // Too short for full analysis
        assert!(!analysis.valid || analysis.fields.is_empty());
    }

    #[test]
    fn analyze_ecat_coe_frame_valid_coe() {
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
        assert!(analysis.valid);
        assert!(!analysis.summary.is_empty());
        // Should have MBX Length, MBX Address, MBX Type + SDO fields
        assert!(analysis.fields.len() >= 3);
    }

    #[test]
    fn analyze_ecat_coe_frame_read_request() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 5,
            index: 0x1018,
            sub_index: 1,
            data: vec![],
            is_write: false,
        };
        let frame = sdo.build_coe_frame();
        let mut combined = frame.mailbox_header.clone();
        combined.extend_from_slice(&frame.coe_data);
        let analysis = analyze_ecat_coe_frame(&combined);
        assert!(analysis.valid);
        assert!(analysis.summary.contains("Upload Init"));
    }

    #[test]
    fn ecat_coe_frame_summary_format() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 3,
            index: 0x1017,
            sub_index: 0,
            data: vec![0xE8, 0x03],
            is_write: true,
        };
        let frame = sdo.build_coe_frame();
        assert!(frame.summary.contains("Write"));
        assert!(frame.summary.contains("Slave=3"));
        assert!(frame.summary.contains("0x1017"));
    }

    #[test]
    fn ecat_coe_frame_mailbox_header_length() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 1,
            index: 0x6040,
            sub_index: 0,
            data: vec![0x06, 0x00],
            is_write: true,
        };
        let frame = sdo.build_coe_frame();
        // Mailbox header should be 6 bytes
        assert_eq!(frame.mailbox_header.len(), 6);
        // Last byte should be 0x03 (CoE type)
        assert_eq!(frame.mailbox_header[5], 0x03);
    }

    #[test]
    fn ecat_coe_frame_empty_data_read() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 1,
            index: 0x1000,
            sub_index: 0,
            data: vec![],
            is_write: false,
        };
        let frame = sdo.build_coe_frame();
        assert!(!frame.coe_data.is_empty());
        // Upload request command byte should be 0x40
        assert_eq!(frame.coe_data[2], 0x40);
    }
    #[test]
    fn sdo_request_cob_id_range() {
        for node_id in 1..=127u8 {
            let r = CanopenSdoRequest {
                node_id,
                action: SdoAction::UploadRequest,
                index: 0x1000,
                sub_index: 0,
                payload: vec![],
            };
            let f = r.build();
            // RSDO cob_id = 0x600 + node_id
            assert_eq!(f.cob_id, 0x600 + node_id as u16);
        }
    }

    #[test]
    fn sdo_download_cob_id_matches_upload() {
        let upload = CanopenSdoRequest {
            node_id: 5,
            action: SdoAction::UploadRequest,
            index: 0x1000,
            sub_index: 0,
            payload: vec![],
        };
        let download = CanopenSdoRequest {
            node_id: 5,
            action: SdoAction::DownloadExpedited,
            index: 0x1000,
            sub_index: 0,
            payload: vec![0x01],
        };
        // Both use same cob_id (RSDO)
        assert_eq!(upload.build().cob_id, download.build().cob_id);
    }
    #[test]
    fn nmt_reset_node_command() {
        let f = build_nmt(1, NmtCommand::ResetNode);
        assert_eq!(f.cob_id, 0x000);
        assert_eq!(f.data[0], 0x81);
        assert_eq!(f.data[1], 0x01);
    }

    #[test]
    fn nmt_reset_communication_command() {
        let f = build_nmt(1, NmtCommand::ResetCommunication);
        assert_eq!(f.cob_id, 0x000);
        assert_eq!(f.data[0], 0x82);
    }

    #[test]
    fn nmt_enter_pre_operational() {
        let f = build_nmt(1, NmtCommand::EnterPreOperational);
        assert_eq!(f.data[0], 0x80);
    }
    #[test]
    fn pdo_config_build_bool_values() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Enable".into(),
                index: 0x6000,
                sub_index: 1,
                bit_length: 1,
                data_type: PdoDataType::Bool,
            }],
            ..Default::default()
        };
        let frame_on = cfg.build_from_values(&[1.0]);
        assert_ne!(frame_on.data[0] & 0x01, 0);

        let frame_off = cfg.build_from_values(&[0.0]);
        assert_eq!(frame_off.data[0] & 0x01, 0);
    }

    #[test]
    fn pdo_config_build_u8_value() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Speed".into(),
                index: 0x6000,
                sub_index: 1,
                bit_length: 8,
                data_type: PdoDataType::U8,
            }],
            ..Default::default()
        };
        let frame = cfg.build_from_values(&[200.0]);
        assert_eq!(frame.data[0], 200);
    }
    #[test]
    fn canopen_frame_fields() {
        let f = CanopenFrame {
            cob_id: 0x185,
            data: vec![0x01, 0x02, 0x03],
        };
        assert_eq!(f.cob_id, 0x185);
        assert_eq!(f.data.len(), 3);
    }
    #[test]
    fn object_dict_name_store_restore() {
        assert_eq!(object_dict_name(0x1010, 0), "Store Parameters");
        assert_eq!(object_dict_name(0x1010, 1), "Store Parameters");
        assert_eq!(object_dict_name(0x1011, 0), "Restore Default Parameters");
    }

    #[test]
    fn object_dict_name_emcy_objects() {
        assert_eq!(object_dict_name(0x1014, 0), "EMCY COB-ID");
        assert_eq!(object_dict_name(0x1015, 0), "EMCY Inhibit Time");
    }

    #[test]
    fn object_dict_name_heartbeat_objects() {
        assert_eq!(object_dict_name(0x1016, 0), "Consumer Heartbeat Time");
        assert_eq!(object_dict_name(0x1017, 0), "Producer Heartbeat Time");
    }

    // ECC (EtherCAT CoE + CiA 301) Full Closure Tests
    #[test]
    fn nmt_state_from_heartbeat_all_codes() {
        assert_eq!(
            NmtState::from_heartbeat_code(0x00),
            Some(NmtState::Initializing)
        );
        assert_eq!(NmtState::from_heartbeat_code(0x04), Some(NmtState::Stopped));
        assert_eq!(
            NmtState::from_heartbeat_code(0x05),
            Some(NmtState::Operational)
        );
        assert_eq!(
            NmtState::from_heartbeat_code(0x7F),
            Some(NmtState::PreOperational)
        );
    }

    #[test]
    fn nmt_state_from_heartbeat_invalid() {
        assert_eq!(NmtState::from_heartbeat_code(0x01), None);
        assert_eq!(NmtState::from_heartbeat_code(0x02), None);
        assert_eq!(NmtState::from_heartbeat_code(0x03), None);
        assert_eq!(NmtState::from_heartbeat_code(0xFF), None);
    }

    #[test]
    fn nmt_state_heartbeat_code_roundtrip() {
        for state in [
            NmtState::Initializing,
            NmtState::Stopped,
            NmtState::Operational,
            NmtState::PreOperational,
        ] {
            let code = state.heartbeat_code();
            let decoded = NmtState::from_heartbeat_code(code);
            assert_eq!(decoded, Some(state), "roundtrip failed for {:?}", state);
        }
    }

    #[test]
    fn nmt_state_display() {
        assert_eq!(format!("{}", NmtState::Initializing), "Initializing");
        assert_eq!(format!("{}", NmtState::PreOperational), "Pre-Operational");
        assert_eq!(format!("{}", NmtState::Operational), "Operational");
        assert_eq!(format!("{}", NmtState::Stopped), "Stopped");
    }

    #[test]
    fn nmt_state_default_is_initializing() {
        assert_eq!(NmtState::default(), NmtState::Initializing);
    }

    #[test]
    fn nmt_state_serde_roundtrip() {
        for state in [
            NmtState::Initializing,
            NmtState::PreOperational,
            NmtState::Operational,
            NmtState::Stopped,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let restored: NmtState = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, state);
        }
    }
    #[test]
    fn emcy_error_register_default_no_errors() {
        let r = EmcyErrorRegister::default();
        assert_eq!(r.bits, 0);
        assert!(!r.generic_error());
        assert!(!r.current_error());
        assert!(!r.voltage_error());
        assert!(!r.temperature_error());
        assert!(!r.communication_error());
        assert!(!r.device_profile_error());
        assert!(!r.manufacturer_error());
    }

    #[test]
    fn emcy_error_register_each_bit() {
        let tests: &[(u8, fn(&EmcyErrorRegister) -> bool, &str)] = &[
            (0x01, |r| r.generic_error(), "generic"),
            (0x02, |r| r.current_error(), "current"),
            (0x04, |r| r.voltage_error(), "voltage"),
            (0x08, |r| r.temperature_error(), "temperature"),
            (0x10, |r| r.communication_error(), "communication"),
            (0x20, |r| r.device_profile_error(), "device_profile"),
            (0x80, |r| r.manufacturer_error(), "manufacturer"),
        ];
        for &(bit, checker, name) in tests {
            let r = EmcyErrorRegister { bits: bit };
            assert!(checker(&r), "bit 0x{:02X} ({}) should be set", bit, name);
            // All other bits should be unset
            let others = EmcyErrorRegister { bits: !bit & 0x7F };
            assert!(
                !checker(&others),
                "bit 0x{:02X} ({}) should not be set on others",
                bit,
                name
            );
        }
    }

    #[test]
    fn emcy_error_register_describe_no_error() {
        let r = EmcyErrorRegister { bits: 0 };
        let desc = r.describe();
        assert_eq!(desc, vec!["No Error"]);
    }

    #[test]
    fn emcy_error_register_describe_multiple() {
        let r = EmcyErrorRegister { bits: 0x03 }; // generic + current
        let desc = r.describe();
        assert_eq!(desc, vec!["Generic Error", "Current Error"]);
    }

    #[test]
    fn emcy_error_register_describe_all_bits() {
        let r = EmcyErrorRegister { bits: 0xFF };
        let desc = r.describe();
        assert_eq!(desc.len(), 7);
        assert!(desc.contains(&"Generic Error"));
        assert!(desc.contains(&"Manufacturer Error"));
    }

    #[test]
    fn emcy_error_register_serde_roundtrip() {
        let r = EmcyErrorRegister { bits: 0x15 };
        let json = serde_json::to_string(&r).unwrap();
        let restored: EmcyErrorRegister = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.bits, 0x15);
    }
    #[test]
    fn emcy_error_class_from_code_all_classes() {
        assert_eq!(
            EmcyErrorClass::from_code(0x1000),
            EmcyErrorClass::GenericError
        );
        assert_eq!(
            EmcyErrorClass::from_code(0x1FFF),
            EmcyErrorClass::GenericError
        );
        assert_eq!(EmcyErrorClass::from_code(0x2000), EmcyErrorClass::Current);
        assert_eq!(EmcyErrorClass::from_code(0x2FFF), EmcyErrorClass::Current);
        assert_eq!(EmcyErrorClass::from_code(0x3000), EmcyErrorClass::Voltage);
        assert_eq!(
            EmcyErrorClass::from_code(0x4000),
            EmcyErrorClass::Temperature
        );
        assert_eq!(
            EmcyErrorClass::from_code(0x5000),
            EmcyErrorClass::DeviceHardware
        );
        assert_eq!(
            EmcyErrorClass::from_code(0x6000),
            EmcyErrorClass::DeviceSoftware
        );
        assert_eq!(
            EmcyErrorClass::from_code(0x7000),
            EmcyErrorClass::Monitoring
        );
        assert_eq!(EmcyErrorClass::from_code(0x8000), EmcyErrorClass::External);
        assert_eq!(
            EmcyErrorClass::from_code(0xF000),
            EmcyErrorClass::AdditionalHardware
        );
        assert_eq!(
            EmcyErrorClass::from_code(0xFF00),
            EmcyErrorClass::DeviceSpecific
        );
    }

    #[test]
    fn emcy_error_class_from_code_unknown() {
        match EmcyErrorClass::from_code(0x0042) {
            EmcyErrorClass::Unknown(c) => assert_eq!(c, 0x0042),
            other => panic!("Expected Unknown, got {:?}", other),
        }
    }

    #[test]
    fn emcy_error_class_display() {
        assert_eq!(format!("{}", EmcyErrorClass::GenericError), "Generic Error");
        assert_eq!(format!("{}", EmcyErrorClass::Current), "Current");
        assert_eq!(format!("{}", EmcyErrorClass::Voltage), "Voltage");
        assert_eq!(format!("{}", EmcyErrorClass::Temperature), "Temperature");
        assert_eq!(
            format!("{}", EmcyErrorClass::DeviceHardware),
            "Device Hardware"
        );
        assert_eq!(
            format!("{}", EmcyErrorClass::DeviceSoftware),
            "Device Software"
        );
        assert_eq!(format!("{}", EmcyErrorClass::Monitoring), "Monitoring");
        assert_eq!(format!("{}", EmcyErrorClass::External), "External");
        assert_eq!(
            format!("{}", EmcyErrorClass::AdditionalHardware),
            "Additional Hardware"
        );
        assert_eq!(
            format!("{}", EmcyErrorClass::DeviceSpecific),
            "Device-Specific"
        );
        assert_eq!(
            format!("{}", EmcyErrorClass::Unknown(0x42)),
            "Unknown(0x0042)"
        );
    }

    #[test]
    fn emcy_describe_code_known_codes() {
        assert_eq!(
            EmcyErrorClass::describe_code(0x1000),
            "Error Reset / No Error"
        );
        assert_eq!(EmcyErrorClass::describe_code(0x1001), "Generic Error");
        assert_eq!(
            EmcyErrorClass::describe_code(0x2110),
            "CAN overrun (objects lost)"
        );
        assert_eq!(
            EmcyErrorClass::describe_code(0x3100),
            "Input voltage too high"
        );
        assert_eq!(
            EmcyErrorClass::describe_code(0x4210),
            "Ambient temperature too high"
        );
        assert_eq!(EmcyErrorClass::describe_code(0x5100), "Power supply fault");
        assert_eq!(
            EmcyErrorClass::describe_code(0x6100),
            "Software reset (watchdog)"
        );
        assert_eq!(EmcyErrorClass::describe_code(0x7100), "Sensor fault");
        assert_eq!(EmcyErrorClass::describe_code(0x8100), "CAN bus off");
        assert_eq!(
            EmcyErrorClass::describe_code(0xFF01),
            "Manufacturer-specific: motor stall detected"
        );
    }

    #[test]
    fn emcy_describe_code_unknown() {
        assert_eq!(
            EmcyErrorClass::describe_code(0xAAAA),
            "Unknown EMCY error code"
        );
    }
    #[test]
    fn nmt_command_all_count() {
        assert_eq!(NmtCommand::all().len(), 5);
    }

    #[test]
    fn nmt_command_code_values() {
        assert_eq!(NmtCommand::StartRemoteNode.code(), 0x01);
        assert_eq!(NmtCommand::StopRemoteNode.code(), 0x02);
        assert_eq!(NmtCommand::EnterPreOperational.code(), 0x80);
        assert_eq!(NmtCommand::ResetNode.code(), 0x81);
        assert_eq!(NmtCommand::ResetCommunication.code(), 0x82);
    }

    #[test]
    fn nmt_command_display() {
        assert_eq!(
            format!("{}", NmtCommand::StartRemoteNode),
            "Start Remote Node (0x01)"
        );
        assert_eq!(
            format!("{}", NmtCommand::StopRemoteNode),
            "Stop Remote Node (0x02)"
        );
        assert_eq!(
            format!("{}", NmtCommand::EnterPreOperational),
            "Enter Pre-Operational (0x80)"
        );
        assert_eq!(format!("{}", NmtCommand::ResetNode), "Reset Node (0x81)");
        assert_eq!(
            format!("{}", NmtCommand::ResetCommunication),
            "Reset Communication (0x82)"
        );
    }

    #[test]
    fn nmt_command_serde_roundtrip() {
        for cmd in NmtCommand::all() {
            let json = serde_json::to_string(cmd).unwrap();
            let restored: NmtCommand = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, *cmd);
        }
    }
    #[test]
    fn sdo_action_all_count() {
        assert_eq!(SdoAction::all().len(), 2);
    }

    #[test]
    fn sdo_action_display() {
        assert_eq!(
            format!("{}", SdoAction::UploadRequest),
            "Upload Request (Read)"
        );
        assert_eq!(
            format!("{}", SdoAction::DownloadExpedited),
            "Download Expedited (Write)"
        );
    }

    #[test]
    fn sdo_action_serde_roundtrip() {
        for action in SdoAction::all() {
            let json = serde_json::to_string(action).unwrap();
            let restored: SdoAction = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, *action);
        }
    }
    #[test]
    fn build_heartbeat_producer_sdo_cob_id() {
        let f = build_heartbeat_producer_sdo(5, 1000);
        // RSDO cob_id = 0x600 + node_id
        assert_eq!(f.cob_id, 0x605);
    }

    #[test]
    fn build_heartbeat_producer_sdo_index() {
        let f = build_heartbeat_producer_sdo(1, 500);
        // Index 0x1017 = Producer Heartbeat Time
        assert_eq!(f.data[1], 0x17);
        assert_eq!(f.data[2], 0x10);
    }

    #[test]
    fn build_heartbeat_producer_sdo_payload() {
        let f = build_heartbeat_producer_sdo(1, 1000);
        // 1000 = 0x03E8, little-endian
        assert_eq!(f.data[4], 0xE8);
        assert_eq!(f.data[5], 0x03);
    }
    #[test]
    fn build_pdo_basic() {
        let f = build_pdo(0x181, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(f.cob_id, 0x181);
        assert_eq!(f.data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn build_pdo_truncates_to_8_bytes() {
        let data = vec![0xAA; 20];
        let f = build_pdo(0x281, &data);
        assert_eq!(f.data.len(), 8, "PDO data should be truncated to 8 bytes");
    }

    #[test]
    fn build_pdo_empty_data() {
        let f = build_pdo(0x181, &[]);
        assert!(f.data.is_empty());
    }
    #[test]
    fn decode_values_u16() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Speed".into(),
                bit_length: 16,
                data_type: PdoDataType::U16,
                ..Default::default()
            }],
            ..Default::default()
        };
        let data = (1000u16).to_le_bytes();
        let results = cfg.decode_values(&data);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "Speed");
        assert_eq!(results[0].2, 1000.0);
    }

    #[test]
    fn decode_values_i16_negative() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Torque".into(),
                bit_length: 16,
                data_type: PdoDataType::I16,
                ..Default::default()
            }],
            ..Default::default()
        };
        let data = (-500i16).to_le_bytes();
        let results = cfg.decode_values(&data);
        assert_eq!(results[0].2, -500.0);
    }

    #[test]
    fn decode_values_u8() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Mode".into(),
                bit_length: 8,
                data_type: PdoDataType::U8,
                ..Default::default()
            }],
            ..Default::default()
        };
        let results = cfg.decode_values(&[200]);
        assert_eq!(results[0].2, 200.0);
    }

    #[test]
    fn decode_values_i8_negative() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Offset".into(),
                bit_length: 8,
                data_type: PdoDataType::I8,
                ..Default::default()
            }],
            ..Default::default()
        };
        let results = cfg.decode_values(&[0xFE]); // -2 as i8
        assert_eq!(results[0].2, -2.0);
    }

    #[test]
    fn decode_values_u32() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Position".into(),
                bit_length: 32,
                data_type: PdoDataType::U32,
                ..Default::default()
            }],
            ..Default::default()
        };
        let data = (100000u32).to_le_bytes();
        let results = cfg.decode_values(&data);
        assert_eq!(results[0].2, 100000.0);
    }

    #[test]
    fn decode_values_i32_negative() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Error".into(),
                bit_length: 32,
                data_type: PdoDataType::I32,
                ..Default::default()
            }],
            ..Default::default()
        };
        let data = (-12345i32).to_le_bytes();
        let results = cfg.decode_values(&data);
        assert_eq!(results[0].2, -12345.0);
    }

    #[test]
    fn decode_values_f32() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Current".into(),
                bit_length: 32,
                data_type: PdoDataType::F32,
                ..Default::default()
            }],
            ..Default::default()
        };
        let data = (3.25f32).to_le_bytes();
        let results = cfg.decode_values(&data);
        assert!((results[0].2 - 3.25).abs() < 0.01);
    }

    #[test]
    fn decode_values_bool() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Enable".into(),
                bit_length: 1,
                data_type: PdoDataType::Bool,
                ..Default::default()
            }],
            ..Default::default()
        };
        let results_on = cfg.decode_values(&[0x01]);
        assert_eq!(results_on[0].2, 1.0);
        assert_eq!(results_on[0].1, "TRUE");

        let results_off = cfg.decode_values(&[0x00]);
        assert_eq!(results_off[0].2, 0.0);
        assert_eq!(results_off[0].1, "FALSE");
    }

    #[test]
    fn decode_values_insufficient_data_returns_zero() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Big".into(),
                bit_length: 32,
                data_type: PdoDataType::U32,
                ..Default::default()
            }],
            ..Default::default()
        };
        // Only 2 bytes available, need 4
        let results = cfg.decode_values(&[0x01, 0x02]);
        assert_eq!(results[0].2, 0.0);
    }

    #[test]
    fn decode_values_multiple_mappings() {
        let cfg = PdoConfig {
            mappings: vec![
                PdoMappingEntry {
                    name: "Speed".into(),
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                    ..Default::default()
                },
                PdoMappingEntry {
                    name: "Torque".into(),
                    bit_length: 16,
                    data_type: PdoDataType::I16,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut data = Vec::new();
        data.extend_from_slice(&500u16.to_le_bytes());
        data.extend_from_slice(&(-100i16).to_le_bytes());
        let results = cfg.decode_values(&data);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].2, 500.0);
        assert_eq!(results[1].2, -100.0);
    }

    #[test]
    fn decode_values_display_format() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "F32".into(),
                bit_length: 32,
                data_type: PdoDataType::F32,
                ..Default::default()
            }],
            ..Default::default()
        };
        let data = (1.2345f32).to_le_bytes();
        let results = cfg.decode_values(&data);
        // F32 should show 4 decimal places
        assert!(
            results[0].1.contains('.'),
            "F32 display should have decimal: {}",
            results[0].1
        );
    }
    #[test]
    fn sdo_download_1byte_expedited() {
        let r = CanopenSdoRequest {
            node_id: 1,
            action: SdoAction::DownloadExpedited,
            index: 0x6040,
            sub_index: 0,
            payload: vec![0x06],
        };
        let f = r.build();
        // 1-byte: ccs=1, e=1, s=1, n=3 → 0x2F
        assert_eq!(f.data[0], 0x2F);
    }

    #[test]
    fn sdo_download_3byte_expedited() {
        let r = CanopenSdoRequest {
            node_id: 1,
            action: SdoAction::DownloadExpedited,
            index: 0x6040,
            sub_index: 0,
            payload: vec![0x01, 0x02, 0x03],
        };
        let f = r.build();
        // 3-byte: ccs=1, e=1, s=1, n=1 → 0x27
        assert_eq!(f.data[0], 0x27);
    }
    #[test]
    fn build_nmt_all_commands_data_length() {
        for cmd in NmtCommand::all() {
            let f = build_nmt(1, *cmd);
            assert_eq!(f.data.len(), 2, "NMT frame data should be 2 bytes");
            assert_eq!(f.cob_id, 0x000, "NMT cob_id should be 0x000");
        }
    }
    #[test]
    fn pdo_direction_serde_roundtrip() {
        for dir in [PdoDirection::Transmit, PdoDirection::Receive] {
            let json = serde_json::to_string(&dir).unwrap();
            let restored: PdoDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, dir);
        }
    }

    #[test]
    fn pdo_data_type_serde_roundtrip() {
        let types = [
            PdoDataType::Bool,
            PdoDataType::U8,
            PdoDataType::I8,
            PdoDataType::U16,
            PdoDataType::I16,
            PdoDataType::U32,
            PdoDataType::I32,
            PdoDataType::F32,
        ];
        for dt in types {
            let json = serde_json::to_string(&dt).unwrap();
            let restored: PdoDataType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, dt);
        }
    }
    #[test]
    fn pdo_build_decode_roundtrip_u16() {
        let cfg = PdoConfig {
            mappings: vec![PdoMappingEntry {
                name: "Val".into(),
                bit_length: 16,
                data_type: PdoDataType::U16,
                ..Default::default()
            }],
            ..Default::default()
        };
        let frame = cfg.build_from_values(&[12345.0]);
        let decoded = cfg.decode_values(&frame.data);
        assert_eq!(decoded[0].2, 12345.0);
    }

    #[test]
    fn pdo_build_decode_roundtrip_multi() {
        let cfg = PdoConfig {
            mappings: vec![
                PdoMappingEntry {
                    name: "A".into(),
                    bit_length: 16,
                    data_type: PdoDataType::U16,
                    ..Default::default()
                },
                PdoMappingEntry {
                    name: "B".into(),
                    bit_length: 8,
                    data_type: PdoDataType::U8,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let frame = cfg.build_from_values(&[500.0, 42.0]);
        let decoded = cfg.decode_values(&frame.data);
        assert_eq!(decoded[0].2, 500.0);
        assert_eq!(decoded[1].2, 42.0);
    }
    #[test]
    fn emcy_error_register_reserved_bit_6() {
        let r = EmcyErrorRegister { bits: 0x40 };
        // Bit 6 is not mapped to any named error
        assert!(!r.generic_error());
        assert!(!r.communication_error());
        assert!(!r.manufacturer_error());
    }
    #[test]
    fn analyze_nmt_frame() {
        let f = build_nmt(1, NmtCommand::StartRemoteNode);
        let analysis = analyze_canopen_frame(f.cob_id, &f.data);
        assert_eq!(analysis.role, "NMT");
        assert!(analysis.valid);
        assert_eq!(analysis.node_id, 0);
        assert!(analysis.summary.contains("Start"));
        assert!(analysis.summary.contains("Node 1"));
        assert!(analysis.fields.len() >= 2);
    }

    #[test]
    fn analyze_nmt_insufficient_data() {
        let analysis = analyze_canopen_frame(0x000, &[0x01]);
        assert_eq!(analysis.role, "NMT");
        assert!(!analysis.valid);
    }

    #[test]
    fn analyze_rsdo_upload_request() {
        let r = CanopenSdoRequest {
            node_id: 5,
            action: SdoAction::UploadRequest,
            index: 0x1018,
            sub_index: 1,
            payload: vec![],
        };
        let f = r.build();
        let analysis = analyze_canopen_frame(f.cob_id, &f.data);
        assert_eq!(analysis.role, "RSDO");
        assert!(analysis.valid);
        assert!(analysis.summary.contains("Upload Init"));
        assert!(analysis.summary.contains("0x1018"));
    }

    #[test]
    fn analyze_rsdo_download() {
        let r = CanopenSdoRequest {
            node_id: 1,
            action: SdoAction::DownloadExpedited,
            index: 0x1017,
            sub_index: 0,
            payload: vec![0xE8, 0x03],
        };
        let f = r.build();
        let analysis = analyze_canopen_frame(f.cob_id, &f.data);
        assert_eq!(analysis.role, "RSDO");
        assert!(analysis.valid);
        assert!(analysis.summary.contains("Download Init"));
    }

    #[test]
    fn analyze_sdo_insufficient_data() {
        let analysis = analyze_canopen_frame(0x601, &[0x40]);
        assert_eq!(analysis.role, "RSDO");
        assert!(!analysis.valid);
    }

    #[test]
    fn analyze_emcy_frame() {
        let data = [0x00, 0x10, 0x01, 0, 0, 0, 0, 0];
        let analysis = analyze_canopen_frame(0x081, &data);
        assert_eq!(analysis.role, "EMCY");
        assert!(analysis.valid);
        assert!(analysis.summary.contains("EMCY"));
        assert!(analysis.summary.contains("Generic Error"));
    }

    #[test]
    fn analyze_emcy_insufficient_data() {
        let analysis = analyze_canopen_frame(0x081, &[0x00]);
        assert_eq!(analysis.role, "EMCY");
        assert!(!analysis.valid);
    }

    #[test]
    fn analyze_heartbeat_operational() {
        let analysis = analyze_canopen_frame(0x705, &[0x05]);
        assert_eq!(analysis.role, "Heartbeat");
        assert!(analysis.valid);
        assert!(analysis.summary.contains("Operational"));
        assert!(analysis.summary.contains("Node 5"));
    }

    #[test]
    fn analyze_heartbeat_pre_operational() {
        let analysis = analyze_canopen_frame(0x701, &[0x7F]);
        assert_eq!(analysis.role, "Heartbeat");
        assert!(analysis.valid);
        assert!(analysis.summary.contains("Pre-operational"));
    }

    #[test]
    fn analyze_heartbeat_boot_up() {
        let analysis = analyze_canopen_frame(0x701, &[0x00]);
        assert_eq!(analysis.role, "Heartbeat");
        assert!(analysis.valid);
        assert!(analysis.summary.contains("Boot-up"));
    }

    #[test]
    fn analyze_heartbeat_no_data() {
        let analysis = analyze_canopen_frame(0x701, &[]);
        assert_eq!(analysis.role, "Heartbeat");
        assert!(!analysis.valid);
    }

    #[test]
    fn analyze_tpdo_frame() {
        let analysis = analyze_canopen_frame(0x181, &[0x01, 0x02, 0x03]);
        assert!(analysis.role.contains("PDO"));
        assert!(analysis.valid);
        assert_eq!(analysis.fields.len(), 3);
    }

    #[test]
    fn analyze_rpdo_frame() {
        let analysis = analyze_canopen_frame(0x201, &[0xAA, 0xBB]);
        assert!(analysis.role.contains("PDO"));
        assert!(analysis.valid);
    }

    #[test]
    fn analyze_non_standard_frame() {
        let analysis = analyze_canopen_frame(0x780, &[0x01, 0x02]);
        assert_eq!(analysis.role, "Non-Standard");
        assert!(analysis.valid);
    }

    #[test]
    fn analyze_sync_frame() {
        let analysis = analyze_canopen_frame(0x080, &[]);
        assert_eq!(analysis.role, "SYNC");
        assert!(analysis.valid);
    }

    #[test]
    fn analyze_node_id_extraction() {
        // node_id = cob_id & 0x7F
        let analysis = analyze_canopen_frame(0x705, &[0x05]);
        assert_eq!(analysis.node_id, 5);

        let analysis2 = analyze_canopen_frame(0x181, &[0x01]);
        assert_eq!(analysis2.node_id, 1);
    }
    #[test]
    fn canopen_frame_analysis_fields() {
        let f = build_nmt(1, NmtCommand::StartRemoteNode);
        let analysis = analyze_canopen_frame(f.cob_id, &f.data);
        // Check field info structure
        assert!(!analysis.fields[0].name.is_empty());
        assert!(!analysis.fields[0].raw_hex.is_empty());
        assert!(!analysis.fields[0].decoded.is_empty());
    }
    #[test]
    fn analyze_ecat_coe_write_has_sdo_fields() {
        let sdo = EcatCoeSdoRequest {
            slave_addr: 2,
            index: 0x6040,
            sub_index: 0,
            data: vec![0x06, 0x00],
            is_write: true,
        };
        let frame = sdo.build_coe_frame();
        let mut combined = frame.mailbox_header.clone();
        combined.extend_from_slice(&frame.coe_data);
        let analysis = analyze_ecat_coe_frame(&combined);
        assert!(analysis.valid);
        // Should have MBX Length, MBX Address, MBX Type + CoE SDO Cmd + OD Index + SubIndex
        assert!(
            analysis.fields.len() >= 5,
            "Expected >=5 fields, got {}",
            analysis.fields.len()
        );
    }
    #[test]
    fn can_protocol_type_serde_roundtrip() {
        for pt in [
            CanProtocolType::Standard,
            CanProtocolType::Fd,
            CanProtocolType::EtherCatCoE,
        ] {
            let json = serde_json::to_string(&pt).unwrap();
            let restored: CanProtocolType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, pt);
        }
    }

    // 严格补全：剩余未测试公共函数
    #[test]
    fn fd_len_to_dlc_standard_range() {
        for len in 0..=8 {
            assert_eq!(fd_len_to_dlc(len), len as u8);
        }
    }

    #[test]
    fn fd_len_to_dlc_extended() {
        assert_eq!(fd_len_to_dlc(9), 9);
        assert_eq!(fd_len_to_dlc(12), 9);
        assert_eq!(fd_len_to_dlc(13), 10);
        assert_eq!(fd_len_to_dlc(16), 10);
        assert_eq!(fd_len_to_dlc(17), 11);
        assert_eq!(fd_len_to_dlc(20), 11);
        assert_eq!(fd_len_to_dlc(21), 12);
        assert_eq!(fd_len_to_dlc(24), 12);
        assert_eq!(fd_len_to_dlc(25), 13);
        assert_eq!(fd_len_to_dlc(32), 13);
        assert_eq!(fd_len_to_dlc(33), 14);
        assert_eq!(fd_len_to_dlc(48), 14);
        assert_eq!(fd_len_to_dlc(49), 15);
        assert_eq!(fd_len_to_dlc(64), 15);
        assert_eq!(fd_len_to_dlc(100), 15);
    }
    #[test]
    fn is_fd_valid_len_standard() {
        for len in 0..=8 {
            assert!(is_fd_valid_len(len), "{} should be valid", len);
        }
    }

    #[test]
    fn is_fd_valid_len_extended() {
        for len in [12, 16, 20, 24, 32, 48, 64] {
            assert!(is_fd_valid_len(len), "{} should be valid", len);
        }
    }

    #[test]
    fn is_fd_valid_len_invalid() {
        for len in [9, 10, 11, 13, 14, 15, 17, 25, 33, 50, 100] {
            assert!(!is_fd_valid_len(len), "{} should be invalid", len);
        }
    }
    #[test]
    fn pdo_config_from_json_valid() {
        let cfg = PdoConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let restored = PdoConfig::from_json(&json);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().name, "PDO1");
    }

    #[test]
    fn pdo_config_from_json_invalid() {
        assert!(PdoConfig::from_json("not json").is_none());
        assert!(PdoConfig::from_json("").is_none());
        // Empty object fails because required fields are missing
        assert!(PdoConfig::from_json("{}").is_none());
    }
    #[test]
    fn preset_pdo_configs_non_empty() {
        let presets = preset_pdo_configs();
        assert!(!presets.is_empty(), "Should have at least one preset");
    }

    #[test]
    fn preset_pdo_configs_valid_structure() {
        let presets = preset_pdo_configs();
        for preset in &presets {
            assert!(!preset.name.is_empty());
            assert!(preset.cob_id > 0);
            assert!(!preset.mappings.is_empty());
        }
    }

    #[test]
    fn preset_pdo_configs_unique_names() {
        let presets = preset_pdo_configs();
        let names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len(), "Preset names should be unique");
    }
    #[test]
    fn fd_dlc_len_roundtrip_consistency() {
        // fd_len_to_dlc(fd_dlc_to_len(dlc)) == dlc for valid DLC values
        for dlc in 0..=15u8 {
            let len = fd_dlc_to_len(dlc);
            let back = fd_len_to_dlc(len);
            assert_eq!(
                back, dlc,
                "roundtrip failed: dlc={} → len={} → dlc={}",
                dlc, len, back
            );
        }
    }
}
