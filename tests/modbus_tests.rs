use a3ot_modbus_protocol::{ModbusTCPUnit, ModbusRTU, RegisterType, ModbusTransportError};

#[cfg(test)]
mod tcp_tests {
    use super::*;

    #[test]
    fn test_tcp_builder_success() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build();

        assert!(modbus.is_ok());
    }

    #[test]
    fn test_tcp_builder_missing_device_id() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .build();

        assert!(matches!(modbus, Err(ModbusTransportError::DeviceIdMissing)));
    }

    #[test]
    fn test_tcp_builder_invalid_address() {
        let modbus = ModbusTCPUnit::builder()
            .address(70000) // > 65535
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build();

        assert!(matches!(modbus, Err(ModbusTransportError::Protocol(_))));
    }

    #[test]
    fn test_tcp_read_request_format() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let request = modbus.create_read_request().unwrap();

        // MBAP header (7 bytes) + PDU (5 bytes) = 12 bytes total
        assert_eq!(request.len(), 12);

        // Transaction ID (increments)
        assert_eq!(request[0], 0x00);
        assert_eq!(request[1], 0x01);

        // Protocol ID (always 0)
        assert_eq!(request[2], 0x00);
        assert_eq!(request[3], 0x00);

        // Length (PDU + unit_id = 6)
        assert_eq!(request[4], 0x00);
        assert_eq!(request[5], 0x06);

        // Unit ID
        assert_eq!(request[6], 0x01);

        // Function code (0x03 for holding registers)
        assert_eq!(request[7], 0x03);

        // Start address (100)
        assert_eq!(request[8], 0x00);
        assert_eq!(request[9], 0x64);

        // Length (10)
        assert_eq!(request[10], 0x00);
        assert_eq!(request[11], 0x0A);
    }

    #[test]
    fn test_tcp_transaction_id_increments() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let req1 = modbus.create_read_request().unwrap();
        let req2 = modbus.create_read_request().unwrap();

        // Transaction ID should increment
        assert_eq!(req1[1], 0x01);
        assert_eq!(req2[1], 0x02);
    }

    #[test]
    fn test_tcp_parse_valid_response() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Valid TCP response: MBAP + PDU
        let response = vec![
            0x00, 0x01, // Transaction ID
            0x00, 0x00, // Protocol ID
            0x00, 0x07, // Length (7 = unit + fc + byte_count + 4 data bytes)
            0x01,       // Unit ID
            0x03,       // Function code
            0x04,       // Byte count (2 registers * 2 bytes)
            0x12, 0x34, // Register 1 = 0x1234
            0x56, 0x78, // Register 2 = 0x5678
        ];

        let result = modbus.parse_response(&response);
        assert!(result.is_ok());

        // Now get the values
        let values = modbus.get();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], 0x1234);
        assert_eq!(values[1], 0x5678);
    }

    #[test]
    fn test_tcp_parse_frame_too_short() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let response = vec![0x00, 0x01, 0x00]; // Only 3 bytes

        let result = modbus.parse_response(&response);
        assert!(matches!(result, Err(ModbusTransportError::FrameTooShort)));
    }

    #[test]
    fn test_tcp_parse_invalid_protocol_id() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let response = vec![
            0x00, 0x01, // Transaction ID
            0x00, 0x01, // Invalid Protocol ID (should be 0)
            0x00, 0x07,
            0x01,
            0x03,
            0x04,
            0x12, 0x34,
            0x56, 0x78,
        ];

        let result = modbus.parse_response(&response);
        assert!(matches!(result, Err(ModbusTransportError::InvalidProtocolId(1))));
    }

    #[test]
    fn test_tcp_parse_unit_id_mismatch() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let response = vec![
            0x00, 0x01,
            0x00, 0x00,
            0x00, 0x07,
            0x02,       // Wrong unit ID (expected 1)
            0x03,
            0x04,
            0x12, 0x34,
            0x56, 0x78,
        ];

        let result = modbus.parse_response(&response);
        assert!(matches!(
            result,
            Err(ModbusTransportError::UnitIdMismatch { expected: 1, received: 2 })
        ));
    }

    #[test]
    fn test_tcp_write_request() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Set data using new API
        modbus.set(&[0x1234, 0x5678]).unwrap();

        let request = modbus.create_write_request().unwrap();

        // MBAP (7) + FC (1) + Addr (2) + Count (2) + ByteCount (1) + Data (4) = 17
        assert_eq!(request.len(), 17);
        assert_eq!(request[7], 0x10); // Function code 0x10 (write multiple)

        // Verify address
        assert_eq!(request[8], 0x00);
        assert_eq!(request[9], 0x64); // 100

        // Verify count
        assert_eq!(request[10], 0x00);
        assert_eq!(request[11], 0x02); // 2 registers

        // Verify byte count
        assert_eq!(request[12], 0x04); // 4 bytes

        // Verify data
        assert_eq!(request[13], 0x12);
        assert_eq!(request[14], 0x34);
        assert_eq!(request[15], 0x56);
        assert_eq!(request[16], 0x78);
    }

    #[test]
    fn test_tcp_write_without_setting_data() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Don't set any data - should fail
        let result = modbus.create_write_request();
        assert!(matches!(result, Err(ModbusTransportError::Protocol(_))));
    }

    #[test]
    fn test_tcp_write_partial_data() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(3)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Set only first 2 values out of 3
        modbus.set_to(0, 0x1234).unwrap();
        modbus.set_to(1, 0x5678).unwrap();
        // Don't set index 2

        // Should fail - missing value at index 2
        let result = modbus.create_write_request();
        assert!(matches!(result, Err(ModbusTransportError::Protocol(_))));
    }

    #[test]
    fn test_tcp_custom_write_command() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(1)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .with_write_cmd(0x10) // Force multi-write for single register
            .build()
            .unwrap();

        modbus.set(&[0x1234]).unwrap();
        let request = modbus.create_write_request().unwrap();

        // Should use 0x10 instead of default 0x06
        assert_eq!(request[7], 0x10);
    }

    #[test]
    fn test_tcp_set_to_with_index() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(5)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Test set_to with different index types
        assert!(modbus.set_to(0u8, 100).is_ok());
        assert!(modbus.set_to(1u16, 200).is_ok());
        assert!(modbus.set_to(2usize, 300).is_ok());
    }

    #[test]
    fn test_tcp_value_overflow() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Try to set value > u16::MAX
        let result = modbus.set_to(0, 70000);
        assert!(matches!(result, Err(ModbusTransportError::ValueOverflow(70000, 0))));

        // Try to set negative value
        let result = modbus.set_to(1, -100);
        assert!(matches!(result, Err(ModbusTransportError::ValueOverflow(-100, 1))));
    }
}


