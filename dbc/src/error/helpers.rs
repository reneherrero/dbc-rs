use super::Error;

/// Helper function to check if a length exceeds a maximum limit.
///
/// This centralizes the common pattern of checking collection lengths against MAX limits.
/// Returns `Some(error)` if the limit is exceeded, `None` otherwise.
///
/// This is an internal helper function and not part of the public API.
#[inline]
pub(crate) fn check_max_limit<E>(len: usize, max: usize, error: E) -> Option<E> {
    if len > max { Some(error) } else { None }
}

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

/// Helper function to convert `Error::Validation` to a specific `Error` variant,
/// preserving line number information.
///
/// Similar to `map_val_error` but also preserves the line number from the original error
/// in the converted error.
#[inline]
pub(crate) fn map_val_error_with_line<F, G>(err: Error, variant: F, fallback: G) -> Error
where
    F: FnOnce(&'static str) -> Error,
    G: FnOnce() -> Error,
{
    let line = err.line();
    let mapped = match err {
        Error::Validation(msg) => variant(msg),
        _ => fallback(),
    };
    if let Some(line) = line {
        mapped.with_line(line)
    } else {
        mapped
    }
}
