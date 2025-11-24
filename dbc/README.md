# dbc

A clean, zero-dependency DBC (CAN Database) file parser and editor for Rust.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/crates/l/dbc-rs.svg)](https://github.com/yourusername/dbc-rs/blob/main/LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/rustc-1.70.0+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/yourusername/dbc-rs/workflows/CI/badge.svg)](https://github.com/yourusername/dbc-rs/actions)

## Minimum Supported Rust Version (MSRV)

The minimum supported Rust version is **1.70.0**. The crate is tested against this version and may use features available in it or later.

## Features

- ✅ **Zero dependencies** - Pure Rust implementation
- ✅ **no_std + alloc support** - Works on embedded targets without the standard library
- ✅ **Full editing & writing** - Modify and save DBC files with the same structs
- ✅ **Feature flag control** - Optional `std` feature for desktop conveniences
- ✅ **Well tested** - Tested with real-world DBC files

## Quick Start

```rust
use dbc_rs::{Dbc, Message, Signal};

// Parse a DBC file
let content = std::fs::read_to_string("example.dbc").unwrap();
let mut dbc = Dbc::parse(&content).expect("invalid dbc");

// Access messages and signals (read-only)
if let Some(engine_msg) = dbc.messages.iter().find(|m| m.id == 256) {
    if let Some(rpm) = engine_msg.signals.iter().find(|s| s.name() == "RPM") {
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
| 29-bit Extended CAN IDs | ⚠️ | ⚠️ | Parsed as u32 but not validated |
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
- Extended CAN IDs: Supported in data structures but not validated
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
VERSION "1.0";

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C"

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
```


## Limitations

1. **Byte Order & Sign**: Currently, all signals are written with `@1+` (big-endian, unsigned) regardless of how they were parsed. The parser extracts these values but doesn't store them.

2. **Signal Receivers**: Receiver nodes in signal definitions are not parsed or preserved.

3. **Extended Features**: Many advanced DBC features (attributes, value tables, comments, etc.) are not yet supported. Files containing these features will parse successfully but the extended data will be lost on save.

4. **Validation**: Limited validation of CAN IDs, DLC values, and signal bit ranges.

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

