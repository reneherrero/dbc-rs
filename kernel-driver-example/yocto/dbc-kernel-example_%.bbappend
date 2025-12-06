# BitBake append file for dbc-kernel-example
#
# Use this file to customize the recipe for specific kernel versions
# or machine configurations.
#
# Place in: <your-layer>/recipes-kernel/dbc-kernel-example/dbc-kernel-example_%.bbappend

# Example: Override Rust target for different architectures
# RUST_TARGET:arm = "arm-unknown-none"
# RUST_TARGET:aarch64 = "aarch64-unknown-none"

# Example: Add kernel module parameters
# module_conf_dbc_kernel_example = "options dbc_kernel_example debug=1"

# Example: Auto-load on boot
# KERNEL_MODULE_AUTOLOAD += "dbc_kernel_example"

# Example: Module dependencies
# KERNEL_MODULE_PROBECONF += "dbc_kernel_example"

