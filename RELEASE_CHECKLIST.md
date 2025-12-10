# Release Checklist

This checklist ensures all steps are completed before publishing a new release of `dbc-rs`.

## Pre-Release Preparation

### 1. Code Quality Checks

- [ ] **All tests pass**
  ```bash
  cargo test --workspace
  ```

- [ ] **Clippy checks pass (all targets)**
  ```bash
  # Default target (with std) - matches CI workflow
  cargo clippy --all-targets -p dbc-rs -- -D warnings
  
  # no_std target (clippy only checks, doesn't build, so debug build issues don't apply)
  cargo clippy --no-default-features --target thumbv7m-none-eabi -p dbc-rs -- -D warnings
  
  # dbc-cli
  cargo clippy --all-targets -p dbc-cli -- -D warnings
  ```
  
  **Note**: The CI workflow runs clippy checks for both std and no_std modes.

- [ ] **Code formatting is consistent**
  ```bash
  cargo fmt -- --check
  ```

- [ ] **Documentation builds without warnings**
  ```bash
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p dbc-rs
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p dbc-cli
  ```

- [ ] **Code coverage meets threshold (80%)**
  ```bash
  cargo llvm-cov --workspace --tests --lib --ignore-filename-regex "^(tests|examples|benches)/" --fail-under-lines 80
  ```

### 2. Build Verification

- [ ] **Builds successfully with `std` feature**
  ```bash
  cargo build --release -p dbc-rs
  ```

- [ ] **Builds successfully in `no_std` mode**
  ```bash
  cargo build --release --no-default-features --target thumbv7m-none-eabi -p dbc-rs
  ```

- [ ] **Builds successfully on MSRV (1.85.0)**
  ```bash
  rustup toolchain install 1.85.0
  rustup run 1.85.0 cargo build --release -p dbc-rs
  rustup run 1.85.0 cargo test -p dbc-rs
  ```

### 3. Documentation Updates

- [ ] **CHANGELOG.md updated**
  - [ ] All changes since last release documented
  - [ ] Version number updated
  - [ ] Release date added
  - [ ] Breaking changes clearly marked
  - [ ] Migration guide added if needed

- [ ] **README.md reviewed**
  - [ ] Examples are up-to-date
  - [ ] Feature list is accurate
  - [ ] Links are working
  - [ ] Badges are correct

- [ ] **dbc/README.md reviewed**
  - [ ] Examples are up-to-date
  - [ ] Feature support table is accurate
  - [ ] Security section is current
  - [ ] References are correct

- [ ] **API documentation is complete**
  - [ ] All public APIs have doc comments
  - [ ] Examples in doc comments compile
  - [ ] Error conditions documented

- [ ] **SECURITY.md reviewed**
  - [ ] All fixed issues marked as resolved
  - [ ] Date updated if audit was refreshed

### 4. Version Updates

- [ ] **Workspace Cargo.toml version updated**
  ```toml
  [workspace.package]
  version = "X.Y.Z"  # Update this
  ```

- [ ] **Version strings in code updated** (if any)
  - [ ] Check `dbc/src/lib.rs` for `PKG_VERSION` constant (uses `env!` macro, auto-updates; only available with `std` feature)

- [ ] **Git tag prepared**
  - [ ] Tag name follows format: `vX.Y.Z` (e.g., `v0.1.0`)
  - [ ] Tag message includes release notes

### 5. Security & Compliance

- [ ] **Security audit reviewed**
  - [ ] All critical issues resolved
  - [ ] Known vulnerabilities documented
  - [ ] DoS protections verified

- [ ] **License files present**
  - [ ] dbc/LICENSE-APACHE
  - [ ] dbc/LICENSE-MIT
  - [ ] dbc/LICENSE-COMMERCIAL
  - [ ] dbc/LICENSING.md

- [ ] **Dependencies audited** (if any added in future)
  ```bash
  cargo audit  # If cargo-audit is installed
  ```

### 6. File Synchronization

- [ ] **SECURITY.md reviewed**
  - [ ] Version and date updated for release
  - [ ] All security issues documented

### 7. Final Verification

- [ ] **All CI checks pass**
  - [ ] **dbc-rs Library Workflow** (`.github/workflows/dbc-rs.yml`) successful
    - [ ] `test-std` job passes (tests with std on latest stable)
    - [ ] `test-no-std` job passes (tests no_std on latest stable)
    - [ ] `test-std-msrv` job passes (tests with std on MSRV 1.85.0)
    - [ ] `test-no-std-msrv` job passes (tests no_std on MSRV 1.85.0)
    - [ ] `lint` job passes (clippy checks for std and no_std modes)
    - [ ] `fmt` job passes (formatting check)
    - [ ] `doc` job passes (documentation builds)
    - [ ] `coverage` job passes (code coverage ≥80%)
  - [ ] **dbc-cli Workflow** (`.github/workflows/dbc-cli.yml`) successful (if CLI changes were made)
    - [ ] `test` job passes (tests on latest stable)
    - [ ] `test-msrv` job passes (tests on MSRV 1.85.0)
    - [ ] `lint` job passes (clippy checks)
    - [ ] `fmt` job passes (formatting check)

