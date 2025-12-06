// SPDX-License-Identifier: GPL-2.0
/*
 * C glue code for dbc-kernel-example Rust kernel module
 *
 * This file provides the minimal C interface required by the kernel
 * build system to link the Rust module.
 */

#include <linux/module.h>
#include <linux/kernel.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("dbc-rs contributors");
MODULE_DESCRIPTION("Example kernel driver using dbc-rs for DBC parsing");
MODULE_VERSION("0.1.0");

// The actual module implementation is in Rust (lib.rs)
// This C file is a placeholder for the kernel build system

static int __init kernel_module_init(void)
{
    // Rust module initialization happens in lib.rs
    return 0;
}

static void __exit kernel_module_exit(void)
{
    // Rust module cleanup happens in lib.rs
}

module_init(kernel_module_init);
module_exit(kernel_module_exit);

