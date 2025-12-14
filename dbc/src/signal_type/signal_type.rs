//! Signal Type definition (SGTYPE_)

use crate::compat::String;

/// Represents a Signal Type definition (SGTYPE_)
///
/// Signal Types define reusable signal type definitions with a name and size.
/// Signals can reference these types using SIG_TYPE_REF_.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct SignalType {
    name: String<{ crate::MAX_NAME_SIZE }>,
    size: u8,
}

impl SignalType {
    /// Create a new SignalType
    #[cfg_attr(not(feature = "std"), allow(dead_code))] // Only used in std parsing code
    pub(crate) fn new(name: String<{ crate::MAX_NAME_SIZE }>, size: u8) -> Self {
        Self { name, size }
    }

    /// Get the signal type name
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the signal type size (in bits)
    #[must_use]
    pub fn size(&self) -> u8 {
        self.size
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_creation() {
        let name = crate::validate_name("SignalType1").unwrap();
        let signal_type = SignalType::new(name, 16);
        assert_eq!(signal_type.name(), "SignalType1");
        assert_eq!(signal_type.size(), 16);
    }

    #[test]
    fn test_signal_type_equality() {
        let name1 = crate::validate_name("SignalType1").unwrap();
        let name2 = crate::validate_name("SignalType1").unwrap();
        let st1 = SignalType::new(name1, 16);
        let st2 = SignalType::new(name2, 16);
        assert_eq!(st1, st2);
    }
}
