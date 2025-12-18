//! Compatibility layer for `alloc` and `heapless` abstractions.
//!
//! This module provides `String<{N}>`, `Vec<T, {N}>`, and `BTreeMap<K, V, {N}>` types that abstract over
//! `alloc` types (with `alloc` feature) and `heapless` types (with `heapless` feature).
//!
//! # Common Type Aliases
//!
//! - [`Name`] - A name string with `MAX_NAME_SIZE` limit (identifiers, signal names, etc.)
//! - [`ValueDescEntry`] - A value description entry `(value, description)`
//! - [`ValueDescEntries`] - A collection of value description entries

#[cfg(all(not(feature = "alloc"), not(feature = "heapless")))]
compile_error!("Either the `alloc` or `heapless` feature must be enabled");

#[cfg(feature = "alloc")]
extern crate alloc;

mod btree_map;
mod string;
mod vec;

pub use btree_map::BTreeMap;
pub use string::String;
pub use vec::Vec;

use crate::{Error, MAX_NAME_SIZE, MAX_VALUE_DESCRIPTIONS, Result};

// ============================================================================
// Common Type Aliases
// ============================================================================

/// A name string with `MAX_NAME_SIZE` limit.
///
/// Used for identifiers like signal names, message names, node names, etc.
pub type Name = String<{ MAX_NAME_SIZE }>;

/// A value description entry: `(numeric_value, description_string)`.
///
/// Used in VAL_ statements to map signal values to human-readable text.
pub type ValueDescEntry = (u64, Name);

/// A collection of value description entries.
pub type ValueDescEntries = Vec<ValueDescEntry, { MAX_VALUE_DESCRIPTIONS }>;

/// Helper function to validate and convert a string to a [`Name`] with `MAX_NAME_SIZE` limit.
///
/// This centralizes the common pattern of converting a string reference to
/// [`Name`] with proper error handling.
///
/// # Errors
///
/// Returns `Error::Expected` with `MAX_NAME_SIZE_EXCEEDED` message if the name
/// exceeds `MAX_NAME_SIZE` (32 characters by default, per DBC specification).
#[inline]
pub fn validate_name<S: AsRef<str>>(name: S) -> Result<Name> {
    let name_str: &str = name.as_ref();

    // Explicitly check length before conversion to ensure MAX_NAME_SIZE enforcement
    // This check works for both alloc and heapless features
    if name_str.len() > MAX_NAME_SIZE {
        return Err(Error::expected(Error::MAX_NAME_SIZE_EXCEEDED));
    }

    // Convert to compat::String - this will also check the limit internally,
    // but we've already checked above for clarity and early error reporting
    String::try_from(name_str).map_err(|_| Error::expected(Error::MAX_NAME_SIZE_EXCEEDED))
}
