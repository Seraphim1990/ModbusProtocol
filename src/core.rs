use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModbusUnitError {
    #[error("Invalid address: {0} > 65535")]
    InvalidAddress(i32),

    #[error("Start address is empty")]
    AddressIsEmpty,

    #[error("Invalid length: {0} < 0 or {0} > 65535")]
    InvalidLength(i32),

    #[error("Invalid range: {0} + {1} = {2} > 65535")]
    RangeToMatch(i32, i32, i32),


    #[error("Register type is empty")]
    InvalidRegisterType,

    #[error("Read command {0} < 0 or {0} > 255")]
    InvalidReadCommand(i32),

    #[error("Write command {0} < 0 or {0} > 255")]
    InvalidWriteCommand(i32),

    #[error("Multi write command {0} < 0 or {0} > 255")]
    InvalidWriteMultiCommand(i32),

    #[error("Type {0:?} haven't write command")]
    InvalidRegisterTypeForWriteCommand(RegisterType),

    #[error("Value {0} overflow u16 or i16")]
    ValueOverflow(i32),

    #[error("Invalid coil value {0} at index {1}, expected 0 or 1")]
    InvalidCoilValue(i32, usize),

    #[error("Empty response received")]
    EmptyResponse,

    #[error("Modbus exception: function code {0:#x}, exception code {1:#x}")]
    ModbusException(u8, u8),

    #[error("Unexpected function code: expected {0:#x}, got {1:#x}")]
    UnexpectedFunctionCode(u8, u8),

    #[error("Invalid response length")]
    InvalidResponseLength,

    #[error("Data length mismatch: expected max {expected}, got {actual}")]
    DataLengthMismatch { expected: usize, actual: usize },
}

#[derive(Copy, Clone, Debug, )]
pub enum RegisterType {
    CoilRegister,
    DiscreteRegister,
    HoldingRegister,
    InputRegister,
}

pub struct ModbusUnit {
    start_addr: u16,
    length: u16,
    register_type: RegisterType,
    read_cmd: Option<i32>,
    write_cmd: Option<i32>,
    multi_write_cmd: Option<i32>,
}

pub struct ModbusUnitBuilder {
    start_addr: Option<i32>,
    length: Option<i32>,
    register_type: Option<RegisterType>,
    spec_read_cmd: Option<i32>,
    spec_write_cmd: Option<i32>,
    spec_multi_write_cmd: Option<i32>,
}

impl ModbusUnitBuilder {
    pub fn address(&mut self, addr: i32) -> &mut Self {
        self.start_addr = Some(addr);
        self
    }

    pub fn length(&mut self, length: i32) -> &mut Self {
        self.length = Some(length);
        self
    }

    pub fn register_type(&mut self, register_type: RegisterType) -> &mut Self {
        self.register_type = Some(register_type);
        self
    }

    pub fn with_read_cmd(&mut self, spec_read_cmd: i32) -> &mut Self {
        self.spec_read_cmd = Some(spec_read_cmd);
        self
    }

    pub fn with_write_cmd(&mut self, spec_write_cmd: i32) -> &mut Self {
        self.spec_write_cmd = Some(spec_write_cmd);
        self
    }

    pub fn with_multi_write_cmd(&mut self, multi_write_cmd: i32) -> &mut Self {
        self.spec_multi_write_cmd = Some(multi_write_cmd);
        self
    }

    pub fn build(self) -> Result<ModbusUnit, ModbusUnitError> {
        let start_addr = match self.start_addr {
            Some(addr) => {
                if addr < 0 || addr > 65535 {
                    return Err(ModbusUnitError::InvalidAddress(addr));
                }
                addr
            } ,
            None => return Err(ModbusUnitError::AddressIsEmpty),
        };
        let reg_type = match self.register_type {
            Some(_reg_type) => _reg_type,
            None => return Err(ModbusUnitError::InvalidRegisterType),
        };
        let length = match self.length {
            Some(length) => {
                if length < 0 || length > 65535 {
                    return Err(ModbusUnitError::InvalidLength(length));
                }
                length
            },
            None => 1,
        };
        let end_addr = start_addr + length;
        if end_addr > 65535 {
            return Err(ModbusUnitError::RangeToMatch(start_addr, length, end_addr));
        }
        let read_cmd = match self.spec_read_cmd {
            Some(spec_read_cmd) => {
                if spec_read_cmd < 0 || spec_read_cmd > 255 {
                    return Err(ModbusUnitError::InvalidReadCommand(spec_read_cmd));
                }
                Some(spec_read_cmd)
            },
            None => None,
        };
        let write_cmd = match self.spec_write_cmd {
            Some(spec_write_cmd) => {
                if spec_write_cmd < 0 || spec_write_cmd > 255 {
                    return Err(ModbusUnitError::InvalidWriteCommand(spec_write_cmd));
                }
                Some(spec_write_cmd)
            },
            None => None,
        };
        let multi_write_cmd = match self.spec_multi_write_cmd {
            Some(spec_multi_write_cmd) => {
                if spec_multi_write_cmd < 0 || spec_multi_write_cmd > 255 {
                    return Err(ModbusUnitError::InvalidWriteMultiCommand(spec_multi_write_cmd));
                }
                Some(spec_multi_write_cmd)
            },
            None => None,
        };
        Ok(
            ModbusUnit {
                start_addr: start_addr as u16,
                length: length as u16,
                register_type: reg_type,
                read_cmd: read_cmd,
                write_cmd: write_cmd,
                multi_write_cmd: multi_write_cmd,
            }
        )
    }
}

