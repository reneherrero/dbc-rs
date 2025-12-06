# dbc-kernel-example

Example Linux kernel driver demonstrating `dbc-rs` usage in a Rust kernel module.

## Status

**EXPERIMENTAL**: This is a skeleton implementation. Not production-ready.

## Platform

- **Development Board**: PocketBeagle 2 Rev A1
- **SoC**: Texas Instruments AM6254
- **Kernel**: Linux with Rust support (rust-for-linux)
- **Build System**: Yocto Project with meta-ti layer

## Prerequisites

- Linux kernel 6.1+ with Rust support (`CONFIG_RUST=y`)
- Rust nightly toolchain
- Kernel source tree with Rust bindings
- Build tools: `make`, `cargo`

### Build Dependencies

**Essential build tools (Debian/Ubuntu):**
```bash
sudo apt-get install -y \
    flex bison libelf-dev libncurses-dev bc libssl-dev \
    build-essential gcc-aarch64-linux-gnu binutils-aarch64-linux-gnu
```

**Rust toolchain (for kernel Rust support):**
```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
```

## Build Methods

### Yocto/meta-ti (Recommended)

```bash
# Add recipe to your Yocto layer
cp yocto/dbc-kernel-example.bb <your-layer>/recipes-kernel/dbc-kernel-example/
bitbake dbc-kernel-example
```

See [yocto/README.md](yocto/README.md) for details.

### Standalone Build

1. **Get TI Kernel Sources**:
   ```bash
   git clone https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git
   cd ti-linux-kernel
   git checkout ti-linux-6.12.y
   ```

2. **Configure Kernel**:
   ```bash
   cd ti-linux-kernel
   make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- defconfig
   echo "CONFIG_RUST=y" >> .config
   make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- olddefconfig
   ```

3. **Build Kernel** (optional, for full kernel build):
   ```bash
   # Full build (takes 30+ minutes)
   make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- -j$(nproc)
   
   # Or just prepare headers (faster, for module building)
   make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- prepare
   ```

4. **Build Module**:
   ```bash
   export KDIR=/path/to/ti-linux-kernel
   export ARCH=arm64
   export CROSS_COMPILE=aarch64-linux-gnu-
   cd kernel-driver-example
   make
   ```

5. **Load/Unload**:
   ```bash
   sudo make install    # Load module
   make status          # Check status
   sudo make remove     # Unload module
   ```

**Note**: Verify TI kernel has Rust support. If not, apply rust-for-linux patches or use rust-for-linux upstream with TI patches.

## Project Structure

```
kernel-driver-example/
├── Cargo.toml          # Rust project configuration
├── Makefile            # Kernel build integration
├── kernel_module.c     # C glue code
├── setup-ti-kernel.sh  # TI kernel setup helper
├── yocto/              # Yocto recipes
├── src/lib.rs          # Rust kernel module
└── test-*.sh           # Test scripts
```

## Testing

### Quick Tests

```bash
# Compilation test (uses mock, no kernel required)
./test-compilation.sh

# Runtime test (requires built module and root)
sudo ./test-runtime.sh

# Full integration test
./test-integration.sh --runtime
```

### API Verification

Verify compatibility with rust-for-linux kernel alloc API:

```bash
# From repository root
cd ../..
./scripts/verify-kernel-api.sh
```

This compares our mock implementation with the real kernel API.

### Test Scripts

- **test-compilation.sh**: Tests Cargo compilation and kernel module build
- **test-runtime.sh**: Tests module loading, verification, and unloading
- **test-integration.sh**: Runs both compilation and runtime tests

### Testing with Real Kernel Crate

1. **Set up kernel source**:
   ```bash
   ./setup-ti-kernel.sh  # Or set KDIR manually
   ```

2. **Update Cargo.toml** (uncomment or add):
   ```toml
   [dependencies]
   kernel = { path = "/path/to/kernel/rust" }
   # Or use git:
   # kernel = { git = "https://github.com/Rust-for-Linux/linux.git", package = "kernel", branch = "rust" }
   ```

3. **Test compilation**:
   ```bash
   cargo check --features kernel
   ```

### Known Differences (Mock vs Real API)

- **Error Type**: Mock uses `Result<(), ()>`, real uses `Result<(), AllocError>`
- **Impact**: Minimal - we don't use error values in our code
- **Internal Implementation**: Mock uses `alloc` crate, real uses kernel allocators
- **Impact**: None for API compatibility

## Implementation Notes

### Dependencies

- `dbc-rs`: DBC parsing library (kernel feature enabled)
- `kernel`: Rust-for-Linux kernel bindings

### Error Handling

All allocation operations return `Result` types:

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse_bytes(data)?;  // Returns Result
```

### Memory Allocation

Kernel feature uses `kernel::alloc`:
- `String::try_from()` instead of `String::from()`
- `Vec::try_push()` instead of `Vec::push()`
- All operations are fallible and return `Result`

## Environment Variables

Set these for building kernel modules:

```bash
export KDIR=/path/to/ti-linux-kernel
export ARCH=arm64
export CROSS_COMPILE=aarch64-linux-gnu-
```

## Troubleshooting

**"flex: not found"**: Install dependencies: `sudo apt-get install -y flex bison libelf-dev libncurses-dev`

**"can't find crate `kernel`"**: Use mock for testing, or configure kernel crate in `Cargo.toml`

**"Kernel directory not found"**: Set `KDIR` environment variable or use `setup-ti-kernel.sh`

**"Rust target script not found"**: Enable `CONFIG_RUST=y` in kernel config and run `make prepare`

**"No rule to make target 'rust/alloc/alloc.rs'"**: Ensure `CONFIG_RUST=y` is set and run `make prepare`

**"Module not found"**: Build module first: `make KDIR=/path/to/kernel`

**"Operation not permitted"**: Run with root privileges: `sudo ./test-runtime.sh`

**"Cross-compiler not found"**: Install: `sudo apt-get install -y gcc-aarch64-linux-gnu`

## TI Kernel Sources

### Repository

```bash
git clone https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git
git checkout ti-linux-6.12.y  # Adjust version as needed
```

### Device Tree

AM6254 device tree: `arch/arm64/boot/dts/ti/k3-am62-pocketbeagle2.dts`

### Rust Support

**Verify Rust is enabled:**
```bash
grep CONFIG_RUST .config
ls -la rust/alloc/  # Should exist if Rust is enabled
```

**If not available**, you may need to:
- Apply rust-for-linux patches to TI kernel
- Use rust-for-linux upstream with TI patches

## Limitations

1. **Experimental**: The `kernel` feature in `dbc-rs` is experimental
2. **Skeleton Only**: Basic structure, not full functionality
3. **No File Operations**: Character device file operations not yet implemented

## References

- [TI Linux Kernel Repository](https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git)
- [Rust for Linux](https://rust-for-linux.com/)
- [dbc-rs Documentation](../dbc/README.md)
- [PocketBeagle 2](https://github.com/beagleboard/pocketbeagle-2)
- [AM6254 SoC](https://www.ti.com/product/AM625#tech-docs)

## License

GPL-2.0 (required for kernel modules)
