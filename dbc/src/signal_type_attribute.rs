//! Signal Type Attribute Value (BA_SGTYPE_)
//!
//! Assigns a value to a signal type attribute.

/// Signal Type Attribute Value (BA_SGTYPE_)
///
/// Assigns a value to a signal type attribute.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct SignalTypeAttribute {
    name: std::string::String,
    signal_type_name: std::string::String,
    value: crate::attributes::AttributeValue,
}

#[cfg(feature = "std")]
impl SignalTypeAttribute {
    /// Create a new SignalTypeAttribute
    pub(crate) fn new(
        name: std::string::String,
        signal_type_name: std::string::String,
        value: crate::attributes::AttributeValue,
    ) -> Self {
        Self {
            name,
            signal_type_name,
            value,
        }
    }

    /// Get the attribute name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the signal type name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn signal_type_name(&self) -> &str {
        &self.signal_type_name
    }

    /// Get the attribute value
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn value(&self) -> &crate::attributes::AttributeValue {
        &self.value
    }
}