impl ModbusUnit {
    pub fn builder() -> ModbusUnitBuilder {
        ModbusUnitBuilder {
            start_addr: None,
            length: None,
            register_type: None,
            spec_read_cmd: None,
            spec_write_cmd: None,
            spec_multi_write_cmd: None,
        }
    }

    pub fn create_read_request(&self) -> Result<Vec<u8>, ModbusUnitError> {
        let mut msg: [u8; 5] = [0; 5];
        let command = match self.read_cmd {
            Some(cmd) => cmd as u8,
            None => self.get_read_command(),
        };
        msg[0] = command;
        msg[1] = (self.start_addr >> 8) as u8;
        msg[2] = self.start_addr as u8;
        msg[3] = (self.length >> 8) as u8;
        msg[4] = self.length as u8;
        Ok(Vec::from(msg)) // no err. all data for read validate in builder
    }

    fn get_read_command(&self) -> u8 {
        match self.register_type {
            RegisterType::CoilRegister => 0x01,
            RegisterType::DiscreteRegister => 0x02,
            RegisterType::HoldingRegister => 0x03,
            RegisterType::InputRegister => 0x04,
        }
    }

    pub fn get_write_request(&self, data: &[i32]) -> Result<Vec<u8>, ModbusUnitError> {
        // Validate data length matches unit length
        if data.len() > self.length as usize {
            return Err(ModbusUnitError::DataLengthMismatch {
                expected: self.length as usize,
                actual: data.len(),
            });
        }

        let cmd = self.get_write_command(data.len())?;

        match self.register_type {
            RegisterType::CoilRegister => self.get_for_body_for_coils_write(data, cmd),
            RegisterType::HoldingRegister => self.get_for_body_for_holding_write(data, cmd),
            _ => Err(ModbusUnitError::InvalidRegisterTypeForWriteCommand(self.register_type))
        }
    }

    fn get_write_command(&self, length: usize) -> Result<u8, ModbusUnitError> {
        match length {
            1 => self.get_single_write_command(),
            _ => self.get_multi_write_command()
        }
    }

    fn get_single_write_command(&self) -> Result<u8, ModbusUnitError> {
        let cmd = match self.write_cmd {
            Some(cmd) => cmd as u8,
            None => match self.register_type {
                RegisterType::CoilRegister => 0x05,
                RegisterType::HoldingRegister => 0x06,
                _ => return Err(ModbusUnitError::InvalidRegisterTypeForWriteCommand(self.register_type))
            }
        };
        Ok(cmd)
    }

    fn get_multi_write_command(&self) -> Result<u8, ModbusUnitError> {
        let cmd = match self.multi_write_cmd {
            Some(cmd) => cmd as u8,
            None => match self.register_type {
                RegisterType::CoilRegister => 0x0F,
                RegisterType::HoldingRegister => 0x10,
                _ => return Err(ModbusUnitError::InvalidRegisterTypeForWriteCommand(self.register_type))
            }
        };
        Ok(cmd)
    }

    fn get_for_body_for_holding_write(&self, data: &[i32], cmd: u8) -> Result<Vec<u8>, ModbusUnitError> {
        let request_len = 3 + { if data.len() == 1 {2} else {3 + data.len() * 2} }; // ← Змінено: +3 замість +2
        let mut result: Vec<u8> = Vec::with_capacity(request_len);
        result.push(cmd);
        result.push((self.start_addr >> 8) as u8);
        result.push(self.start_addr as u8);

        if data.len() > 1 {
            result.push((data.len() >> 8) as u8);
            result.push(data.len() as u8);
            result.push((data.len() * 2) as u8);  // ← ДОДАНО ByteCount!
        }

        for item in data.iter() {
            let val = match i16::try_from(*item) {
                Ok(val) => val,
                Err(_) => return Err(ModbusUnitError::ValueOverflow(*item))
            };
            result.push((val >> 8) as u8);
            result.push(val as u8);
        }
        Ok(result)
    }

