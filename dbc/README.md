# dbc

A clean, zero-dependency DBC (CAN Database) file parser and editor for Rust.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/crates/l/dbc-rs.svg)](https://github.com/reneherrero/dbc-rs/blob/main/LICENSING.md)
[![MSRV](https://img.shields.io/badge/rustc-1.74.0+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-rs%20Library%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-rs.yml)

## Minimum Supported Rust Version (MSRV)

The minimum supported Rust version is **1.74.0**. The crate is tested against this version and may use features available in it or later.

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

**Example (force std explicitly):**

```toml
[dependencies]
dbc-rs = { version = "1", features = ["std"] }
```

## Internationalization (i18n)

Error messages can be localized at build time using feature flags. The language is selected during compilation and cannot be changed at runtime.

### Supported Languages

The following languages are currently supported:

| Language | Feature Flag | Code |
|----------|-------------|------|
| English (default) | *(none)* | `en` |
| French | `lang-fr` | `fr` |
| Spanish | `lang-es` | `es` |
| German | `lang-de` | `de` |
| Japanese | `lang-ja` | `ja` |

### Selecting a Language

To use a specific language, enable the corresponding feature flag:

**Example (French):**

```toml
[dependencies]
dbc-rs = { version = "1", features = ["lang-fr"] }
```

**Example (German):**

```toml
[dependencies]
dbc-rs = { version = "1", features = ["lang-de"] }
```

**Example (Command line):**

```bash
cargo build --features lang-fr    # Build with French error messages
cargo build --features lang-ja    # Build with Japanese error messages
```

**Note:** Language features are mutually exclusive. If multiple language features are enabled, the last one in the feature list will be used. The default language (English) is used when no language feature is specified.

### Translation Status

⚠️ **Warning**: The translations have been generated and may contain errors or inaccuracies. They have not been fully verified by native speakers. If you encounter translation issues, please report them or contribute improvements (see Contributing section below).

### Contributing Translations

Contributions to improve existing translations or add new languages are welcome!

#### Updating Existing Translations

1. Navigate to `src/error/lang/` directory
2. Open the language file you want to update (e.g., `fr.rs` for French)
3. Edit the string constants with improved translations
4. Submit a pull request with your changes

#### Adding a New Language

1. Create a new file in `src/error/lang/` (e.g., `it.rs` for Italian)
2. Copy the structure from `en.rs` and translate all constants
3. Add the language module to `src/error/lang/mod.rs`:
   ```rust
   mod it;
   
   #[cfg(feature = "lang-it")]
   use it as lang;
   ```
4. Add the feature flag to `Cargo.toml`:
   ```toml
   lang-it = []  # Italian
   ```
5. Update this README with the new language in the Supported Languages table
6. Submit a pull request

**Translation Guidelines:**
- Maintain the same constant names across all language files
- Keep format placeholders (`{}`) in the same positions
- Ensure technical terms are accurately translated
- Test that error messages display correctly with the new language

## DBC Format Feature Support

This table shows which DBC file format features are currently implemented:

### Core Features

| Feature | Statement | Parse | Write | Notes |
|---------|-----------|-------|-------|-------|
| **Version** | `VERSION` | ✅ | ✅ | Database version string |
| **New Symbols** | `NS_` | ⚠️ | ❌ | Parsed but ignored (no errors) |
| **Bit Timing** | `BS_` | ⚠️ | ❌ | Parsed but ignored (no errors) |
| **Bus Nodes** | `BU_` | ✅ | ✅ | List of ECUs on the bus |
| **Messages** | `BO_` | ✅ | ✅ | CAN message definitions |
| **Signals** | `SG_` | ✅ | ✅ | Signal definitions (see limitations below) |
| **Comments** | `//` | ✅ | ❌ | Single-line comments parsed but not preserved |

### Signal Features (SG_)

| Feature | Parse | Write | Notes |
|---------|-------|-------|-------|
| Signal name | ✅ | ✅ | |
| Start bit | ✅ | ✅ | |
| Length (bits) | ✅ | ✅ | |
| Byte order (`@0`/`@1`) | ⚠️ | ❌ | Parsed but not stored; write always uses `@1+` |
| Sign (`+`/`-`) | ⚠️ | ❌ | Parsed but not stored; write always uses `+` |
| Factor (scaling) | ✅ | ✅ | |
| Offset | ✅ | ✅ | |
| Min value | ✅ | ✅ | |
| Max value | ✅ | ✅ | |
| Unit | ✅ | ✅ | |
| Receivers | ❌ | ❌ | Receiver nodes not parsed or written |

### Extended Features

| Feature | Statement | Parse | Write | Notes |
|---------|-----------|-------|-------|-------|
| **Value Tables** | `VAL_TABLE_` | ❌ | ❌ | Named enumeration tables |
| **Value Descriptions** | `VAL_` | ❌ | ❌ | Enum values for signals |
| **Comments** | `CM_` | ❌ | ❌ | Structured comments for messages/signals/nodes |
| **Attribute Definitions** | `BA_DEF_` | ❌ | ❌ | Custom attribute definitions |
| **Default Attributes** | `BA_DEF_DEF_` | ❌ | ❌ | Default values for attributes |
| **Attributes** | `BA_` | ❌ | ❌ | Attribute assignments |
| **Signal Groups** | `SIG_GROUP_` | ❌ | ❌ | Group related signals |
| **Signal Value Types** | `SIG_VALTYPE_` | ❌ | ❌ | Link signals to value tables |
| **Environment Variables** | `EV_` | ❌ | ❌ | Environment variable definitions |
| **Signal Type References** | `SIG_TYPE_REF_` | ❌ | ❌ | Reference signal types |
| **Signal Multiplexing** | `SG_MUL_VAL_` | ❌ | ❌ | Multiplexed signals |
| **Message Transmitters** | `BO_TX_BU_` | ❌ | ❌ | Multiple transmitters per message |
| **Node Relations** | `BU_SG_REL_`, `BU_EV_REL_`, `BU_BO_REL_` | ❌ | ❌ | Node relationship definitions |

### Advanced Features

| Feature | Parse | Write | Notes |
|---------|-------|-------|-------|
| 29-bit Extended CAN IDs | ✅ | ✅ | Parsed as u32 but not validated |
| Signal multiplexing | ❌ | ❌ | |
| Signal type definitions | ❌ | ❌ | |
| Category definitions | ❌ | ❌ | |
| Filters | ❌ | ❌ | |

## Implementation Status Summary

### ✅ Fully Implemented
- Basic DBC file structure (VERSION, BU_, BO_, SG_)
- Signal parsing with scaling, offset, min/max, units
- Message and signal editing
- Round-trip save/load for basic features

### ⚠️ Partially Implemented
- Signal byte order and sign: Parsed but not stored; always written as `@1+`
- Comments: Single-line `//` comments are ignored during parsing

### ❌ Not Yet Implemented
- Value tables and enumerations (VAL_TABLE_, VAL_)
- Structured comments (CM_)
- Attributes (BA_DEF_, BA_DEF_DEF_, BA_)
- Signal groups (SIG_GROUP_)
- Environment variables (EV_)
- Signal receivers in signal definitions
- Signal multiplexing
- Advanced node relationships

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

let content = std::fs::read_to_string("example.dbc")?;
let dbc = Dbc::parse(&content)?;

println!("Version: {}", dbc.version().to_string());
println!("Nodes: {}", dbc.nodes().to_string());
println!("Number of messages: {}", dbc.messages().len());
```

### Error Handling

```rust
use dbc_rs::{Dbc, Error};

match Dbc::parse(invalid_content) {
    Ok(dbc) => println!("Parsed successfully: {} messages", dbc.messages().len()),
    Err(Error::InvalidData(msg)) => eprintln!("Data error: {}", msg),
    Err(Error::Signal(msg)) => eprintln!("Signal error: {}", msg),
}
```

### Round-Trip: Parse, Modify, Save

```rust
use dbc_rs::{Dbc, Version, Nodes, Message, Signal, ByteOrder, Receivers};

// Parse existing DBC
let dbc = Dbc::parse(&content)?;

// Create new signal
let new_signal = Signal::new(
    "NewSignal",
    32,
    8,
    ByteOrder::BigEndian,
    true,
    1.0,
    0.0,
    0.0,
    255.0,
    Some("unit"),
    Receivers::Broadcast,
)?;

// Create new message with the signal
let new_message = Message::new(
    1024,
    "NewMessage",
    8,
    "ECM",
    vec![new_signal],
)?;

// Note: Dbc doesn't have a mutable API yet, so you'd need to:
// 1. Extract all data via getters
// 2. Create new Dbc with modified data
let version = dbc.version();
let nodes = dbc.nodes();
let mut messages: Vec<Message> = dbc.messages().to_vec();
messages.push(new_message);

let modified_dbc = Dbc::new(version.clone(), nodes.clone(), messages)?;

// Save back to DBC format
let saved_content = modified_dbc.save();
std::fs::write("modified.dbc", saved_content)?;
```

### Finding Messages and Signals

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;

// Find message by ID
let engine_msg = dbc.messages().iter().find(|m| m.id() == 256);

// Find signal by name in a specific message
if let Some(msg) = engine_msg {
    let rpm_signal = msg.find_signal("RPM");
    if let Some(sig) = rpm_signal {
        println!("RPM: factor={}, offset={}", sig.factor(), sig.offset());
    }
}

// Iterate all messages and signals
for message in dbc.messages() {
    println!("Message {} (ID: {})", message.name(), message.id());
    for signal in message.signals() {
        println!("  Signal: {} ({} bits)", signal.name(), signal.length());
    }
}
```

### Validation Example

```rust
use dbc_rs::{Message, Signal, ByteOrder, Receivers};

// This will fail validation - signal overlaps
let signal1 = Signal::new("Signal1", 0, 16, ByteOrder::BigEndian, true, 1.0, 0.0, 0.0, 100.0, None, Receivers::None)?;
let signal2 = Signal::new("Signal2", 8, 16, ByteOrder::BigEndian, true, 1.0, 0.0, 0.0, 100.0, None, Receivers::None)?;

// This will return an error due to signal overlap
match Message::new(256, "Test", 8, "ECM", vec![signal1, signal2]) {
    Ok(_) => println!("Message created successfully"),
    Err(e) => println!("Validation failed: {}", e),
}
```

### no_std Usage

```rust
// In a no_std environment (with alloc)
use dbc_rs::Dbc;

// Parse from a byte slice (no file I/O needed)
let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
let dbc = Dbc::parse_bytes(dbc_bytes)?;

// Access data
let version = dbc.version();
let messages = dbc.messages();

// Save to string (can be written to storage later)
let saved = dbc.save();
```

### Working with Different Input Sources

```rust
use dbc_rs::Dbc;

// From string slice
let dbc1 = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;

// From bytes
let bytes = b"VERSION \"1.0\"...";
let dbc2 = Dbc::parse_bytes(bytes)?;

// From String
let string = String::from("VERSION \"1.0\"...");
let dbc3 = Dbc::parse_from(string)?;

// From std::io::Read (requires std feature)
#[cfg(feature = "std")]
{
    use std::fs::File;
    let file = File::open("example.dbc")?;
    let dbc4 = Dbc::from_reader(file)?;
}
```

## Limitations

1. **Byte Order & Sign**: Currently, all signals are written with `@1+` (big-endian, unsigned) regardless of how they were parsed. The parser extracts these values but doesn't store them.

2. **Signal Receivers**: Receiver nodes in signal definitions are not parsed or preserved.

3. **Extended Features**: Many advanced DBC features (attributes, value tables, comments, etc.) are not yet supported. Files containing these features will parse successfully but the extended data will be lost on save.

4. **Validation**: Limited validation of CAN IDs, DLC values, and signal bit ranges.

## Usage Patterns

### Common Workflows

#### Parse and Inspect

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;

// Get version information
let version = dbc.version();
println!("DBC Version: {}", version.to_string());

// List all nodes
let nodes = dbc.nodes();
println!("Nodes: {}", nodes.to_string());

// Iterate messages
for message in dbc.messages() {
    println!("Message: {} (ID: 0x{:X}, DLC: {})", 
             message.name(), message.id(), message.dlc());
    
    // Iterate signals in message
    for signal in message.signals() {
        println!("  Signal: {} ({} bits, factor: {}, offset: {})",
                 signal.name(), signal.length(), signal.factor(), signal.offset());
    }
}
```

#### Find Specific Data

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;

// Find message by ID
let engine_msg = dbc.messages().iter().find(|m| m.id() == 256);

// Find signal by name
if let Some(msg) = engine_msg {
    if let Some(rpm) = msg.find_signal("RPM") {
        // Calculate physical value from raw CAN data
        let raw_value: u16 = 4000; // Example raw CAN value
        let physical_value = (raw_value as f64 * rpm.factor()) + rpm.offset();
        println!("RPM: {:.2} {}", physical_value, rpm.unit().unwrap_or(""));
    }
}
```

#### Create and Modify DBC Files

```rust
use dbc_rs::{Dbc, Version, Nodes, Message, Signal, ByteOrder, Receivers};

// Create from scratch
let version = Version::new(1, Some(0), None)?;
let nodes = Nodes::new(&["ECM", "TCM", "BCM"]);

let signal = Signal::new(
    "EngineSpeed",
    0,
    16,
    ByteOrder::BigEndian,
    true,
    0.25,
    0.0,
    0.0,
    8000.0,
    Some("rpm"),
    Receivers::Broadcast,
)?;

let message = Message::new(256, "EngineData", 8, "ECM", vec![signal])?;
let dbc = Dbc::new(version, nodes, vec![message])?;

// Save to string
let dbc_string = dbc.save();
```

#### Modify Existing DBC

```rust
use dbc_rs::{Dbc, Message, Signal, ByteOrder, Receivers};

let dbc = Dbc::parse(&content)?;

// Extract current data
let version = dbc.version();
let nodes = dbc.nodes();
let mut messages: Vec<Message> = dbc.messages().to_vec();

// Add new message
let new_signal = Signal::new(
    "NewSignal",
    0,
    8,
    ByteOrder::BigEndian,
    true,
    1.0,
    0.0,
    0.0,
    255.0,
    None,
    Receivers::Broadcast,
)?;

let new_message = Message::new(1024, "NewMessage", 8, "ECM", vec![new_signal])?;
messages.push(new_message);

// Create modified DBC
let modified_dbc = Dbc::new(version.clone(), nodes.clone(), messages)?;
let saved = modified_dbc.save();
```

### Best Practices

1. **Always handle errors**: Use `?` operator or `match` to handle parsing errors gracefully
2. **Validate before creating**: Use constructors (`new()`) which include validation
3. **Use getters**: Access data through getter methods, not direct field access
4. **Clone when needed**: Clone `Version` and `Nodes` when creating new `Dbc` instances
5. **Check signal ranges**: Verify signal min/max values match your application requirements

### Anti-Patterns to Avoid

❌ **Don't**: Access fields directly (they're private)
```rust
// This won't compile
let id = message.id; // Error: field is private
```

✅ **Do**: Use getter methods
```rust
let id = message.id(); // Correct
```

❌ **Don't**: Ignore validation errors
```rust
// This might panic or create invalid data
let signal = Signal::new(/* invalid params */).unwrap();
```

✅ **Do**: Handle errors properly
```rust
match Signal::new(/* params */) {
    Ok(signal) => { /* use signal */ },
    Err(e) => eprintln!("Validation failed: {}", e),
}
```

## Performance Notes

### Memory Usage

- **String Storage**: Uses `Box<str>` for strings to reduce memory overhead compared to `String`
- **Pre-allocation**: Vectors are pre-allocated with estimated capacity to reduce reallocations
- **No String Interning**: Each string is stored independently (no deduplication)

### Parsing Performance

- **Linear Complexity**: Parsing is O(n) where n is the file size
- **No Streaming**: Entire file is parsed into memory (suitable for typical DBC file sizes)
- **Validation**: Validation occurs during parsing and construction, adding minimal overhead

### Recommendations

- For very large DBC files (>10MB), consider splitting into multiple files
- If parsing many files, reuse `Dbc` instances when possible
- Use `parse_bytes()` instead of `parse()` if you already have bytes to avoid UTF-8 validation

## Troubleshooting

### Common Issues

#### "Message ID out of valid range"

**Problem**: CAN ID exceeds maximum allowed value (0x1FFFFFFF for extended IDs)

**Solution**: 
- Standard 11-bit IDs: Use 0-0x7FF (0-2047)
- Extended 29-bit IDs: Use 0x800-0x1FFFFFFF (2048-536870911)

```rust
// ❌ Invalid
let msg = Message::new(0x20000000, "Test", 8, "ECM", vec![])?; // Too large

// ✅ Valid
let msg = Message::new(0x1FFFFFFF, "Test", 8, "ECM", vec![])?; // Max extended ID
```

#### "Signal extends beyond CAN message"

**Problem**: Signal start_bit + length exceeds message size (DLC * 8 bits)

**Solution**: Ensure signals fit within message boundaries

```rust
// ❌ Invalid: Signal extends to bit 64, but DLC=8 means max bit is 63
let sig = Signal::new("Test", 56, 16, ...)?; // 56 + 16 = 72 > 64
let msg = Message::new(256, "Test", 8, "ECM", vec![sig])?;

// ✅ Valid: Signal fits within 8-byte message (0-63)
let sig = Signal::new("Test", 0, 64, ...)?; // 0 + 64 = 64 (fits in 8 bytes)
let msg = Message::new(256, "Test", 8, "ECM", vec![sig])?;
```

#### "Signal overlap detected"

**Problem**: Multiple signals occupy overlapping bit ranges

**Solution**: Ensure signals don't overlap

```rust
// ❌ Invalid: Signals overlap at bits 8-15
let sig1 = Signal::new("Signal1", 0, 16, ...)?;  // Bits 0-15
let sig2 = Signal::new("Signal2", 8, 16, ...)?; // Bits 8-23 (overlaps!)

// ✅ Valid: Signals don't overlap
let sig1 = Signal::new("Signal1", 0, 16, ...)?;  // Bits 0-15
let sig2 = Signal::new("Signal2", 16, 16, ...)?; // Bits 16-31 (no overlap)
```

#### "Sender not in nodes"

**Problem**: Message sender is not listed in the nodes

**Solution**: Add the sender to the nodes list

```rust
// ❌ Invalid: "ECM" not in nodes
let nodes = Nodes::new(&["TCM"]);
let msg = Message::new(256, "Test", 8, "ECM", vec![])?;
let dbc = Dbc::new(version, nodes, vec![msg])?; // Error!

// ✅ Valid: "ECM" is in nodes
let nodes = Nodes::new(&["ECM", "TCM"]);
let msg = Message::new(256, "Test", 8, "ECM", vec![])?;
let dbc = Dbc::new(version, nodes, vec![msg])?; // OK
```

#### "Duplicate message ID"

**Problem**: Multiple messages have the same CAN ID

**Solution**: Use unique CAN IDs for each message

```rust
// ❌ Invalid: Duplicate IDs
let msg1 = Message::new(256, "Message1", 8, "ECM", vec![])?;
let msg2 = Message::new(256, "Message2", 8, "TCM", vec![])?; // Same ID!

// ✅ Valid: Unique IDs
let msg1 = Message::new(256, "Message1", 8, "ECM", vec![])?;
let msg2 = Message::new(512, "Message2", 8, "TCM", vec![])?; // Different IDs
```

### Debugging Tips

1. **Enable verbose error messages**: Error messages include context about what failed
2. **Check validation**: Use constructors (`new()`) which validate input
3. **Verify DBC format**: Ensure your DBC file follows the correct format
4. **Test with minimal examples**: Start with simple DBC files to isolate issues

## Architecture & Design

### Design Principles

1. **Immutability**: All data structures are immutable after creation (read-only access)
2. **Validation**: Input validation occurs at construction time, not at use time
3. **no_std First**: Designed to work in `no_std` environments with `alloc`
4. **Zero Dependencies**: No external dependencies to keep the library lightweight
5. **Memory Efficiency**: Uses `Box<str>` and pre-allocated vectors to minimize memory usage

### Module Structure

```
dbc-rs/
├── lib.rs          # Main library entry point, re-exports
├── dbc.rs          # DBC file structure and parsing
├── message.rs      # CAN message definitions
├── signal.rs       # Signal definitions with validation
├── nodes.rs        # Node/ECU management
├── version.rs      # Version string parsing
└── error/
    ├── mod.rs      # Error types
    ├── messages.rs # Error message formatting
    └── lang/       # Internationalized error messages
        ├── en.rs   # English (default)
        ├── fr.rs   # French
        ├── es.rs   # Spanish
        ├── de.rs   # German
        └── ja.rs   # Japanese
```

### Data Flow

1. **Parsing**: `Dbc::parse()` → tokenizes input → validates → creates structures
2. **Construction**: `new()` methods → validate input → create immutable structures
3. **Access**: Getter methods provide read-only access to internal data
4. **Serialization**: `save()` / `to_dbc_string()` → convert structures back to DBC format

### Validation Strategy

- **Parse-time validation**: Basic format validation during parsing
- **Construction-time validation**: Comprehensive validation in `new()` methods
- **Shared validation**: Same validation logic used in both parsing and construction
- **Early failure**: Validation errors are returned immediately, not deferred

### Error Handling

- **Result-based**: All fallible operations return `Result<T, Error>`
- **Categorized errors**: `Error::InvalidData` for parsing issues, `Error::Signal` for signal-specific issues
- **Internationalized**: Error messages can be localized at build time
- **Descriptive**: Error messages include context about what failed and why

## Contributing

Contributions are welcome! Areas that need work:

- Full byte order and sign support in signals
- Signal receiver parsing
- Value tables and enumerations (VAL_TABLE_, VAL_)
- Structured comments (CM_)
- Attributes (BA_DEF_, BA_, etc.)
- Environment variables (EV_)
- Signal multiplexing support
- **Translation improvements** - Help verify and improve error message translations (see Internationalization section above)

## License

dbc-rs is available under a **dual-licensing** model:

- **Open Source**: MIT OR Apache-2.0 (default) - See [LICENSING.md](LICENSING.md) for details
- **Commercial**: Available for proprietary use - See [LICENSING.md](LICENSING.md) for terms

For most users, the open-source license (MIT OR Apache-2.0) is sufficient. Commercial licenses are available for organizations that need additional flexibility or legal protection.

## References

- [DBC Format Specification](DBC_FORMAT_SPEC.md) - Detailed format documentation
- Vector Informatik: "DBC File Format Documentation Version 01/2007"
- CAN Specification (ISO 11898)

### Test DBC Files

The test suite includes several DBC files in `tests/data/`:

- **`complete.dbc`** - Comprehensive test file with multiple messages, signals, and extended DBC features (attributes, value tables, etc.). Created for testing parser robustness with real-world-like complexity.
- **`simple.dbc`** - Basic 2-message file for testing core parsing functionality.
- **`multiplexed.dbc`** - Tests multiple sensors and actuators with various signal types.
- **`minimal.dbc`** - Minimal valid DBC file for testing edge cases.
- **`extended_ids.dbc`** - Tests higher message IDs and small signal sizes.
- **`broadcast_signals.dbc`** - Tests broadcast receivers (`*`) and specific receiver nodes.

All test files are custom-created for this project to ensure comprehensive test coverage. For additional DBC files, consider:

- [commaai/opendbc](https://github.com/commaai/opendbc) - Open-source collection of DBC files for various vehicle models
- [CSS Electronics DBC Editor](https://www.csselectronics.com/pages/dbc-editor-can-bus-database) - Sample DBC files and documentation

