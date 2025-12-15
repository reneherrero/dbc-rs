//! Signal Type Attribute Definition (BA_DEF_SGTYPE_)
//!
//! Defines a custom attribute for signal types.

/// Signal Type Attribute Definition (BA_DEF_SGTYPE_)
///
/// Defines a custom attribute for signal types.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct SignalTypeAttributeDefinition {
    name: std::string::String,
    value_type: crate::attributes::AttributeValueType,
}

#[cfg(feature = "std")]
impl SignalTypeAttributeDefinition {
    /// Create a new SignalTypeAttributeDefinition
    pub(crate) fn new(
        name: std::string::String,
        value_type: crate::attributes::AttributeValueType,
    ) -> Self {
        Self { name, value_type }
    }

    /// Get the attribute name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the attribute value type
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn value_type(&self) -> &crate::attributes::AttributeValueType {
        &self.value_type
    }
}
