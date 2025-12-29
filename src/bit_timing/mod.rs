#[cfg(feature = "std")]
pub mod builder;
mod impls;
mod parse;
#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
pub use builder::BitTimingBuilder;

/// Represents the bit timing section from a DBC file.
///
/// The `BS_:` statement in a DBC file specifies CAN bus timing parameters.
/// This section is **required** in DBC files but is typically empty as
/// bit timing configuration is obsolete in modern CAN systems.
///
/// # Format
///
/// ```text
/// BS_:                        (empty - most common)
/// BS_: 500                    (baudrate only)
/// BS_: 500 : 12,34            (baudrate with BTR1, BTR2)
/// ```
///
/// # Fields
///
/// - `baudrate` - Optional CAN bus baudrate in kbps
/// - `btr1` - Optional Bus Timing Register 1 value
/// - `btr2` - Optional Bus Timing Register 2 value
///
/// # Notes
///
/// - This section is **obsolete** and not processed by modern CAN tools
/// - The keyword `BS_:` is required but values are typically omitted
/// - BTR values are only present if baudrate is specified
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BitTiming {
    baudrate: Option<u32>,
    btr1: Option<u32>,
    btr2: Option<u32>,
}
