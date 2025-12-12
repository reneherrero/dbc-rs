use super::error_impl::Error;

/// Helper function to convert `Error::Validation` to a specific `Error` variant.
///
/// This centralizes the common pattern of converting validation errors to specific error variants
/// with a fallback for non-validation errors.
///
/// # Arguments
///
/// * `err` - The `Error` to convert
/// * `variant` - A closure that maps a validation message to the appropriate `Error` variant
/// * `fallback` - A closure that generates a fallback `Error` for non-validation errors
///
/// # Example
///
/// This is an internal helper function used throughout the codebase to convert
/// `Error::Validation` variants to specific error variants with proper
/// error message handling. It's not part of the public API.
#[inline]
pub(crate) fn map_val_error<F, G>(err: Error, variant: F, fallback: G) -> Error
where
    F: FnOnce(&'static str) -> Error,
    G: FnOnce() -> Error,
{
    match err {
        Error::Validation(msg) => variant(msg),
        _ => fallback(),
    }
}
