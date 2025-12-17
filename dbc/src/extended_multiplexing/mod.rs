//! Extended Multiplexing definition (SG_MUL_VAL_)
//!
//! Represents extended multiplexing entries that define which multiplexer switch values
//! activate specific multiplexed signals.
use crate::{MAX_NAME_SIZE, compat::String, compat::Vec};

mod core;
mod parse;

#[cfg(feature = "std")]
mod builder;

#[cfg(feature = "std")]
pub use builder::ExtendedMultiplexingBuilder;

/// Extended Multiplexing definition (SG_MUL_VAL_)
///
/// Represents extended multiplexing entries that define which multiplexer switch values
/// activate specific multiplexed signals.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedMultiplexing {
    message_id: u32,
    signal_name: String<{ MAX_NAME_SIZE }>,
    multiplexer_switch: String<{ MAX_NAME_SIZE }>,
    value_ranges: Vec<(u64, u64), 64>, // Max 64 ranges per extended multiplexing entry
}
