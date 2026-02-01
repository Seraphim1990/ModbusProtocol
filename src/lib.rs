// lib.rs

mod core;
mod modbus_tcp;
mod modbus_rtu;

pub use core::{RegisterType};
pub use modbus_rtu::{ModbusRTU, ModbusRTUBuilder};
pub use modbus_tcp::{ModbusTCPUnit, ModbusTCPUnitBuilder};

pub use core::{ModbusUnit, ModbusUnitBuilder, ModbusUnitError};

#[derive(Debug, thiserror::Error)]
pub enum ModbusTransportError {
    #[error("Frame too short")]
    FrameTooShort,

    #[error("Invalid protocol ID: {0}")]
    InvalidProtocolId(u16),

    #[error("Unit ID mismatch: expected {expected}, received {received}")]
    UnitIdMismatch { expected: u8, received: u8 },

    #[error("CRC mismatch: expected {expected:#06x}, received {received:#06x}")]
    CrcMismatch { expected: u16, received: u16 },

    #[error("Device ID not set")]
    DeviceIdMissing,

    #[error("Protocol error: {0}")]
    Protocol(#[from] ModbusUnitError),

    #[error("Value overflow:{0} as u16 at index {1}")]
    ValueOverflow(i32, usize),

    #[error("Invalid index at set")]
    InvalidIndexAtSet,
}