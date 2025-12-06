# BitBake recipe for dbc-kernel-example kernel module
#
# This recipe builds the dbc-kernel-example Rust kernel module for
# PocketBeagle 2 (AM6254) using meta-ti kernel sources.
#
# Prerequisites:
# - meta-ti layer in Yocto build
# - Kernel with Rust support enabled (CONFIG_RUST=y)
# - Rust toolchain for kernel (provided by kernel build system)

SUMMARY = "DBC parsing kernel module using dbc-rs"
DESCRIPTION = "Example Linux kernel driver demonstrating dbc-rs usage in a Rust kernel module for PocketBeagle 2"
LICENSE = "GPL-2.0-only"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/GPL-2.0-only;md5=801f80980d171dd6425610833a22dbe6"

# Inherit kernel module class
inherit module

# Module name
MODULE_NAME = "dbc_kernel_example"

# Source files
# Note: Adjust SRC_URI based on where recipe is placed in your layer
# Option 1: If recipe is in layer root, use relative path
SRC_URI = "file://${TOPDIR}/../kernel-driver-example"

# Option 2: If using git repository
# SRC_URI = "git://github.com/your-org/dbc-rs.git;protocol=https;branch=main;subpath=kernel-driver-example"
# SRCREV = "${AUTOREV}"

# Source directory (relative to recipe location)
S = "${WORKDIR}/kernel-driver-example"

# Kernel source directory (provided by kernel module class)
# This will point to the kernel built by meta-ti
KERNEL_SRC = "${STAGING_KERNEL_DIR}"

# Rust target for ARM64 (AM6254)
RUST_TARGET = "aarch64-unknown-none"

# Build dependencies
DEPENDS += "rust-native"
DEPENDS += "virtual/kernel"
DEPENDS += "rust-llvm-native"

# Module build
do_compile() {
    # Build Rust library
    cd ${S}
    
    # Set up Rust environment for kernel
    export RUST_TARGET="${RUST_TARGET}"
    export KDIR="${KERNEL_SRC}"
    export ARCH="arm64"
    export CROSS_COMPILE="${TARGET_PREFIX}"
    
    # Build Rust library
    cargo build --release \
        --target "${RUST_TARGET}" \
        --target-dir "${B}/target" \
        || bbnote "Rust build may require kernel Rust support"
    
    # Build kernel module
    oe_runmake -C "${KERNEL_SRC}" \
        M="${S}" \
        ARCH=arm64 \
        CROSS_COMPILE="${TARGET_PREFIX}" \
        modules
}

do_install() {
    # Install kernel module
    install -d ${D}${nonarch_base_libdir}/modules/${KERNEL_VERSION}/extra
    install -m 0644 ${B}/${MODULE_NAME}.ko \
        ${D}${nonarch_base_libdir}/modules/${KERNEL_VERSION}/extra/
}

# Module files
FILES:${PN} += "${nonarch_base_libdir}/modules/${KERNEL_VERSION}/extra/${MODULE_NAME}.ko"

# Auto-load module (optional)
KERNEL_MODULE_AUTOLOAD += "${MODULE_NAME}"

# Module dependencies (if needed)
# KERNEL_MODULE_PROBECONF += "${MODULE_NAME}"
# module_conf_${MODULE_NAME} = "options ${MODULE_NAME} param=value"

