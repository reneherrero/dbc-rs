#!/bin/bash
# Integration test script for kernel module
#
# This script runs both compilation and runtime tests.
# It provides a comprehensive test of the kernel module integration.
#
# Usage:
#   ./test-integration.sh [KDIR=/path/to/kernel]
#
# For runtime tests (requires root):
#   sudo ./test-integration.sh --runtime

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

RUNTIME_TESTS=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --runtime)
            RUNTIME_TESTS=true
            shift
            ;;
        *)
            # Treat as KDIR
            export KDIR="$1"
            shift
            ;;
    esac
done

echo "=== Kernel Module Integration Test ==="
echo ""

# Run compilation tests
echo "Running compilation tests..."
if ./test-compilation.sh; then
    echo "✅ Compilation tests passed"
else
    echo "❌ Compilation tests failed"
    exit 1
fi
echo ""

# Run runtime tests if requested
if [ "$RUNTIME_TESTS" = true ]; then
    echo "Running runtime tests..."
    if sudo ./test-runtime.sh; then
        echo "✅ Runtime tests passed"
    else
        echo "❌ Runtime tests failed"
        exit 1
    fi
else
    echo "Skipping runtime tests (use --runtime to enable)"
    echo "  Note: Runtime tests require root privileges"
fi

echo ""
echo "=== Integration Test Complete ==="
echo "✅ All tests passed"
echo ""

