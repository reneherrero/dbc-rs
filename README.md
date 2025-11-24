# dbc-rs Workspace

[![CI](https://github.com/yourusername/dbc-rs/workflows/CI/badge.svg)](https://github.com/yourusername/dbc-rs/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yourusername/dbc-rs/blob/main/LICENSE-MIT)

A Rust workspace containing the DBC (CAN Database) file parser library and command-line tools.

## Projects

### [`dbc`](./dbc/)

The core library for parsing, editing, and writing DBC files. Supports both `std` and `no_std` environments.

**Features:**
- Zero dependencies
- `no_std` + `alloc` support
- Full DBC file parsing and writing
- Internationalization support (i18n)
- Comprehensive validation

### [`dbc-cli`](./dbc-cli/)

Command-line interface for working with DBC files.

**Features:**
- Version information
- DBC file manipulation (coming soon)

## Quick Start

### Using the Library

Add to your `Cargo.toml`:

```toml
[dependencies]
dbc-rs = { path = "./dbc" }
```

### Using the CLI

```bash
cargo run --bin dbc-cli -- version
```

## Building

Build all workspace members:

```bash
cargo build
```

Build without default features (no_std):

```bash
cargo build --no-default-features
```

## Testing

Run all tests:

```bash
cargo test
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](dbc/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](dbc/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

For commercial licensing, see [LICENSING.md](dbc/LICENSING.md).

