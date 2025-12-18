# dbc-rs

A clean, zero-dependency DBC (CAN Database) file parser and editor for Rust.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/crates/l/dbc-rs.svg)](https://github.com/reneherrero/dbc-rs/blob/main/dbc/LICENSING.md)
[![MSRV](https://img.shields.io/badge/rustc-1.85.0+-blue.svg)](https://www.rust-lang.org)
[![CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-rs%20Library%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-rs.yml)

## Features

- ✅ **Zero dependencies** with `alloc`/`std` features (optional `heapless` for embedded)
- ✅ **no_std support** - Works on embedded targets
- ✅ **Full editing & writing** - Modify and save DBC files
- ✅ **Well tested** - Tested with real-world DBC files

For design principles, module structure, and internal details, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Quick Start

```rust
use dbc_rs::Dbc;
let content = std::fs::read_to_string("example.dbc").unwrap();
let dbc = Dbc::parse(&content).expect("invalid dbc");

if let Some(engine_msg) = dbc.messages().iter().find(|m| m.id() == 256) {
    if let Some(rpm) = engine_msg.signals().iter().find(|s| s.name() == "RPM") {
        println!("RPM factor: {}", rpm.factor());
    }
}
```

## Feature Flags

**⚠️ Important:** You **MUST** enable exactly one of: `std`, `alloc`, or `heapless`. See [ARCHITECTURE.md](ARCHITECTURE.md#feature-flags) for details.

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | ✅ | Full std library support (includes `alloc`). Zero dependencies. |
| `alloc` | ❌ | Heap allocation without std. Requires global allocator. Zero dependencies. |
| `heapless` | ❌ | Stack allocation, no allocator needed. One dependency: [`heapless`](https://crates.io/crates/heapless). |
| `embedded-can` | ❌ | `decode_frame()` via [`embedded-can`](https://crates.io/crates/embedded-can) `Frame` trait. |

**Cargo.toml examples:**
```toml
# Default: std enabled
dbc-rs = "1"

# no_std with heap allocation
dbc-rs = { version = "1", default-features = false, features = ["alloc"] }

# no_std with stack allocation
dbc-rs = { version = "1", default-features = false, features = ["heapless"] }

# With embedded-can Frame decoding support
dbc-rs = { version = "1", features = ["embedded-can"] }

# no_std with stack allocation + embedded-can (embedded targets)
dbc-rs = { version = "1", default-features = false, features = ["heapless", "embedded-can"] }
```

## DBC Format Support

### Core Features ✅
- **Version** (`VERSION`), **Nodes** (`BU_`), **Messages** (`BO_`), **Signals** (`SG_`), **Value Descriptions** (`VAL_`)
- All signal features: start bit, length, byte order, sign, factor, offset, min/max, unit, receivers
- Extended CAN IDs (bit 31 flag per DBC spec)
- **Signal multiplexing**: Basic (`M`, `m0`, `m1`) and extended (`SG_MUL_VAL_`)

### Limitations ❌
Not implemented: Value tables (`VAL_TABLE_`), structured comments (`CM_`), attributes (`BA_*`), signal groups, environment variables (`EV_`).

**Note:** `NS_` and `BS_` are parsed but ignored. Single-line `//` comments are parsed but not preserved on save.

## Examples

### Basic Parsing (works with either `std`, `alloc`, or `heapless`)

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;
println!("Messages: {}", dbc.messages().len());
```

### Creating DBC Files

```rust
use dbc_rs::{ByteOrder, DbcBuilder, MessageBuilder, NodesBuilder, ReceiversBuilder};
use dbc_rs::{SignalBuilder, VersionBuilder};

let dbc = DbcBuilder::new()
    .version(VersionBuilder::new().version("1.0"))
    .nodes(NodesBuilder::new().add_node("ECM"))
    .add_message(
        MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(
                SignalBuilder::new()
                    .name("EngineSpeed")
                    .start_bit(0)
                    .length(16)
                    .byte_order(ByteOrder::BigEndian)
                    .unsigned(true)
                    .factor(0.25)
                    .offset(0.0)
                    .min(0.0)
                    .max(16000.0)
                    .unit("rpm")
                    .receivers(ReceiversBuilder::new().none())
            )
    )
    .build()?;

let dbc_string = dbc.to_dbc_string();
```

### Decoding CAN Messages

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&content)?;

// Decode a standard CAN message (11-bit ID)
let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];
let decoded = dbc.decode(256, &payload, false)?;

for signal in decoded.iter() {
    println!("{}: {} {:?}", signal.name, signal.value, signal.unit);
}

// Decode an extended CAN message (29-bit ID)
let decoded_ext = dbc.decode(0x400, &payload, true)?;
```

With the `embedded-can` feature, you can decode frames directly:

```rust,ignore
use dbc_rs::Dbc;

let decoded = dbc.decode_frame(can_frame)?;
```

### Error Handling

```rust
use dbc_rs::{Dbc, Error};

match Dbc::parse(invalid_content) {
    Ok(dbc) => println!("Parsed: {} messages", dbc.messages().len()),
    Err(Error::Expected(msg)) => eprintln!("Expected {}", msg),
    Err(Error::UnexpectedEof) => eprintln!("Unexpected end of input"),
    Err(Error::Validation(msg)) => eprintln!("Validation error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security & Limits

Capacity limits prevent resource exhaustion (DoS protection). Limits are configurable at build time via environment variables. See [ARCHITECTURE.md](ARCHITECTURE.md#build-time-configuration) for the full list.

```bash
# Example: Reduce limits for embedded targets
DBC_MAX_MESSAGES=512 cargo build --no-default-features --features heapless --target thumbv7em-none-eabihf
```

**Note:** With `heapless`, most limits must be powers of 2.

## Troubleshooting

- **"Message ID out of valid range"**: Standard 11-bit (0-0x7FF) or Extended 29-bit with bit 31 set (0x80000000-0x9FFFFFFF)
- **"Signal extends beyond message"**: Ensure `start_bit + length <= DLC * 8`
- **"Signal overlap"**: Signals must not occupy overlapping bit ranges
- **"Sender not in nodes"**: Add message sender to nodes list
- **"Duplicate message ID"**: Use unique CAN IDs

## Contributing

Contributions welcome! Areas needing work: Value tables, structured comments, attributes, environment variables, signal multiplexing.

## License

Available under **MIT OR Apache-2.0** (open source) or commercial licensing. See [LICENSING.md](LICENSING.md) for details.

## References

- [Architecture](ARCHITECTURE.md) - Internal design and module structure
- [DBC Format Specification](SPECIFICATIONS.md)
- [Security Audit](SECURITY.md)
- Vector Informatik: "DBC File Format Documentation Version 01/2007"
- [commaai/opendbc](https://github.com/commaai/opendbc) - Open-source DBC files
