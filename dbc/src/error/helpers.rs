use super::error_impl::Error;
use super::parse_error_impl::ParseError;

/// Helper function to convert `Error::Validation` to a specific `ParseError` variant.
///
/// This centralizes the common pattern of converting validation errors to parse errors
/// with a fallback for non-validation errors.
///
/// # Arguments
///
/// * `err` - The `Error` to convert
/// * `variant` - A closure that maps a validation message to the appropriate `ParseError` variant
/// * `fallback` - A closure that generates a fallback `ParseError` for non-validation errors
///
/// # Example
///
/// This is an internal helper function used throughout the codebase to convert
/// `Error::Validation` variants to specific `ParseError` variants with proper
/// error message handling. It's not part of the public API.
#[inline]
pub(crate) fn map_val_error<F, G>(err: Error, variant: F, fallback: G) -> ParseError
where
    F: FnOnce(&'static str) -> ParseError,
    G: FnOnce() -> ParseError,
{
    match err {
        Error::Validation(msg) => variant(msg),
        _ => fallback(),
    }
}
