# Makefile for dbc-kernel-example
#
# This Makefile integrates the Rust kernel module with the Linux kernel build system.
#
# Prerequisites:
# - Linux kernel source with Rust support enabled
# - rust-for-linux toolchain
# - KDIR environment variable pointing to kernel source
#
# For TI AM6254 (PocketBeagle 2):
#   KDIR=/path/to/ti-linux-kernel
#   ARCH=arm64
#   CROSS_COMPILE=aarch64-linux-gnu-

obj-m += dbc_kernel_example.o

# The kernel build system will compile src/lib.rs directly
# We only need to link against the dbc-rs static library
dbc_kernel_example-rusty-objs := src/lib.o
# Hardcode path since RUST_TARGET may not be available when kernel build system runs
dbc_kernel_example-objs := \
    target/aarch64-unknown-none/release/libdbc_rs.a \
    kernel_module.o

# Kernel source directory (override with KDIR=...)
# Default: running kernel headers
# For TI kernel: export KDIR=/path/to/ti-linux-kernel
KDIR ?= /lib/modules/$(shell uname -r)/build

# Cross-compilation support for ARM64 (AM6254)
# For TI kernel on AM6254: export CROSS_COMPILE=aarch64-linux-gnu-
CROSS_COMPILE ?=

# Rust target for kernel (override with ARCH=...)
ARCH ?= $(shell uname -m)

# Get Rust target from kernel (if available)
# For TI kernel, this requires: make ARCH=$(ARCH) prepare
RUST_TARGET := $(shell if [ -f "$(KDIR)/scripts/rust/rust_target.sh" ]; then \
	$(KDIR)/scripts/rust/rust_target.sh; \
	elif [ "$(ARCH)" = "arm64" ]; then \
	echo "aarch64-unknown-none"; \
	else \
	echo "x86_64-unknown-none"; \
	fi)

# Cargo build directory
CARGO_BUILD_DIR := target/$(RUST_TARGET)/release

# Default target
all:
	@echo "Building dbc-kernel-example kernel module"
	@echo "KDIR: $(KDIR)"
	@echo "ARCH: $(ARCH)"
	@echo "CROSS_COMPILE: $(CROSS_COMPILE)"
	@echo "RUST_TARGET: $(RUST_TARGET)"
	@echo ""
	@# Check if kernel has Rust support
	@# Note: Kernel doesn't generate Cargo.toml, so we check for rust/kernel directory instead
	@if [ ! -f "$(KDIR)/scripts/rust/rust_target.sh" ] && [ ! -d "$(KDIR)/rust/kernel" ]; then \
		echo "ERROR: Kernel Rust support not found!"; \
		echo "  KDIR: $(KDIR)"; \
		echo ""; \
		echo "For TI kernel, ensure:"; \
		echo "  1. Set environment variables:"; \
		echo "     export KDIR=/home/rene/Source/dbc-rs/ti-linux-kernel"; \
		echo "     export ARCH=arm64"; \
		echo "     export CROSS_COMPILE=aarch64-linux-gnu-"; \
		echo "  2. Kernel is configured: cd ti-linux-kernel && make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- defconfig"; \
		echo "  3. Rust is enabled: echo 'CONFIG_RUST=y' >> .config && make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- olddefconfig"; \
		echo "  4. Kernel is prepared: make ARCH=arm64 CROSS_COMPILE=aarch64-linux-gnu- prepare"; \
		echo ""; \
		exit 1; \
	fi
	@echo "Step 1: Building Rust library (dbc-rs only)..."
	@if [ -n "$(CROSS_COMPILE)" ]; then \
		echo "Cross-compiling for $(ARCH)"; \
		cargo build --release --target $(RUST_TARGET) --target-dir target -p dbc-rs; \
	else \
		cargo build --release --target $(RUST_TARGET) --target-dir target -p dbc-rs; \
	fi
	@echo "Converting rlib to static library..."
	@# rlib files are actually .a files, just rename it
	@if [ -f "target/$(RUST_TARGET)/release/libdbc_rs.rlib" ]; then \
		cp target/$(RUST_TARGET)/release/libdbc_rs.rlib target/$(RUST_TARGET)/release/libdbc_rs.a; \
		echo "Created libdbc_rs.a from rlib"; \
	else \
		echo "ERROR: libdbc_rs.rlib not found!"; \
		exit 1; \
	fi
	@echo ""
	@echo "Step 2: Building kernel module..."
	@$(MAKE) -C $(KDIR) M=$$PWD ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) modules

clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@$(MAKE) -C $(KDIR) M=$$PWD ARCH=$(ARCH) CROSS_COMPILE=$(CROSS_COMPILE) clean
	@rm -f *.o *.ko *.mod.c *.mod *.symvers *.order

# Install module (requires root)
install: all
	@echo "Installing kernel module..."
	@sudo insmod dbc_kernel_example.ko

# Remove module (requires root)
remove:
	@echo "Removing kernel module..."
	@sudo rmmod dbc_kernel_example || true

# Check module status
status:
	@lsmod | grep dbc_kernel_example || echo "Module not loaded"

# View kernel logs
logs:
	@dmesg | tail -20 | grep -i dbc || echo "No dbc-related log entries"

# Run compilation tests
test-compile:
	@./test-compilation.sh

# Run runtime tests (requires root)
test-runtime:
	@sudo ./test-runtime.sh

# Run all integration tests
test: test-compile
	@echo "Run 'make test-runtime' (as root) for runtime tests"

.PHONY: all clean install remove status logs test test-compile test-runtime

