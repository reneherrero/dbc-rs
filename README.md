# dbc-rs

A lightweight Rust library for seamlessly parsing and editing DBC (CAN Database) files, with robust support for encoding and decoding messages and signals.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSING.md)

## Features

- **Zero dependencies** - Pure Rust, no external runtime dependencies
- **no_std compatible** - Works on embedded targets (Cortex-M, RISC-V)
- **Flexible memory** - Heap (`alloc`) or stack (`heapless`) allocation
- **Memory safe** - `forbid(unsafe_code)` enforced
- **Full read/write** - Parse, modify, and serialize DBC files

## Quick Start

```toml
[dependencies]
dbc-rs = "0.3"
```

### Decode CAN Frames

```rust
use dbc_rs::Dbc;

// Parse DBC content
let dbc = Dbc::parse(dbc_content)?;

// Decode a CAN frame (ID 0x100, 8 bytes, standard 11-bit ID)
let signals = dbc.decode(0x100, &frame_data, false)?;
for signal in signals {
    println!("{}: {:.2} {}", signal.name, signal.value, signal.unit);
}
```

### Encode Signal Values

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(dbc_content)?;

// Encode signal values into a CAN frame payload (standard 11-bit ID)
let payload = dbc.encode(0x100, &[
    ("RPM", 2500.0),
    ("Temperature", 85.0),
], false)?;
```

### Embedded (`no_std`)

```toml
[dependencies]
dbc-rs = { version = "0.3", default-features = false, features = ["heapless"] }
```

See [`examples/`](./examples/) for complete working examples:
- `decode.rs` - Signal decoding basics
- `decode_frame.rs` - Using `embedded-can` Frame trait
- `encode.rs` - Encoding signals to CAN payloads
- `create_dbc.rs` - Building DBC files programmatically
- `parse_std.rs` / `parse_no_std.rs` - Parsing in different environments

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `std` | Standard library with builders | Yes |
| `alloc` | Heap allocation | Via `std` |
| `heapless` | Stack-only for `no_std` | No |
| `embedded-can` | `embedded-can` crate integration | No |

## Documentation

- [API Reference](https://docs.rs/dbc-rs)
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Internal design
- [SPECIFICATIONS.md](./SPECIFICATIONS.md) - DBC format reference
- [SECURITY.md](./SECURITY.md) - Security audit

## License

MIT OR Apache-2.0. See [LICENSING.md](./LICENSING.md).
