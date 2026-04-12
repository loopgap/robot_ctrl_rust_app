use super::packet::crc16_modbus;
use serde::{Deserialize, Serialize};

/// Modbus 功能码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModbusFunction {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

impl ModbusFunction {
    pub fn all() -> &'static [ModbusFunction] {
        &[
            Self::ReadCoils,
            Self::ReadDiscreteInputs,
            Self::ReadHoldingRegisters,
            Self::ReadInputRegisters,
            Self::WriteSingleCoil,
            Self::WriteSingleRegister,
            Self::WriteMultipleCoils,
            Self::WriteMultipleRegisters,
        ]
    }

    pub fn code(&self) -> u8 {
        *self as u8
    }

    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Self::ReadCoils
                | Self::ReadDiscreteInputs
                | Self::ReadHoldingRegisters
                | Self::ReadInputRegisters
        )
    }
}

impl std::fmt::Display for ModbusFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadCoils => write!(f, "01 - Read Coils"),
            Self::ReadDiscreteInputs => write!(f, "02 - Read Discrete Inputs"),
            Self::ReadHoldingRegisters => write!(f, "03 - Read Holding Registers"),
            Self::ReadInputRegisters => write!(f, "04 - Read Input Registers"),
            Self::WriteSingleCoil => write!(f, "05 - Write Single Coil"),
            Self::WriteSingleRegister => write!(f, "06 - Write Single Register"),
            Self::WriteMultipleCoils => write!(f, "0F - Write Multiple Coils"),
            Self::WriteMultipleRegisters => write!(f, "10 - Write Multiple Registers"),
        }
    }
}

/// Modbus RTU 帧
#[derive(Debug, Clone)]
pub struct ModbusFrame {
    pub slave_id: u8,
    pub function: ModbusFunction,
    pub start_address: u16,
    pub quantity: u16,
    pub write_values: Vec<u16>,
}

impl Default for ModbusFrame {
    fn default() -> Self {
        Self {
            slave_id: 1,
            function: ModbusFunction::ReadHoldingRegisters,
            start_address: 0,
            quantity: 10,
            write_values: Vec::new(),
        }
    }
}

impl ModbusFrame {
    /// 构建 Modbus RTU 请求帧
    pub fn build_rtu_request(&self) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.push(self.slave_id);
        frame.push(self.function.code());

        match self.function {
            ModbusFunction::ReadCoils
            | ModbusFunction::ReadDiscreteInputs
            | ModbusFunction::ReadHoldingRegisters
            | ModbusFunction::ReadInputRegisters => {
                frame.extend_from_slice(&self.start_address.to_be_bytes());
                frame.extend_from_slice(&self.quantity.to_be_bytes());
            }
            ModbusFunction::WriteSingleCoil => {
                frame.extend_from_slice(&self.start_address.to_be_bytes());
                let val: u16 = if self.write_values.first().copied().unwrap_or(0) != 0 {
                    0xFF00
                } else {
                    0x0000
                };
                frame.extend_from_slice(&val.to_be_bytes());
            }
            ModbusFunction::WriteSingleRegister => {
                frame.extend_from_slice(&self.start_address.to_be_bytes());
                let val = self.write_values.first().copied().unwrap_or(0);
                frame.extend_from_slice(&val.to_be_bytes());
            }
            ModbusFunction::WriteMultipleCoils => {
                frame.extend_from_slice(&self.start_address.to_be_bytes());
                let qty = self.quantity;
                frame.extend_from_slice(&qty.to_be_bytes());
                let byte_count = (qty as usize).div_ceil(8) as u8;
                frame.push(byte_count);
                // 将 write_values 打包为位
                for i in 0..byte_count as usize {
                    let mut byte_val: u8 = 0;
                    for bit in 0..8 {
                        let idx = i * 8 + bit;
                        if idx < self.write_values.len() && self.write_values[idx] != 0 {
                            byte_val |= 1 << bit;
                        }
                    }
                    frame.push(byte_val);
                }
            }
            ModbusFunction::WriteMultipleRegisters => {
                frame.extend_from_slice(&self.start_address.to_be_bytes());
                let qty = self.write_values.len() as u16;
                frame.extend_from_slice(&qty.to_be_bytes());
                frame.push((qty * 2) as u8);
                for &val in &self.write_values {
                    frame.extend_from_slice(&val.to_be_bytes());
                }
            }
        }

        // 添加 CRC16
        let crc = crc16_modbus(&frame);
        frame.extend_from_slice(&crc.to_le_bytes());

