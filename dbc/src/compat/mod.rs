//! Compatibility layer for alloc vs kernel::alloc
//!
//! This module provides type aliases and helper functions to abstract
//! over the differences between `alloc` and `kernel::alloc` APIs.
//!
//! Strategy: Separate modules for alloc vs kernel modes.
//! The `alloc` and `kernel` features are mutually exclusive.

// Only use kernel module if alloc is not enabled
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
mod kernel;
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
pub use kernel::*;

// Only use alloc module if kernel is not enabled
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
mod alloc;
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
pub use alloc::*;
