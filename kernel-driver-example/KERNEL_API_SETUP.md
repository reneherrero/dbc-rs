# Testing Against Rust-for-Linux Kernel Alloc API

This guide explains how to test `dbc-rs` against the rust-for-linux kernel alloc API.

## Prerequisites

1. **Rust-for-Linux Kernel Source**
   - Clone: `git clone -b rust https://github.com/Rust-for-Linux/linux.git`
   - Or use TI kernel with rust-for-linux patches

2. **Kernel Build Environment**
   - Kernel configured with `CONFIG_RUST=y`
   - Rust toolchain (nightly)
   - Kernel build system

3. **Kernel Bindings**
   - Build kernel to generate Rust bindings
   - Bindings are in `rust/bindings/bindings_generated.rs`

## Setup Methods

### Method 1: Using Verification Script

We provide a script to verify API compatibility (in the main repository):

```bash
cd ../..  # Go to repository root
./scripts/verify-kernel-api.sh
```

This script:
- Clones rust-for-linux if needed
- Extracts kernel alloc API signatures
- Compares with our mock implementation
- Reports discrepancies

### Method 2: Direct Kernel Crate Reference

To test with actual kernel crate in `kernel-driver-example`:

1. **Set up kernel source**:
   ```bash
   cd kernel-driver-example
   ./setup-ti-kernel.sh  # Or manually set KDIR
   ```

2. **Configure kernel**:
   ```bash
   cd $KDIR
   make ARCH=arm64 defconfig
   # Enable CONFIG_RUST=y in .config
   ```

3. **Build kernel bindings**:
   ```bash
   make ARCH=arm64 rustavailable
   make ARCH=arm64 rustfmtcheck  # Generates bindings
   ```

4. **Update Cargo.toml** in `kernel-driver-example`:
   ```toml
   [dependencies]
   kernel = { path = "../rust-for-linux/rust" }
   ```

5. **Test compilation**:
   ```bash
   cd kernel-driver-example
   cargo check --features kernel
   ```

### Method 3: Using Git Dependency

Reference rust-for-linux kernel crate directly:

```toml
[dependencies]
kernel = { git = "https://github.com/Rust-for-Linux/linux.git", package = "kernel", branch = "rust" }
```

**Note**: This requires the kernel crate to be published or accessible via git.

## API Verification Checklist

When testing against real kernel API, verify:

### String API
- [ ] `String::try_from(s: &str) -> Result<Self, AllocError>` exists
- [ ] `String::try_push_str(&mut self, s: &str) -> Result<(), AllocError>` exists
- [ ] `String::as_str(&self) -> &str` exists
- [ ] `Deref<Target = str>` is implemented
- [ ] `Display` trait is implemented

### Vec API
- [ ] `Vec::new() -> Self` exists
- [ ] `Vec::try_push(&mut self, item: T) -> Result<(), AllocError>` exists
- [ ] `Vec::try_extend_from_slice(&mut self, other: &[T]) -> Result<(), AllocError>` exists
- [ ] `Vec::len(&self) -> usize` exists
- [ ] `Vec::is_empty(&self) -> bool` exists
- [ ] `Deref<Target = [T]>` is implemented
- [ ] `DerefMut<Target = [T]>` is implemented

### Error Type
- [ ] Error type is `AllocError` (not `()`)
- [ ] Error can be converted to `kernel::error::Error`
- [ ] Error handling patterns match our implementation

## Known Differences

### Our Mock vs Real API

1. **Error Type**:
   - Mock: `Result<(), ()>`
   - Real: `Result<(), AllocError>`
   - **Impact**: Minimal - we don't use error values in our code

2. **Internal Implementation**:
   - Mock: Uses `alloc` crate internally
   - Real: Uses kernel allocators (kmalloc, etc.)
   - **Impact**: None for API compatibility

3. **Additional Methods**:
   - Real API may have additional methods we don't use
   - **Impact**: None - we only use methods we need

## Testing Workflow

1. **Run verification script** (from repository root):
   ```bash
   cd ../..  # Go to repository root
   ./scripts/verify-kernel-api.sh
   ```

2. **Review API comparison**:
   - Check extracted signatures
   - Compare with `../../dbc/src/kernel_mock.rs`
   - Note any discrepancies

3. **Test compilation** (if kernel environment available):
   ```bash
   # Add kernel dependency to Cargo.toml (uncomment or add):
   # [dependencies.kernel]
   # path = "/path/to/kernel/rust"
   
   # Then test compilation:
   cargo check --features kernel
   ```

4. **Run integration tests**:
   ```bash
   cargo test --features kernel --test kernel_integration_test
   ```

5. **Update mock if needed**:
   - If discrepancies found, update `dbc/src/kernel_mock.rs`
   - Update tests accordingly
   - Document changes

## Troubleshooting

### Kernel Crate Not Found

If `cargo check` fails with "can't find crate `kernel`":

1. Verify kernel source path is correct
2. Ensure kernel bindings are generated
3. Check `Cargo.toml` kernel dependency path

### Compilation Errors

If compilation fails:

1. Check kernel version compatibility
2. Verify Rust toolchain version matches kernel requirements
3. Check feature flags are correct

### API Mismatches

If API signatures don't match:

1. Check rust-for-linux branch/version
2. Verify we're looking at correct API files
3. Update mock to match real API
4. Document differences

## References

- [Rust-for-Linux Repository](https://github.com/Rust-for-Linux/linux)
- [Kernel Alloc Module](https://github.com/Rust-for-Linux/linux/tree/rust/rust/kernel/alloc)
- [dbc-rs Kernel Mock](../../dbc/src/kernel_mock.rs)
- [dbc-rs Repository Root](../../)

