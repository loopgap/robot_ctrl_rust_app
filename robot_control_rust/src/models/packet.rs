use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// 数据字段类型定义
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Endianness {
    Little,
    Big,
}

impl std::fmt::Display for Endianness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Little => write!(f, "LE"),
            Self::Big => write!(f, "BE"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Bytes,  // 原始字节
    Ascii,  // ASCII 字符串
    HexStr, // 十六进制字符串
}

impl FieldType {
    pub fn all() -> &'static [FieldType] {
        &[
            Self::U8,
            Self::U16,
            Self::U32,
            Self::U64,
            Self::I8,
            Self::I16,
            Self::I32,
            Self::I64,
            Self::F32,
            Self::F64,
            Self::Bool,
            Self::Bytes,
            Self::Ascii,
            Self::HexStr,
        ]
    }

    pub fn byte_size(&self) -> Option<usize> {
        match self {
            Self::U8 | Self::I8 | Self::Bool => Some(1),
            Self::U16 | Self::I16 => Some(2),
            Self::U32 | Self::I32 | Self::F32 => Some(4),
            Self::U64 | Self::I64 | Self::F64 => Some(8),
            Self::Bytes | Self::Ascii | Self::HexStr => None, // 可变长
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ═══════════════════════════════════════════════════════════════
// 数据包字段
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketField {
    pub name: String,
    pub field_type: FieldType,
    pub endianness: Endianness,
    pub value_str: String,
    pub enabled: bool,
    pub comment: String,
}

impl Default for PacketField {
    fn default() -> Self {
        Self {
            name: "field".into(),
            field_type: FieldType::U8,
            endianness: Endianness::Little,
            value_str: "0".into(),
            enabled: true,
            comment: String::new(),
        }
    }
}

impl PacketField {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.field_type {
            FieldType::U8 => {
                let v = self.value_str.trim().parse::<u8>().unwrap_or(0);
                vec![v]
            }
            FieldType::U16 => {
                let v = self.value_str.trim().parse::<u16>().unwrap_or(0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::U32 => {
                let v = self.value_str.trim().parse::<u32>().unwrap_or(0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::U64 => {
                let v = self.value_str.trim().parse::<u64>().unwrap_or(0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::I8 => {
                let v = self.value_str.trim().parse::<i8>().unwrap_or(0);
                vec![v as u8]
            }
            FieldType::I16 => {
                let v = self.value_str.trim().parse::<i16>().unwrap_or(0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::I32 => {
                let v = self.value_str.trim().parse::<i32>().unwrap_or(0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::I64 => {
                let v = self.value_str.trim().parse::<i64>().unwrap_or(0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::F32 => {
                let v = self.value_str.trim().parse::<f32>().unwrap_or(0.0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::F64 => {
                let v = self.value_str.trim().parse::<f64>().unwrap_or(0.0);
                match self.endianness {
                    Endianness::Little => v.to_le_bytes().to_vec(),
                    Endianness::Big => v.to_be_bytes().to_vec(),
                }
            }
            FieldType::Bool => {
                let v = self.value_str.trim().parse::<bool>().unwrap_or(false);
                vec![v as u8]
            }
            FieldType::Bytes | FieldType::HexStr => parse_hex_string(&self.value_str),
            FieldType::Ascii => self.value_str.as_bytes().to_vec(),
        }
    }

    /// Parse a value from raw bytes according to field type and endianness.
    /// Returns (parsed_string, bytes_consumed). Returns None if buffer too short.
    pub fn from_bytes(
        field_type: FieldType,
        endianness: Endianness,
        data: &[u8],
        var_len: usize,
    ) -> Option<(String, usize)> {
        match field_type {
            FieldType::U8 => {
                if data.is_empty() {
                    return None;
                }
                Some((format!("{}", data[0]), 1))
            }
            FieldType::U16 => {
                if data.len() < 2 {
                    return None;
                }
                let v = match endianness {
                    Endianness::Little => u16::from_le_bytes([data[0], data[1]]),
                    Endianness::Big => u16::from_be_bytes([data[0], data[1]]),
                };
                Some((format!("{}", v), 2))
            }
            FieldType::U32 => {
                if data.len() < 4 {
                    return None;
                }
                let v = match endianness {
                    Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                    Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                };
                Some((format!("{}", v), 4))
            }
            FieldType::U64 => {
                if data.len() < 8 {
                    return None;
                }
                let arr: [u8; 8] = data[..8].try_into().ok()?;
                let v = match endianness {
                    Endianness::Little => u64::from_le_bytes(arr),
                    Endianness::Big => u64::from_be_bytes(arr),
                };
                Some((format!("{}", v), 8))
            }
            FieldType::I8 => {
                if data.is_empty() {
                    return None;
                }
                Some((format!("{}", data[0] as i8), 1))
            }
            FieldType::I16 => {
                if data.len() < 2 {
                    return None;
                }
                let v = match endianness {
                    Endianness::Little => i16::from_le_bytes([data[0], data[1]]),
                    Endianness::Big => i16::from_be_bytes([data[0], data[1]]),
                };
                Some((format!("{}", v), 2))
            }
            FieldType::I32 => {
                if data.len() < 4 {
                    return None;
                }
                let v = match endianness {
                    Endianness::Little => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                    Endianness::Big => i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                };
                Some((format!("{}", v), 4))
            }
            FieldType::I64 => {
                if data.len() < 8 {
                    return None;
                }
                let arr: [u8; 8] = data[..8].try_into().ok()?;
                let v = match endianness {
                    Endianness::Little => i64::from_le_bytes(arr),
                    Endianness::Big => i64::from_be_bytes(arr),
                };
                Some((format!("{}", v), 8))
            }
            FieldType::F32 => {
                if data.len() < 4 {
                    return None;
                }
                let v = match endianness {
                    Endianness::Little => f32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                    Endianness::Big => f32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                };
                Some((format!("{}", v), 4))
            }
            FieldType::F64 => {
                if data.len() < 8 {
                    return None;
                }
                let arr: [u8; 8] = data[..8].try_into().ok()?;
                let v = match endianness {
                    Endianness::Little => f64::from_le_bytes(arr),
                    Endianness::Big => f64::from_be_bytes(arr),
                };
                Some((format!("{}", v), 8))
            }
            FieldType::Bool => {
                if data.is_empty() {
                    return None;
                }
                Some((format!("{}", data[0] != 0), 1))
            }
            FieldType::Bytes | FieldType::HexStr => {
                let len = if var_len > 0 {
                    var_len.min(data.len())
                } else {
                    data.len()
                };
                let hex = data[..len]
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");
                Some((hex, len))
            }
            FieldType::Ascii => {
                let len = if var_len > 0 {
                    var_len.min(data.len())
                } else {
                    data.len()
                };
                let s = String::from_utf8_lossy(&data[..len]).to_string();
                Some((s, len))
            }
        }
    }

    /// Get the numeric value as f64 (for visualization). Returns None for non-numeric types.
    pub fn numeric_from_bytes(
        field_type: FieldType,
        endianness: Endianness,
        data: &[u8],
    ) -> Option<f64> {
        match field_type {
            FieldType::U8 => data.first().map(|&v| v as f64),
            FieldType::U16 if data.len() >= 2 => {
                let v = match endianness {
                    Endianness::Little => u16::from_le_bytes([data[0], data[1]]),
                    Endianness::Big => u16::from_be_bytes([data[0], data[1]]),
                };
                Some(v as f64)
            }
            FieldType::U32 if data.len() >= 4 => {
                let v = match endianness {
                    Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                    Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                };
                Some(v as f64)
            }
            FieldType::I8 => data.first().map(|&v| (v as i8) as f64),
            FieldType::I16 if data.len() >= 2 => {
                let v = match endianness {
                    Endianness::Little => i16::from_le_bytes([data[0], data[1]]),
                    Endianness::Big => i16::from_be_bytes([data[0], data[1]]),
                };
                Some(v as f64)
            }
            FieldType::I32 if data.len() >= 4 => {
                let v = match endianness {
                    Endianness::Little => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                    Endianness::Big => i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                };
                Some(v as f64)
            }
            FieldType::F32 if data.len() >= 4 => {
                let v = match endianness {
                    Endianness::Little => f32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                    Endianness::Big => f32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                };
                Some(v as f64)
            }
            FieldType::F64 if data.len() >= 8 => {
                let arr: [u8; 8] = data[..8].try_into().ok()?;
                let v = match endianness {
                    Endianness::Little => f64::from_le_bytes(arr),
                    Endianness::Big => f64::from_be_bytes(arr),
                };
                Some(v)
            }
            FieldType::Bool => data.first().map(|&v| if v != 0 { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 解析后的数据包
// ═══════════════════════════════════════════════════════════════

/// 解析后的单个字段
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedField {
    pub name: String,
    pub field_type: FieldType,
    pub value_str: String,
    pub value_f64: Option<f64>,
    pub raw_bytes: Vec<u8>,
}

/// 解析后的完整数据包
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedPacket {
    pub template_name: String,
    pub fields: Vec<ParsedField>,
    pub checksum_ok: bool,
    pub raw: Vec<u8>,
    pub timestamp: String,
}

impl ParsedPacket {
    /// Get a field value as f64 by name (for visualization)
    pub fn field_value(&self, name: &str) -> Option<f64> {
        self.fields
            .iter()
            .find(|f| f.name == name)
            .and_then(|f| f.value_f64)
    }
}

/// 数据包解析器 - 用模板匹配并解析接收到的二进制数据
#[derive(Debug, Clone)]
pub struct PacketParser {
    templates: Vec<PacketTemplate>,
}

impl PacketParser {
    pub fn new(templates: Vec<PacketTemplate>) -> Self {
        Self { templates }
    }

    pub fn update_templates(&mut self, templates: Vec<PacketTemplate>) {
        self.templates = templates;
    }

    /// Try to parse raw data against all registered templates.
    /// Returns the first successful parse, or None.
    pub fn try_parse(&self, data: &[u8]) -> Option<ParsedPacket> {
        for tmpl in &self.templates {
            if let Some(parsed) = self.parse_with_template(data, tmpl) {
                return Some(parsed);
            }
        }
        None
    }

    /// Parse raw data with a specific template
    pub fn parse_with_template(&self, data: &[u8], tmpl: &PacketTemplate) -> Option<ParsedPacket> {
        let header = parse_hex_string(&tmpl.header_hex);
        let tail = parse_hex_string(&tmpl.tail_hex);

        // Check minimum length
        let min_len = header.len() + tail.len() + if tmpl.include_length { 1 } else { 0 };
        if data.len() < min_len {
            return None;
        }

        // Check header match
        if !data.starts_with(&header) {
            return None;
        }

        // Check tail match
        if !tail.is_empty() && !data.ends_with(&tail) {
            return None;
        }

        let mut offset = header.len();

        // Read length byte if present
        let payload_len = if tmpl.include_length {
            if offset >= data.len() {
                return None;
            }
            let len = data[offset] as usize;
            offset += 1;
            len
        } else {
            // Calculate: total - header - tail - checksum_len
            let checksum_len = match tmpl.checksum_type {
                ChecksumType::None => 0,
                ChecksumType::Sum8 | ChecksumType::Xor | ChecksumType::Crc8 => 1,
                ChecksumType::Crc16Modbus | ChecksumType::Crc16Ccitt => 2,
                ChecksumType::Crc32 => 4,
            };
            data.len() - header.len() - tail.len() - checksum_len
        };

        let payload_start = offset;
        let payload_end = offset + payload_len;
        if payload_end > data.len() {
            return None;
        }

        // Verify checksum
        let checksum_ok = if tmpl.checksum_type != ChecksumType::None {
            let check_region = &data[header.len()..payload_end];
            let expected = compute_checksum(tmpl.checksum_type, check_region);
            let checksum_start = payload_end;
            let checksum_end = checksum_start + expected.len();
            if checksum_end > data.len() - tail.len() {
                false
            } else {
                &data[checksum_start..checksum_end] == expected.as_slice()
            }
        } else {
            true
        };

        // Parse fields from payload
        let payload = &data[payload_start..payload_end];
        let mut field_offset = 0;
        let mut parsed_fields = Vec::new();

        for field_def in &tmpl.fields {
            if !field_def.enabled {
                continue;
            }
            if field_offset >= payload.len() {
                break;
            }

            let remaining = &payload[field_offset..];
            let var_len = match field_def.field_type.byte_size() {
                Some(sz) => sz,
                None => remaining.len(), // variable-length: use remaining bytes
            };

            if let Some((value_str, consumed)) = PacketField::from_bytes(
                field_def.field_type,
                field_def.endianness,
                remaining,
                var_len,
            ) {
                let value_f64 = PacketField::numeric_from_bytes(
                    field_def.field_type,
                    field_def.endianness,
                    remaining,
                );
                parsed_fields.push(ParsedField {
                    name: field_def.name.clone(),
                    field_type: field_def.field_type,
                    value_str,
                    value_f64,
                    raw_bytes: remaining[..consumed].to_vec(),
                });
                field_offset += consumed;
            } else {
                break;
            }
        }

        Some(ParsedPacket {
            template_name: tmpl.name.clone(),
            fields: parsed_fields,
            checksum_ok,
            raw: data.to_vec(),
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// 校验方式
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumType {
    None,
    Sum8, // 字节累加和
    Xor,  // 异或校验
    Crc8,
    Crc16Modbus,
    Crc16Ccitt,
    Crc32,
}

impl ChecksumType {
    pub fn all() -> &'static [ChecksumType] {
        &[
            Self::None,
            Self::Sum8,
            Self::Xor,
            Self::Crc8,
            Self::Crc16Modbus,
            Self::Crc16Ccitt,
            Self::Crc32,
        ]
    }
}

impl std::fmt::Display for ChecksumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Sum8 => write!(f, "SUM8"),
            Self::Xor => write!(f, "XOR"),
            Self::Crc8 => write!(f, "CRC8"),
            Self::Crc16Modbus => write!(f, "CRC16/Modbus"),
            Self::Crc16Ccitt => write!(f, "CRC16/CCITT"),
            Self::Crc32 => write!(f, "CRC32"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// 数据包模板
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketTemplate {
    pub name: String,
    pub header_hex: String,
    pub fields: Vec<PacketField>,
    pub checksum_type: ChecksumType,
    pub tail_hex: String,
    pub include_length: bool,
    pub description: String,
}

impl Default for PacketTemplate {
    fn default() -> Self {
        Self {
            name: "New Packet".into(),
            header_hex: "AA".into(),
            fields: vec![PacketField::default()],
            checksum_type: ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        }
    }
}

impl PacketTemplate {
    pub fn build(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Header
        packet.extend(parse_hex_string(&self.header_hex));

        // Payload
        let mut payload = Vec::new();
        for field in &self.fields {
            if field.enabled {
                payload.extend(field.to_bytes());
            }
        }

        // Length
        if self.include_length {
            packet.push(payload.len() as u8);
        }

        packet.extend(&payload);

        // Checksum
        let check_data = &packet[parse_hex_string(&self.header_hex).len()..];
        let checksum_bytes = compute_checksum(self.checksum_type, check_data);
        packet.extend(&checksum_bytes);

        // Tail
        packet.extend(parse_hex_string(&self.tail_hex));

        packet
    }

    pub fn builtin_templates() -> Vec<PacketTemplate> {
        vec![
            PacketTemplate {
                name: "Robot Status Query".into(),
                header_hex: "AA".into(),
                fields: vec![PacketField {
                    name: "CMD".into(),
                    field_type: FieldType::U8,
                    value_str: "1".into(),
                    ..Default::default()
                }],
                checksum_type: ChecksumType::Sum8,
                tail_hex: "55".into(),
                include_length: true,
                description: "查询机器人状态".into(),
            },
            PacketTemplate {
                name: "Motor Control".into(),
                header_hex: "AA".into(),
                fields: vec![
                    PacketField {
                        name: "CMD".into(),
                        field_type: FieldType::U8,
                        value_str: "2".into(),
                        ..Default::default()
                    },
                    PacketField {
                        name: "Speed".into(),
                        field_type: FieldType::F32,
                        value_str: "0.0".into(),
                        ..Default::default()
                    },
                ],
                checksum_type: ChecksumType::Sum8,
                tail_hex: "55".into(),
                include_length: true,
                description: "电机速度控制".into(),
            },
            PacketTemplate {
                name: "Emergency Stop".into(),
                header_hex: "AA".into(),
                fields: vec![
                    PacketField {
                        name: "CMD".into(),
                        field_type: FieldType::U8,
                        value_str: "255".into(),
                        ..Default::default()
                    },
                    PacketField {
                        name: "Flag".into(),
                        field_type: FieldType::U8,
                        value_str: "1".into(),
                        ..Default::default()
                    },
                ],
                checksum_type: ChecksumType::Sum8,
                tail_hex: "55".into(),
                include_length: true,
                description: "紧急停止".into(),
            },
        ]
    }
}

// ═══════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════

pub fn parse_hex_string(s: &str) -> Vec<u8> {
    let cleaned = s.trim().replace([' ', ','], "");
    let hex_str = if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
        &cleaned[2..]
    } else {
        &cleaned
    };
    hex_str
        .as_bytes()
        .chunks(2)
        .filter_map(|chunk| {
            let s = std::str::from_utf8(chunk).ok()?;
            u8::from_str_radix(s, 16).ok()
        })
        .collect()
}

pub fn bytes_to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn compute_checksum(checksum_type: ChecksumType, data: &[u8]) -> Vec<u8> {
    match checksum_type {
        ChecksumType::None => vec![],
        ChecksumType::Sum8 => {
            let sum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
            vec![sum]
        }
        ChecksumType::Xor => {
            let xor: u8 = data.iter().fold(0u8, |acc, &b| acc ^ b);
            vec![xor]
        }
        ChecksumType::Crc8 => {
            let mut crc: u8 = 0xFF;
            for &byte in data {
                crc ^= byte;
                for _ in 0..8 {
                    if crc & 0x80 != 0 {
                        crc = (crc << 1) ^ 0x31;
                    } else {
                        crc <<= 1;
                    }
                }
            }
            vec![crc]
        }
        ChecksumType::Crc16Modbus => {
            let crc = crc16_modbus(data);
            crc.to_le_bytes().to_vec()
        }
        ChecksumType::Crc16Ccitt => {
            let mut crc: u16 = 0xFFFF;
            for &byte in data {
                crc ^= (byte as u16) << 8;
                for _ in 0..8 {
                    if crc & 0x8000 != 0 {
                        crc = (crc << 1) ^ 0x1021;
                    } else {
                        crc <<= 1;
                    }
                }
            }
            crc.to_be_bytes().to_vec()
        }
        ChecksumType::Crc32 => {
            let mut crc: u32 = 0xFFFFFFFF;
            for &byte in data {
                crc ^= byte as u32;
                for _ in 0..8 {
                    if crc & 1 != 0 {
                        crc = (crc >> 1) ^ 0xEDB88320;
                    } else {
                        crc >>= 1;
                    }
                }
            }
            (crc ^ 0xFFFFFFFF).to_le_bytes().to_vec()
        }
    }
}

pub fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── parse_hex_string ──────────────
    #[test]
    fn test_parse_hex_empty() {
        assert!(parse_hex_string("").is_empty());
    }

    #[test]
    fn test_parse_hex_simple() {
        assert_eq!(parse_hex_string("AA55"), vec![0xAA, 0x55]);
    }

    #[test]
    fn test_parse_hex_with_spaces() {
        assert_eq!(
            parse_hex_string("AA 55 00 FF"),
            vec![0xAA, 0x55, 0x00, 0xFF]
        );
    }

    #[test]
    fn test_parse_hex_with_0x_prefix() {
        assert_eq!(parse_hex_string("0xAA55"), vec![0xAA, 0x55]);
    }

    #[test]
    fn test_parse_hex_with_commas() {
        assert_eq!(parse_hex_string("AA,55,00"), vec![0xAA, 0x55, 0x00]);
    }

    #[test]
    fn test_parse_hex_lowercase() {
        assert_eq!(parse_hex_string("aa bb cc"), vec![0xAA, 0xBB, 0xCC]);
    }

    // ─── bytes_to_hex ──────────────────
    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(bytes_to_hex(&[0xAA, 0x55, 0x00]), "AA 55 00");
    }

    #[test]
    fn test_bytes_to_hex_empty() {
        assert_eq!(bytes_to_hex(&[]), "");
    }

    // ─── CRC16 Modbus ─────────────────
    #[test]
    fn test_crc16_modbus_known_value() {
        // CRC16/Modbus for [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]
        // The function returns native u16; LE bytes would be [0xCD, 0xC5]
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
        let crc = crc16_modbus(&data);
        assert_eq!(
            crc, 0xCDC5,
            "CRC16/Modbus should be 0xCDC5, got 0x{:04X}",
            crc
        );
    }

    #[test]
    fn test_crc16_modbus_empty() {
        let crc = crc16_modbus(&[]);
        assert_eq!(
            crc, 0xFFFF,
            "CRC of empty data should be initial value 0xFFFF"
        );
    }

    #[test]
    fn test_crc16_modbus_single_byte() {
        let crc = crc16_modbus(&[0x00]);
        // CRC16 Modbus for [0x00] should be deterministic
        assert_ne!(crc, 0xFFFF);
    }

    // ─── Checksum compute ─────────────
    #[test]
    fn test_checksum_none() {
        assert!(compute_checksum(ChecksumType::None, &[1, 2, 3]).is_empty());
    }

    #[test]
    fn test_checksum_sum8() {
        let result = compute_checksum(ChecksumType::Sum8, &[1, 2, 3, 4]);
        assert_eq!(result, vec![10]);
    }

    #[test]
    fn test_checksum_sum8_overflow() {
        let result = compute_checksum(ChecksumType::Sum8, &[0xFF, 0x01]);
        assert_eq!(result, vec![0x00]); // wrapping: 255 + 1 = 0
    }

    #[test]
    fn test_checksum_xor() {
        let result = compute_checksum(ChecksumType::Xor, &[0xAA, 0x55]);
        assert_eq!(result, vec![0xFF]);
    }

    #[test]
    fn test_checksum_crc32() {
        let result = compute_checksum(ChecksumType::Crc32, &[0x00]);
        assert_eq!(result.len(), 4);
    }

    // ─── PacketField ──────────────────
    #[test]
    fn test_field_u8() {
        let f = PacketField {
            field_type: FieldType::U8,
            value_str: "42".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![42]);
    }

    #[test]
    fn test_field_u16_le() {
        let f = PacketField {
            field_type: FieldType::U16,
            endianness: Endianness::Little,
            value_str: "256".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![0x00, 0x01]); // 256 LE = 0x0100
    }

    #[test]
    fn test_field_u16_be() {
        let f = PacketField {
            field_type: FieldType::U16,
            endianness: Endianness::Big,
            value_str: "256".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![0x01, 0x00]); // 256 BE = 0x0100
    }

    #[test]
    fn test_field_f32_le() {
        let f = PacketField {
            field_type: FieldType::F32,
            endianness: Endianness::Little,
            value_str: "1.0".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), 1.0f32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_field_bool_true() {
        let f = PacketField {
            field_type: FieldType::Bool,
            value_str: "true".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![1]);
    }

    #[test]
    fn test_field_bool_false() {
        let f = PacketField {
            field_type: FieldType::Bool,
            value_str: "false".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![0]);
    }

    #[test]
    fn test_field_ascii() {
        let f = PacketField {
            field_type: FieldType::Ascii,
            value_str: "ABC".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![0x41, 0x42, 0x43]);
    }

    #[test]
    fn test_field_hex_str() {
        let f = PacketField {
            field_type: FieldType::HexStr,
            value_str: "AABB".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_field_invalid_number_defaults_to_zero() {
        let f = PacketField {
            field_type: FieldType::U16,
            value_str: "not_a_number".into(),
            ..Default::default()
        };
        assert_eq!(f.to_bytes(), vec![0x00, 0x00]);
    }

    // ─── PacketTemplate ───────────────
    #[test]
    fn test_packet_template_build() {
        let tmpl = PacketTemplate {
            header_hex: "AA".into(),
            fields: vec![PacketField {
                field_type: FieldType::U8,
                value_str: "1".into(),
                enabled: true,
                ..Default::default()
            }],
            checksum_type: ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            ..Default::default()
        };
        let pkt = tmpl.build();
        assert_eq!(pkt[0], 0xAA, "Header should be 0xAA");
        assert_eq!(*pkt.last().unwrap(), 0x55, "Tail should be 0x55");
        assert!(pkt.len() >= 4, "Packet too short: {:?}", pkt);
    }

    #[test]
    fn test_builtin_templates_not_empty() {
        let templates = PacketTemplate::builtin_templates();
        assert!(!templates.is_empty());
        for tmpl in &templates {
            let pkt = tmpl.build();
            assert!(
                !pkt.is_empty(),
                "Template '{}' produced empty packet",
                tmpl.name
            );
        }
    }

    // ─── FieldType ────────────────────
    #[test]
    fn test_field_type_byte_sizes() {
        assert_eq!(FieldType::U8.byte_size(), Some(1));
        assert_eq!(FieldType::U16.byte_size(), Some(2));
        assert_eq!(FieldType::U32.byte_size(), Some(4));
        assert_eq!(FieldType::U64.byte_size(), Some(8));
        assert_eq!(FieldType::F32.byte_size(), Some(4));
        assert_eq!(FieldType::F64.byte_size(), Some(8));
        assert_eq!(FieldType::Bool.byte_size(), Some(1));
        assert_eq!(FieldType::Bytes.byte_size(), None);
        assert_eq!(FieldType::Ascii.byte_size(), None);
    }

    // ─── Endianness ───────────────────
    #[test]
    fn test_endianness_display() {
        assert_eq!(format!("{}", Endianness::Little), "LE");
        assert_eq!(format!("{}", Endianness::Big), "BE");
    }

    // ─── Cross-platform: u32/u64 field LE/BE roundtrip
    #[test]
    fn test_field_u32_roundtrip() {
        let val: u32 = 0x12345678;
        let f_le = PacketField {
            field_type: FieldType::U32,
            endianness: Endianness::Little,
            value_str: val.to_string(),
            ..Default::default()
        };
        let bytes = f_le.to_bytes();
        assert_eq!(
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            val
        );

        let f_be = PacketField {
            field_type: FieldType::U32,
            endianness: Endianness::Big,
            value_str: val.to_string(),
            ..Default::default()
        };
        let bytes = f_be.to_bytes();
        assert_eq!(
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            val
        );
    }

    // ─── from_bytes parsing ───────────────
    #[test]
    fn test_from_bytes_u8() {
        let (v, n) = PacketField::from_bytes(FieldType::U8, Endianness::Little, &[42], 0).unwrap();
        assert_eq!(v, "42");
        assert_eq!(n, 1);
    }

    #[test]
    fn test_from_bytes_u16_le() {
        let data = 1000u16.to_le_bytes();
        let (v, n) = PacketField::from_bytes(FieldType::U16, Endianness::Little, &data, 0).unwrap();
        assert_eq!(v, "1000");
        assert_eq!(n, 2);
    }

    #[test]
    fn test_from_bytes_u16_be() {
        let data = 1000u16.to_be_bytes();
        let (v, n) = PacketField::from_bytes(FieldType::U16, Endianness::Big, &data, 0).unwrap();
        assert_eq!(v, "1000");
        assert_eq!(n, 2);
    }

    #[test]
    fn test_from_bytes_i16_negative() {
        let data = (-500i16).to_le_bytes();
        let (v, n) = PacketField::from_bytes(FieldType::I16, Endianness::Little, &data, 0).unwrap();
        assert_eq!(v, "-500");
        assert_eq!(n, 2);
    }

    #[test]
    fn test_from_bytes_f32() {
        let data = std::f32::consts::PI.to_le_bytes();
        let (v, n) = PacketField::from_bytes(FieldType::F32, Endianness::Little, &data, 0).unwrap();
        assert!(v.starts_with("3.14"));
        assert_eq!(n, 4);
    }

    #[test]
    fn test_from_bytes_bool() {
        let (v, _) = PacketField::from_bytes(FieldType::Bool, Endianness::Little, &[1], 0).unwrap();
        assert_eq!(v, "true");
        let (v, _) = PacketField::from_bytes(FieldType::Bool, Endianness::Little, &[0], 0).unwrap();
        assert_eq!(v, "false");
    }

    #[test]
    fn test_from_bytes_ascii() {
        let (v, n) =
            PacketField::from_bytes(FieldType::Ascii, Endianness::Little, b"Hello", 5).unwrap();
        assert_eq!(v, "Hello");
        assert_eq!(n, 5);
    }

    #[test]
    fn test_from_bytes_too_short() {
        assert!(PacketField::from_bytes(FieldType::U16, Endianness::Little, &[1], 0).is_none());
        assert!(PacketField::from_bytes(FieldType::F32, Endianness::Little, &[1, 2], 0).is_none());
    }

    #[test]
    fn test_from_bytes_u32_roundtrip() {
        let val = 0xDEADBEEFu32;
        let bytes_le = val.to_le_bytes();
        let (v, _) =
            PacketField::from_bytes(FieldType::U32, Endianness::Little, &bytes_le, 0).unwrap();
        assert_eq!(v, format!("{}", val));
    }

    #[test]
    fn test_numeric_from_bytes_f32() {
        let val = 2.5f32;
        let bytes = val.to_le_bytes();
        let result =
            PacketField::numeric_from_bytes(FieldType::F32, Endianness::Little, &bytes).unwrap();
        assert!((result - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_numeric_from_bytes_bool() {
        assert_eq!(
            PacketField::numeric_from_bytes(FieldType::Bool, Endianness::Little, &[1]).unwrap(),
            1.0
        );
        assert_eq!(
            PacketField::numeric_from_bytes(FieldType::Bool, Endianness::Little, &[0]).unwrap(),
            0.0
        );
    }

    #[test]
    fn test_numeric_from_bytes_ascii_returns_none() {
        assert!(
            PacketField::numeric_from_bytes(FieldType::Ascii, Endianness::Little, b"hello")
                .is_none()
        );
    }

    // ─── PacketParser ─────────────────
    #[test]
    fn test_parser_roundtrip() {
        let tmpl = PacketTemplate {
            name: "Test".into(),
            header_hex: "AA".into(),
            fields: vec![
                PacketField {
                    name: "CMD".into(),
                    field_type: FieldType::U8,
                    value_str: "5".into(),
                    ..Default::default()
                },
                PacketField {
                    name: "Value".into(),
                    field_type: FieldType::U16,
                    endianness: Endianness::Little,
                    value_str: "1000".into(),
                    ..Default::default()
                },
            ],
            checksum_type: ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        };

        let built = tmpl.build();
        let parser = PacketParser::new(vec![tmpl]);
        let parsed = parser.try_parse(&built).unwrap();

        assert_eq!(parsed.template_name, "Test");
        assert!(parsed.checksum_ok);
        assert_eq!(parsed.fields.len(), 2);
        assert_eq!(parsed.fields[0].name, "CMD");
        assert_eq!(parsed.fields[0].value_str, "5");
        assert_eq!(parsed.fields[1].name, "Value");
        assert_eq!(parsed.fields[1].value_str, "1000");
    }

    #[test]
    fn test_parser_wrong_header() {
        let tmpl = PacketTemplate {
            header_hex: "AA".into(),
            tail_hex: "55".into(),
            ..Default::default()
        };
        let parser = PacketParser::new(vec![tmpl]);
        assert!(parser.try_parse(&[0xBB, 0x01, 0x00, 0x00, 0x55]).is_none());
    }

    #[test]
    fn test_parser_checksum_mismatch() {
        let tmpl = PacketTemplate {
            name: "Test".into(),
            header_hex: "AA".into(),
            fields: vec![PacketField {
                name: "CMD".into(),
                field_type: FieldType::U8,
                value_str: "1".into(),
                ..Default::default()
            }],
            checksum_type: ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        };

        // Build valid then corrupt checksum
        let mut pkt = tmpl.build();
        let tail_pos = pkt.len() - 1;
        let check_pos = tail_pos - 1;
        pkt[check_pos] ^= 0xFF; // corrupt checksum

        let parser = PacketParser::new(vec![tmpl]);
        let parsed = parser.try_parse(&pkt).unwrap();
        assert!(!parsed.checksum_ok);
    }

    #[test]
    fn test_parser_no_checksum() {
        let tmpl = PacketTemplate {
            name: "Simple".into(),
            header_hex: "AA".into(),
            fields: vec![PacketField {
                name: "Data".into(),
                field_type: FieldType::U8,
                value_str: "42".into(),
                ..Default::default()
            }],
            checksum_type: ChecksumType::None,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        };

        let built = tmpl.build();
        let parser = PacketParser::new(vec![tmpl]);
        let parsed = parser.try_parse(&built).unwrap();
        assert!(parsed.checksum_ok);
        assert_eq!(parsed.fields[0].value_str, "42");
    }

    #[test]
    fn test_parser_f32_field() {
        let tmpl = PacketTemplate {
            name: "FloatPkt".into(),
            header_hex: "AA".into(),
            fields: vec![PacketField {
                name: "Speed".into(),
                field_type: FieldType::F32,
                endianness: Endianness::Little,
                value_str: "3.14".into(),
                ..Default::default()
            }],
            checksum_type: ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        };

        let built = tmpl.build();
        let parser = PacketParser::new(vec![tmpl]);
        let parsed = parser.try_parse(&built).unwrap();
        assert!(parsed.fields[0].value_f64.is_some());
        let v = parsed.fields[0].value_f64.unwrap();
        assert!((v - std::f64::consts::PI).abs() < 0.01);
    }

    #[test]
    fn test_parser_multi_template() {
        let tmpl_a = PacketTemplate {
            name: "TypeA".into(),
            header_hex: "AA".into(),
            tail_hex: "55".into(),
            fields: vec![PacketField {
                name: "A".into(),
                field_type: FieldType::U8,
                value_str: "1".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let tmpl_b = PacketTemplate {
            name: "TypeB".into(),
            header_hex: "BB".into(),
            tail_hex: "55".into(),
            fields: vec![PacketField {
                name: "B".into(),
                field_type: FieldType::U8,
                value_str: "2".into(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let parser = PacketParser::new(vec![tmpl_a, tmpl_b.clone()]);
        let pkt = tmpl_b.build();
        let parsed = parser.try_parse(&pkt).unwrap();
        assert_eq!(parsed.template_name, "TypeB");
    }

    #[test]
    fn test_parsed_packet_field_value() {
        let parsed = ParsedPacket {
            template_name: "Test".into(),
            fields: vec![
                ParsedField {
                    name: "Speed".into(),
                    field_type: FieldType::F32,
                    value_str: "3.14".into(),
                    value_f64: Some(std::f64::consts::PI),
                    raw_bytes: vec![],
                },
                ParsedField {
                    name: "Status".into(),
                    field_type: FieldType::U8,
                    value_str: "1".into(),
                    value_f64: Some(1.0),
                    raw_bytes: vec![],
                },
            ],
            checksum_ok: true,
            raw: vec![],
            timestamp: String::new(),
        };
        assert!((parsed.field_value("Speed").unwrap() - std::f64::consts::PI).abs() < 0.01);
        assert!(parsed.field_value("NonExist").is_none());
    }
}
