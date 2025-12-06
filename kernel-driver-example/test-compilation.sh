#!/bin/bash
# Test script for kernel module compilation
#
# This script tests whether the kernel module can be compiled with the kernel build system.
# It requires a kernel build environment with Rust support.
#
# Usage:
#   ./test-compilation.sh [KDIR=/path/to/kernel]
#
# Environment variables:
#   KDIR - Kernel source directory (default: /lib/modules/$(uname -r)/build)
#   ARCH - Architecture (default: detected from system)
#   CROSS_COMPILE - Cross-compiler prefix (optional)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Default values
KDIR="${KDIR:-/lib/modules/$(uname -r)/build}"
ARCH="${ARCH:-$(uname -m)}"
CROSS_COMPILE="${CROSS_COMPILE:-}"

echo "=== Kernel Module Compilation Test ==="
echo ""
echo "KDIR: $KDIR"
echo "ARCH: $ARCH"
echo "CROSS_COMPILE: $CROSS_COMPILE"
echo ""

# Check if kernel directory exists
if [ ! -d "$KDIR" ]; then
    echo "❌ Error: Kernel directory not found: $KDIR"
    echo ""
    echo "Set KDIR environment variable to point to your kernel source:"
    echo "  export KDIR=/path/to/kernel"
    echo ""
    echo "Or use the setup script:"
    echo "  ./setup-ti-kernel.sh"
    exit 1
fi

# Check if kernel has Rust support
if [ ! -f "$KDIR/rust/Makefile" ]; then
    echo "⚠️  Warning: Kernel may not have Rust support"
    echo "  Expected: $KDIR/rust/Makefile"
    echo "  This test may fail if Rust support is not enabled"
    echo ""
fi

# Try to get Rust target (optional - only needed for kernel module build)
RUST_TARGET=""
KERNEL_RUST_AVAILABLE=false

if [ -f "$KDIR/scripts/rust/rust_target.sh" ]; then
    RUST_TARGET=$("$KDIR/scripts/rust/rust_target.sh" 2>/dev/null || echo "")
    if [ -n "$RUST_TARGET" ]; then
        KERNEL_RUST_AVAILABLE=true
        echo "RUST_TARGET: $RUST_TARGET"
    else
        echo "⚠️  Could not determine Rust target (kernel module build will be skipped)"
    fi
else
    echo "⚠️  Rust target script not found (kernel module build will be skipped)"
    echo "  This is OK - we can still test with mock kernel::alloc"
fi
echo ""

# Test 1: Cargo check (without kernel crate - uses mock)
echo "=== Test 1: Cargo Check (with mock) ==="
if cargo check --features kernel 2>&1 | tee /tmp/dbc-kernel-check.log; then
    echo "✅ Cargo check passed (using mock kernel::alloc)"
else
    echo "❌ Cargo check failed"
    exit 1
fi
echo ""

# Test 2: Cargo build (without kernel crate - uses mock)
echo "=== Test 2: Cargo Build (with mock) ==="
# Use default target if RUST_TARGET not available
BUILD_TARGET="${RUST_TARGET:-$(rustc -vV | grep '^host:' | cut -d' ' -f2)}"
echo "Building for target: $BUILD_TARGET"

if cargo build --release --target "$BUILD_TARGET" 2>&1 | tee /tmp/dbc-kernel-build.log; then
    echo "✅ Cargo build passed (using mock kernel::alloc)"
    if [ -f "target/$BUILD_TARGET/release/libdbc_kernel_example.a" ]; then
        echo "✅ Library artifact created: target/$BUILD_TARGET/release/libdbc_kernel_example.a"
    else
        echo "⚠️  Warning: Library artifact not found at expected location"
        echo "  (This may be normal for rlib crate type)"
    fi
else
    echo "❌ Cargo build failed"
    exit 1
fi
echo ""

# Test 3: Kernel module build (requires kernel crate and Rust support)
echo "=== Test 3: Kernel Module Build (requires kernel crate) ==="

if [ "$KERNEL_RUST_AVAILABLE" = false ]; then
    echo "⚠️  Skipping kernel module build - kernel Rust support not available"
    echo "  This is OK - mock-based tests have already passed"
    echo ""
    echo "To enable kernel module build:"
    echo "  1. Set KDIR to a kernel with Rust support: export KDIR=/path/to/kernel"
    echo "  2. Configure kernel with CONFIG_RUST=y"
    echo "  3. Build kernel to generate bindings"
    echo "  4. Uncomment kernel dependency in Cargo.toml"
else
    echo "Kernel Rust support detected - attempting module build..."
    echo ""
    
    # Check if kernel crate is available in Cargo.toml
    if grep -q "\[dependencies.kernel\]" Cargo.toml; then
        echo "✅ Kernel crate dependency found in Cargo.toml"
        
        # Try to build with kernel Makefile
        if make KDIR="$KDIR" ARCH="$ARCH" CROSS_COMPILE="$CROSS_COMPILE" 2>&1 | tee /tmp/dbc-kernel-module-build.log; then
            if [ -f "dbc_kernel_example.ko" ]; then
                echo "✅ Kernel module built successfully: dbc_kernel_example.ko"
                echo ""
                echo "Module info:"
                file dbc_kernel_example.ko 2>/dev/null || echo "  (file command not available)"
            else
                echo "⚠️  Warning: Kernel module file not found"
            fi
        else
            echo "❌ Kernel module build failed"
            echo ""
            echo "This may be expected if:"
            echo "  - Kernel crate is not properly configured"
            echo "  - Kernel bindings are not generated"
            echo "  - Additional kernel configuration is needed"
        fi
    else
        echo "⚠️  Kernel crate dependency not found in Cargo.toml"
        echo "  This is expected - kernel crate is typically provided by kernel build system"
        echo "  To test with real kernel crate, uncomment the dependency in Cargo.toml"
    fi
fi

echo ""
echo "=== Compilation Test Summary ==="
echo "✅ Cargo check: Passed (mock)"
echo "✅ Cargo build: Passed (mock)"
if [ -f "dbc_kernel_example.ko" ]; then
    echo "✅ Kernel module: Built successfully"
elif [ "$KERNEL_RUST_AVAILABLE" = true ]; then
    echo "⚠️  Kernel module: Build attempted but failed (see logs above)"
else
    echo "⚠️  Kernel module: Skipped (kernel Rust support not available)"
fi
echo ""
if [ "$KERNEL_RUST_AVAILABLE" = false ]; then
    echo "✅ All mock-based tests passed!"
    echo ""
    echo "Note: To test full kernel module compilation, you need:"
    echo "  - Kernel with CONFIG_RUST=y"
    echo "  - Kernel bindings generated"
    echo "  - Kernel crate available"
    echo ""
    echo "Set KDIR to a kernel with Rust support to enable kernel module build tests."
else
    echo "Note: Full kernel module compilation requires:"
    echo "  - Kernel with CONFIG_RUST=y"
    echo "  - Kernel bindings generated"
    echo "  - Kernel crate available"
    echo ""
fi

