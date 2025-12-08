//! Alloc-specific implementations for the compatibility layer

// Re-exports
pub use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

/// Convert a string-like value to a String
pub fn str_to_string(s: impl AsRef<str>) -> String {
    alloc::string::String::from(s.as_ref())
}

/// Convert a Display type to a String
pub fn display_to_string<T: core::fmt::Display>(value: T) -> String {
    value.to_string()
}

/// Format two Display values with a separator
pub fn format_two(a: impl core::fmt::Display, sep: &str, b: impl core::fmt::Display) -> String {
    alloc::format!("{}{}{}", a, sep, b)
}

/// Create a new Vec with the specified capacity
pub fn vec_with_capacity<T>(capacity: usize) -> alloc::vec::Vec<T> {
    alloc::vec::Vec::with_capacity(capacity)
}

/// Convert an iterator of items to Vec<String>
/// Each item must implement ToString
pub fn strings_from_iter<I, T>(iter: I) -> Vec<String>
where
    I: IntoIterator<Item = T>,
    T: ToString,
{
    iter.into_iter().map(|s| s.to_string()).collect()
}
