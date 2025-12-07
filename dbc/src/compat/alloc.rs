//! Alloc-specific implementations for the compatibility layer

// Re-exports
pub use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

// Type aliases
pub use alloc::string::String as AllocString;
pub use alloc::vec::Vec as AllocVec;

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

// Legacy functions (kept for backward compatibility)
#[allow(dead_code)]
pub fn alloc_string_from(s: &str) -> AllocString {
    alloc::string::String::from(s)
}

#[allow(dead_code)]
pub fn alloc_format(args: core::fmt::Arguments<'_>) -> AllocString {
    alloc::fmt::format(args)
}

#[allow(dead_code)]
pub fn vec_push<T>(vec: &mut AllocVec<T>, item: T) {
    vec.push(item);
}
