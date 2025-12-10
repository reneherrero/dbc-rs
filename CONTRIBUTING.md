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

### Basic Commands

```bash
# Build
cargo check --all-targets
cargo check --target thumbv7m-none-eabi --no-default-features --package dbc-rs

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --no-default-features --target thumbv7m-none-eabi --package dbc-rs -- -D warnings
```

**Note**: The pre-commit hook automatically runs clippy and formatting checks.

### Testing and Quality Checks

For comprehensive testing procedures, build verification, and code coverage requirements, see [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) sections 1-2.

**Quick reference:**
- **Tests**: Must pass in both `std` and `no_std` modes
- **Coverage**: Minimum 80% (see RELEASE_CHECKLIST.md for details)
- **MSRV**: Must work with Rust 1.85.0

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
- See [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) for comprehensive testing procedures

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
- [ ] All tests pass (see [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) for testing procedures)
- [ ] Clippy passes without warnings (see [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) section 1)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated

### CI/CD Workflows

The project uses GitHub Actions for continuous integration. Workflows automatically run on pushes and pull requests to the `main` branch.

- **dbc-rs Library Workflow** (`.github/workflows/dbc-rs.yml`): Tests library with `std`/`no_std`, MSRV, linting, formatting, docs, and coverage
- **dbc-cli Workflow** (`.github/workflows/dbc-cli.yml`): Tests CLI application

For detailed workflow information and CI verification procedures, see [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) section 7.

**Best Practices:**
- Wait for CI checks to pass before merging PRs
- Fix CI failures locally before pushing
- Workflows use path-based triggers to reduce unnecessary runs

## Project Structure

```
dbc-rs/
├── dbc/                  # Main library crate
│   ├── src/
│   │   ├── dbc/         # DBC file structure & builder
│   │   ├── message/     # CAN message definitions & builder
│   │   ├── signal/      # Signal definitions & builder
│   │   ├── nodes/       # Node/ECU management & builder
│   │   ├── receivers/   # Signal receivers & builder
│   │   ├── version/     # Version information & builder
│   │   └── error/       # Error types & i18n (en, de, es, fr, ja)
│   ├── tests/           # Integration tests & test data
│   ├── examples/        # Example code (std, no_std, builder)
│   └── benches/         # Benchmark tests
├── dbc-cli/             # Command-line interface
└── .github/workflows/   # CI/CD workflows
```

## Areas for Contribution

### High Priority

- Additional DBC format features (attributes, value tables, comments, etc.)
- Performance optimizations
- More comprehensive test coverage
- Documentation improvements

### Medium Priority

- More example code
- Benchmarking and performance analysis
- Additional DBC file test cases

### Low Priority

- CLI enhancements
- Additional tooling
- Website/documentation site

## Questions?

If you have questions or need help:

- Open an issue on GitHub
- Check existing issues and discussions
- Review the documentation in the README files

## License

By contributing to dbc-rs, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0). See [LICENSING.md](dbc/LICENSING.md) for details.

Thank you for contributing! 🎉

