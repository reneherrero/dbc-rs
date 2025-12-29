use std::{string::String, vec::Vec};

/// Builder for creating `ValueDescriptions` programmatically.
///
/// This builder allows you to construct value descriptions when building DBC files
/// programmatically. It validates that entries are within limits and non-empty.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::ValueDescriptionsBuilder;
///
/// let value_descriptions = ValueDescriptionsBuilder::new()
///     .add_entry(0, "Park")
///     .add_entry(1, "Reverse")
///     .add_entry(2, "Neutral")
///     .add_entry(3, "Drive")
///     .build()?;
///
/// assert_eq!(value_descriptions.get(0), Some("Park"));
/// assert_eq!(value_descriptions.get(1), Some("Reverse"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Validation
///
/// The builder validates:
/// - At least one entry is required (non-empty)
/// - Maximum of 64 value descriptions (MAX_VALUE_DESCRIPTIONS)
/// - Description names must not exceed MAX_NAME_SIZE
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct ValueDescriptionsBuilder {
    entries: Vec<(u64, String)>,
}

mod build;
mod impls;
