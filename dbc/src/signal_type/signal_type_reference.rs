//! Signal Type Reference (SIG_TYPE_REF_)

use crate::compat::String;

/// Represents a Signal Type Reference (SIG_TYPE_REF_)
///
/// Links a signal to a signal type definition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct SignalTypeReference {
    message_id: u32,
    signal_name: String<{ crate::MAX_NAME_SIZE }>,
    type_name: String<{ crate::MAX_NAME_SIZE }>,
}

impl SignalTypeReference {
    /// Create a new SignalTypeReference
    #[cfg_attr(not(feature = "std"), allow(dead_code))] // Only used in std parsing code
    pub(crate) fn new(
        message_id: u32,
        signal_name: String<{ crate::MAX_NAME_SIZE }>,
        type_name: String<{ crate::MAX_NAME_SIZE }>,
    ) -> Self {
        Self {
            message_id,
            signal_name,
            type_name,
        }
    }

    /// Get the message ID
    #[must_use]
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Get the signal name
    #[must_use]
    pub fn signal_name(&self) -> &str {
        self.signal_name.as_str()
    }

    /// Get the signal type name
    #[must_use]
    pub fn type_name(&self) -> &str {
        self.type_name.as_str()
    }
}

impl crate::signal_type::SignalTypeName for SignalTypeReference {
    fn signal_type_name(&self) -> &str {
        SignalTypeReference::type_name(self)
    }
}

#[cfg(feature = "std")]
impl SignalTypeReference {
    /// Parse a Signal Type Reference (SIG_TYPE_REF_)
    ///
    /// Format: SIG_TYPE_REF_ message_id signal_name : type_name ;
    /// Example: SIG_TYPE_REF_ 256 RPM : SignalType1;
    pub(crate) fn parse(parser: &mut crate::parser::Parser) -> crate::error::Result<Self> {
        parser.skip_newlines_and_spaces();
        let message_id = parser.parse_u32()?;
        parser.skip_newlines_and_spaces();
        let signal_name = parser.parse_identifier()?;
        let signal_name = crate::validate_name(signal_name)?;
        parser.skip_newlines_and_spaces();
        parser.expect(b":")?;
        parser.skip_newlines_and_spaces();
        let type_name = parser.parse_identifier()?;
        let type_name = crate::validate_name(type_name)?;
        parser.skip_newlines_and_spaces();
        // Semicolon is optional but common
        if parser.starts_with(b";") {
            parser.expect(b";").ok();
        }
        Ok(SignalTypeReference::new(message_id, signal_name, type_name))
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_reference_creation() {
        let signal_name = crate::validate_name("RPM").unwrap();
        let type_name = crate::validate_name("SignalType1").unwrap();
        let ref_ = SignalTypeReference::new(256, signal_name, type_name);
        assert_eq!(ref_.message_id(), 256);
        assert_eq!(ref_.signal_name(), "RPM");
        assert_eq!(ref_.type_name(), "SignalType1");
    }

    #[test]
    fn test_signal_type_reference_equality() {
        let signal_name1 = crate::validate_name("RPM").unwrap();
        let type_name1 = crate::validate_name("SignalType1").unwrap();
        let signal_name2 = crate::validate_name("RPM").unwrap();
        let type_name2 = crate::validate_name("SignalType1").unwrap();
        let ref1 = SignalTypeReference::new(256, signal_name1, type_name1);
        let ref2 = SignalTypeReference::new(256, signal_name2, type_name2);
        assert_eq!(ref1, ref2);
    }

    #[test]
    fn test_parse() {
        let content = b"256 RPM : SignalType1;";
        let mut parser = crate::Parser::new(content).unwrap();
        // Parser should be positioned after the keyword
        let ref_ = SignalTypeReference::parse(&mut parser).unwrap();
        assert_eq!(ref_.message_id(), 256);
        assert_eq!(ref_.signal_name(), "RPM");
        assert_eq!(ref_.type_name(), "SignalType1");
    }
}
