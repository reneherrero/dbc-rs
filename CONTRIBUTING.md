# Contributing to dbc-rs

Thank you for your interest in contributing to dbc-rs! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.85.0 or later (see [MSRV](dbc/README.md#minimum-supported-rust-version-msrv))
- Git
- Basic familiarity with Rust and the DBC file format

### Setting Up the Development Environment

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/dbc-rs.git
   cd dbc-rs
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run the tests:
   ```bash
   cargo test
   ```

## Development Workflow

### Making Changes

1. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. Make your changes following the coding standards below

3. Ensure all tests pass:
   ```bash
   cargo test
   ```

4. Run clippy to catch common issues:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

5. Format your code:
   ```bash
   cargo fmt
   ```

6. Check that the code compiles in both `std` and `no_std` modes:
   ```bash
   cargo check --all-targets
   cargo check --all-targets --no-default-features
   ```

### Coding Standards

#### Code Style

- Follow the existing code style in the project
- Use `cargo fmt` to format your code (configuration is in `rustfmt.toml`)
- Maximum line length is 100 characters
- Use 4 spaces for indentation in Rust files

#### Documentation

- **All public APIs must be documented** with doc comments (`///`)
- Use code examples in documentation when helpful
- Document error conditions and return values
- Follow Rust documentation conventions

#### Error Handling

- Use `Result<T, Error>` for fallible operations
- Provide descriptive error messages
- Use appropriate error variants (`Error::Signal`, `Error::Message`, etc.)

#### Testing

- Write tests for new functionality
- Include both positive and negative test cases
- Test edge cases and error conditions
- Ensure tests pass in both `std` and `no_std` modes

#### Safety

- **No `unsafe` code** - The project explicitly disallows unsafe code
- Avoid `unwrap()` and `expect()` in production code (tests are fine)
- Use proper error handling with `Result` types

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
4. Update `CHANGELOG.md` if it exists (or create one if needed)
5. Reference any related issues in your PR description

#### PR Checklist

- [ ] Code follows the project's style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Clippy passes without warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] Changes work in both `std` and `no_std` modes
- [ ] No `unsafe` code introduced
- [ ] Commit messages are clear and descriptive

## Project Structure

```
dbc-rs/
├── dbc/              # Main library crate
│   ├── src/
│   │   ├── dbc.rs     # DBC file structure and parsing
│   │   ├── message.rs # CAN message definitions
│   │   ├── signal.rs  # Signal definitions
│   │   ├── nodes.rs   # Node/ECU management
│   │   ├── version.rs # Version parsing
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

