# dbc-rs

A clean, zero-dependency DBC (CAN Database) file parser and editor for Rust.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/crates/l/dbc-rs.svg)](https://github.com/reneherrero/dbc-rs/blob/main/dbc/LICENSING.md)
[![MSRV](https://img.shields.io/badge/rustc-1.85.0+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-rs%20Library%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-rs.yml)

## Minimum Supported Rust Version (MSRV)

The minimum supported Rust version is **1.85.0**. The crate is tested against this version and may use features available in it or later.

## Features

- ✅ **Minimal dependencies** - Pure Rust implementation
- ✅ **no_std support** - Works on embedded targets without the standard library
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

The crate supports multiple levels of functionality through feature flags:

| Feature | Default | Description |
|---------|---------|-------------|
| `alloc` | ❌ | **REQUIRED** (unless using `heapless`): Enables heap-allocated collections via `alloc` crate. Uses `Vec` for dynamic collections. Requires global allocator. |
| `heapless` | ❌ | **REQUIRED** (unless using `alloc`): Enables stack-allocated, bounded collections (no `alloc` needed). Uses fixed-size arrays with capacity limits. |
| `std` | ✅ | Includes `alloc` + standard library features (I/O helpers, file operations, builders, string formatting). |

**⚠️ Important:** You **MUST** enable either `alloc` OR `heapless` (or use `std` which includes `alloc`). The library will not compile without one of these features.

**Feature Hierarchy:**
- **`alloc`**: Heap-allocated `Vec` collections (requires global allocator). Choose this for systems with dynamic memory allocation.
- **`heapless`**: Stack-allocated, bounded collections (no allocator needed). Choose this for embedded systems without allocators. Capacity limits apply.
- **`std`** (default): Includes `alloc` + std library features (builders, I/O, formatting). Best for desktop/server applications.

**Note:** You use either `alloc` OR `heapless`, not both. The `std` feature automatically includes `alloc`.

**Examples:**

```toml
# Default: std enabled (includes alloc)
[dependencies]
dbc-rs = "1"

# no_std with heap-allocated collections (requires global allocator)
[dependencies]
dbc-rs = { version = "1", default-features = false, features = ["alloc"] }

# no_std with stack-allocated, bounded collections (no allocator needed)
[dependencies]
dbc-rs = { version = "1", default-features = false, features = ["heapless"] }

# ❌ This will NOT compile - you must enable either alloc or heapless
# [dependencies]
# dbc-rs = { version = "1", default-features = false }
```

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
| **Value Descriptions** | `VAL_` | ✅ | ✅ | Value descriptions for signals and messages |
| **Comments** | `//` | ✅ | ❌ | Single-line comments parsed but not preserved |

### Signal Features ✅

All signal features are fully supported: name, start bit, length, byte order (`@0`/`@1`), sign (`+`/`-`), factor, offset, min/max values, unit, and receivers (Broadcast `*`, specific nodes, or None).

### Advanced Features

| Feature | Parse | Write | Notes |
|---------|-------|-------|-------|
| 29-bit Extended CAN IDs | ✅ | ✅ | Validated (range: 2048-536870911) |

### Extended Features ❌

Not yet implemented: Value tables (`VAL_TABLE_`), structured comments (`CM_`), attributes (`BA_DEF_`, `BA_DEF_DEF_`, `BA_`), signal groups (`SIG_GROUP_`), environment variables (`EV_`), signal multiplexing, and advanced node relationships.

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

// Note: to_string() requires std feature
#[cfg(feature = "std")]
{
    println!("Version: {}", dbc.version().map(|v| v.to_string()).unwrap_or_default());
    println!("Nodes: {}", dbc.nodes().to_string());
}
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
use dbc_rs::{Dbc, VersionBuilder, NodesBuilder, MessageBuilder, SignalBuilder, ByteOrder, ReceiversBuilder};

// Create from scratch
let version = VersionBuilder::new().version("1.0").build()?;
let nodes = NodesBuilder::new().add_node("ECM").add_node("TCM").build()?;

let signal = SignalBuilder::new()
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
    .receivers(ReceiversBuilder::new().broadcast())
    .build()?;

let message = MessageBuilder::new()
    .id(256)
    .name("EngineData")
    .dlc(8)
    .sender("ECM")
    .add_signal(signal)
    .build()?;

let dbc = DbcBuilder::new()
    .version(version)
    .nodes(nodes)
    .add_message(message)
    .build()?;

// Save to string
let dbc_string = dbc.to_dbc_string();
```

### Parsing with Options

```rust
use dbc_rs::{Dbc, ParseOptions};

// Use lenient mode to allow signals that extend beyond message boundaries
// This is useful for parsing real-world DBC files that may have technically
// invalid but commonly used signal definitions
let options = ParseOptions::lenient();
let dbc = Dbc::parse_with_options(&content, options)?;
```

### Modifying Existing DBC

```rust
use dbc_rs::{Dbc, DbcBuilder, MessageBuilder, SignalBuilder, ByteOrder, Receivers};

let dbc = Dbc::parse(&content)?;

// Create builder from existing DBC (preserves version, nodes, and messages)
let modified_dbc = DbcBuilder::new(Some(&dbc))
    .add_message(
        MessageBuilder::new()
            .id(1024)
            .name("NewMessage")
            .dlc(8)
            .sender("ECM")
            .add_signal(
                SignalBuilder::new()
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
                    .build()?,
            )
            .build()?,
    )
    .build()?;
```

### no_std Usage

```rust
use dbc_rs::Dbc;

// Parse from string slice or bytes
let dbc = Dbc::parse(dbc_content)?;
// or
let dbc = Dbc::parse_bytes(dbc_bytes)?;

// Access data (read-only in no_std, builders require std)
let version = dbc.version();
let messages = dbc.messages();
```

### File I/O (requires `std` feature)

```rust
use dbc_rs::Dbc;

// Read from file
let content = std::fs::read_to_string("example.dbc")?;
let dbc = Dbc::parse(&content)?;

// Save to file
let dbc_string = dbc.to_dbc_string();
std::fs::write("output.dbc", dbc_string)?;
```

### Error Handling

```rust
use dbc_rs::{Dbc, Error, ParseError};

match Dbc::parse(invalid_content) {
    Ok(dbc) => println!("Parsed: {} messages", dbc.messages().len()),
    Err(Error::ParseError(ParseError::Expected(msg))) => eprintln!("Parse error: Expected {}", msg),
    Err(Error::ParseError(ParseError::UnexpectedEof)) => eprintln!("Parse error: Unexpected end of input"),
    Err(Error::InvalidData(msg)) => eprintln!("Data error: {}", msg),
    Err(Error::Signal(msg)) => eprintln!("Signal error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security & Resource Limits

For security reasons (DoS protection), the library enforces capacity limits that prevent resource exhaustion. Default limits accommodate typical DBC files (typically < 1000 messages, < 10 nodes). These limits are configurable at build time (see [Build-Time Configuration](#build-time-configuration)). Attempting to exceed limits results in a validation error.

For a comprehensive security audit, see [SECURITY.md](SECURITY.md).

## Limitations

1. **Extended Features**: Advanced DBC features (attributes, value tables, structured comments, signal multiplexing, etc.) are not yet supported. Files containing these features parse successfully but extended data is lost on save.
2. **Comments**: Single-line `//` comments are parsed but not preserved when saving.
3. **Feature Restrictions**: 
   - Builders and file I/O require `std` feature
   - Without `std`: Use `parse()` or `parse_bytes()` for parsing only

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
3. **no_std First**: Designed for `no_std` environments, with optional `std` feature
4. **Zero Dependencies**: No external dependencies
5. **Memory Efficiency**: Supports both heap-allocated (`alloc`) and stack-allocated (`heapless`) collections

### Error Handling

- **Result-based**: All fallible operations return `Result<T>`
- **Categorized errors**: 
  - High-level (`std` only): `Error::Signal`, `Error::Message`, `Error::Dbc`, `Error::Version`, `Error::Nodes`, `Error::InvalidData`
  - Low-level (`no_std` compatible): `Error::ParseError(ParseError::...)` with variants like `Expected`, `UnexpectedEof`, `InvalidChar`
  - Other: `Error::Decoding`, `Error::Validation`
- **Feature-dependent**: With `std`, errors include detailed `String` messages; without it, static `&'static str` messages

## Build-Time Configuration

Capacity limits are configurable at build time via environment variables (generated by `build.rs`):

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `DBC_MAX_MESSAGES` | `10000` | Maximum number of messages per DBC file |
| `DBC_MAX_SIGNALS_PER_MESSAGE` | `64` | Maximum number of signals per message |
| `DBC_MAX_NODES` | `256` | Maximum number of nodes in the bus |
| `DBC_MAX_VALUE_DESCRIPTIONS` | `64` | Maximum number of value descriptions |
| `DBC_MAX_RECEIVER_NODES` | `64` | Maximum number of receiver nodes per signal |
| `DBC_MAX_NAME_SIZE` | `64` | Maximum length of names (messages, signals, nodes, etc.) |

**Example:**
```bash
# Reduce capacity limits for embedded targets (recommended for heapless)
DBC_MAX_MESSAGES=500 cargo build --release --no-default-features --features heapless --target thumbv7em-none-eabihf
```

**Performance Notes:**
- **`alloc`/`std`**: Heap-allocated, dynamic sizing
- **`heapless`**: Stack-allocated, fixed-size arrays (default 10000 messages may cause stack overflow on embedded; reduce to 100-500)
- **Parsing**: O(n) complexity, entire file parsed into memory

## Contributing

Contributions are welcome! Areas that need work:

- Value tables (VAL_TABLE_) - Note: VAL_ (value descriptions) is now supported
- Structured comments (CM_)
- Attributes (BA_DEF_, BA_, etc.)
- Environment variables (EV_)
- Signal multiplexing support

## License

dbc-rs is available under a **dual-licensing** model:

- **Open Source**: MIT OR Apache-2.0 (default) - See [LICENSING.md](LICENSING.md) for details
- **Commercial**: Available for proprietary use - See [LICENSING.md](LICENSING.md) for terms

For most users, the open-source license (MIT OR Apache-2.0) is sufficient.

## References

- [DBC Format Specification](SPECIFICATIONS.md) - Detailed format documentation
- [Security Audit](SECURITY.md) - Comprehensive security review
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
