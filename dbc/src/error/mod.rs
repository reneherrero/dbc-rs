mod lang;

#[path = "error.rs"]
mod error_impl;
mod helpers;

pub use error_impl::Error;
pub(crate) use helpers::{check_max_limit, map_val_error};

/// Result type alias for operations that can return an `Error`.
pub type Result<T> = core::result::Result<T, Error>;
