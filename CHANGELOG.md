# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- (Future changes will be listed here)

## [0.1.0-alpha] - 2024-11-25

**Note**: This is an alpha release. The API may change before the stable 1.0.0 release.

### Added

#### Core Functionality
- **DBC File Parsing**: Complete parser for DBC (CAN Database) file format
  - Parse version information (`VERSION`)
  - Parse node definitions (`BU_`)
  - Parse CAN messages (`BO_`) with ID, name, DLC, and sender
  - Parse signals (`SG_`) with full attribute support
  - Support for standard (11-bit) and extended (29-bit) CAN IDs
- **DBC File Writing**: Save DBC files with `Dbc::save()` method
  - Preserves structure and formatting
  - Supports all parsed elements
- **Generic Input Sources**: Parse from multiple input types
  - `Dbc::parse()` - from string slice
  - `Dbc::parse_bytes()` - from byte slice
  - `Dbc::parse_from()` - from any `AsRef<str>`
  - `Dbc::from_reader()` - from `std::io::Read` (requires `std` feature)

#### Builder Pattern
- **Builder API** for all complex structs:
  - `Dbc::builder()` - Construct DBC files programmatically
  - `Message::builder()` - Construct CAN messages
  - `Signal::builder()` - Construct signals with validation
  - `Version::builder()` - Construct version strings
  - `Nodes::builder()` - Construct node lists
- **Validation Methods**: `validate()` method on all builders for pre-build validation

#### Validation
- **CAN ID Range Validation**:
  - Standard IDs: 0-0x7FF (0-2047)
  - Extended IDs: 0x800-0x1FFFFFFF (2048-536870911)
- **Signal Validation**:
  - Signal overlap detection within messages
  - Signal boundary checking (start_bit + length within message size)
  - Min/max range validation
  - Length validation (1-64 bits)
- **Message Validation**:
  - Duplicate message ID detection
  - Sender node validation (must exist in nodes list)
  - DLC validation (1-8 bytes)
- **Node Validation**:
  - Duplicate node name detection (case-sensitive)
  - Non-empty node list validation

#### Error Handling
- **Categorized Error Types**:
  - `Error::Signal` - Signal-specific errors
  - `Error::Message` - Message-specific errors
  - `Error::Dbc` - DBC file-level errors
  - `Error::Version` - Version parsing errors
  - `Error::Nodes` - Node-related errors
  - `Error::InvalidData` - General parsing errors
- **Internationalized Error Messages**: Build-time language selection
  - English (default)
  - French (`lang-fr` feature)
  - Spanish (`lang-es` feature)
  - German (`lang-de` feature)
  - Japanese (`lang-ja` feature)

#### no_std Support
- **Full `no_std` + `alloc` support** for embedded systems
- **Feature flags**: `std` feature for standard library integration
- **Zero dependencies**: Pure Rust implementation
- **Memory efficient**: Uses `Box<str>` for string storage

#### Documentation
- **Comprehensive API Documentation**: All public APIs fully documented
- **Module-level Documentation**: Detailed library overview
- **Code Examples**: Examples for common use cases
- **DBC Format Specification**: Detailed format documentation (`DBC_FORMAT_SPEC.md`)
- **Contributing Guide**: Development guidelines (`CONTRIBUTING.md`)
- **Usage Patterns**: Best practices and anti-patterns documented

#### Developer Experience
- **Read-only API**: Immutable data structures with getter methods
- **Type Safety**: Strong typing for all DBC elements
- **Debug Support**: `Debug` trait implementations
- **Clone Support**: `Clone` for `Message`, `Signal`, `Version`, `Nodes`
- **PartialEq Support**: Equality comparison for testing

#### Testing
- **Comprehensive Test Suite**: 151+ tests covering:
  - Parsing edge cases
  - Validation scenarios
  - Error conditions
  - Round-trip parsing (parse → save → parse)
- **Integration Tests**: Real-world DBC file parsing
- **Test Data**: Multiple DBC files for various scenarios:
  - `complete.dbc` - Comprehensive test file
  - `simple.dbc` - Basic functionality
  - `multiplexed.dbc` - Multiple signals
  - `minimal.dbc` - Edge cases
  - `extended_ids.dbc` - Extended CAN IDs
  - `broadcast_signals.dbc` - Broadcast receivers

#### Code Quality
- **Linting**: `clippy` with `-D warnings`
- **Formatting**: `rustfmt` with consistent style
- **Documentation**: `#![deny(missing_docs)]` for all public APIs
- **Safety**: `#![deny(unsafe_code)]` - no unsafe code
- **CI/CD**: Comprehensive GitHub Actions workflows
  - Tests for `std` and `no_std` modes
  - MSRV testing (Rust 1.85.0)
  - Latest Rust toolchain testing
  - Clippy and rustfmt checks
  - Documentation generation checks
  - Language feature testing

#### Workspace Structure
- **Rust Workspace**: Organized as a workspace with:
  - `dbc` - Core library crate
  - `dbc-cli` - Command-line interface
- **Dual Licensing**: MIT OR Apache-2.0 (open source) + Commercial license option

### Changed
- N/A (Initial release)

### Deprecated
- N/A (Initial release)

### Removed
- N/A (Initial release)

### Fixed
- N/A (Initial release)

### Security
- **No unsafe code**: Entire codebase uses safe Rust only
- **Input validation**: All inputs are validated before processing
- **Memory safety**: Rust's ownership system prevents memory issues
- **No dependencies**: Zero external dependencies reduces attack surface

## Version History

- **0.1.0-alpha** (2024-12-XX): Initial alpha release

---

## Types of Changes

- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** for vulnerability fixes

[0.1.0-alpha]: https://github.com/reneherrero/dbc-rs/releases/tag/v0.1.0-alpha

