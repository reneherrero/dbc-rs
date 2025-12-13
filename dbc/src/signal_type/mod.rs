//! Signal Types module
//!
//! This module contains all signal type-related structures and parsing logic.

mod signal_extended_value_type;
mod signal_type;
mod signal_type_reference;
mod signal_type_value;

pub use signal_extended_value_type::SignalExtendedValueType;
pub use signal_type::SignalType;
pub use signal_type_reference::SignalTypeReference;
pub use signal_type_value::SignalTypeValue;

#[cfg(feature = "std")]
pub(crate) mod parse;
