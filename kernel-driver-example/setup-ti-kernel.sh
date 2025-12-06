#!/bin/bash
# Setup script for TI kernel development environment
#
# This script helps set up the environment for building the dbc-kernel-example
# module against TI's Linux kernel sources for AM6254/PocketBeagle 2.
# It creates a git submodule for the TI kernel repository.

set -e

echo "=== TI Kernel Setup for dbc-kernel-example ==="
echo ""

# Default paths (relative to repository root)
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
TI_KERNEL_SUBMODULE="${TI_KERNEL_SUBMODULE:-ti-linux-kernel}"
TI_KERNEL_DIR="${REPO_ROOT}/${TI_KERNEL_SUBMODULE}"
KERNEL_BRANCH="${KERNEL_BRANCH:-ti-linux-6.12.y}"
TI_KERNEL_URL="https://git.ti.com/git/ti-linux-kernel/ti-linux-kernel.git"

echo "Repository root: $REPO_ROOT"
echo "TI Kernel submodule: $TI_KERNEL_SUBMODULE"
echo "TI Kernel directory: $TI_KERNEL_DIR"
echo "Kernel branch: $KERNEL_BRANCH"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not in a git repository"
    echo "Initialize git repository first: git init"
    exit 1
fi

# Check if submodule already exists
if [ -f "$REPO_ROOT/.gitmodules" ] && grep -q "\[submodule \"$TI_KERNEL_SUBMODULE\"\]" "$REPO_ROOT/.gitmodules"; then
    echo "Submodule '$TI_KERNEL_SUBMODULE' already exists in .gitmodules"
    
    # Check if submodule directory exists and is initialized
    if [ -d "$TI_KERNEL_DIR" ] && [ -f "$TI_KERNEL_DIR/.git" ]; then
        echo "Submodule directory exists and is initialized"
        echo "Updating submodule..."
        cd "$REPO_ROOT"
        git submodule update --init --recursive "$TI_KERNEL_SUBMODULE"
        
        # Checkout specified branch if not already on it
        cd "$TI_KERNEL_DIR"
        CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "detached")
        if [ "$CURRENT_BRANCH" != "$KERNEL_BRANCH" ]; then
            echo "Checking out branch: $KERNEL_BRANCH"
            git checkout "$KERNEL_BRANCH" || {
                echo "Warning: Could not checkout $KERNEL_BRANCH"
                echo "Available branches:"
                git branch -r | head -10
            }
        fi
    else
        echo "Submodule directory missing or not initialized. Initializing..."
        cd "$REPO_ROOT"
        git submodule update --init --recursive "$TI_KERNEL_SUBMODULE"
    fi
else
    # Add submodule
    echo "Adding TI kernel as git submodule..."
    echo ""
    echo "This will:"
    echo "  1. Clone $TI_KERNEL_URL"
    echo "  2. Add it as a submodule at $TI_KERNEL_SUBMODULE"
    echo "  3. Checkout branch $KERNEL_BRANCH"
    echo ""
    read -p "Continue? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cd "$REPO_ROOT"
        git submodule add -b "$KERNEL_BRANCH" "$TI_KERNEL_URL" "$TI_KERNEL_SUBMODULE" || {
            echo "Error: Failed to add submodule"
            echo "If branch doesn't exist, you may need to:"
            echo "  1. Add submodule without branch: git submodule add $TI_KERNEL_URL $TI_KERNEL_SUBMODULE"
            echo "  2. cd $TI_KERNEL_SUBMODULE"
            echo "  3. git checkout $KERNEL_BRANCH"
            exit 1
        }
        echo "Submodule added successfully"
    else
        echo "Aborted"
        exit 1
    fi
fi

# Check if kernel has Rust support
echo "Checking for Rust support in kernel..."
if [ -f "$TI_KERNEL_DIR/.config" ]; then
    if grep -q "CONFIG_RUST=y" "$TI_KERNEL_DIR/.config"; then
        echo "✓ Rust support is enabled"
    else
        echo "⚠ Rust support is NOT enabled"
        echo "  You may need to enable CONFIG_RUST=y in kernel config"
    fi
else
    echo "⚠ Kernel not configured. Run:"
    echo "  cd $TI_KERNEL_DIR"
    echo "  make ARCH=arm64 defconfig"
    echo "  # Then enable CONFIG_RUST=y"
fi

# Verify submodule is ready
if [ ! -d "$TI_KERNEL_DIR" ]; then
    echo "Error: Submodule directory not found at $TI_KERNEL_DIR"
    exit 1
fi

echo ""
echo "=== Environment Variables ==="
echo ""
echo "Add these to your shell environment:"
echo ""
echo "export KDIR=$TI_KERNEL_DIR"
echo "export ARCH=arm64"
echo "export CROSS_COMPILE=aarch64-linux-gnu-"
echo ""
echo "Or create a .env file in this directory with:"
echo "KDIR=$TI_KERNEL_DIR"
echo "ARCH=arm64"
echo "CROSS_COMPILE=aarch64-linux-gnu-"
echo ""

# Update submodule info
echo "=== Submodule Information ==="
echo ""
echo "To update the submodule later:"
echo "  git submodule update --remote $TI_KERNEL_SUBMODULE"
echo ""
echo "To clone repository with submodules:"
echo "  git clone --recursive <repository-url>"
echo "  # OR after clone:"
echo "  git submodule update --init --recursive"
echo ""

