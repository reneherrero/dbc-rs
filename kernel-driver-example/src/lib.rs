//! Example Linux kernel driver using dbc-rs
//!
//! This is a skeleton example demonstrating how to use dbc-rs in a Linux kernel module.
//! It provides a basic character device that can parse DBC files.
//!
//! # Platform
//! - Target: PocketBeagle 2 Rev A1 (AM6254 SoC)
//! - Kernel: Linux with Rust support
//!
//! # Status
//! EXPERIMENTAL: This is a skeleton implementation. Not production-ready.

#![no_std]

use kernel::prelude::*;

kernel::module! {
    type: DbcDriver,
    name: "dbc_kernel_example",
    author: "dbc-rs contributors",
    description: "Example kernel driver using dbc-rs for DBC parsing",
    license: "GPL",
}

/// Kernel module state
struct DbcDriver {
    _dev: Pin<Box<chrdev::Registration>>,
}

impl kernel::Module for DbcDriver {
    fn init(_module: &'static ThisModule) -> kernel::error::Result<Self> {
        pr_info!("dbc-kernel-example: Initializing DBC kernel driver\n");

        // Register character device
        let mut chrdev_reg = chrdev::Registration::new_pinned(cstr!("dbc_example"), 0, &[])?;

        // TODO: Add file operations for DBC parsing
        // chrdev_reg.as_mut().register()?;

        Ok(DbcDriver { _dev: chrdev_reg })
    }
}

impl Drop for DbcDriver {
    fn drop(&mut self) {
        pr_info!("dbc-kernel-example: Cleaning up DBC kernel driver\n");
    }
}

/// Example function demonstrating dbc-rs usage in kernel context
///
/// This function shows how to parse a DBC file using dbc-rs with kernel::alloc.
/// In a real implementation, this would read from a file or user-space buffer.
///
/// # Errors
/// Returns kernel error code if DBC parsing fails
fn example_parse_dbc(dbc_data: &[u8]) -> kernel::error::Result {
    use dbc_rs::Dbc;

    // Parse DBC data
    // Note: In kernel context, error handling must use Result types
    // The dbc-rs crate with kernel feature handles kernel::alloc Result types
    // Convert bytes to str (kernel alloc String)
    let dbc_str = core::str::from_utf8(dbc_data).map_err(|_| kernel::error::code::EINVAL)?;
    let dbc = Dbc::parse(dbc_str).map_err(|_e| {
        pr_err!("dbc-kernel-example: Failed to parse DBC\n");
        kernel::error::code::EINVAL
    })?;

    pr_info!(
        "dbc-kernel-example: Parsed DBC with {} messages\n",
        dbc.messages().len()
    );

    Ok(())
}

// Note: Kernel modules typically don't use standard Rust tests
// Testing is done via kernel build system and runtime verification
