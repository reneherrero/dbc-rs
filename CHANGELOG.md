# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- (Future changes will be listed here)

## [0.1.0-beta.3] - 2025-12-08

### Added

- **VAL Support**: Added support for value descriptions (VAL_ lines in DBC files)
  - Parse value descriptions for signals and messages
  - Support for message-specific and global value descriptions
  - Value descriptions accessible through `Dbc::value_descriptions()` method
  - Builder support for creating value descriptions programmatically

### Fixed

- **no_std Build Compatibility**: Fixed compilation errors in no_std mode
  - Split `validate` function into feature-specific versions (alloc/kernel vs no_std)
  - Removed conditional parameter that caused compilation errors
  - All builds now compile successfully in no_std mode

- **Kernel Mock API Verification**: Improved kernel alloc API mock implementation
  - Removed unused `CapacityOverflow` variant from `TryReserveErrorKind`
  - Verified all methods match real kernel alloc API patterns
  - All clippy warnings resolved

## [0.1.0-beta.2] - 2025-12-07

### Changed

- **Compatibility Layer Refactoring**: Improved `compat` module structure
  - Split into separate `alloc.rs` and `kernel.rs` modules
  - Reduced conditional compilation complexity
  - Better separation of concerns between alloc and kernel implementations

- **Code Quality Improvements**: Enhanced code maintainability
  - Combined use statements where possible
  - Removed unused imports across all modules
  - Fixed all clippy warnings with `-D warnings`
  - Optimized import organization

- **Test Organization**: Reorganized tests by feature
  - Tests split by feature flags (no_std, alloc, std, kernel)
  - Better test isolation and maintainability
  - All tests pass with `--no-default-features`, `--features alloc`, and `--features kernel`

- **Documentation Improvements**: Enhanced doctest compatibility
  - Fixed all doctests to work in no_std environments
  - Removed `println!` calls from doctests
  - Added `rust,no_run` markers for doctests requiring alloc features
  - Improved error reporting examples

### Fixed

- **no_std Build Compatibility**: Fixed compilation errors in no_std mode
  - Split `validate` function into feature-specific versions (alloc/kernel vs no_std)
  - Removed conditional parameter that caused compilation errors
  - All builds now compile successfully in no_std mode

- **Parser Line Number Tracking**: Added line number tracking to parser
  - `Parser` now tracks current line number for better error reporting
  - Public `line()` getter method available for future error reporting enhancements

- **Builder Usage in Tests**: Cleaned up test organization
  - Removed builder usage from object test modules
  - Builders only used in their respective builder test modules
  - Improved test clarity and maintainability

- **Error Handling**: Improved error message helpers
  - Centralized error string conversion in `error` module
  - Reduced code duplication in error handling

- **Release Checklist**: Verified and corrected all commands
  - Fixed clippy commands to match CI workflow
  - Updated code coverage command to match CI
  - Verified all build and test commands

### Documentation

- **Release Checklist**: Updated `RELEASE_CHECKLIST.md`
  - Verified all commands work correctly
  - Fixed clippy check commands
  - Updated code coverage command
  - Added notes about experimental kernel feature

## [0.1.0-beta.1] - 2024-12-06

**Note**: This is the first beta release. The API is now stable and ready for wider testing.

### Added

- **Configurable Parsing Options**: `ParseOptions` struct for customizing parsing behavior
  - `ParseOptions::strict()` - Strict boundary validation (default)
  - `ParseOptions::lenient()` - Allow signals that extend beyond message boundaries
  - `Dbc::parse_with_options()` - Parse with custom options
  - Useful for parsing real-world DBC files with technically invalid but commonly used signal definitions

- **Property-Based Testing**: Comprehensive property-based tests using `proptest`
  - Tests parsing round-trips with randomly generated DBC content
  - Validates parser robustness against edge cases

- **Benchmark Suite**: Performance benchmarks using `criterion`
  - Parsing performance benchmarks
  - String conversion benchmarks
  - Available with `alloc` feature

- **Enhanced Error Messages**: Improved error context and recovery suggestions
  - Signal overlap errors now mention multiplexing as a potential solution
  - DLC errors include message name, ID, and valid range suggestions
  - Signal length errors include signal name and valid range
  - Start bit errors include signal name and valid range
  - All error messages provide actionable recovery suggestions

### Changed

- **Relaxed Node Validation**: Empty node lists are now allowed (DBC spec compliant)
  - Removed requirement for at least one node in `BU_` line
  - `Dbc::nodes()` may return an empty collection
  - `DbcBuilder` no longer requires nodes

- **Improved Error Context**: Error messages now include relevant values
  - Message errors include message name and ID
  - Signal errors include signal name
  - DLC errors show actual DLC value and valid range
  - Signal length errors show actual length and valid range

### Fixed

- **Documentation Warnings**: Fixed unclosed HTML tags in documentation comments
- **Clippy Warnings**: Resolved all Clippy linter warnings
- **Build Issues**: Worked around rustc/LLVM bug for `thumbv7m-none-eabi` debug builds
  - CI now uses `cargo check` and `cargo build --release` for this target
- **Test Coverage**: Increased test coverage for edge cases
  - Tests for empty nodes
  - Tests for Unicode in names
  - Tests for boundary validation with strict/lenient modes

### Documentation

- **Release Checklist**: Added comprehensive `RELEASE_CHECKLIST.md` for release procedures
- **API Documentation**: All public APIs fully documented with examples
- **Code Examples**: Updated examples to include `ParseOptions` usage

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

#### no_std Support
- **Full `no_std` + `alloc` support** for embedded systems
- **Feature flags**: `std` feature for standard library integration
- **Zero dependencies**: Pure Rust implementation
- **Memory efficient**: Uses `Box<str>` for string storage

#### Documentation
- **Comprehensive API Documentation**: All public APIs fully documented
- **Module-level Documentation**: Detailed library overview
- **Code Examples**: Examples for common use cases
- **DBC Format Specification**: Detailed format documentation (`SPECIFICATIONS.md`)
- **Contributing Guide**: Development guidelines (`CONTRIBUTING.md`)
- **Usage Patterns**: Best practices and anti-patterns documented

#### Developer Experience
- **Read-only API**: Immutable data structures with getter methods
- **Type Safety**: Strong typing for all DBC elements
- **Debug Support**: `Debug` trait implementations
- **Clone Support**: `Clone` for `Message`, `Signal`, `Version`, `Nodes`
- **PartialEq Support**: Equality comparison for testing

#### Testing
- **Comprehensive Test Suite**: 161+ tests covering:
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

- **0.1.0-beta.3** (2025-12-08): Added VAL support, fixed no_std build compatibility, improved kernel mock API
- **0.1.0-beta.2** (2025-12-07): Code quality improvements, test reorganization, doctest fixes
- **0.1.0-beta.1** (2024-12-06): First beta release - API stable, ready for wider testing
- **0.1.0-alpha** (2024-11-25): Initial alpha release

---

## Types of Changes

- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** for vulnerability fixes

[0.1.0-beta.3]: https://github.com/reneherrero/dbc-rs/releases/tag/v0.1.0-beta.3
[0.1.0-beta.2]: https://github.com/reneherrero/dbc-rs/releases/tag/v0.1.0-beta.2
[0.1.0-beta.1]: https://github.com/reneherrero/dbc-rs/releases/tag/v0.1.0-beta.1
[0.1.0-alpha]: https://github.com/reneherrero/dbc-rs/releases/tag/v0.1.0-alpha