    fn get_for_body_for_coils_write(&self, data: &[i32], cmd: u8) -> Result<Vec<u8>, ModbusUnitError> {
        // Validate: all values must be 0 or 1
        for (i, &val) in data.iter().enumerate() {
            if val != 0 && val != 1 {
                return Err(ModbusUnitError::InvalidCoilValue(val, i));
            }
        }

        let capacity = if data.len() == 1 {
            5  // cmd + addr(2) + value(2)
        } else {
            let byte_count = (data.len() + 7) / 8;
            6 + byte_count  // cmd + addr(2) + count(2) + byte_count(1) + data
        };

        let mut result: Vec<u8> = Vec::with_capacity(capacity);
        result.push(cmd);
        result.push((self.start_addr >> 8) as u8);
        result.push(self.start_addr as u8);

        if data.len() == 1 {
            // Single coil: 0xFF00 for true, 0x0000 for false
            let val = if data[0] != 0 { 0xFF } else { 0x00 };
            result.push(val);
            result.push(0x00);
        } else {
            // Multiple coils
            result.push((data.len() >> 8) as u8);
            result.push(data.len() as u8);

            // Calculate byte count
            let byte_count = (data.len() + 7) / 8;
            result.push(byte_count as u8);

            // Pack bits into bytes
            let mut bytes = vec![0u8; byte_count];
            for (i, &bit) in data.iter().enumerate() {
                if bit != 0 {
                    bytes[i / 8] |= 1 << (i % 8);
                }
            }
            result.extend(bytes);
        }
        Ok(result)
    }
    pub fn parse_response(&self, pdu: &[u8]) -> Result<Vec<u16>, ModbusUnitError> {
        if pdu.is_empty() {
            return Err(ModbusUnitError::EmptyResponse);
        }

        let function_code = pdu[0];

        // Check for Modbus exception (function code | 0x80)
        if (function_code & 0x80) != 0 {
            let exception_code = if pdu.len() > 1 { pdu[1] } else { 0 };
            return Err(ModbusUnitError::ModbusException(function_code, exception_code));
        }

        // Verify function code matches expected
        let expected_fc = self.get_read_command();
        if function_code != expected_fc {
            return Err(ModbusUnitError::UnexpectedFunctionCode(expected_fc, function_code));
        }

        // Parse based on register type
        match self.register_type {
            RegisterType::HoldingRegister | RegisterType::InputRegister => {
                self.parse_holding_registers(pdu)
            }
            RegisterType::CoilRegister | RegisterType::DiscreteRegister => {
                self.parse_coils(pdu)
            }
        }
    }

    fn parse_holding_registers(&self, pdu: &[u8]) -> Result<Vec<u16>, ModbusUnitError> {
        if pdu.len() < 2 {
            return Err(ModbusUnitError::InvalidResponseLength);
        }

        let byte_count = pdu[1] as usize;
        let expected_bytes = self.length as usize * 2;

        if byte_count != expected_bytes || pdu.len() < 2 + byte_count {
            return Err(ModbusUnitError::InvalidResponseLength);
        }

        let mut result:Vec<u16> = Vec::with_capacity(self.length as usize);

        for i in 0..self.length as usize {
            let offset = 2 + i * 2;
            let value = ((pdu[offset] as u16) << 8) | (pdu[offset + 1] as u16);
            result.push(value);
        }
        Ok(result)
    }

    fn parse_coils(&self, pdu: &[u8]) -> Result<Vec<u16>, ModbusUnitError> {
        if pdu.len() < 2 {
            return Err(ModbusUnitError::InvalidResponseLength);
        }

        let byte_count = pdu[1] as usize;
        let expected_bytes = (self.length as usize + 7) / 8;

        if byte_count != expected_bytes || pdu.len() < 2 + byte_count {
            return Err(ModbusUnitError::InvalidResponseLength);
        }

        let mut result = Vec::with_capacity(self.length as usize);

        for i in 0..self.length as usize {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let bit_value = (pdu[2 + byte_idx] >> bit_idx) & 0x01;
            result.push(bit_value as u16);
        }
        Ok(result)
    }
}
