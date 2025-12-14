//! Signal Type definitions
//!
//! This module contains all signal type-related structures:
//! - `SignalType`: Signal type definitions (SGTYPE_)
//! - `SignalTypeReference`: Signal type references (SIG_TYPE_REF_)
//! - `SignalTypeValue`: Signal type value descriptions (SGTYPE_VAL_)
//! - `SignalExtendedValueType`: Extended value types (SIG_VALTYPE_)
//!
//! ## Relationships
//!
//! - `SignalTypeReference` and `SignalTypeValue` both reference a `SignalType` by name
//!   through their `type_name()` methods, which should match a `SignalType`'s `name()`

mod signal_extended_value_type;
mod signal_type_def;
mod signal_type_reference;
mod signal_type_value;

pub use signal_extended_value_type::SignalExtendedValueType;
pub use signal_type_def::SignalType;
pub use signal_type_reference::SignalTypeReference;
pub use signal_type_value::SignalTypeValue;

/// Trait for types that reference a SignalType by name
///
/// This trait unifies the common pattern where `SignalTypeReference` and `SignalTypeValue`
/// reference a `SignalType` by name through their `type_name()` methods, which should
/// match a `SignalType`'s `name()`.
#[allow(dead_code)] // Public API trait, may be used by library consumers
pub trait SignalTypeName {
    /// Get the name of the referenced signal type
    fn signal_type_name(&self) -> &str;
}
