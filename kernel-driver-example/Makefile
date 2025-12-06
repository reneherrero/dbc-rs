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

dbc_kernel_example-objs := \
    target/$(ARCH)/release/libdbc_kernel_example.a \
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
RUST_TARGET := $(shell $(KDIR)/scripts/rust/rust_target.sh)

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
	@echo "Step 1: Building Rust library..."
	@if [ -n "$(CROSS_COMPILE)" ]; then \
		echo "Cross-compiling for $(ARCH)"; \
		cargo build --release --target $(RUST_TARGET) --target-dir target; \
	else \
		cargo build --release --target $(RUST_TARGET) --target-dir target; \
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

