//! Parsing logic for Signal Types

use crate::error::Result;
use crate::parser::Parser;

use super::{SignalType, SignalTypeReference, SignalTypeValue};

/// Parse a Signal Type definition (SGTYPE_)
///
/// Format: SGTYPE_ type_name : size ;
/// Example: SGTYPE_ SignalType1 : 16;
pub(crate) fn parse_signal_type(parser: &mut Parser) -> Result<SignalType> {
    parser.skip_newlines_and_spaces();
    let type_name = parser.parse_identifier()?;
    let type_name = crate::validate_name(type_name)?;
    parser.skip_newlines_and_spaces();
    parser.expect(b":")?;
    parser.skip_newlines_and_spaces();
    let size = parser.parse_u32()? as u8;
    parser.skip_newlines_and_spaces();
    // Semicolon is optional but common
    if parser.starts_with(b";") {
        parser.expect(b";").ok();
    }
    Ok(SignalType::new(type_name, size))
}

/// Parse a Signal Type Reference (SIG_TYPE_REF_)
///
/// Format: SIG_TYPE_REF_ message_id signal_name : type_name ;
/// Example: SIG_TYPE_REF_ 256 RPM : SignalType1;
pub(crate) fn parse_signal_type_reference(parser: &mut Parser) -> Result<SignalTypeReference> {
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

/// Parse Signal Type Value Descriptions (SGTYPE_VAL_)
///
/// Format: SGTYPE_VAL_ type_name value "description" value "description" ... ;
/// Example: SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One" 2 "Two";
pub(crate) fn parse_signal_type_values(
    parser: &mut Parser,
) -> Result<std::vec::Vec<SignalTypeValue>> {
    parser.skip_newlines_and_spaces();
    let type_name = parser.parse_identifier()?;
    let type_name = crate::validate_name(type_name)?;
    parser.skip_newlines_and_spaces();

    let mut values = std::vec::Vec::new();

    loop {
        parser.skip_newlines_and_spaces();
        if parser.starts_with(b";") {
            parser.expect(b";").ok();
            break;
        }
        if parser.is_empty() {
            break;
        }

        // Parse value
        if let Ok(value) = parser.parse_u32() {
            parser.skip_newlines_and_spaces();
            // Parse description (quoted string)
            let description = if parser.expect(b"\"").is_ok() {
                let desc_bytes = parser.take_until_quote(false, 1024).ok();
                if let Some(desc_bytes) = desc_bytes {
                    core::str::from_utf8(desc_bytes)
                        .ok()
                        .and_then(|s| crate::validate_name(s).ok())
                        .unwrap_or_default()
                } else {
                    crate::compat::String::<{ crate::MAX_NAME_SIZE }>::new()
                }
            } else {
                crate::compat::String::<{ crate::MAX_NAME_SIZE }>::new()
            };
            values.push(SignalTypeValue::new(
                type_name.clone(),
                value as u64,
                description,
            ));
        } else {
            // Not a value, might be end of line or semicolon
            if parser.starts_with(b";") {
                parser.expect(b";").ok();
            }
            break;
        }
    }

    Ok(values)
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_parse_signal_type() {
        let content = b"SignalType1 : 16;";
        let mut parser = Parser::new(content).unwrap();
        // Parser should be positioned after the keyword
        let signal_type = parse_signal_type(&mut parser).unwrap();
        assert_eq!(signal_type.name(), "SignalType1");
        assert_eq!(signal_type.size(), 16);
    }

    #[test]
    fn test_parse_signal_type_reference() {
        let content = b"256 RPM : SignalType1;";
        let mut parser = Parser::new(content).unwrap();
        // Parser should be positioned after the keyword
        let ref_ = parse_signal_type_reference(&mut parser).unwrap();
        assert_eq!(ref_.message_id(), 256);
        assert_eq!(ref_.signal_name(), "RPM");
        assert_eq!(ref_.type_name(), "SignalType1");
    }

    #[test]
    fn test_parse_signal_type_values() {
        let content = b"SignalType1 0 \"Zero\" 1 \"One\" 2 \"Two\";";
        let mut parser = Parser::new(content).unwrap();
        // Parser should be positioned after the keyword
        let values = parse_signal_type_values(&mut parser).unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].value(), 0);
        assert_eq!(values[0].description(), "Zero");
        assert_eq!(values[1].value(), 1);
        assert_eq!(values[1].description(), "One");
        assert_eq!(values[2].value(), 2);
        assert_eq!(values[2].description(), "Two");
    }
}
