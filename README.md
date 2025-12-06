# dbc-rs

[![dbc-rs CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-rs%20Library%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-rs.yml)
[![dbc-cli CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-cli%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-cli.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/reneherrero/dbc-rs/blob/main/LICENSING.md)

A Rust workspace containing the DBC (CAN Database) file parser library and command-line tools.

## Projects

### [`dbc-rs`](./dbc/)

The core library for parsing, editing, and writing DBC files. Supports both `std` and `no_std` environments.

**Features:**
- ✅ **Zero dependencies** - Pure Rust implementation
- ✅ **no_std + alloc support** - Works on embedded targets without the standard library
- ✅ **Full editing & writing** - Modify and save DBC files with the same structs
- ✅ **Feature flag control** - Optional `std` feature for desktop conveniences
- ✅ **Internationalized errors** - Build-time language selection (English, French, Spanish, German, Japanese)
- ✅ **Comprehensive validation** - Signal overlap detection, boundary checking, and more
- ✅ **Well tested** - Tested with real-world DBC files

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

### Code Coverage

Code coverage is tracked automatically in CI. To check coverage locally:

```bash
# Install cargo-llvm-cov using prebuilt binary (recommended)
host=$(rustc -vV | grep '^host:' | cut -d' ' -f2)
curl --proto '=https' --tlsv1.2 -fsSL \
  "https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$host.tar.gz" \
  | tar xzf - -C "$HOME/.cargo/bin"

# Generate coverage report
cargo llvm-cov --all-features --workspace

# Generate HTML report (opens in browser)
cargo llvm-cov --all-features --workspace --html
```

**Note**: Prebuilt binaries are recommended because `cargo install` may fail with MSRV (1.85.0). CI uses latest stable Rust and works automatically.

The project aims for at least 80% code coverage. Coverage reports are automatically generated in CI and posted as PR comments.

## Security

For a comprehensive security audit, see [SECURITY.md](dbc/SECURITY.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

For commercial licensing, see [LICENSING.md](LICENSING.md).

