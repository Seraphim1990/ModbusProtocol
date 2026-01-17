# a3ot_modbus_protocol

A pure Rust implementation of the Modbus protocol (TCP and RTU) with a clean, type-safe API.

## Features

- ✅ **Modbus TCP** - Full MBAP header handling with transaction IDs
- ✅ **Modbus RTU** - CRC-16 calculation and validation
- ✅ **Read operations** - Holding Registers, Input Registers, Coils, Discrete Inputs
- ✅ **Write operations** - Single and multiple register/coil writes
- ✅ **Custom function codes** - Override standard commands for non-compliant devices

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
a3ot_modbus_protocol = "0.1.0"
```

## Quick Start

### Modbus TCP

```rust
use a3ot_modbus_protocol::{ModbusTCP, RegisterType};

// Create a Modbus TCP client
let mut modbus = ModbusTCP::builder()
    .address(100)
    .length(10)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .build()?;

// Generate read request
let request = modbus.create_read_request()?;
// Send `request` over TCP socket...

// Parse response
let values: Vec<u16> = modbus.parse_response(&response)?;
```

### Modbus RTU

```rust
use a3ot_modbus_protocol::{ModbusRTU, RegisterType};

// Create a Modbus RTU client
let modbus = ModbusRTU::builder()
    .address(200)
    .length(5)
    .register_type(RegisterType::CoilRegister)
    .device_id(2)
    .build()?;

// Generate read request with CRC
let request = modbus.create_read_request()?;
// Send `request` over serial port...

// Parse response (validates CRC automatically)
let values: Vec<u16> = modbus.parse_response(&response)?;
```

### Write Operations

**Note:** For universality, all write operations accept `Vec<i32>` as input, including boolean coil values (0 or 1). Read operations always return `Vec<u16>`, with boolean values represented as 0 or 1.

```rust
// Write multiple holding registers
let mut modbus = ModbusTCP::builder()
    .address(100)
    .length(3)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .build()?;

let data = vec![0x1234, 0x5678, 0xABCD];  // i32 values
let request = modbus.create_write_request(&data)?;

// Write coils (binary values as i32)
let modbus = ModbusTCP::builder()
    .address(50)
    .length(8)
    .register_type(RegisterType::CoilRegister)
    .device_id(1)
    .build()?;

let coils = vec![1, 0, 1, 1, 0, 0, 1, 0];  // i32: 0 or 1
let request = modbus.create_write_request(&coils)?;

// Read returns u16 values
let values: Vec<u16> = modbus.parse_response(&response)?;
// For coils: values will be [1, 0, 1, 1, 0, 0, 1, 0] as u16
```

### Custom Function Codes

Sometimes Modbus devices accept non-standard commands. This library provides the ability to override default function codes for such cases.

**Important:** If your device follows the standard Modbus protocol, you don't need to explicitly specify commands - they will be selected automatically based on the register type.

```rust
// Example: Device that requires 0x10 for single register write instead of standard 0x06
let modbus = ModbusTCP::builder()
    .address(100)
    .length(1)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .with_write_cmd(0x10)  // Override default 0x06
    .build()?;

// You can also override read and multi-write commands
let modbus = ModbusTCP::builder()
    .address(200)
    .length(10)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .with_read_cmd(0x04)         // Custom read command
    .with_multi_write_cmd(0x17)  // Custom multi-write command
    .build()?;
```

## Supported Register Types

```rust
pub enum RegisterType {
    CoilRegister,          // Read: 0x01, Write: 0x05/0x0F
    DiscreteRegister,      // Read: 0x02 (read-only)
    HoldingRegister,       // Read: 0x03, Write: 0x06/0x10
    InputRegister,         // Read: 0x04 (read-only)
}
```

## Error Handling

All operations return `Result` types with detailed error information:

```rust
use a3ot_modbus_protocol::ModbusTransportError;

match modbus.parse_response(&response) {
    Ok(values) => println!("Read {} registers", values.len()),
    Err(ModbusTransportError::CrcMismatch { expected, received }) => {
        eprintln!("CRC error: expected {:#x}, got {:#x}", expected, received);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```


## License

MIT
