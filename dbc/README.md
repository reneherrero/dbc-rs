# dbc

A clean, zero-dependency DBC (CAN Database) file parser and editor for Rust.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/crates/l/dbc-rs.svg)](https://github.com/reneherrero/dbc-rs/blob/main/LICENSING.md)
[![MSRV](https://img.shields.io/badge/rustc-1.85.0+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-rs%20Library%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-rs.yml)

## Minimum Supported Rust Version (MSRV)

The minimum supported Rust version is **1.85.0**. The crate is tested against this version and may use features available in it or later.

## Features

- ✅ **Zero dependencies** - Pure Rust implementation
- ✅ **no_std + alloc support** - Works on embedded targets without the standard library
- ✅ **Full editing & writing** - Modify and save DBC files with the same structs
- ✅ **Feature flag control** - Optional `std` feature for desktop conveniences
- ✅ **Well tested** - Tested with real-world DBC files

## Quick Start

```rust
use dbc_rs::Dbc;

// Parse a DBC file
let content = std::fs::read_to_string("example.dbc").unwrap();
let dbc = Dbc::parse(&content).expect("invalid dbc");

// Access messages and signals (read-only)
if let Some(engine_msg) = dbc.messages().iter().find(|m| m.id() == 256) {
    if let Some(rpm) = engine_msg.signals().iter().find(|s| s.name() == "RPM") {
        println!("RPM factor: {}", rpm.factor());
    }
}
```

## Feature Flags

The crate is `#![no_std]` + `alloc` friendly. The following features are available:

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | ✅       | Enables standard library integration (I/O helpers, tests). Disable it for pure `no_std` builds. |

**Example (no_std build):**
```toml
[dependencies]
dbc-rs = { version = "1", default-features = false }
```

## Internationalization (i18n)

Error messages can be localized at build time using feature flags. The language is selected during compilation and cannot be changed at runtime.

### Supported Languages

| Language | Feature Flag | Code |
|----------|-------------|------|
| English (default) | *(none)* | `en` |
| French | `lang-fr` | `fr` |
| Spanish | `lang-es` | `es` |
| German | `lang-de` | `de` |
| Japanese | `lang-ja` | `ja` |

**Example:**
```toml
[dependencies]
dbc-rs = { version = "1", features = ["lang-fr"] }
```

**Note:** Language features are mutually exclusive. If multiple language features are enabled, the last one in the feature list will be used.

⚠️ **Warning**: Translations have been generated and may contain errors. They have not been fully verified by native speakers. Contributions to improve translations are welcome! See [Contributing](#contributing) section.

## DBC Format Feature Support

### Core Features ✅

| Feature | Statement | Parse | Write | Notes |
|---------|-----------|-------|-------|-------|
| **Version** | `VERSION` | ✅ | ✅ | Database version string |
| **New Symbols** | `NS_` | ⚠️ | ❌ | Parsed but ignored |
| **Bit Timing** | `BS_` | ⚠️ | ❌ | Parsed but ignored |
| **Bus Nodes** | `BU_` | ✅ | ✅ | List of ECUs on the bus |
| **Messages** | `BO_` | ✅ | ✅ | CAN message definitions |
| **Signals** | `SG_` | ✅ | ✅ | Signal definitions |
| **Comments** | `//` | ✅ | ❌ | Single-line comments parsed but not preserved |

### Signal Features ✅

All signal features are fully supported: name, start bit, length, byte order (`@0`/`@1`), sign (`+`/`-`), factor, offset, min/max values, unit, and receivers (Broadcast `*`, specific nodes, or None).

### Extended Features ❌

Not yet implemented: Value tables (`VAL_TABLE_`, `VAL_`), structured comments (`CM_`), attributes (`BA_DEF_`, `BA_DEF_DEF_`, `BA_`), signal groups (`SIG_GROUP_`), environment variables (`EV_`), signal multiplexing, and advanced node relationships.

### Advanced Features

| Feature | Parse | Write | Notes |
|---------|-------|-------|-------|
| 29-bit Extended CAN IDs | ✅ | ✅ | Validated (range: 2048-536870911) |
| Signal multiplexing | ❌ | ❌ | |

## Example DBC File

```dbc
VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C"

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
```

## Examples

### Basic Parsing

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;
println!("Version: {}", dbc.version().to_string());
println!("Nodes: {}", dbc.nodes().to_string());
println!("Messages: {}", dbc.messages().len());
```

### Finding Messages and Signals

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;

// Find message by ID
if let Some(msg) = dbc.messages().iter().find(|m| m.id() == 256) {
    // Find signal by name
    if let Some(sig) = msg.signals().find("RPM") {
        println!("RPM: factor={}, offset={}", sig.factor(), sig.offset());
    }
}
```

### Creating and Modifying DBC Files

```rust
use dbc_rs::{Dbc, Version, Nodes, Message, Signal, ByteOrder, Receivers};

// Create from scratch
let version = Version::builder().major(1).minor(0).build()?;
let nodes = Nodes::builder().add_node("ECM").add_node("TCM").build();

let signal = Signal::builder()
    .name("EngineSpeed")
    .start_bit(0)
    .length(16)
    .byte_order(ByteOrder::BigEndian)
    .unsigned(true)
    .factor(0.25)
    .offset(0.0)
    .min(0.0)
    .max(8000.0)
    .unit("rpm")
    .receivers(Receivers::Broadcast)
    .build()?;

let message = Message::builder()
    .id(256)
    .name("EngineData")
    .dlc(8)
    .sender("ECM")
    .add_signal(signal)
    .build()?;

let dbc = Dbc::builder()
    .version(version)
    .nodes(nodes)
    .add_message(message)
    .build()?;

// Save to string
let dbc_string = dbc.to_dbc_string();
```

### Modifying Existing DBC

```rust
use dbc_rs::{Dbc, Message, Signal, ByteOrder, Receivers};

let dbc = Dbc::parse(&content)?;

// Extract current data
let version = dbc.version();
let nodes = dbc.nodes();
let mut messages: Vec<Message> = dbc.messages().iter().cloned().collect();

// Add new message
let new_signal = Signal::builder()
    .name("NewSignal")
    .start_bit(0)
    .length(8)
    .byte_order(ByteOrder::BigEndian)
    .unsigned(true)
    .factor(1.0)
    .offset(0.0)
    .min(0.0)
    .max(255.0)
    .receivers(Receivers::Broadcast)
    .build()?;

let new_message = Message::builder()
    .id(1024)
    .name("NewMessage")
    .dlc(8)
    .sender("ECM")
    .add_signal(new_signal)
    .build()?;

messages.push(new_message);

// Create modified DBC
let modified_dbc = Dbc::builder()
    .version(version.clone())
    .nodes(nodes.clone())
    .messages(messages)
    .build()?;
```

### no_std Usage

```rust
use dbc_rs::Dbc;

// Parse from bytes (no file I/O needed)
let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
let dbc = Dbc::parse_bytes(dbc_bytes)?;

// Access data
let version = dbc.version();
let messages = dbc.messages();

// Save to string
let saved = dbc.to_dbc_string();
```

### Error Handling

```rust
use dbc_rs::{Dbc, Error};

match Dbc::parse(invalid_content) {
    Ok(dbc) => println!("Parsed: {} messages", dbc.messages().len()),
    Err(Error::InvalidData(msg)) => eprintln!("Data error: {}", msg),
    Err(Error::Signal(msg)) => eprintln!("Signal error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security & Resource Limits

For security reasons (DoS protection), the library enforces the following limits:

- **Maximum 256 nodes** per DBC file
- **Maximum 64 receiver nodes** per signal
- **Maximum 10,000 messages** per DBC file
- **Maximum 64 signals** per message
- **Maximum 256 characters** for unit strings

Attempting to exceed these limits will result in a validation error. These limits accommodate typical DBC file sizes (typically < 1000 messages and < 10 nodes).

For a comprehensive security audit, see [SECURITY_AUDIT.md](SECURITY_AUDIT.md).

## Limitations

1. **Extended Features**: Many advanced DBC features (attributes, value tables, structured comments, etc.) are not yet supported. Files containing these features will parse successfully but the extended data will be lost on save.
2. **Comments**: Single-line `//` comments are parsed but not preserved when saving.
3. **Signal Multiplexing**: Multiplexed signals are not yet supported.

## Troubleshooting

### Common Issues

**"Message ID out of valid range"**
- Standard 11-bit IDs: Use 0-0x7FF (0-2047)
- Extended 29-bit IDs: Use 0x800-0x1FFFFFFF (2048-536870911)

**"Signal extends beyond CAN message"**
- Ensure `start_bit + length <= DLC * 8`

**"Signal overlap detected"**
- Ensure signals don't occupy overlapping bit ranges

**"Sender not in nodes"**
- Add the message sender to the nodes list

**"Duplicate message ID"**
- Use unique CAN IDs for each message

## Architecture & Design

### Design Principles

1. **Immutability**: All data structures are immutable after creation
2. **Validation**: Input validation occurs at construction time
3. **no_std First**: Designed to work in `no_std` environments with `alloc`
4. **Zero Dependencies**: No external dependencies
5. **Memory Efficiency**: Uses `Box<str>` and pre-allocated vectors

### Error Handling

- **Result-based**: All fallible operations return `Result<T>`
- **Categorized errors**: `Error::Signal`, `Error::Message`, `Error::Dbc`, `Error::Version`, `Error::Nodes`, `Error::InvalidData`
- **Internationalized**: Error messages can be localized at build time
- **Descriptive**: Error messages include context about what failed

## Performance Notes

- **Memory Usage**: Uses `Box<str>` for strings, pre-allocated vectors
- **Parsing**: O(n) complexity, entire file parsed into memory
- **Recommendations**: For very large DBC files (>10MB), consider splitting into multiple files

## Contributing

Contributions are welcome! Areas that need work:

- Value tables and enumerations (VAL_TABLE_, VAL_)
- Structured comments (CM_)
- Attributes (BA_DEF_, BA_, etc.)
- Environment variables (EV_)
- Signal multiplexing support
- **Translation improvements** - Help verify and improve error message translations

### Adding a New Language

1. Create a new file in `src/error/lang/` (e.g., `it.rs` for Italian)
2. Copy the structure from `en.rs` and translate all constants
3. Add the language module to `src/error/lang/mod.rs`:
   ```rust
   mod it;
   #[cfg(feature = "lang-it")]
   use it as lang;
   ```
4. Add the feature flag to `Cargo.toml`: `lang-it = []`
5. Update this README with the new language
6. Submit a pull request

**Translation Guidelines:**
- Maintain the same constant names across all language files
- Keep format placeholders (`{}`) in the same positions
- Ensure technical terms are accurately translated

## License

dbc-rs is available under a **dual-licensing** model:

- **Open Source**: MIT OR Apache-2.0 (default) - See [LICENSING.md](LICENSING.md) for details
- **Commercial**: Available for proprietary use - See [LICENSING.md](LICENSING.md) for terms

For most users, the open-source license (MIT OR Apache-2.0) is sufficient.

## References

- [DBC Format Specification](DBC_SPECIFICATIONS.md) - Detailed format documentation
- [Security Audit](SECURITY_AUDIT.md) - Comprehensive security review
- Vector Informatik: "DBC File Format Documentation Version 01/2007"
- CAN Specification (ISO 11898)

### Test DBC Files

The test suite includes several DBC files in `tests/data/`:
- **`complete.dbc`** - Comprehensive test file with multiple messages and signals
- **`simple.dbc`** - Basic 2-message file
- **`multiplexed.dbc`** - Tests multiple sensors and actuators
- **`minimal.dbc`** - Minimal valid DBC file
- **`extended_ids.dbc`** - Tests higher message IDs
- **`broadcast_signals.dbc`** - Tests broadcast receivers

For additional DBC files, consider:
- [commaai/opendbc](https://github.com/commaai/opendbc) - Open-source collection of DBC files
- [CSS Electronics DBC Editor](https://www.csselectronics.com/pages/dbc-editor-can-bus-database) - Sample DBC files
