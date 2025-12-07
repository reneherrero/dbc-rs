//! Kernel-specific implementations for the compatibility layer

// Re-exports
pub use crate::kernel::alloc::{string::String, vec::Vec};
pub use alloc::boxed::Box;

// Type aliases
pub use crate::kernel::alloc::string::String as AllocString;
pub use crate::kernel::alloc::vec::Vec as AllocVec;

/// Convert a string-like value to a String
pub fn str_to_string(s: impl AsRef<str>) -> String {
    crate::kernel::alloc::string::String::from_str(s.as_ref())
}

/// Convert a Display type to a String
pub fn display_to_string<T: core::fmt::Display>(value: T) -> String {
    let formatted = alloc::format!("{}", value);
    crate::kernel::alloc::string::String::from_str(&formatted)
}

/// Format two Display values with a separator
pub fn format_two(a: impl core::fmt::Display, sep: &str, b: impl core::fmt::Display) -> String {
    let formatted = alloc::format!("{}{}{}", a, sep, b);
    crate::kernel::alloc::string::String::from_str(&formatted)
}

/// Create a new Vec with the specified capacity (best-effort)
///
/// Always returns `alloc::vec::Vec` for compatibility with code that expects the standard alloc Vec.
pub fn vec_with_capacity<T>(capacity: usize) -> alloc::vec::Vec<T> {
    use crate::kernel::alloc::vec::Vec as KernelVec;
    let mut vec = KernelVec::new();
    // Attempt to reserve capacity (best-effort, may fail silently in kernel mode)
    let _ = vec.try_reserve(capacity);
    // Extract the inner alloc::vec::Vec (kernel Vec wraps it)
    vec.into()
}

// Legacy functions (kept for backward compatibility)
#[allow(dead_code)]
pub fn alloc_string_from(s: &str) -> AllocString {
    crate::kernel::alloc::string::String::from_str(s)
}

#[allow(dead_code)]
pub fn alloc_format(args: core::fmt::Arguments<'_>) -> AllocString {
    // In kernel mode, we need to format to a temporary string first
    // This is a limitation - we'll need to handle this differently
    // For now, use alloc::format and convert
    let s = alloc::fmt::format(args);
    crate::kernel::alloc::string::String::from_str(s.as_str())
}

#[allow(dead_code)]
pub fn vec_push<T>(vec: &mut AllocVec<T>, item: T) {
    vec.push(item);
}