        frame
    }

    /// 构建 Modbus TCP 请求帧 (MBAP header + PDU)
    pub fn build_tcp_request(&self, transaction_id: u16) -> Vec<u8> {
        let mut pdu = Vec::new();
        pdu.push(self.slave_id);
        pdu.push(self.function.code());

        match self.function {
            ModbusFunction::ReadCoils
            | ModbusFunction::ReadDiscreteInputs
            | ModbusFunction::ReadHoldingRegisters
            | ModbusFunction::ReadInputRegisters => {
                pdu.extend_from_slice(&self.start_address.to_be_bytes());
                pdu.extend_from_slice(&self.quantity.to_be_bytes());
            }
            ModbusFunction::WriteSingleCoil => {
                pdu.extend_from_slice(&self.start_address.to_be_bytes());
                let val: u16 = if self.write_values.first().copied().unwrap_or(0) != 0 {
                    0xFF00
                } else {
                    0x0000
                };
                pdu.extend_from_slice(&val.to_be_bytes());
            }
            ModbusFunction::WriteSingleRegister => {
                pdu.extend_from_slice(&self.start_address.to_be_bytes());
                let val = self.write_values.first().copied().unwrap_or(0);
                pdu.extend_from_slice(&val.to_be_bytes());
            }
            ModbusFunction::WriteMultipleRegisters => {
                pdu.extend_from_slice(&self.start_address.to_be_bytes());
                let qty = self.write_values.len() as u16;
                pdu.extend_from_slice(&qty.to_be_bytes());
                pdu.push((qty * 2) as u8);
                for &val in &self.write_values {
                    pdu.extend_from_slice(&val.to_be_bytes());
                }
            }
            _ => {}
        }

        // MBAP Header
        let mut frame = Vec::new();
        frame.extend_from_slice(&transaction_id.to_be_bytes()); // Transaction ID
        frame.extend_from_slice(&0u16.to_be_bytes()); // Protocol ID (0 = Modbus)
        frame.extend_from_slice(&(pdu.len() as u16).to_be_bytes()); // Length
        frame.extend(&pdu);

        frame
    }

    /// 解析 Modbus RTU 响应
    pub fn parse_rtu_response(data: &[u8]) -> Option<ModbusResponse> {
        if data.len() < 5 {
            return None;
        }

        // 验证 CRC
        let payload = &data[..data.len() - 2];
        let received_crc = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
        let calc_crc = crc16_modbus(payload);
        if received_crc != calc_crc {
            return None;
        }

        let slave_id = data[0];
        let function_code = data[1];

        // 异常响应
        if function_code & 0x80 != 0 {
            if data.len() < 5 {
                return None;
            }
            return Some(ModbusResponse {
                slave_id,
                function_code: function_code & 0x7F,
                data: Vec::new(),
                is_error: true,
                error_code: Some(data[2]),
            });
        }

        let byte_count = data[2] as usize;
        if data.len() < 3 + byte_count + 2 {
            return None; // 数据不完整
        }
        let response_data = data[3..3 + byte_count].to_vec();

        Some(ModbusResponse {
            slave_id,
            function_code,
            data: response_data,
            is_error: false,
            error_code: None,
        })
    }
}

/// Modbus 响应
#[derive(Debug, Clone)]
pub struct ModbusResponse {
    pub slave_id: u8,
    pub function_code: u8,
    pub data: Vec<u8>,
    pub is_error: bool,
    pub error_code: Option<u8>,
}