/*
#[cfg(test)]
mod rtu_tests {
    use super::*;

    #[test]
    fn test_rtu_builder_success() {
        let modbus = ModbusRTU::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build();

        assert!(modbus.is_ok());
    }

    #[test]
    fn test_rtu_read_request_format() {
        let modbus = ModbusRTU::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let request = modbus.create_read_request().unwrap();

        // Unit ID (1) + PDU (5) + CRC (2) = 8 bytes
        assert_eq!(request.len(), 8);

        // Unit ID
        assert_eq!(request[0], 0x01);

        // Function code
        assert_eq!(request[1], 0x03);

        // Address
        assert_eq!(request[2], 0x00);
        assert_eq!(request[3], 0x64);

        // Length
        assert_eq!(request[4], 0x00);
        assert_eq!(request[5], 0x0A);

        // CRC is at the end
        assert!(request.len() == 8);
    }

    #[test]
    fn test_rtu_parse_valid_response() {
        let modbus = ModbusRTU::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Valid RTU response with correct CRC
        let mut response = vec![
            0x01,       // Unit ID
            0x03,       // Function code
            0x04,       // Byte count
            0x12, 0x34, // Register 1
            0x56, 0x78, // Register 2
        ];

        // Calculate and append CRC
        let crc = calculate_test_crc(&response);
        response.push(crc as u8);
        response.push((crc >> 8) as u8);

        let result = modbus.parse_response(&response);
        assert!(result.is_ok());

        let values = modbus.get();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], 0x1234);
        assert_eq!(values[1], 0x5678);
    }

    #[test]
    fn test_rtu_parse_crc_mismatch() {
        let modbus = ModbusRTU::builder()
            .address(100)
            .length(2)
            .register_type(RegisterType::HoldingRegister)
            .device_id(1)
            .build()
            .unwrap();

        let response = vec![
            0x01,
            0x03,
            0x04,
            0x12, 0x34,
            0x56, 0x78,
            0x00, 0x00, // Wrong CRC
        ];

        let result = modbus.parse_response(&response);
        assert!(matches!(result, Err(ModbusTransportError::CrcMismatch { .. })));
    }

    #[test]
    fn test_rtu_coils_write() {
        let modbus = ModbusRTU::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::CoilRegister)
            .device_id(1)
            .build()
            .unwrap();

        let data = vec![1, 0, 1, 1, 0, 0, 0, 0, 1, 0];
        modbus.set(&data).unwrap();

        let request = modbus.create_write_request().unwrap();

        // Unit (1) + FC (1) + Addr (2) + Count (2) + ByteCount (1) + Data (2) + CRC (2) = 11
        assert_eq!(request.len(), 11);
        assert_eq!(request[1], 0x0F); // Write multiple coils
    }

    #[test]
    fn test_rtu_invalid_coil_value() {
        let modbus = ModbusRTU::builder()
            .address(100)
            .length(5)
            .register_type(RegisterType::CoilRegister)
            .device_id(1)
            .build()
            .unwrap();

        let data = vec![1, 0, 2, 0, 1]; // Invalid value: 2

        let result = modbus.set(&data);
        assert!(matches!(result, Err(ModbusTransportError::ValueOverflow(2, 2))));
    }

    fn calculate_test_crc(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }
}
*/
#[cfg(test)]
mod coil_tests {
    use super::*;

