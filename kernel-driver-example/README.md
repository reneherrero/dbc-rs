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

1. **Linux kernel with Rust support**
   - Kernel version: 6.1+ with Rust support enabled
   - Rust-for-Linux toolchain installed
   - Kernel source tree available

2. **Rust toolchain**
   - Rust nightly (required for kernel development)
   - rust-for-linux bindings

3. **Build tools**
   - `make`
   - `cargo`
   - Kernel build system

## Build Methods

### Method 1: Yocto/meta-ti (Recommended for PocketBeagle 2)

See [yocto/README.md](yocto/README.md) for Yocto integration instructions.

Quick start:
```bash
# Add recipe to your Yocto layer
cp yocto/dbc-kernel-example.bb <your-layer>/recipes-kernel/dbc-kernel-example/

# Build
bitbake dbc-kernel-example
```

### Method 2: Standalone Kernel Build

### 1. Get TI Kernel Sources

Clone the TI Linux kernel repository:

```bash
git clone https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git
cd ti-linux-kernel
git checkout ti-linux-6.12.y  # Or appropriate version for AM6254
```

**REQUIRES VERIFICATION**: Check if TI kernel has Rust support enabled. If not, you may need to:
- Apply rust-for-linux patches to TI kernel, OR
- Use rust-for-linux upstream kernel and apply TI patches

### 2. Configure Kernel

Ensure your kernel has Rust support enabled:

```bash
cd ti-linux-kernel
make ARCH=arm64 defconfig  # Or use your board config
# Edit .config or use menuconfig:
# CONFIG_RUST=y
# CONFIG_RUST_IS_AVAILABLE=y
```

### 3. Set Kernel Source Path

```bash
export KDIR=/path/to/ti-linux-kernel
export ARCH=arm64
export CROSS_COMPILE=aarch64-linux-gnu-  # For cross-compilation
```

### 4. Build Module

```bash
cd kernel-driver-example
make
```

### 4. Load Module

```bash
sudo make install
```

### 5. Check Status

```bash
make status
make logs
```

### 6. Unload Module

```bash
sudo make remove
```

## Project Structure

```
kernel-driver-example/
├── Cargo.toml          # Rust project configuration
├── Makefile            # Kernel build integration (standalone)
├── kernel_module.c     # C glue code (minimal)
├── setup-ti-kernel.sh  # Helper script for TI kernel setup
├── yocto/              # Yocto/bitbake recipes
│   ├── dbc-kernel-example.bb        # Main recipe
│   ├── dbc-kernel-example_%.bbappend # Append file template
│   └── README.md                    # Yocto integration guide
├── src/
│   └── lib.rs         # Rust kernel module implementation
└── README.md          # This file
```

## Quick Start with TI Kernel

Run the setup script:

```bash
./setup-ti-kernel.sh
```

This will:
1. Check for TI kernel sources
2. Offer to clone if missing
3. Verify Rust support configuration
4. Display required environment variables
```

## Implementation Notes

### Dependencies

- `dbc-rs`: DBC parsing library (kernel feature enabled)
- `kernel`: Rust-for-Linux kernel bindings

### Error Handling

All allocation operations in kernel context return `Result` types. The `dbc-rs` crate with `kernel` feature handles this:

```rust
use dbc_rs::Dbc;

let dbc = Dbc::parse_bytes(data)?;  // Returns Result
```

### Memory Allocation

The kernel feature uses `kernel::alloc` instead of standard `alloc`:
- `String::try_from()` instead of `String::from()`
- `Vec::try_push()` instead of `Vec::push()`
- All operations are fallible and return `Result`

## Testing

See [INTEGRATION_TESTING.md](INTEGRATION_TESTING.md) for comprehensive testing guide.

### Quick Test

```bash
# Compilation test (works with mock, no kernel required)
./test-compilation.sh

# Runtime test (requires built module and root)
sudo ./test-runtime.sh

# Full integration test
./test-integration.sh --runtime
```

## Limitations

1. **Experimental**: The `kernel` feature in `dbc-rs` is experimental
2. **Skeleton Only**: This example provides basic structure, not full functionality
3. **No File Operations**: Character device file operations are not yet implemented
4. **Testing**: Full kernel module testing requires kernel build environment (see INTEGRATION_TESTING.md)

## TI Kernel Sources

### Official TI Kernel Repository

```bash
# Clone TI kernel
git clone https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git

# Checkout branch for your kernel version
git checkout ti-linux-6.12.y  # Example: adjust version as needed

# For AM6254/PocketBeagle 2, check branches:
git branch -a | grep am62
```

### Kernel Branches

TI maintains branches for different kernel versions:
- `ti-linux-6.12.y` - Kernel 6.12.x
- `ti-linux-6.1.y` - Kernel 6.1.x
- Check `git branch -a` for available branches

### Device Tree Files

AM6254 device tree files are located at:
```
arch/arm64/boot/dts/ti/k3-am62-pocketbeagle2.dts
```

### Rust Support Status

**REQUIRES VERIFICATION**: TI kernel may not have Rust support enabled by default. Options:

1. **Check if TI kernel has Rust support**:
   ```bash
   cd ti-linux-kernel
   grep CONFIG_RUST .config
   ```

2. **If not available**, you may need to:
   - Apply rust-for-linux patches to TI kernel
   - Use rust-for-linux upstream and apply TI-specific patches
   - Request Rust support from TI

## References

- [TI Linux Kernel Repository](https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git)
- [Rust for Linux](https://rust-for-linux.com/)
- [dbc-rs Documentation](../dbc/README.md)
- [PocketBeagle 2](https://github.com/beagleboard/pocketbeagle-2)
- [AM6254 SoC](https://www.ti.com/product/AM625#tech-docs)
- [Testing Against Kernel API](KERNEL_API_SETUP.md) - Guide for testing with rust-for-linux kernel alloc API

## License

GPL-2.0 (required for kernel modules)

