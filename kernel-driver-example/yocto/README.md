# Yocto Integration for dbc-kernel-example

This directory contains Yocto/bitbake recipes for building the dbc-kernel-example kernel module for PocketBeagle 2 using meta-ti.

## Prerequisites

1. **Yocto Project** (Kirkstone or later recommended)
2. **meta-ti layer** - TI-specific Yocto layer
3. **meta-rust** (optional) - Rust support layer for Yocto
4. **Kernel with Rust support** - CONFIG_RUST=y in kernel config

## Setup

### 1. Create Yocto Layer (if needed)

```bash
# Create a custom layer for your project
bitbake-layers create-layer ../meta-dbc-example
bitbake-layers add-layer ../meta-dbc-example
```

### 2. Add Recipe to Layer

Copy the recipe to your layer:

```bash
mkdir -p <your-layer>/recipes-kernel/dbc-kernel-example
cp yocto/dbc-kernel-example.bb <your-layer>/recipes-kernel/dbc-kernel-example/
```

### 5. Enable Rust in Kernel

Create a kernel configuration fragment or append file:

```bash
# Create kernel config fragment
mkdir -p <your-layer>/recipes-kernel/linux/files
cat > <your-layer>/recipes-kernel/linux/files/rust.cfg <<EOF
CONFIG_RUST=y
CONFIG_RUST_IS_AVAILABLE=y
EOF
```

Create kernel append file:

```bash
# <your-layer>/recipes-kernel/linux/linux-ti_%.bbappend
FILESEXTRAPATHS:prepend := "${THISDIR}/files:"
SRC_URI += "file://rust.cfg"
```

Or add to `local.conf`:

```bitbake
# Enable Rust in kernel
KERNEL_FEATURES:append = " cfg/rust.scc"
```

**REQUIRES VERIFICATION**: Check if meta-ti kernel recipe supports Rust. You may need to:
- Apply rust-for-linux patches to meta-ti kernel recipe
- Use a custom kernel recipe with Rust support

### 4. Configure IDE (Optional but Recommended)

See [IDE Configuration](#ide-configuration) section below for VSCode and other editor setup.

### 5. Build Module

```bash
bitbake dbc-kernel-example
```

## Integration with meta-ti

### PocketBeagle 2 Machine Configuration

The PocketBeagle 2 is typically configured in meta-ti. Ensure your `MACHINE` is set:

```bash
MACHINE = "beagleplay"  # Or appropriate machine for PocketBeagle 2
```

### Kernel Recipe

meta-ti provides kernel recipes. The module will build against the kernel provided by:

```bitbake
PREFERRED_PROVIDER_virtual/kernel = "linux-ti"
```

### Device Tree

PocketBeagle 2 device tree is in meta-ti:
- `arch/arm64/boot/dts/ti/k3-am62-pocketbeagle2.dts`

## Build Output

The built module will be in:

```
tmp/work/<machine>/dbc-kernel-example/<version>/image/lib/modules/<kernel-version>/extra/dbc_kernel_example.ko
```

## Installation

The module is automatically included in the rootfs if added to `IMAGE_INSTALL`:

```bitbake
IMAGE_INSTALL:append = " dbc-kernel-example"
```

## Manual Installation

After building, copy the module to target:

```bash
# From build directory
scp tmp/work/*/dbc-kernel-example/*/image/lib/modules/*/extra/dbc_kernel_example.ko root@<target-ip>:/lib/modules/$(uname -r)/extra/

# On target
insmod /lib/modules/$(uname -r)/extra/dbc_kernel_example.ko
```

## Troubleshooting

### Rust Support Not Available

If kernel doesn't have Rust support:

1. Check kernel config: `bitbake -e virtual/kernel | grep CONFIG_RUST`
2. Add Rust support to kernel recipe
3. Verify rust-for-linux patches are applied

### Cross-Compilation Issues

Ensure Rust target is installed:

```bash
rustup target add aarch64-unknown-none
```

### Kernel Module Build Fails

Check kernel source path:

```bash
bitbake -e dbc-kernel-example | grep KERNEL_SRC
```

## References

- [Yocto Project Documentation](https://docs.yoctoproject.org/)
- [meta-ti Layer](https://git.yoctoproject.org/meta-ti)
- [Rust for Linux](https://rust-for-linux.com/)
- [PocketBeagle 2](https://github.com/beagleboard/pocketbeagle-2)

