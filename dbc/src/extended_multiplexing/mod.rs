//! Extended Multiplexing definition (SG_MUL_VAL_)
//!
//! Represents extended multiplexing entries that define which multiplexer switch values
//! activate specific multiplexed signals.
//!
//! Extended multiplexing allows signals to be active for ranges of multiplexer switch
//! values, rather than just a single value (as with basic `m0`, `m1` multiplexing).
//!
//! # DBC Format
//!
//! ```text
//! SG_MUL_VAL_ <message_id> <signal_name> <switch_name> <range1>,<range2>,... ;
//! ```
//!
//! Where each range is `min-max` (inclusive).
//!
//! # Example
//!
//! ```text
//! SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
//! ```
//!
//! This means `Signal_A` is decoded when `Mux1` has a value between 0-5 OR 10-15.

use crate::compat::{Name, Vec};

mod impls;
mod parse;

/// A value range (min, max) for extended multiplexing
type ValueRange = (u64, u64);
/// Maximum 64 ranges per extended multiplexing entry
type ValueRanges = Vec<ValueRange, 64>;

#[cfg(feature = "std")]
mod builder;

#[cfg(feature = "std")]
pub use builder::ExtendedMultiplexingBuilder;

/// Extended Multiplexing definition (SG_MUL_VAL_)
///
/// Represents extended multiplexing entries that define which multiplexer switch values
/// activate specific multiplexed signals. This is an extension to basic multiplexing
/// (`m0`, `m1`, etc.) that allows signals to be active for **ranges** of switch values.
///
/// # When to Use Extended Multiplexing
///
/// Extended multiplexing is useful when:
/// - A signal should be active for multiple switch values (e.g., 0-5 and 10-15)
/// - Multiple multiplexer switches control a single signal (AND logic)
/// - The activation pattern is more complex than a single value
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc = Dbc::parse(r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 500 MuxMessage : 8 ECM
///  SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *
///
/// SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
/// "#)?;
///
/// // Get extended multiplexing entries for message 500
/// let entries: Vec<_> = dbc.extended_multiplexing_for_message(500).collect();
/// assert_eq!(entries.len(), 1);
///
/// let entry = entries[0];
/// assert_eq!(entry.message_id(), 500);
/// assert_eq!(entry.signal_name(), "Signal_A");
/// assert_eq!(entry.multiplexer_switch(), "Mux1");
/// assert_eq!(entry.value_ranges(), &[(5, 10)]);
///
/// // Decode - Signal_A will only be decoded when Mux1 is 5-10
/// let payload = [0x07, 0x00, 0xE8, 0x03, 0x00, 0x00, 0x00, 0x00]; // Mux1=7
/// let decoded = dbc.decode(500, &payload, false)?;
/// assert!(decoded.iter().any(|s| s.name == "Signal_A"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Multiple Ranges
///
/// A signal can be active for multiple non-contiguous ranges:
///
/// ```text
/// SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15,20-25 ;
/// ```
///
/// # Multiple Switches (AND Logic)
///
/// When multiple `SG_MUL_VAL_` entries exist for the same signal with different
/// switches, ALL switches must match their respective ranges for the signal to
/// be decoded (AND logic):
///
/// ```text
/// SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
/// SG_MUL_VAL_ 500 Signal_A Mux2 20-25 ;
/// ```
///
/// Here, `Signal_A` is only decoded when `Mux1` is 5-10 AND `Mux2` is 20-25.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedMultiplexing {
    message_id: u32,
    signal_name: Name,
    multiplexer_switch: Name,
    value_ranges: ValueRanges,
}
