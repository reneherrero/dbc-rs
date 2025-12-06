#!/bin/bash
# Test script for kernel module runtime behavior
#
# This script tests loading, using, and unloading the kernel module.
# It requires root privileges and a running kernel with the module loaded.
#
# Usage:
#   sudo ./test-runtime.sh
#
# Prerequisites:
#   - Kernel module must be built (dbc_kernel_example.ko)
#   - Root privileges
#   - Kernel with Rust support

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODULE_NAME="dbc_kernel_example"
MODULE_FILE="${MODULE_NAME}.ko"

echo "=== Kernel Module Runtime Test ==="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ Error: This script must be run as root"
    echo "  Usage: sudo ./test-runtime.sh"
    exit 1
fi

# Check if module file exists
if [ ! -f "$MODULE_FILE" ]; then
    echo "❌ Error: Kernel module not found: $MODULE_FILE"
    echo ""
    echo "Build the module first:"
    echo "  make KDIR=/path/to/kernel"
    exit 1
fi

echo "Module file: $MODULE_FILE"
echo ""

# Test 1: Check module dependencies
echo "=== Test 1: Module Dependencies ==="
if command -v modinfo >/dev/null 2>&1; then
    echo "Module information:"
    modinfo "$MODULE_FILE" || echo "  (modinfo failed)"
    echo ""
else
    echo "⚠️  modinfo not available"
    echo ""
fi

# Test 2: Load module
echo "=== Test 2: Load Module ==="
if lsmod | grep -q "^${MODULE_NAME}"; then
    echo "⚠️  Module already loaded, removing first..."
    rmmod "$MODULE_NAME" || true
    sleep 1
fi

echo "Loading module: $MODULE_FILE"
if insmod "$MODULE_FILE" 2>&1 | tee /tmp/dbc-kernel-load.log; then
    echo "✅ Module loaded successfully"
else
    echo "❌ Failed to load module"
    echo ""
    echo "Check kernel logs:"
    dmesg | tail -20
    exit 1
fi
echo ""

# Test 3: Verify module is loaded
echo "=== Test 3: Verify Module Loaded ==="
if lsmod | grep -q "^${MODULE_NAME}"; then
    echo "✅ Module is loaded:"
    lsmod | grep "^${MODULE_NAME}"
else
    echo "❌ Module not found in lsmod"
    exit 1
fi
echo ""

# Test 4: Check kernel logs
echo "=== Test 4: Kernel Logs ==="
echo "Recent kernel messages:"
dmesg | tail -10 | grep -i dbc || echo "  (No dbc-related messages found)"
echo ""

# Test 5: Check device file (if created)
echo "=== Test 5: Device File ==="
if [ -e "/dev/dbc_example" ]; then
    echo "✅ Device file exists: /dev/dbc_example"
    ls -l /dev/dbc_example
else
    echo "⚠️  Device file not found: /dev/dbc_example"
    echo "  (This is expected if file operations are not implemented)"
fi
echo ""

# Test 6: Unload module
echo "=== Test 6: Unload Module ==="
echo "Unloading module..."
if rmmod "$MODULE_NAME" 2>&1 | tee /tmp/dbc-kernel-unload.log; then
    echo "✅ Module unloaded successfully"
else
    echo "❌ Failed to unload module"
    echo ""
    echo "Check if module is in use:"
    lsmod | grep "^${MODULE_NAME}"
    exit 1
fi
echo ""

# Test 7: Verify module is unloaded
echo "=== Test 7: Verify Module Unloaded ==="
if lsmod | grep -q "^${MODULE_NAME}"; then
    echo "❌ Module still loaded"
    exit 1
else
    echo "✅ Module successfully unloaded"
fi
echo ""

# Test 8: Check cleanup logs
echo "=== Test 8: Cleanup Logs ==="
echo "Kernel messages after unload:"
dmesg | tail -5 | grep -i dbc || echo "  (No dbc-related messages found)"
echo ""

echo "=== Runtime Test Summary ==="
echo "✅ Module loaded: Success"
echo "✅ Module verified: Success"
echo "✅ Module unloaded: Success"
echo ""
echo "Note: Full functionality testing requires:"
echo "  - File operations implementation"
echo "  - DBC parsing integration"
echo "  - User-space test programs"
echo ""

