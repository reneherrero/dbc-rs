//! Compatibility layer for alloc vs kernel::alloc
//!
//! This module provides type aliases and helper functions to abstract
//! over the differences between `alloc` and `kernel::alloc` APIs.
//!
//! Strategy: Separate modules for alloc vs kernel modes.
//! The `alloc` and `kernel` features are mutually exclusive.
//! Kernel takes priority when both are available (kernel wraps alloc internally).

#![cfg(any(feature = "alloc", feature = "kernel"))]

// Kernel takes priority when both are available (kernel wraps alloc internally)
// This allows kernel to work even if alloc is transitively enabled by dependencies
#[cfg(feature = "kernel")]
mod kernel;
#[cfg(feature = "kernel")]
pub use kernel::*;

// Only use alloc module if kernel is not enabled
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
mod alloc;
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
pub use alloc::*;
