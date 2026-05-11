use crate::models::*;

pub struct ProtocolHub {
    pub modbus_frame: ModbusFrame,
    pub modbus_registers: Vec<u16>,
    pub modbus_response_log: Vec<String>,
    pub canopen_log: Vec<String>,
    pub canopen_pdo_configs: Vec<PdoConfig>,
    pub packet_templates: Vec<PacketTemplate>,
    pub packet_parser: PacketParser,
    pub parsed_packets: Vec<ParsedPacket>,
}

impl ProtocolHub {
    pub fn new() -> Self {
        Self {
            modbus_frame: ModbusFrame::default(),
            modbus_registers: Vec::new(),
            modbus_response_log: Vec::new(),
            canopen_log: Vec::new(),
            canopen_pdo_configs: Vec::new(),
            packet_templates: Vec::new(),
            packet_parser: PacketParser::new(Vec::new()),
            parsed_packets: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default() {
        let hub = ProtocolHub::new();
        assert!(hub.packet_templates.is_empty());
        assert!(hub.parsed_packets.is_empty());
        assert!(hub.modbus_registers.is_empty());
        assert!(hub.modbus_response_log.is_empty());
        assert!(hub.canopen_log.is_empty());
    }
}
