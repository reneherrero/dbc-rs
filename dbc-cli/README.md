# dbc-cli

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSING.md)
[![CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-cli%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-cli.yml)

Command-line interface for DBC (CAN Database) file manipulation.

## Installation

### From Source

```bash
cargo install --path dbc-cli
```

### From Crates.io (when published)

```bash
cargo install dbc-cli
```

## Usage

### Version

Print version information:

```bash
dbc-cli version
```

Or simply:

```bash
dbc-cli
```

## Development

Build the CLI:

```bash
cargo build --bin dbc-cli
```

Run the CLI:

```bash
cargo run --bin dbc-cli
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

For commercial licensing, see [LICENSING.md](LICENSING.md).

