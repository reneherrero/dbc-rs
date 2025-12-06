#!/bin/bash
# Script to verify dbc-rs kernel mock against rust-for-linux source code
#
# This script:
# 1. Clones rust-for-linux repository (if not present)
# 2. Extracts kernel alloc API signatures
# 3. Compares with our mock implementation
# 4. Reports any discrepancies

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_FOR_LINUX_DIR="${RUST_FOR_LINUX_DIR:-$REPO_ROOT/rust-for-linux}"
RUST_FOR_LINUX_URL="https://github.com/Rust-for-Linux/linux.git"
RUST_FOR_LINUX_BRANCH="${RUST_FOR_LINUX_BRANCH:-rust}"

echo "=== Rust-for-Linux Kernel Alloc API Verification ==="
echo ""
echo "Repository root: $REPO_ROOT"
echo "Rust-for-Linux directory: $RUST_FOR_LINUX_DIR"
echo "Branch: $RUST_FOR_LINUX_BRANCH"
echo ""

# Check if rust-for-linux directory exists
if [ ! -d "$RUST_FOR_LINUX_DIR" ]; then
    echo "Rust-for-Linux directory not found at: $RUST_FOR_LINUX_DIR"
    echo ""
    read -p "Clone rust-for-linux repository? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Cloning rust-for-linux repository..."
        echo "Note: This is a large repository and may take several minutes"
        git clone --depth 1 --branch "$RUST_FOR_LINUX_BRANCH" "$RUST_FOR_LINUX_URL" "$RUST_FOR_LINUX_DIR" || {
            echo "Error: Failed to clone repository"
            echo "You may need to:"
            echo "  1. Check your internet connection"
            echo "  2. Verify the branch name: $RUST_FOR_LINUX_BRANCH"
            echo "  3. Clone manually: git clone -b $RUST_FOR_LINUX_BRANCH $RUST_FOR_LINUX_URL $RUST_FOR_LINUX_DIR"
            exit 1
        }
        echo "Repository cloned successfully"
    else
        echo "Aborted. Set RUST_FOR_LINUX_DIR to point to an existing rust-for-linux repository"
        exit 1
    fi
else
    echo "Rust-for-Linux directory found"
    cd "$RUST_FOR_LINUX_DIR"
    
    # Check if it's a git repository
    if [ -d ".git" ]; then
        echo "Updating repository..."
        git fetch origin "$RUST_FOR_LINUX_BRANCH" || echo "Warning: Could not fetch updates"
        CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "detached")
        if [ "$CURRENT_BRANCH" != "$RUST_FOR_LINUX_BRANCH" ]; then
            echo "Checking out branch: $RUST_FOR_LINUX_BRANCH"
            git checkout "$RUST_FOR_LINUX_BRANCH" || echo "Warning: Could not checkout branch"
        fi
    fi
fi

# Verify rust-for-linux directory structure
if [ ! -d "$RUST_FOR_LINUX_DIR/rust" ]; then
    echo "Error: rust-for-linux directory does not contain 'rust' subdirectory"
    echo "This may not be a valid rust-for-linux repository"
    exit 1
fi

echo ""
echo "=== Extracting Kernel Alloc API Signatures ==="
echo ""

# Create temporary directory for extracted API info
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Extract String API
echo "Extracting String API..."
STRING_API_FILE="$TEMP_DIR/string_api.txt"
# Try multiple possible locations
STRING_LOCATIONS=(
    "$RUST_FOR_LINUX_DIR/rust/kernel/alloc/string.rs"
    "$RUST_FOR_LINUX_DIR/rust/alloc/string.rs"
)

STRING_FOUND=false
for LOC in "${STRING_LOCATIONS[@]}"; do
    if [ -f "$LOC" ]; then
        grep -E "(pub fn|pub struct String|impl.*String)" "$LOC" > "$STRING_API_FILE" || true
        echo "  Found String API in: ${LOC#$RUST_FOR_LINUX_DIR/}"
        STRING_FOUND=true
        break
    fi
done

if [ "$STRING_FOUND" = false ]; then
    echo "  Warning: String API file not found in expected locations"
    echo "  Searching for String definition..."
    find "$RUST_FOR_LINUX_DIR/rust" -name "*.rs" -type f -exec grep -l "pub struct String" {} \; | head -5
fi

