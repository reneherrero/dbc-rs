//! Compatibility layer for `alloc` and `heapless` abstractions.
//!
//! This module provides `String<{N}>` and `Vec<T, {N}>` types that abstract over
//! `alloc::string::String`/`alloc::vec::Vec` (with `alloc` feature) and
//! `heapless::String<N>`/`heapless::Vec<T, N>` (with `heapless` feature).

#[cfg(all(not(feature = "alloc"), not(feature = "heapless")))]
compile_error!("Either the `alloc` or `heapless` feature must be enabled");

#[cfg(feature = "alloc")]
extern crate alloc;

mod string;
mod vec;

pub use string::String;
pub use vec::Vec;

use crate::{Error, MAX_NAME_SIZE, Result};

/// Helper function to validate and convert a string to a name with MAX_NAME_SIZE limit.
///
/// This centralizes the common pattern of converting a string reference to
/// `String<{ MAX_NAME_SIZE }>` with proper error handling.
///
/// # Errors
///
/// Returns `Error::Expected` with `MAX_NAME_SIZE_EXCEEDED` message if the name
/// exceeds `MAX_NAME_SIZE` (32 characters by default, per DBC specification).
#[inline]
pub fn validate_name<S: AsRef<str>>(name: S) -> Result<String<{ MAX_NAME_SIZE }>> {
    let name_str: &str = name.as_ref();

    // Explicitly check length before conversion to ensure MAX_NAME_SIZE enforcement
    // This check works for both alloc and heapless features
    if name_str.len() > MAX_NAME_SIZE {
        return Err(Error::Expected(Error::MAX_NAME_SIZE_EXCEEDED));
    }

    // Convert to compat::String - this will also check the limit internally,
    // but we've already checked above for clarity and early error reporting
    String::try_from(name_str).map_err(|_| Error::Expected(Error::MAX_NAME_SIZE_EXCEEDED))
}
