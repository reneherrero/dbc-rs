# dbc-rs

A production-ready Rust workspace for parsing, editing, and manipulating DBC (CAN Database) files. Designed for both embedded systems and desktop applications.

[![Crates.io](https://img.shields.io/crates/v/dbc-rs.svg)](https://crates.io/crates/dbc-rs)
[![Documentation](https://docs.rs/dbc-rs/badge.svg)](https://docs.rs/dbc-rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/reneherrero/dbc-rs/blob/main/dbc/LICENSING.md)
[![MSRV](https://img.shields.io/badge/rustc-1.85.0+-blue.svg)](https://www.rust-lang.org)
[![dbc-rs CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-rs%20Library%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-rs.yml)
[![dbc-cli CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-cli%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-cli.yml)

## Table of Contents

- [Overview](#overview)
- [Key Advantages](#key-advantages)
- [Projects](#projects)
  - [DBC Core Library - `dbc-rs`](#dbc-core-library---dbc-rs)
  - [Command-Line Interface - `dbc-cli`](#command-line-interface---dbc-cli)
- [Quick Start](#quick-start)
  - [Installation](#installation)
  - [Basic Usage](#basic-usage)
  - [Embedded Usage (no_std)](#embedded-usage-no_std)
  - [Creating and Editing DBC Files](#creating-and-editing-dbc-files)
- [Building](#building)
- [Testing](#testing)
- [Code Coverage](#code-coverage)
- [Security](#security)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

## Overview

dbc-rs provides a comprehensive, type-safe solution for working with DBC files in Rust. The library is designed with embedded systems in mind, offering full `no_std` support while maintaining compatibility with standard library environments.

## Key Advantages

| Feature | dbc-rs | Typical Alternatives |
|---------|--------|---------------------|
| **Dependencies** | Zero dependencies (pure Rust) | Often includes heavy dependencies |
| **no_std Support** | Full support with `alloc` and `heapless` features | Limited or no support |
| **Memory Model** | Flexible: heap (`alloc`) or stack (`heapless`) allocation | Typically heap-only |
| **Memory Safety** | `forbid(unsafe_code)` - guaranteed safety | Varies by implementation |
| **Type Safety** | Strong typing throughout | Varies by implementation |
| **Edit & Write** | Full read/write support | Often read-only |

## Projects

### DBC Core Library - [`dbc-rs`](./dbc/)

A flexible DBC parser and editor that works across environments, from microcontrollers to servers.

```rust
use dbc_rs::Dbc;

// Parse DBC files in any environment
let dbc = Dbc::parse(dbc_content)?;

// Decode a real CAN frame from your vehicle's ECU
let can_id = 0x100;  // Engine data message ID
let can_payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];

// Decode all signals in the message at once (false = standard 11-bit ID)
let decoded = dbc.decode(can_id, &can_payload, false)?;

// Process decoded signals with proper units
for signal in decoded {
    match signal.name {
        "RPM" => println!("Engine RPM: {:.0} {}", signal.value, signal.unit.unwrap_or("")),
        "EngineTemp" => println!("Temperature: {:.1}{}", signal.value, signal.unit.unwrap_or("")),
        _ => println!("{}: {:.2} {}", signal.name, signal.value, signal.unit.unwrap_or("")),
    }
}
```

**Key Features:**
- **Zero dependencies** - Pure Rust implementation with no external runtime dependencies
- **no_std compatible** - Works seamlessly on Cortex-M, RISC-V, and other embedded targets
- **Flexible memory allocation** - Choose heap (`alloc`) or stack (`heapless`) allocation based on your platform constraints
- **Memory safety** - `forbid(unsafe_code)` ensures guaranteed safety at compile time
- **Full editing capabilities** - Parse, modify, and save DBC files with complete round-trip support
- **Comprehensive testing** - 190+ tests with 80%+ code coverage, validated with real-world DBC files
- **Security audited** - DoS protection, resource limits, and comprehensive security review

**Use Cases:**
- Automotive CAN bus applications
- Embedded CAN tools and analyzers
- CAN data visualization and analysis
- DBC file manipulation and conversion tools
- Educational and research projects

### Command-Line Interface - [`dbc-cli`](./dbc-cli/)

A command-line tool for working with DBC files. Aims to be a `candump` replacement in Rust with scripting support (Python FFI).

**Features:**
- Version information
- DBC file manipulation (coming soon)
- CAN message decoding
- Scripting support

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dbc-rs = "0.1.0-rc.3"
```

For embedded targets without standard library:

```toml
[dependencies]
dbc-rs = { version = "0.1.0-rc.3", default-features = false, features = ["heapless"] }
```


### Basic Usage

Parse a DBC file:

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse(&std::fs::read_to_string("example.dbc")?)?;

// Access messages
for message in dbc.messages().iter() {
    println!("Message: {} (ID: {})", message.name(), message.id());
    
    // Access signals
    for signal in message.signals().iter() {
        println!("  Signal: {} - {} {}", 
            signal.name(), 
            signal.factor(), 
            signal.unit().unwrap_or("")
        );
    }
}
```

### Embedded Usage (no_std)

```rust
// Works on embedded targets without std
let dbc = Dbc::parse_bytes(can_dbc_bytes)?;
let version = dbc.version();
let messages = dbc.messages();
```

### Creating and Editing DBC Files

```rust
use dbc_rs::{DbcBuilder, MessageBuilder, SignalBuilder, ByteOrder, VersionBuilder};

let dbc = DbcBuilder::new()
    .version(VersionBuilder::new().version("2.0"))
    .add_message(
        MessageBuilder::new()
            .id(256)
            .name("EngineData")
            .dlc(8)
            .sender("ECM")
            .add_signal(
                SignalBuilder::new()
                    .name("RPM")
                    .start_bit(0)
                    .length(16)
                    .byte_order(ByteOrder::BigEndian)
                    .factor(0.25)
                    .build()?
            )
            .build()?
    )
    .build()?;

// Serialize to DBC format
let dbc_string = dbc.to_dbc_string();
```

## Building

Build all workspace members:

```bash
cargo build
```

Build for embedded targets (no_std):

```bash
# For embedded targets, reduce MAX_MESSAGES to avoid stack overflow
DBC_MAX_MESSAGES=512 cargo build --no-default-features --features heapless --target thumbv7em-none-eabihf -p dbc-rs
```

**Note:** The `-p dbc-rs` flag ensures only the library is built (excluding `dbc-cli` which requires `std`). The `DBC_MAX_MESSAGES` environment variable reduces stack allocation for memory-constrained targets.

## Testing

Run all tests:

```bash
cargo test --workspace
```

Test specific feature configurations:

```bash
# Test with alloc feature
cargo test --no-default-features --features alloc -p dbc-rs

# Test with heapless feature (requires increased stack size)
RUST_MIN_STACK=8388608 cargo test --no-default-features --features heapless -p dbc-rs --lib
```

## Code Coverage

The project maintains **80%+ code coverage** across all feature configurations. Coverage is tracked automatically in CI and detailed reports are posted as PR comments.

To check coverage locally:

```bash
# Install cargo-llvm-cov using prebuilt binary (recommended)
host=$(rustc -vV | grep '^host:' | cut -d' ' -f2)
curl --proto '=https' --tlsv1.2 -fsSL \
  "https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$host.tar.gz" \
  | tar xzf - -C "$HOME/.cargo/bin"

# Generate coverage report
cargo llvm-cov --workspace --fail-under-lines 80

# Generate HTML report
cargo llvm-cov --workspace --html
```

**Note**: Prebuilt binaries are recommended because `cargo install` may fail with MSRV (1.85.0). CI uses latest stable Rust and works automatically.

## Security

dbc-rs takes security seriously. The library has undergone comprehensive security review and implements multiple layers of protection:

- **Comprehensive security audit** - See [SECURITY.md](dbc/SECURITY.md) for detailed analysis
- **DoS protection** - Resource limits prevent exhaustion attacks on all collections and strings
- **Memory safety** - Zero `unsafe` code throughout the codebase—verified by automated checks
- **Input validation** - All inputs validated at construction time with clear error messages

## Documentation

- **[API Reference](https://docs.rs/dbc-rs)** - Complete API documentation
- **[dbc/README.md](./dbc/README.md)** - Library usage, feature flags, examples
- **[dbc/ARCHITECTURE.md](./dbc/ARCHITECTURE.md)** - Internal design, module structure, build configuration
- **[dbc-cli/README.md](./dbc-cli/README.md)** - CLI tool documentation
- **[CONTRIBUTING.md](./CONTRIBUTING.md)** - Contribution guidelines

## Contributing

Contributions are welcome and appreciated. Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines and development setup.

**Areas that would benefit from contributions:**
- Value tables (`VAL_TABLE_`)
- Structured comments (`CM_`)
- Attributes (`BA_DEF_`, `BA_`, etc.)
- Signal groups (`SIG_GROUP_`)
- Environment variables (`EV_`)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](dbc/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](dbc/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

For commercial licensing options, see [LICENSING.md](dbc/LICENSING.md).
