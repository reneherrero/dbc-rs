//! Signal Type Value Descriptions (SGTYPE_VAL_)

use crate::compat::String;

/// Represents Signal Type Value Descriptions (SGTYPE_VAL_)
///
/// Value descriptions for signal types, similar to VAL_ but for types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct SignalTypeValue {
    type_name: String<{ crate::MAX_NAME_SIZE }>,
    value: u64,
    description: String<{ crate::MAX_NAME_SIZE }>,
}

impl SignalTypeValue {
    /// Create a new SignalTypeValue
    pub(crate) fn new(
        type_name: String<{ crate::MAX_NAME_SIZE }>,
        value: u64,
        description: String<{ crate::MAX_NAME_SIZE }>,
    ) -> Self {
        Self {
            type_name,
            value,
            description,
        }
    }

    /// Get the signal type name
    #[must_use]
    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }

    /// Get the value
    #[must_use]
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Get the description
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_value_creation() {
        let type_name = crate::validate_name("SignalType1").unwrap();
        let description = crate::validate_name("Zero").unwrap();
        let value = SignalTypeValue::new(type_name, 0, description);
        assert_eq!(value.type_name(), "SignalType1");
        assert_eq!(value.value(), 0);
        assert_eq!(value.description(), "Zero");
    }

    #[test]
    fn test_signal_type_value_equality() {
        let type_name1 = crate::validate_name("SignalType1").unwrap();
        let description1 = crate::validate_name("Zero").unwrap();
        let type_name2 = crate::validate_name("SignalType1").unwrap();
        let description2 = crate::validate_name("Zero").unwrap();
        let val1 = SignalTypeValue::new(type_name1, 0, description1);
        let val2 = SignalTypeValue::new(type_name2, 0, description2);
        assert_eq!(val1, val2);
    }
}

