# a3ot_modbus_protocol

Чиста Rust реалізація протоколу Modbus (TCP та RTU) з чистим, типобезпечним API.

## Можливості

- ✅ **Modbus TCP** - Повна обробка MBAP заголовків з transaction ID
- ✅ **Modbus RTU** - Розрахунок та валідація CRC-16
- ✅ **Операції читання** - Holding Registers, Input Registers, Coils, Discrete Inputs
- ✅ **Операції запису** - Запис одного та декількох регістрів/котушок
- ✅ **Кастомні команди** - Перевизначення стандартних команд для нестандартних пристроїв

## Встановлення

Додайте до вашого `Cargo.toml`:

```toml
[dependencies]
a3ot_modbus_protocol = "0.1.0"
```

## Швидкий старт

### Modbus TCP

```rust
use a3ot_modbus_protocol::{ModbusTCP, RegisterType};

// Створення Modbus TCP клієнта
let mut modbus = ModbusTCP::builder()
    .address(100)
    .length(10)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .build()?;

// Генерація запиту на читання
let request = modbus.create_read_request()?;
// Відправте `request` через TCP сокет...

// Парсинг відповіді
let values: Vec<u16> = modbus.parse_response(&response)?;
```

### Modbus RTU

```rust
use a3ot_modbus_protocol::{ModbusRTU, RegisterType};

// Створення Modbus RTU клієнта
let modbus = ModbusRTU::builder()
    .address(200)
    .length(5)
    .register_type(RegisterType::CoilRegister)
    .device_id(2)
    .build()?;

// Генерація запиту на читання з CRC
let request = modbus.create_read_request()?;
// Відправте `request` через serial порт...

// Парсинг відповіді (автоматична валідація CRC)
let values: Vec<u16> = modbus.parse_response(&response)?;
```

### Операції запису

**Примітка:** Для універсальності всі операції запису приймають `Vec<i32>` на вході, включно з булевими значеннями котушок (0 або 1). Операції читання завжди повертають `Vec<u16>`, де булеві значення представлені як 0 або 1.

```rust
// Запис декількох holding регістрів
let mut modbus = ModbusTCP::builder()
    .address(100)
    .length(3)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .build()?;

let data = vec![0x1234, 0x5678, 0xABCD];  // i32 значення
let request = modbus.create_write_request(&data)?;

// Запис котушок (булеві значення як i32)
let modbus = ModbusTCP::builder()
    .address(50)
    .length(8)
    .register_type(RegisterType::CoilRegister)
    .device_id(1)
    .build()?;

let coils = vec![1, 0, 1, 1, 0, 0, 1, 0];  // i32: 0 або 1
let request = modbus.create_write_request(&coils)?;

// Читання повертає u16 значення
let values: Vec<u16> = modbus.parse_response(&response)?;
// Для котушок: values буде [1, 0, 1, 1, 0, 0, 1, 0] як u16
```

### Кастомні команди функцій

Іноді Modbus пристрої приймають нестандартні команди. Ця бібліотека надає можливість перевизначити стандартні коди функцій для таких випадків.

**Важливо:** Якщо ваш пристрій працює за стандартним протоколом Modbus, вам не потрібно явно вказувати команди - вони будуть обрані автоматично на основі типу регістра.

```rust
// Приклад: Пристрій, що вимагає 0x10 для запису одного регістра замість стандартного 0x06
let modbus = ModbusTCP::builder()
    .address(100)
    .length(1)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .with_write_cmd(0x10)  // Перевизначення стандартного 0x06
    .build()?;

// Ви також можете перевизначити команди читання та множинного запису
let modbus = ModbusTCP::builder()
    .address(200)
    .length(10)
    .register_type(RegisterType::HoldingRegister)
    .device_id(1)
    .with_read_cmd(0x04)         // Кастомна команда читання
    .with_multi_write_cmd(0x17)  // Кастомна команда множинного запису
    .build()?;
```

## Підтримувані типи регістрів

```rust
pub enum RegisterType {
    CoilRegister,          // Читання: 0x01, Запис: 0x05/0x0F
    DiscreteRegister,      // Читання: 0x02 (тільки читання)
    HoldingRegister,       // Читання: 0x03, Запис: 0x06/0x10
    InputRegister,         // Читання: 0x04 (тільки читання)
}
```

## Обробка помилок

Всі операції повертають типи `Result` з детальною інформацією про помилки:

```rust
use a3ot_modbus_protocol::ModbusTransportError;

match modbus.parse_response(&response) {
    Ok(values) => println!("Прочитано {} регістрів", values.len()),
    Err(ModbusTransportError::CrcMismatch { expected, received }) => {
        eprintln!("Помилка CRC: очікувалось {:#x}, отримано {:#x}", expected, received);
    }
    Err(e) => eprintln!("Помилка: {}", e),
}
```

## Архітектура

Бібліотека розділена на три рівні:

1. **Протокол ядра** (модуль `core`) - Чиста генерація та парсинг Modbus PDU
2. **Обгортки транспорту** - TCP (MBAP) та RTU (CRC) фреймінг
3. **Публічне API** - Builder pattern з типобезпекою

Користувачі взаємодіють тільки з `ModbusTCP` та `ModbusRTU` - протокол ядра повністю інкапсульований.



## Ліцензія

MIT
