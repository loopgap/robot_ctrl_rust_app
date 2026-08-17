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

    /// Synchronize the packet parser with current templates.
    /// Must be called after packet_templates is modified.
    pub fn sync_packet_parser(&mut self) {
        self.packet_parser = PacketParser::new(self.packet_templates.clone());
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
        assert!(hub.canopen_pdo_configs.is_empty());
    }

    #[test]
    fn test_sync_packet_parser_with_templates() {
        let mut hub = ProtocolHub::new();
        hub.packet_templates.push(crate::models::PacketTemplate {
            name: "TestTemplate".into(),
            header_hex: "AA".into(),
            fields: vec![],
            checksum_type: crate::models::packet::ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        });
        hub.sync_packet_parser();
        assert_eq!(hub.packet_parser.template_count(), 1);
    }

    #[test]
    fn test_sync_packet_parser_clears_old() {
        let mut hub = ProtocolHub::new();
        hub.packet_templates.push(crate::models::PacketTemplate {
            name: "T1".into(),
            header_hex: "AA".into(),
            fields: vec![],
            checksum_type: crate::models::packet::ChecksumType::Sum8,
            tail_hex: "55".into(),
            include_length: true,
            description: String::new(),
        });
        hub.sync_packet_parser();
        assert_eq!(hub.packet_parser.template_count(), 1);

        // Replace templates and re-sync
        hub.packet_templates.clear();
        hub.packet_templates
            .push(crate::models::PacketTemplate::default());
        hub.packet_templates
            .push(crate::models::PacketTemplate::default());
        hub.sync_packet_parser();
        assert_eq!(hub.packet_parser.template_count(), 2);
    }

    #[test]
    fn test_modbus_registers_growable() {
        let mut hub = ProtocolHub::new();
        hub.modbus_registers.extend_from_slice(&[100, 200, 300]);
        assert_eq!(hub.modbus_registers.len(), 3);
        assert_eq!(hub.modbus_registers[1], 200);
    }

    #[test]
    fn test_response_log_append() {
        let mut hub = ProtocolHub::new();
        hub.modbus_response_log.push("Response 1".into());
        hub.modbus_response_log.push("Response 2".into());
        assert_eq!(hub.modbus_response_log.len(), 2);
        assert_eq!(hub.modbus_response_log[0], "Response 1");
    }

    #[test]
    fn test_canopen_log_append() {
        let mut hub = ProtocolHub::new();
        hub.canopen_log.push("NMT Start".into());
        assert_eq!(hub.canopen_log.len(), 1);
    }
}
