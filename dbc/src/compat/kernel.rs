//! Kernel-specific implementations for the compatibility layer

// Re-exports
pub use crate::kernel::alloc::{string::String, vec::Vec};
pub use alloc::boxed::Box;

// Provide ToString trait that works with kernel String
// This is a compatibility wrapper around alloc::string::ToString
pub trait ToString {
    fn to_string(&self) -> String;
}

// Implement for str
impl ToString for str {
    fn to_string(&self) -> String {
        crate::kernel::alloc::string::String::from_str(self)
    }
}

// Implement for types that implement Display
impl<T: core::fmt::Display> ToString for T {
    fn to_string(&self) -> String {
        let formatted = alloc::format!("{}", self);
        crate::kernel::alloc::string::String::from_str(&formatted)
    }
}

/// Convert a string-like value to a String
pub fn str_to_string(s: impl AsRef<str>) -> String {
    crate::kernel::alloc::string::String::from_str(s.as_ref())
}

/// Convert a Display type to a String
pub fn display_to_string<T: core::fmt::Display>(value: T) -> String {
    // Use alloc::format! since kernel String wraps alloc String internally
    let formatted = alloc::format!("{}", value);
    crate::kernel::alloc::string::String::from_str(&formatted)
}

/// Format two Display values with a separator
pub fn format_two(a: impl core::fmt::Display, sep: &str, b: impl core::fmt::Display) -> String {
    // Use alloc::format! since kernel String wraps alloc String internally
    let formatted = alloc::format!("{}{}{}", a, sep, b);
    crate::kernel::alloc::string::String::from_str(&formatted)
}

/// Create a new Vec with the specified capacity (best-effort)
///
/// Returns alloc Vec for compatibility (kernel Vec wraps alloc Vec internally).
pub fn vec_with_capacity<T>(capacity: usize) -> alloc::vec::Vec<T> {
    let mut vec = crate::kernel::alloc::vec::Vec::new();
    // Attempt to reserve capacity (best-effort, may fail silently in kernel mode)
    let _ = vec.try_reserve(capacity);
    // Convert kernel Vec to alloc Vec (kernel Vec wraps alloc Vec)
    crate::alloc::vec::Vec::from(vec)
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
