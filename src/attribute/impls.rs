//! Accessor implementations for attribute types.

use super::{
    AttributeDefinition, AttributeObjectType, AttributeValue, AttributeValueType, EnumValues,
};

// ============================================================================
// AttributeObjectType
// ============================================================================

impl AttributeObjectType {
    /// Returns the DBC keyword for this object type.
    ///
    /// Returns an empty string for Network (global) attributes.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Network => "",
            Self::Node => "BU_",
            Self::Message => "BO_",
            Self::Signal => "SG_",
        }
    }

    /// Returns true if this is a network-level attribute.
    #[inline]
    #[must_use]
    pub fn is_network(&self) -> bool {
        matches!(self, Self::Network)
    }

    /// Returns true if this is a node attribute.
    #[inline]
    #[must_use]
    pub fn is_node(&self) -> bool {
        matches!(self, Self::Node)
    }

    /// Returns true if this is a message attribute.
    #[inline]
    #[must_use]
    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message)
    }

    /// Returns true if this is a signal attribute.
    #[inline]
    #[must_use]
    pub fn is_signal(&self) -> bool {
        matches!(self, Self::Signal)
    }
}

// ============================================================================
// AttributeValueType
// ============================================================================

impl AttributeValueType {
    /// Returns true if this is an integer type (INT or HEX).
    #[inline]
    #[must_use]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int { .. } | Self::Hex { .. })
    }

    /// Returns true if this is a float type.
    #[inline]
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float { .. })
    }

    /// Returns true if this is a string type.
    #[inline]
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    /// Returns true if this is an enum type.
    #[inline]
    #[must_use]
    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum { .. })
    }

    /// Returns the enum values if this is an enum type.
    #[inline]
    #[must_use]
    pub fn enum_values(&self) -> Option<&EnumValues> {
        match self {
            Self::Enum { values } => Some(values),
            _ => None,
        }
    }

    /// Returns the integer range if this is an INT or HEX type.
    #[inline]
    #[must_use]
    pub fn int_range(&self) -> Option<(i64, i64)> {
        match self {
            Self::Int { min, max } | Self::Hex { min, max } => Some((*min, *max)),
            _ => None,
        }
    }

    /// Returns the float range if this is a FLOAT type.
    #[inline]
    #[must_use]
    pub fn float_range(&self) -> Option<(f64, f64)> {
        match self {
            Self::Float { min, max } => Some((*min, *max)),
            _ => None,
        }
    }
}

// ============================================================================
// AttributeValue
// ============================================================================

impl AttributeValue {
    /// Gets the value as an integer, if applicable.
    #[inline]
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Gets the value as a float, if applicable.
    #[inline]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Gets the value as a string, if applicable.
    ///
    /// This returns the string for both STRING and ENUM attribute values.
    #[inline]
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns true if this is an integer value.
    #[inline]
    #[must_use]
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    /// Returns true if this is a float value.
    #[inline]
    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    /// Returns true if this is a string value.
    #[inline]
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
}

// ============================================================================
// AttributeDefinition
// ============================================================================

impl AttributeDefinition {
    /// Returns the attribute name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the object type this attribute applies to.
    #[inline]
    #[must_use]
    pub fn object_type(&self) -> AttributeObjectType {
        self.object_type
    }

    /// Returns the value type specification.
    #[inline]
    #[must_use]
    pub fn value_type(&self) -> &AttributeValueType {
        &self.value_type
    }
}
