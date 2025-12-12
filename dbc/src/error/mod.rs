pub mod lang;

#[path = "error.rs"]
mod error_impl;
mod helpers;
#[path = "parse_error.rs"]
mod parse_error_impl;

pub use error_impl::Error;
pub(crate) use helpers::map_val_error;
pub use parse_error_impl::ParseError;

/// Result type alias for operations that can return an `Error`.
pub type Result<T> = core::result::Result<T, Error>;

/// Result type alias for low-level parsing operations that can return a `ParseError`.
pub type ParseResult<T> = core::result::Result<T, ParseError>;