impl ModbusResponse {
    /// 将响应数据解析为 u16 寄存器值
    pub fn as_registers(&self) -> Vec<u16> {
        self.data
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_be_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::packet::crc16_modbus;

    #[test]
    fn test_modbus_function_codes() {
        assert_eq!(ModbusFunction::ReadCoils.code(), 0x01);
        assert_eq!(ModbusFunction::ReadHoldingRegisters.code(), 0x03);
        assert_eq!(ModbusFunction::WriteSingleRegister.code(), 0x06);
        assert_eq!(ModbusFunction::WriteMultipleRegisters.code(), 0x10);
    }

    #[test]
    fn test_modbus_function_is_read() {
        assert!(ModbusFunction::ReadCoils.is_read());
        assert!(ModbusFunction::ReadHoldingRegisters.is_read());
        assert!(!ModbusFunction::WriteSingleCoil.is_read());
        assert!(!ModbusFunction::WriteMultipleRegisters.is_read());
    }

    #[test]
    fn test_build_rtu_read_holding() {
        let frame = ModbusFrame {
            slave_id: 1,
            function: ModbusFunction::ReadHoldingRegisters,
            start_address: 0,
            quantity: 10,
            write_values: Vec::new(),
        };
        let rtu = frame.build_rtu_request();
        // [slave_id, fn_code, addr_hi, addr_lo, qty_hi, qty_lo, crc_lo, crc_hi]
        assert_eq!(rtu.len(), 8);
        assert_eq!(rtu[0], 1);
        assert_eq!(rtu[1], 0x03);
        assert_eq!(rtu[2], 0x00);
        assert_eq!(rtu[3], 0x00); // address = 0
        assert_eq!(rtu[4], 0x00);
        assert_eq!(rtu[5], 0x0A); // quantity = 10
                                  // Verify CRC
        let payload = &rtu[..6];
        let expected_crc = crc16_modbus(payload);
        let actual_crc = u16::from_le_bytes([rtu[6], rtu[7]]);
        assert_eq!(actual_crc, expected_crc);
    }

    #[test]
    fn test_build_rtu_write_single_register() {
        let frame = ModbusFrame {
            slave_id: 1,
            function: ModbusFunction::WriteSingleRegister,
            start_address: 100,
            quantity: 1,
            write_values: vec![0x1234],
        };
        let rtu = frame.build_rtu_request();
        assert_eq!(rtu[0], 1);
        assert_eq!(rtu[1], 0x06);
        // address = 100 = 0x0064
        assert_eq!(rtu[2], 0x00);
        assert_eq!(rtu[3], 0x64);
        // value = 0x1234
        assert_eq!(rtu[4], 0x12);
        assert_eq!(rtu[5], 0x34);
    }

    #[test]
    fn test_build_tcp_request() {
        let frame = ModbusFrame::default();
        let tcp = frame.build_tcp_request(42);
        // MBAP header: transaction_id(2) + protocol_id(2) + length(2) + unit_id(1) + fn(1) + ...
        assert!(tcp.len() >= 8);
        assert_eq!(u16::from_be_bytes([tcp[0], tcp[1]]), 42); // transaction ID
        assert_eq!(u16::from_be_bytes([tcp[2], tcp[3]]), 0); // protocol ID
    }

    #[test]
    fn test_parse_rtu_response_valid() {
        // 构造有效的 RTU 响应: slave=1, fn=03, byte_count=4, data=[00 01 00 02], CRC
        let mut resp = vec![0x01, 0x03, 0x04, 0x00, 0x01, 0x00, 0x02];
        let crc = crc16_modbus(&resp);
        resp.extend_from_slice(&crc.to_le_bytes());

        let parsed = ModbusFrame::parse_rtu_response(&resp);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.slave_id, 1);
        assert_eq!(parsed.function_code, 0x03);
        assert!(!parsed.is_error);
        assert_eq!(parsed.data, vec![0x00, 0x01, 0x00, 0x02]);
    }

    #[test]
    fn test_parse_rtu_response_crc_mismatch() {
        let resp = vec![0x01, 0x03, 0x02, 0x00, 0x01, 0xFF, 0xFF]; // bad CRC
        let parsed = ModbusFrame::parse_rtu_response(&resp);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_rtu_response_too_short() {
        let parsed = ModbusFrame::parse_rtu_response(&[0x01, 0x03]);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_rtu_response_error_response() {
        // 异常响应: slave=1, fn=0x83, error_code=0x02, CRC
        let mut resp = vec![0x01, 0x83, 0x02];
        let crc = crc16_modbus(&resp);
        resp.extend_from_slice(&crc.to_le_bytes());

        let parsed = ModbusFrame::parse_rtu_response(&resp);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert!(parsed.is_error);
        assert_eq!(parsed.error_code, Some(0x02));
    }

    #[test]
    fn test_parse_rtu_response_overflow_byte_count() {
        // byte_count=255 但数据不够 → 应该返回 None 而不是 panic
        let mut resp = vec![0x01, 0x03, 0xFF, 0x00];
        let crc = crc16_modbus(&resp);
        resp.extend_from_slice(&crc.to_le_bytes());
        let parsed = ModbusFrame::parse_rtu_response(&resp);
        assert!(
            parsed.is_none(),
            "Should return None for overflow byte_count"
        );
    }

    #[test]
    fn test_modbus_response_as_registers() {
        let resp = ModbusResponse {
            slave_id: 1,
            function_code: 0x03,
            data: vec![0x00, 0x01, 0x00, 0x02, 0x00, 0x03],
            is_error: false,
            error_code: None,
        };
        assert_eq!(resp.as_registers(), vec![1, 2, 3]);
    }

    #[test]
    fn test_modbus_response_as_registers_odd_bytes() {
        let resp = ModbusResponse {
            slave_id: 1,
            function_code: 0x03,
            data: vec![0x00, 0x01, 0x02], // 奇数字节
            is_error: false,
            error_code: None,
        };
        // 最后一个不完整字节应被忽略
        assert_eq!(resp.as_registers(), vec![1]);
    }

    #[test]
    fn test_all_functions_listed() {
        assert_eq!(ModbusFunction::all().len(), 8);
    }
}
