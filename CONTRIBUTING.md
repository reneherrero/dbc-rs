# Contributing to dbc-rs

Thank you for your interest in contributing to dbc-rs! This document provides guidelines and instructions for contributing to the project.

**For release procedures**, see [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md).

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Familiarity with Rust and the DBC file format
- Latest stable release of Rust along with tools and platforms ("targets") (defined in `rust-toolchain.toml`)
- Code coverage is done with `cargo-llvm-cov`

### Setting Up the Development Environment

### Code Coverage

Install `cargo-llvm-cov` using prebuilt binaries (recommended):

```bash
# Get your host target
host=$(rustc -vV | grep '^host:' | cut -d' ' -f2)

# Download and install prebuilt binary
curl --proto '=https' --tlsv1.2 -fsSL \
  "https://github.com/taiki-e/cargo-llvm-cov/releases/latest/download/cargo-llvm-cov-$host.tar.gz" \
  | tar xzf - -C "$HOME/.cargo/bin"
```

**Alternative methods:**

Using `cargo-binstall` (if installed):
```bash
cargo binstall cargo-llvm-cov
```

Using Homebrew (macOS/Linux):
```bash
brew install cargo-llvm-cov
```

**Note**: `cargo install cargo-llvm-cov` may fail with MSRV (1.85.0) due to dependency requirements. Prebuilt binaries are recommended for local development.

### Git Pre-Commit Hook (recommended)

Install git hooks (recommended):
```bash
./setup-git-hooks.sh
```
This installs a pre-commit hook that automatically runs clippy and formatting checks before each commit.

## Development Workflow

### Building

```bash
# Check that everything compiles with the Standard Library
cargo check --all-targets

#Check that dbc-rs compiles with `no_std`
cargo check --target thumbv7m-none-eabi --no-default-features --package dbc-rs
```

### Testing

```bash
cargo test
```

### Code Quality

```bash
# For default target (with std)
cargo clippy --all-targets --all-features -- -D warnings

# For no_std builds (must use --no-default-features)
cargo clippy --no-default-features --target thumbv7m-none-eabi --package dbc-rs -- -D warnings
```
**Note**: 
- If you've installed the git hooks (step 5 in setup), clippy will run atomatically on commit.
- When running clippy for `no_std` targets, you **must** use --no-default-features`, otherwise it will try to use `std` features hich aren't available on embedded targets.

### Formatting Code:

```bash
cargo fmt
```
**Note**: The pre-commit hook also checks formatting automatically.

### Verifying the Documentation

```bash
cargo doc --workspace --no-deps --document-private-items
```

### Code Coverage

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace

# Generate HTML report (opens in browser)
cargo llvm-cov --all-features --workspace --html
```

**Note**: 
- **Minimum threshold**: 80% code coverage
- **Target**: Maintain or improve coverage with each change
- **Focus**: Library code (tests and examples are excluded)

### Coding Standards

#### Code Style

- Follow the existing code style in the project
- Use `cargo fmt` to format your code (configuration is in `rustfmt.toml`)

#### Documentation

- **All public APIs must be documented** with doc comments (`///`)
- Use code examples in documentation when helpful
- Document error conditions and return values
- Follow Rust documentation conventions

#### Error Handling

- Use `Result<T>` for fallible operations
- Use appropriate error variants (`Error::Signal`, `Error::Message`, etc.)

#### Testing

- Write tests for new functionality
- Include both positive and negative test cases
- Test edge cases and error conditions
- Ensure tests pass in both `std` and `no_std` modes

#### Safety

- **No `unsafe` code** - The project explicitly disallows unsafe code
- Avoid `unwrap()` and `expect()` in production code (tests are fine)
- Use proper error handling with the `Result` type

### Commit Messages

Write clear, descriptive commit messages:

```
Short summary (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters. Explain:
- What changed and why
- Any breaking changes
- Related issues

Fixes #123
```

### Pull Requests

1. Update documentation if you're adding new features
2. Add tests for new functionality
3. Ensure all CI checks pass
4. Update `CHANGELOG.md`
5. Reference any related issues in your PR description

#### PR Checklist

- [ ] Code follows the project's guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Clippy passes without warnings with std (`cargo clippy -- -D warnings`)
- [ ] Clippy passes without warnings with no_std (`cargo clippy --no-default-features --target thumbv7m-none-eabi -p dbc-rs - -D warnings`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated

## Project Structure

```
dbc-rs/
├── dbc/              # Main library crate
│   ├── src/
│   │   ├── dbc.rs     # DBC file structure
│   │   ├── message.rs # CAN message definitions
│   │   ├── signal.rs  # Signal definitions
│   │   ├── nodes.rs   # Node/ECU management
│   │   ├── version.rs # Version
│   │   └── error/     # Error types and messages
│   ├── tests/         # Integration tests
│   └── examples/      # Example code
├── dbc-cli/           # Command-line interface
└── .github/workflows/ # CI/CD workflows
```

## Areas for Contribution

### High Priority

- Additional DBC format features (attributes, value tables, comments, etc.)
- Performance optimizations
- More comprehensive test coverage
- Documentation improvements

### Medium Priority

- Additional language translations for error messages
- More example code
- Benchmarking and performance analysis
- Additional DBC file test cases

### Low Priority

- CLI enhancements
- Additional tooling
- Website/documentation site

## Internationalization (i18n)

If you're contributing translations for error messages:

1. Add your language module in `dbc/src/error/lang/`
2. Add the language feature to `dbc/Cargo.toml`
3. Update `dbc/src/error/lang/mod.rs` to include your language
4. Add all required constants (see `en.rs` for reference)
5. Update the README to document the new language option

## Questions?

If you have questions or need help:

- Open an issue on GitHub
- Check existing issues and discussions
- Review the documentation in the README files

## License

By contributing to dbc-rs, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0). See [LICENSING.md](LICENSING.md) for details.

Thank you for contributing! 🎉

