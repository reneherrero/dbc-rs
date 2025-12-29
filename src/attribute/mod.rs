//! Attribute definitions and values for DBC files.
//!
//! This module provides types for parsing and representing DBC attributes:
//! - `BA_DEF_` - Attribute definitions
//! - `BA_DEF_DEF_` - Attribute default values
//! - `BA_` - Attribute value assignments
//!
//! # DBC Attribute System
//!
//! Attributes in DBC files provide metadata for network objects:
//! - Network/database level (global attributes)
//! - Node attributes (BU_)
//! - Message attributes (BO_)
//! - Signal attributes (SG_)
//!
//! # Example
//!
//! ```text
//! BA_DEF_ BO_ "GenMsgCycleTime" INT 0 10000;
//! BA_DEF_DEF_ "GenMsgCycleTime" 100;
//! BA_ "GenMsgCycleTime" BO_ 256 50;
//! ```

mod impls;
pub(crate) mod parse;
#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
mod builder;

use crate::compat::{Name, String, Vec};

#[cfg(feature = "std")]
pub use builder::AttributeDefinitionBuilder;

/// Maximum size for attribute string values.
pub const MAX_ATTRIBUTE_STRING_SIZE: usize = 256;

/// Type alias for attribute string values.
pub type AttributeString = String<MAX_ATTRIBUTE_STRING_SIZE>;

/// Type alias for enum value list in attribute definitions.
pub type EnumValues = Vec<Name, { crate::MAX_ATTRIBUTE_ENUM_VALUES }>;

/// Type alias for attribute definitions collection.
pub type AttributeDefinitions = Vec<AttributeDefinition, { crate::MAX_ATTRIBUTE_DEFINITIONS }>;

/// Object type that an attribute applies to.
///
/// Corresponds to the object_type in `BA_DEF_`:
/// - Empty (no prefix) = Network/database level
/// - `BU_` = Node
/// - `BO_` = Message
/// - `SG_` = Signal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AttributeObjectType {
    /// Network/database level attribute (no object prefix in BA_DEF_)
    #[default]
    Network,
    /// Node attribute (BU_ prefix)
    Node,
    /// Message attribute (BO_ prefix)
    Message,
    /// Signal attribute (SG_ prefix)
    Signal,
}

/// Attribute value type specification from BA_DEF_.
///
/// Defines the type and constraints for an attribute's values.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValueType {
    /// Integer type with min/max range: `INT min max`
    Int {
        /// Minimum allowed value
        min: i64,
        /// Maximum allowed value
        max: i64,
    },
    /// Hexadecimal integer type with min/max range: `HEX min max`
    Hex {
        /// Minimum allowed value
        min: i64,
        /// Maximum allowed value
        max: i64,
    },
    /// Float type with min/max range: `FLOAT min max`
    Float {
        /// Minimum allowed value
        min: f64,
        /// Maximum allowed value
        max: f64,
    },
    /// String type: `STRING`
    String,
    /// Enumeration type: `ENUM "val1","val2",...`
    Enum {
        /// List of valid enum values
        values: EnumValues,
    },
}

/// Concrete attribute value from BA_ or BA_DEF_DEF_.
///
/// Represents the actual value assigned to an attribute.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// Integer value (for INT and HEX types)
    Int(i64),
    /// Floating-point value (for FLOAT type)
    Float(f64),
    /// String value (for STRING and ENUM types)
    String(AttributeString),
}

/// Attribute definition from BA_DEF_.
///
/// Defines an attribute's name, target object type, and value type.
///
/// # Example
///
/// ```text
/// BA_DEF_ BO_ "GenMsgCycleTime" INT 0 10000;
/// ```
///
/// This defines a message attribute named "GenMsgCycleTime" with
/// integer values in range 0-10000.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeDefinition {
    /// Attribute name (quoted identifier in DBC)
    name: Name,
    /// Target object type (Network, Node, Message, Signal)
    object_type: AttributeObjectType,
    /// Value type specification
    value_type: AttributeValueType,
}

impl AttributeDefinition {
    /// Creates a new attribute definition.
    #[inline]
    pub(crate) fn new(
        name: Name,
        object_type: AttributeObjectType,
        value_type: AttributeValueType,
    ) -> Self {
        Self {
            name,
            object_type,
            value_type,
        }
    }
}

/// Target object reference for attribute value assignment.
///
/// Identifies which specific object an attribute value applies to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttributeTarget {
    /// Network/database level (global)
    Network,
    /// Node by name
    Node(Name),
    /// Message by ID
    Message(u32),
    /// Signal by (message_id, signal_name)
    Signal(u32, Name),
}
