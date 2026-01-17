use super::*;
pub struct ModbusTCPBuilder {
    unit_builder: ModbusUnitBuilder,
    device_id: Option<u8>,
}

impl ModbusTCPBuilder {
    pub fn address(mut self, addr: i32) -> Self {
        self.unit_builder.address(addr);
        self
    }

    pub fn length(mut self, length: i32) -> Self {
        self.unit_builder.length(length);
        self
    }

    pub fn register_type(mut self, register_type: RegisterType) -> Self {
        self.unit_builder.register_type(register_type);
        self
    }

    pub fn with_read_cmd(mut self, spec_read_cmd: i32) -> Self {
        self.unit_builder.with_read_cmd(spec_read_cmd);
        self
    }

    pub fn with_write_cmd(mut self, spec_write_cmd: i32) -> Self {
        self.unit_builder.with_write_cmd(spec_write_cmd);
        self
    }

    pub fn with_multi_write_cmd(mut self, multi_write_cmd: i32) -> Self {
        self.unit_builder.with_multi_write_cmd(multi_write_cmd);
        self
    }

    pub fn device_id(mut self, device_id: u8) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn build(self) -> Result<ModbusTCP, ModbusTransportError> {
        let unit = self.unit_builder.build()
            .map_err(ModbusTransportError::Protocol)?;

        let device_id = self.device_id.ok_or(ModbusTransportError::DeviceIdMissing)?;

        Ok(ModbusTCP {
            unit,
            transaction_id: 0,
            device_id,
        })
    }
}

/// Modbus TCP client with encapsulated protocol logic
pub struct ModbusTCP {
    unit: ModbusUnit,
    transaction_id: u16,
    device_id: u8,
}

impl ModbusTCP {
    /// Create new builder for Modbus TCP
    pub fn builder() -> ModbusTCPBuilder {
        ModbusTCPBuilder {
            unit_builder: ModbusUnit::builder(),
            device_id: None,
        }
    }

    /// Generate complete TCP frame for read request
    pub fn create_read_request(&mut self) -> Result<Vec<u8>, ModbusTransportError> {
        let pdu = self.unit.create_read_request()
            .map_err(ModbusTransportError::Protocol)?;
        Ok(self.wrap_tcp(pdu))
    }

    pub fn create_write_request(&mut self, data: &[i32]) -> Result<Vec<u8>, ModbusTransportError> {
        let pdu = self.unit.get_write_request(data)
            .map_err(ModbusTransportError::Protocol)?;
        Ok(self.wrap_tcp(pdu))
    }

    /// Parse TCP response and extract values
    pub fn parse_response(&self, frame: &[u8]) -> Result<Vec<u16>, ModbusTransportError> {
        let pdu = self.unwrap_tcp(frame)?;
        self.unit.parse_response(&pdu)
            .map_err(ModbusTransportError::Protocol)
    }

    fn wrap_tcp(&mut self, pdu: Vec<u8>) -> Vec<u8> {
        self.transaction_id = self.transaction_id.wrapping_add(1);

        let length = (pdu.len() + 1) as u16;
        let mut frame = Vec::with_capacity(7 + pdu.len());

        frame.push((self.transaction_id >> 8) as u8);
        frame.push(self.transaction_id as u8);
        frame.push(0x00);
        frame.push(0x00);
        frame.push((length >> 8) as u8);
        frame.push(length as u8);
        frame.push(self.device_id);
        frame.extend(pdu);

        frame
    }

    fn unwrap_tcp(&self, frame: &[u8]) -> Result<Vec<u8>, ModbusTransportError> {
        if frame.len() < 7 {
            return Err(ModbusTransportError::FrameTooShort);
        }

        let protocol_id = ((frame[2] as u16) << 8) | (frame[3] as u16);
        if protocol_id != 0 {
            return Err(ModbusTransportError::InvalidProtocolId(protocol_id));
        }

        let unit_id = frame[6];
        if unit_id != self.device_id {
            return Err(ModbusTransportError::UnitIdMismatch {
                expected: self.device_id,
                received: unit_id,
            });
        }

        let length = ((frame[4] as u16) << 8) | (frame[5] as u16);
        let expected_len = 6 + length as usize;

        if frame.len() < expected_len {
            return Err(ModbusTransportError::FrameTooShort);
        }

        Ok(frame[7..expected_len].to_vec())
    }
}