- [ ] **Pre-commit hook passes**
  ```bash
  .githooks/pre-commit
  ```

- [ ] **No uncommitted changes**
  ```bash
  git status
  ```

## Release Process

### 8. Git Operations

- [ ] **All changes committed**
  ```bash
  git add .
  git commit -m "Release vX.Y.Z"
  ```

- [ ] **Release branch created** (if using branch strategy)
  ```bash
  git checkout -b release/vX.Y.Z
  ```

- [ ] **Changes pushed to remote**
  ```bash
  git push origin main  # or release branch
  ```

- [ ] **Git tag created**
  ```bash
  git tag -a vX.Y.Z -m "Release vX.Y.Z

  [Brief release notes]
  
  See CHANGELOG.md for full details."
  git push origin vX.Y.Z
  ```

### 9. Publishing to crates.io

- [ ] **Dry-run successful**
  ```bash
  cargo publish --dry-run -p dbc-rs
  ```

- [ ] **Published to crates.io**
  ```bash
  cargo publish -p dbc-rs
  ```
  
  **Note**: 
  - Requires crates.io authentication (`cargo login`)
  - Publishing is permanent (cannot delete, only yank)
  - Wait for CI to pass before publishing if using automated workflows

- [ ] **Verify publication**
  - [ ] Check https://crates.io/crates/dbc-rs
  - [ ] Version appears correctly
  - [ ] README displays correctly
  - [ ] Documentation link works

### 10. Post-Release Tasks

- [ ] **GitHub release created**
  - [ ] Go to https://github.com/reneherrero/dbc-rs/releases/new
  - [ ] Select the tag created in step 8
  - [ ] Title: `vX.Y.Z`
  - [ ] Description: Copy from CHANGELOG.md
  - [ ] Mark as "Latest release" if this is the newest
  - [ ] Attach any release artifacts if needed

- [ ] **Update workspace root README** (if needed)
  - [ ] Update version badges if applicable
  - [ ] Update any version-specific information

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes
- **MINOR** (0.X.0): New features, backwards compatible
- **PATCH** (0.0.X): Bug fixes, backwards compatible

For pre-releases:
- **Alpha**: `0.1.0-alpha.1`, `0.1.0-alpha.2`, etc.
- **Beta**: `0.1.0-beta.1`, `0.1.0-beta.2`, etc.
- **RC**: `0.1.0-rc.1`, `0.1.0-rc.2`, etc.

## Quick Release Command Reference

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Run all checks locally (matches CI workflows)
cargo test --workspace
# Note: The following may fail if there are compilation errors - fix before release
cargo test --no-default-features -p dbc-rs
cargo clippy --all-targets -p dbc-rs -- -D warnings
cargo clippy --no-default-features --target thumbv7m-none-eabi -p dbc-rs -- -D warnings
cargo clippy --all-targets -p dbc-cli -- -D warnings
cargo fmt -- --check
cargo llvm-cov --workspace --fail-under-lines 80
cargo publish --dry-run -p dbc-rs

# 4. Push changes and wait for CI workflows to pass
git add .
git commit -m "Release vX.Y.Z"
git push origin main
# Wait for GitHub Actions workflows to complete:
# - dbc-rs Library Workflow (all jobs must pass)
# - dbc-cli Workflow (if CLI changes were made)

# 5. Create and push tag
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z

# 6. Publish to crates.io
cargo publish -p dbc-rs

# 7. Create GitHub release (via web UI)
```

## Troubleshooting

### If publishing fails:
- Check crates.io authentication: `cargo login`
- Verify package name is available
- Check for duplicate version numbers
- Review error messages for specific issues

### If documentation doesn't appear on docs.rs:
- Wait 10-15 minutes (docs.rs builds are queued)
- Check https://docs.rs/releases/queue for build status
- Verify `[package.metadata.docs.rs]` in Cargo.toml is correct

### If tests fail after version update:
- Verify all version references are updated
- Check that examples in documentation still compile
- Ensure no hardcoded version strings remain

## Notes

- **Never skip steps**: Each step ensures quality and prevents issues
- **Test thoroughly**: Especially test `no_std` builds before releasing
- **Document breaking changes**: Users need clear migration paths
- **Keep SECURITY.md updated**: Review and update for each release
- **Verify CI passes**: Don't publish if CI is failing
- **CI Workflows**: The project uses two separate workflows:
  - `dbc-rs.yml`: Tests the library with multiple configurations:
    - `std` feature (default) on latest stable and MSRV
    - `no_std` mode on latest stable and MSRV
    - Clippy checks for both std and no_std modes
    - Formatting, documentation, and coverage checks
  - `dbc-cli.yml`: Tests the CLI application
  - Both workflows run automatically on pushes and PRs to `main`
  - Workflows use path-based triggers to only run when relevant files change