    #[test]
    fn test_tcp_single_coil_write() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(1)
            .register_type(RegisterType::CoilRegister)
            .device_id(1)
            .build()
            .unwrap();

        modbus.set(&[1]).unwrap();
        let request = modbus.create_write_request().unwrap();

        // MBAP (7) + FC (1) + Addr (2) + Value (2) = 12
        assert_eq!(request.len(), 12);
        assert_eq!(request[7], 0x05); // Single coil write
        assert_eq!(request[10], 0xFF); // True = 0xFF
        assert_eq!(request[11], 0x00);
    }

    #[test]
    fn test_tcp_multi_coil_bit_packing() {
        let mut modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(9)
            .register_type(RegisterType::CoilRegister)
            .device_id(1)
            .build()
            .unwrap();

        // [1,0,1,1,0,0,0,0,1] should pack to [0x0D, 0x01]
        let data = vec![1, 0, 1, 1, 0, 0, 0, 0, 1];
        modbus.set(&data).unwrap();
        let request = modbus.create_write_request().unwrap();

        // Find data bytes (after MBAP + FC + Addr + Count + ByteCount)
        let data_start = 7 + 1 + 2 + 2 + 1; // = 13

        // Byte 0: bits 0-7 = 0b00001101 = 0x0D
        assert_eq!(request[data_start], 0x0D);
        // Byte 1: bit 8 = 0b00000001 = 0x01
        assert_eq!(request[data_start + 1], 0x01);
    }

    #[test]
    fn test_tcp_parse_coils_response() {
        let modbus = ModbusTCPUnit::builder()
            .address(100)
            .length(10)
            .register_type(RegisterType::CoilRegister)
            .device_id(1)
            .build()
            .unwrap();

        // Response with 10 coils: [1,0,1,1,0,0,0,0,1,0]
        let response = vec![
            0x00, 0x01, // TX ID
            0x00, 0x00, // Proto
            0x00, 0x05, // Length
            0x01,       // Unit
            0x01,       // FC (read coils)
            0x02,       // Byte count (2 bytes for 10 bits)
            0x0D,       // Byte 0: bits 0-7
            0x01,       // Byte 1: bits 8-9
        ];

        modbus.parse_response(&response).unwrap();
        let values = modbus.get();

        assert_eq!(values.len(), 10);
        assert_eq!(values, vec![1, 0, 1, 1, 0, 0, 0, 0, 1, 0]);
    }
}