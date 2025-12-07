//! Compatibility layer for alloc vs kernel::alloc
//!
//! This module provides type aliases and helper functions to abstract
//! over the differences between `alloc` and `kernel::alloc` APIs.
//!
//! Strategy: Separate implementations for alloc vs kernel modes.

#![cfg(any(feature = "alloc", feature = "kernel"))]

// Direct re-exports for simplified imports
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
pub use crate::kernel::alloc::{string::String, vec::Vec};
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
pub use alloc::boxed::Box;
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
pub use alloc::{boxed::Box, string::String, vec::Vec};

// Type aliases (for backward compatibility)
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
pub use alloc::string::String as AllocString;
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
pub use alloc::vec::Vec as AllocVec;

#[cfg(all(feature = "kernel", not(feature = "alloc")))]
pub use crate::kernel::alloc::string::String as AllocString;
#[cfg(all(feature = "kernel", not(feature = "alloc")))]
pub use crate::kernel::alloc::vec::Vec as AllocVec;

// Helper functions for String operations
// Note: These are currently unused but may be needed for future kernel compatibility work
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
#[allow(dead_code)]
pub fn alloc_string_from(s: &str) -> AllocString {
    alloc::string::String::from(s)
}

#[cfg(all(feature = "kernel", not(feature = "alloc")))]
#[allow(dead_code)]
pub fn alloc_string_from(s: &str) -> Result<AllocString, ()> {
    crate::kernel::alloc::string::String::try_from(s)
}

// Helper functions for format!
// Note: These are currently unused but may be needed for future kernel compatibility work
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
#[allow(dead_code)]
pub fn alloc_format(args: core::fmt::Arguments<'_>) -> AllocString {
    alloc::fmt::format(args)
}

#[cfg(all(feature = "kernel", not(feature = "alloc")))]
#[allow(dead_code)]
pub fn alloc_format(args: core::fmt::Arguments<'_>) -> Result<AllocString, ()> {
    // In kernel mode, we need to format to a temporary string first
    // This is a limitation - we'll need to handle this differently
    // For now, use alloc::format and convert
    let s = alloc::fmt::format(args);
    crate::kernel::alloc::string::String::try_from(s.as_str())
}

// Helper for Vec operations
// Note: These are currently unused but may be needed for future kernel compatibility work
#[cfg(all(feature = "alloc", not(feature = "kernel")))]
#[allow(dead_code)]
pub fn vec_push<T>(vec: &mut AllocVec<T>, item: T) {
    vec.push(item);
}

#[cfg(all(feature = "kernel", not(feature = "alloc")))]
#[allow(dead_code)]
pub fn vec_push<T>(vec: &mut AllocVec<T>, item: T) -> Result<(), ()> {
    vec.try_push(item)
}