# Extract Vec API
echo "Extracting Vec API..."
VEC_API_FILE="$TEMP_DIR/vec_api.txt"
# Try multiple possible locations
VEC_LOCATIONS=(
    "$RUST_FOR_LINUX_DIR/rust/kernel/alloc/vec.rs"
    "$RUST_FOR_LINUX_DIR/rust/kernel/alloc/vec/mod.rs"
    "$RUST_FOR_LINUX_DIR/rust/alloc/vec/mod.rs"
    "$RUST_FOR_LINUX_DIR/rust/alloc/vec.rs"
)

VEC_FOUND=false
for LOC in "${VEC_LOCATIONS[@]}"; do
    if [ -f "$LOC" ]; then
        grep -E "(pub fn|pub struct Vec|impl.*Vec)" "$LOC" > "$VEC_API_FILE" || true
        echo "  Found Vec API in: ${LOC#$RUST_FOR_LINUX_DIR/}"
        VEC_FOUND=true
        break
    fi
done

if [ "$VEC_FOUND" = false ]; then
    echo "  Warning: Vec API file not found in expected locations"
    echo "  Searching for Vec definition..."
    find "$RUST_FOR_LINUX_DIR/rust" -name "*.rs" -type f -exec grep -l "pub struct Vec" {} \; | head -5
fi

# Display extracted APIs
echo ""
echo "=== String API (from rust-for-linux) ==="
if [ -s "$STRING_API_FILE" ]; then
    cat "$STRING_API_FILE"
else
    echo "  No String API found"
fi

echo ""
echo "=== Vec API (from rust-for-linux) ==="
if [ -s "$VEC_API_FILE" ]; then
    cat "$VEC_API_FILE"
else
    echo "  No Vec API found"
fi

# Compare with our mock
echo ""
echo "=== Comparison with dbc-rs Mock ==="
echo ""
echo "Our mock implements:"
echo "  String::try_from(s: &str) -> Result<Self, ()>"
echo "  String::try_push_str(&mut self, s: &str) -> Result<(), ()>"
echo "  String::as_str(&self) -> &str"
echo ""
echo "  Vec::new() -> Self"
echo "  Vec::try_push(&mut self, item: T) -> Result<(), ()>"
echo "  Vec::try_extend_from_slice(&mut self, other: &[T]) -> Result<(), ()> where T: Clone"
echo "  Vec::len(&self) -> usize"
echo "  Vec::is_empty(&self) -> bool"
echo ""

# Check for key methods
echo "Verifying key methods exist in rust-for-linux..."
MISSING_METHODS=0

if [ -f "$STRING_API_FILE" ]; then
    if ! grep -q "try_from\|try_push_str" "$STRING_API_FILE"; then
        echo "  ⚠ Warning: try_from or try_push_str not found in String API"
        MISSING_METHODS=1
    fi
fi

if [ -f "$VEC_API_FILE" ]; then
    if ! grep -q "try_push\|try_extend" "$VEC_API_FILE"; then
        echo "  ⚠ Warning: try_push or try_extend not found in Vec API"
        MISSING_METHODS=1
    fi
fi

if [ $MISSING_METHODS -eq 0 ]; then
    echo "  ✓ Key methods found"
fi

echo ""
echo "=== Summary ==="
echo ""
echo "To fully verify API compatibility:"
echo "  1. Review the extracted API signatures above"
echo "  2. Compare with dbc/src/kernel_mock.rs"
echo "  3. Check error types (our mock uses (), real API may use AllocError)"
echo "  4. Test compilation with actual kernel crate (requires kernel build environment)"
echo ""
echo "For detailed API documentation, see:"
echo "  $RUST_FOR_LINUX_DIR/rust/kernel/alloc/"
echo ""
echo "To test with actual kernel crate, set up kernel build environment:"
echo "  1. Configure kernel with CONFIG_RUST=y"
echo "  2. Build kernel to generate bindings"
echo "  3. Add kernel dependency to your Cargo.toml:"
echo "     [dependencies.kernel]"
echo "     path = \"/path/to/kernel/rust\""
echo "     # OR use git:"
echo "     # git = \"https://github.com/Rust-for-Linux/linux.git\""
echo "     # package = \"kernel\""
echo "     # branch = \"rust\""
echo "  4. Run: cargo check --features kernel"
echo ""
echo "See kernel-driver-example/KERNEL_API_SETUP.md for detailed instructions."
echo ""

