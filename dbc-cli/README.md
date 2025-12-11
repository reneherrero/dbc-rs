# dbc-cli

A command-line interface for parsing, analyzing, and manipulating DBC (CAN Database) files. Built on top of the `dbc-rs` library, providing a powerful toolset for working with CAN bus database files.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSING.md)
[![CI](https://github.com/reneherrero/dbc-rs/workflows/dbc-cli%20CI/badge.svg)](https://github.com/reneherrero/dbc-rs/actions/workflows/dbc-cli.yml)

## Overview

dbc-cli provides a comprehensive command-line tool for working with DBC files. It enables parsing, validation, storage, and decoding of CAN messages using DBC definitions. The tool is designed to be a practical replacement for `candump` with enhanced capabilities for DBC file manipulation.

## Features

- **DBC File Parsing** - Parse and validate DBC files with comprehensive error reporting
- **Persistent Storage** - Store parsed DBC files in platform-specific cache directories
- **Message Decoding** - Decode CAN messages from candump format using stored DBC definitions
- **File Analysis** - Describe DBC file contents including messages, signals, and nodes
- **Version Information** - Display library version and build information

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

### Version Information

Display the version of the dbc-rs library:

```bash
dbc-cli version
```

Or simply run without arguments:

```bash
dbc-cli
```

### Parse and Store a DBC File

Parse a DBC file, validate it, and store it for later use:

```bash
dbc-cli parse path/to/file.dbc
```

The parsed DBC file is stored in a platform-specific cache directory:
- Linux: `~/.cache/dbc-cli/stored.dbc`
- macOS: `~/Library/Caches/dbc-cli/stored.dbc`
- Windows: `%LOCALAPPDATA%\dbc-cli\cache\stored.dbc`

### Print Stored DBC File

Output the stored DBC file to stdout:

```bash
dbc-cli print
```

This command reads the previously stored DBC file and outputs it in standard DBC format.

### Describe DBC Contents

Display a summary of the stored DBC file contents:

```bash
dbc-cli describe
```

The output includes:
- Version information
- Node list
- Message count and details
- Signal statistics
- Receiver information

Example output:
```
Version: 2.0
Nodes: ECM, TCM, BCM
Messages: 15
Signals: 42
...
```

### Decode CAN Messages

Decode a CAN message from candump format using the stored DBC file:

```bash
dbc-cli decode "1F334455#1122334455667788"
```

The input format is compact candump format: `CAN_ID#DATA` where:
- `CAN_ID` is the CAN message ID in hexadecimal
- `DATA` is the message payload in hexadecimal (up to 8 bytes for standard CAN, 64 bytes for CAN FD)

The command will:
1. Look up the message ID in the stored DBC file
2. Decode all signals within that message
3. Display signal names and decoded values

**Note:** A DBC file must be parsed and stored first using the `parse` command.

### Clear Stored DBC File

Remove the stored DBC file from cache:

```bash
dbc-cli clear
```

This deletes the cached DBC file, allowing you to parse and store a different DBC file.

## Command Reference

| Command | Description |
|---------|-------------|
| `version` | Display version information |
| `parse <file>` | Parse and store a DBC file |
| `print` | Output the stored DBC file |
| `describe` | Display summary of stored DBC file |
| `decode <message>` | Decode a CAN message (candump format) |
| `clear` | Remove stored DBC file from cache |

## Examples

### Basic Workflow

```bash
# Parse a DBC file
dbc-cli parse vehicle.dbc

# View the stored file
dbc-cli print

# Get a summary
dbc-cli describe

# Decode a CAN message
dbc-cli decode "100#0011223344556677"

# Clear and parse a different file
dbc-cli clear
dbc-cli parse different_vehicle.dbc
```

### Integration with candump

dbc-cli can be used in combination with `candump` to decode live CAN traffic:

```bash
# Store the DBC file
dbc-cli parse my_vehicle.dbc

# Pipe candump output to dbc-cli for decoding
candump can0 | while read line; do
    # Extract CAN ID and data, convert to compact format
    can_id=$(echo "$line" | awk '{print $3}')
    can_data=$(echo "$line" | awk '{print $4$5$6$7$8$9$10$11}')
    dbc-cli decode "${can_id}#${can_data}"
done
```

## Development

### Building

Build the CLI tool:

```bash
cargo build --bin dbc-cli
```

### Running

Run the CLI tool directly:

```bash
cargo run --bin dbc-cli -- <command>
```

For example:

```bash
cargo run --bin dbc-cli -- parse example.dbc
cargo run --bin dbc-cli -- describe
```

### Testing

Run tests for the CLI:

```bash
cargo test -p dbc-cli
```

## Architecture

dbc-cli is built on top of the `dbc-rs` library and uses:

- **clap** - Command-line argument parsing
- **dirs** - Platform-specific directory resolution for cache storage
- **dbc-rs** - Core DBC parsing and manipulation library

The tool maintains a single stored DBC file in a platform-specific cache directory, allowing multiple commands to operate on the same DBC definition without requiring file paths for each operation.

## Future Enhancements

Planned features include:

- Support for multiple stored DBC files with named contexts
- Enhanced decoding output with formatting options
- Integration with CAN bus interfaces for live decoding
- Python FFI for scripting support
- Batch processing of CAN log files
- Export capabilities (JSON, CSV, etc.)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../dbc/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../dbc/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

For commercial licensing, see [LICENSING.md](LICENSING.md).
