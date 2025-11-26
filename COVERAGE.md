# Code Coverage

This project tracks code coverage to ensure comprehensive testing and identify untested code paths.

## Coverage Setup

Code coverage is automatically generated in CI using `cargo-llvm-cov`. The coverage job runs on every push and pull request.

## Coverage Goals

- **Minimum threshold**: 80% code coverage
- **Target**: Maintain or improve coverage with each change
- **Focus**: Library code (tests and examples are excluded)

## Running Coverage Locally

### Prerequisites

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

### Generate Coverage Report

Basic coverage report:

```bash
cargo llvm-cov --all-features --workspace
```

HTML report (opens in browser):

```bash
cargo llvm-cov --all-features --workspace --html
```

Coverage for specific package:

```bash
cargo llvm-cov --all-features -p dbc-rs
```

### Coverage Options

- `--all-features`: Test with all features enabled
- `--workspace`: Test all workspace members
- `--html`: Generate HTML report
- `--open`: Open HTML report in browser
- `--lcov`: Generate LCOV format (for external tools)
- `--json`: Generate JSON format

## CI Integration

Coverage is automatically:
- Generated on every push and PR
- Posted as a comment on PRs (markdown format)
- Fails the build if coverage drops below 80%
- Excludes test files and examples from coverage calculation

## Improving Coverage

If coverage is below the threshold:

1. Review the coverage report to identify untested code
2. Add tests for uncovered functions and branches
3. Focus on critical paths and error handling
4. Ensure edge cases are tested

## Coverage Artifacts

Coverage artifacts are automatically ignored by git (see `.gitignore`):
- `llvm-cov-target/`
- `coverage/`
- `*.profraw`
- `*.profdata`

