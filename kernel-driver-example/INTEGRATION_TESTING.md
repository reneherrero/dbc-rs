# Integration Testing Guide

This guide explains how to test the kernel module compilation and runtime behavior.

## Overview

Integration testing for the kernel module involves:

1. **Compilation Testing**: Verify the module compiles with the kernel build system
2. **Runtime Testing**: Verify the module loads, runs, and unloads correctly

## Prerequisites

### For Compilation Tests

- Kernel source with Rust support (`CONFIG_RUST=y`)
- Rust toolchain (nightly)
- Kernel build system
- `KDIR` environment variable pointing to kernel source

### For Runtime Tests

- Built kernel module (`dbc_kernel_example.ko`)
- Root privileges
- Running kernel with Rust support

## Quick Start

### Compilation Tests

```bash
cd kernel-driver-example

# Test compilation (uses mock if kernel crate not available)
./test-compilation.sh

# Or with specific kernel:
KDIR=/path/to/kernel ./test-compilation.sh
```

### Runtime Tests

```bash
cd kernel-driver-example

# Build module first
make KDIR=/path/to/kernel

# Run runtime tests (requires root)
sudo ./test-runtime.sh
```

### Full Integration Test

```bash
cd kernel-driver-example

# Compilation + runtime tests
./test-integration.sh --runtime

# Or just compilation:
./test-integration.sh
```

## Test Scripts

### test-compilation.sh

Tests kernel module compilation:

1. **Cargo Check**: Verifies code compiles with mock kernel::alloc
2. **Cargo Build**: Builds the Rust library
3. **Kernel Module Build**: Builds the kernel module (if kernel crate available)

**Usage**:
```bash
./test-compilation.sh [KDIR=/path/to/kernel]
```

**What it tests**:
- ✅ Cargo compilation with kernel feature
- ✅ Library artifact creation
- ✅ Kernel module build (if kernel crate configured)

### test-runtime.sh

Tests kernel module runtime behavior:

1. **Module Loading**: Loads the module into kernel
2. **Module Verification**: Verifies module is loaded
3. **Kernel Logs**: Checks for initialization messages
4. **Device File**: Checks if device file is created
5. **Module Unloading**: Unloads the module
6. **Cleanup Verification**: Verifies module is unloaded

**Usage**:
```bash
sudo ./test-runtime.sh
```

**What it tests**:
- ✅ Module can be loaded
- ✅ Module appears in lsmod
- ✅ Kernel logs show initialization
- ✅ Module can be unloaded
- ✅ Cleanup messages in logs

### test-integration.sh

Runs both compilation and runtime tests:

**Usage**:
```bash
# Compilation only
./test-integration.sh

# Compilation + runtime
./test-integration.sh --runtime
```

## Using Makefile Targets

The Makefile includes test targets:

```bash
# Compilation tests
make test-compile

# Runtime tests (requires root)
make test-runtime

# All tests
make test
```

## Test Scenarios

### Scenario 1: Mock Testing (No Kernel Required)

```bash
cd kernel-driver-example
cargo check --features kernel
cargo build --release
```

This uses the mock `kernel::alloc` and doesn't require a kernel build environment.

### Scenario 2: Compilation with Kernel Crate

```bash
cd kernel-driver-example

# Set up kernel
export KDIR=/path/to/kernel
export ARCH=arm64

# Configure kernel with CONFIG_RUST=y
cd $KDIR
make ARCH=arm64 defconfig
# Edit .config: CONFIG_RUST=y

# Build kernel bindings
make ARCH=arm64 rustavailable

# Build module
cd /path/to/kernel-driver-example
make
```

### Scenario 3: Runtime Testing

```bash
# Build module
make KDIR=/path/to/kernel

# Load module
sudo insmod dbc_kernel_example.ko

# Check status
make status

# View logs
make logs

# Unload module
make remove
```

## Expected Results

### Compilation Tests

**With Mock** (no kernel crate):
- ✅ Cargo check: Passes
- ✅ Cargo build: Passes
- ⚠️  Kernel module build: Skipped (no kernel crate)

**With Kernel Crate**:
- ✅ Cargo check: Passes
- ✅ Cargo build: Passes
- ✅ Kernel module build: Passes
- ✅ Module file created: `dbc_kernel_example.ko`

### Runtime Tests

**Successful Test**:
- ✅ Module loads without errors
- ✅ Module appears in `lsmod`
- ✅ Kernel logs show initialization message
- ✅ Module unloads cleanly
- ✅ Cleanup message in logs

## Troubleshooting

### Compilation Fails

**Error**: "can't find crate `kernel`"
- **Solution**: Kernel crate not configured. Use mock for testing, or configure kernel crate in `Cargo.toml`

**Error**: "Kernel directory not found"
- **Solution**: Set `KDIR` environment variable or use `setup-ti-kernel.sh`

**Error**: "Rust target script not found"
- **Solution**: Kernel doesn't have Rust support. Enable `CONFIG_RUST=y`

### Runtime Tests Fail

**Error**: "Module not found"
- **Solution**: Build module first: `make KDIR=/path/to/kernel`

**Error**: "Operation not permitted"
- **Solution**: Run with root privileges: `sudo ./test-runtime.sh`

**Error**: "Module already loaded"
- **Solution**: Unload first: `sudo rmmod dbc_kernel_example`

**Error**: "Module in use"
- **Solution**: Check what's using it: `lsof | grep dbc_example` or `fuser /dev/dbc_example`

## CI/CD Integration

For CI/CD, you can run compilation tests without a full kernel environment:

```yaml
# .github/workflows/kernel-test.yml
- name: Test kernel module compilation
  run: |
    cd kernel-driver-example
    ./test-compilation.sh
```

Runtime tests require a kernel build environment and are typically run manually or in specialized CI environments.

## Next Steps

After integration tests pass:

1. **Implement File Operations**: Add character device file operations
2. **Add DBC Parsing**: Integrate dbc-rs parsing in kernel context
3. **User-Space Tests**: Create test programs to interact with the module
4. **Performance Testing**: Measure parsing performance in kernel context

## References

- [Kernel API Setup Guide](KERNEL_API_SETUP.md)
- [Kernel Driver Example README](README.md)
- [dbc-rs Kernel Feature](../dbc/README.md#kernel-support-experimental)

