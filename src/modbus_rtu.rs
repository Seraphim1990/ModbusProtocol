use super::*;

pub struct ModbusRTUBuilder {
    unit_builder: ModbusUnitBuilder,
    device_id: Option<u8>,
}

impl ModbusRTUBuilder {
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

    pub fn build(self) -> Result<ModbusRTU, ModbusTransportError> {
        let unit = self.unit_builder.build()
            .map_err(ModbusTransportError::Protocol)?;

        let device_id = self.device_id.ok_or(ModbusTransportError::DeviceIdMissing)?;

        Ok(ModbusRTU {
            unit,
            device_id,
        })
    }
}

/// Modbus RTU client with encapsulated protocol logic
pub struct ModbusRTU {
    unit: ModbusUnit,
    device_id: u8,
}

impl ModbusRTU {
    /// Create new builder for Modbus RTU
    pub fn builder() -> ModbusRTUBuilder {
        ModbusRTUBuilder {
            unit_builder: ModbusUnit::builder(),
            device_id: None,
        }
    }

    /// Generate complete RTU frame for read request
    pub fn create_read_request(&self) -> Result<Vec<u8>, ModbusTransportError> {
        let pdu = self.unit.create_read_request()
            .map_err(ModbusTransportError::Protocol)?;
        Ok(self.wrap_rtu(pdu))
    }

    /// Generate complete RTU frame for write request
    pub fn create_write_request(&self) -> Result<Vec<u8>, ModbusTransportError> {
        let pdu = self.unit.get_write_request()
            .map_err(ModbusTransportError::Protocol)?;
        Ok(self.wrap_rtu(pdu))
    }

    /// Parse RTU response and extract values
    pub fn parse_response(&self, frame: &[u8]) -> Result<(), ModbusTransportError> {
        let pdu = self.unwrap_rtu(frame)?;
        self.unit.parse_response(&pdu)
            .map_err(ModbusTransportError::Protocol)
    }

    fn wrap_rtu(&self, pdu: Vec<u8>) -> Vec<u8> {
        let mut frame = Vec::with_capacity(1 + pdu.len() + 2);
        frame.push(self.device_id);
        frame.extend(&pdu);

        let crc = Self::calculate_crc(&frame);
        frame.push(crc as u8);
        frame.push((crc >> 8) as u8);

        frame
    }

    fn unwrap_rtu(&self, frame: &[u8]) -> Result<Vec<u8>, ModbusTransportError> {
        if frame.len() < 4 {
            return Err(ModbusTransportError::FrameTooShort);
        }

        let unit_id = frame[0];
        if unit_id != self.device_id {
            return Err(ModbusTransportError::UnitIdMismatch {
                expected: self.device_id,
                received: unit_id,
            });
        }

        let received_crc = (frame[frame.len() - 1] as u16) << 8 | frame[frame.len() - 2] as u16;
        let calculated_crc = Self::calculate_crc(&frame[..frame.len() - 2]);

        if received_crc != calculated_crc {
            return Err(ModbusTransportError::CrcMismatch {
                expected: calculated_crc,
                received: received_crc,
            });
        }

        Ok(frame[1..frame.len() - 2].to_vec())
    }

    fn calculate_crc(data: &[u8]) -> u16 {
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