//! Parser implementations for attribute-related DBC sections.
//!
//! Handles parsing of:
//! - `BA_DEF_` - Attribute definitions
//! - `BA_DEF_DEF_` - Attribute default values
//! - `BA_` - Attribute value assignments

use crate::{
    MAX_NAME_SIZE, Parser,
    attribute::{
        AttributeDefinition, AttributeObjectType, AttributeString, AttributeTarget, AttributeValue,
        AttributeValueType, EnumValues, MAX_ATTRIBUTE_STRING_SIZE,
    },
    compat::Name,
};

impl AttributeDefinition {
    /// Parse a BA_DEF_ entry (after the keyword has been consumed).
    ///
    /// Format: `BA_DEF_ [object_type] "attr_name" value_type ;`
    ///
    /// Returns `None` if parsing fails or if this is an EV_ attribute (not supported).
    pub(crate) fn parse(parser: &mut Parser) -> Option<Self> {
        parser.skip_newlines_and_spaces();

        // Parse object_type (empty, BU_, BO_, SG_, EV_)
        let object_type = if parser.starts_with(b"BU_") {
            parser.expect(b"BU_").ok()?;
            AttributeObjectType::Node
        } else if parser.starts_with(b"BO_") {
            parser.expect(b"BO_").ok()?;
            AttributeObjectType::Message
        } else if parser.starts_with(b"SG_") {
            parser.expect(b"SG_").ok()?;
            AttributeObjectType::Signal
        } else if parser.starts_with(b"EV_") {
            // Skip environment variable attributes (not supported)
            return None;
        } else {
            // Empty = Network/database level
            AttributeObjectType::Network
        };

        parser.skip_newlines_and_spaces();

        // Parse quoted attribute name
        parser.expect(b"\"").ok()?;
        let name_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).ok()?;
        let name_str = core::str::from_utf8(name_bytes).ok()?;
        let name = Name::try_from(name_str).ok()?;

        parser.skip_newlines_and_spaces();

        // Parse value type
        let value_type = Self::parse_value_type(parser)?;

        Some(AttributeDefinition::new(name, object_type, value_type))
    }

    /// Parse the attribute value type specification.
    fn parse_value_type(parser: &mut Parser) -> Option<AttributeValueType> {
        parser.skip_newlines_and_spaces();

        if parser.starts_with(b"INT") {
            parser.expect(b"INT").ok()?;
            let (min, max) = Self::parse_int_range(parser)?;
            Some(AttributeValueType::Int { min, max })
        } else if parser.starts_with(b"HEX") {
            parser.expect(b"HEX").ok()?;
            let (min, max) = Self::parse_int_range(parser)?;
            Some(AttributeValueType::Hex { min, max })
        } else if parser.starts_with(b"FLOAT") {
            parser.expect(b"FLOAT").ok()?;
            let (min, max) = Self::parse_float_range(parser)?;
            Some(AttributeValueType::Float { min, max })
        } else if parser.starts_with(b"STRING") {
            parser.expect(b"STRING").ok()?;
            Some(AttributeValueType::String)
        } else if parser.starts_with(b"ENUM") {
            parser.expect(b"ENUM").ok()?;
            let values = Self::parse_enum_values(parser)?;
            Some(AttributeValueType::Enum { values })
        } else {
            None
        }
    }

    /// Parse integer range: `min max`
    fn parse_int_range(parser: &mut Parser) -> Option<(i64, i64)> {
        parser.skip_newlines_and_spaces();
        let min = parser.parse_i64().ok()?;
        parser.skip_newlines_and_spaces();
        let max = parser.parse_i64().ok()?;
        Some((min, max))
    }

    /// Parse float range: `min max`
    fn parse_float_range(parser: &mut Parser) -> Option<(f64, f64)> {
        parser.skip_newlines_and_spaces();
        let min = parser.parse_f64().ok()?;
        parser.skip_newlines_and_spaces();
        let max = parser.parse_f64().ok()?;
        Some((min, max))
    }

    /// Parse enum values: `"val1","val2",...`
    fn parse_enum_values(parser: &mut Parser) -> Option<EnumValues> {
        let mut values = EnumValues::new();

        loop {
            parser.skip_newlines_and_spaces();

            // Check for end of enum list (semicolon or end of line)
            if parser.starts_with(b";") || parser.at_newline() || parser.is_empty() {
                break;
            }

            // Parse quoted string
            if parser.expect(b"\"").is_err() {
                break;
            }

            let value_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).ok()?;
            let value_str = core::str::from_utf8(value_bytes).ok()?;
            let value = Name::try_from(value_str).ok()?;

            let _ = values.push(value);

            parser.skip_newlines_and_spaces();

            // Skip comma separator if present
            if parser.starts_with(b",") {
                let _ = parser.expect(b",");
            }
        }

        if values.is_empty() {
            None
        } else {
            Some(values)
        }
    }
}

/// Parse a BA_DEF_DEF_ entry (after the keyword has been consumed).
///
/// Format: `BA_DEF_DEF_ "attr_name" default_value ;`
///
/// Returns the attribute name and default value, or None if parsing fails.
pub(crate) fn parse_attribute_default(parser: &mut Parser) -> Option<(Name, AttributeValue)> {
    parser.skip_newlines_and_spaces();

    // Parse quoted attribute name
    parser.expect(b"\"").ok()?;
    let name_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).ok()?;
    let name_str = core::str::from_utf8(name_bytes).ok()?;
    let name = Name::try_from(name_str).ok()?;

    parser.skip_newlines_and_spaces();

    // Parse value
    let value = parse_attribute_value(parser)?;

    Some((name, value))
}

/// Parse a BA_ entry (after the keyword has been consumed).
///
/// Format: `BA_ "attr_name" [object_ref] value ;`
///
/// Returns the attribute name, target, and value, or None if parsing fails.
pub(crate) fn parse_attribute_assignment(
    parser: &mut Parser,
) -> Option<(Name, AttributeTarget, AttributeValue)> {
    parser.skip_newlines_and_spaces();

    // Parse quoted attribute name
    parser.expect(b"\"").ok()?;
    let name_bytes = parser.take_until_quote(false, MAX_NAME_SIZE).ok()?;
    let name_str = core::str::from_utf8(name_bytes).ok()?;
    let name = Name::try_from(name_str).ok()?;

    parser.skip_newlines_and_spaces();

    // Parse target (determines which object type)
    let target = if parser.starts_with(b"BU_") {
        parser.expect(b"BU_").ok()?;
        parser.skip_newlines_and_spaces();
        let node_name_bytes = parser.parse_identifier().ok()?;
        let node_name = Name::try_from(node_name_bytes).ok()?;
        AttributeTarget::Node(node_name)
    } else if parser.starts_with(b"BO_") {
        parser.expect(b"BO_").ok()?;
        parser.skip_newlines_and_spaces();
        let msg_id = parser.parse_u32().ok()?;
        AttributeTarget::Message(msg_id)
    } else if parser.starts_with(b"SG_") {
        parser.expect(b"SG_").ok()?;
        parser.skip_newlines_and_spaces();
        let msg_id = parser.parse_u32().ok()?;
        parser.skip_newlines_and_spaces();
        let signal_name_bytes = parser.parse_identifier().ok()?;
        let signal_name = Name::try_from(signal_name_bytes).ok()?;
        AttributeTarget::Signal(msg_id, signal_name)
    } else if parser.starts_with(b"EV_") {
        // Skip environment variable attributes (not supported)
        return None;
    } else {
        // Network level attribute (no object reference)
        AttributeTarget::Network
    };

    parser.skip_newlines_and_spaces();

    // Parse value
    let value = parse_attribute_value(parser)?;

    Some((name, target, value))
}

/// Parse an attribute value (integer, float, or quoted string).
fn parse_attribute_value(parser: &mut Parser) -> Option<AttributeValue> {
    parser.skip_newlines_and_spaces();

    // Check for quoted string first
    if parser.starts_with(b"\"") {
        parser.expect(b"\"").ok()?;
        let str_bytes = parser.take_until_quote(false, MAX_ATTRIBUTE_STRING_SIZE).ok()?;
        let str_val = core::str::from_utf8(str_bytes).ok()?;
        let attr_str = AttributeString::try_from(str_val).ok()?;
        return Some(AttributeValue::String(attr_str));
    }

    // Check if the number contains a decimal point before parsing
    // This determines whether to return Int or Float
    let remaining = parser.remaining();
    let has_decimal = remaining
        .iter()
        .take_while(|&&b| b == b'-' || b == b'+' || b == b'.' || b.is_ascii_digit())
        .any(|&b| b == b'.');

    if has_decimal {
        // Parse as float if decimal point present
        if let Ok(float_val) = parser.parse_f64() {
            return Some(AttributeValue::Float(float_val));
        }
    } else {
        // Try integer first for values without decimal point
        if let Ok(int_val) = parser.parse_i64() {
            return Some(AttributeValue::Int(int_val));
        }
        // Fall back to float for very large numbers
        if let Ok(float_val) = parser.parse_f64() {
            return Some(AttributeValue::Float(float_val));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ba_def_int() {
        let input = b"BO_ \"GenMsgCycleTime\" INT 0 10000 ;";
        let mut parser = Parser::new(input).unwrap();
        let def = AttributeDefinition::parse(&mut parser).unwrap();
        assert_eq!(def.name(), "GenMsgCycleTime");
        assert_eq!(def.object_type(), AttributeObjectType::Message);
        assert!(matches!(
            def.value_type(),
            AttributeValueType::Int { min: 0, max: 10000 }
        ));
    }

    #[test]
    fn test_parse_ba_def_string() {
        let input = b"\"BusType\" STRING ;";
        let mut parser = Parser::new(input).unwrap();
        let def = AttributeDefinition::parse(&mut parser).unwrap();
        assert_eq!(def.name(), "BusType");
        assert_eq!(def.object_type(), AttributeObjectType::Network);
        assert!(matches!(def.value_type(), AttributeValueType::String));
    }

    #[test]
    fn test_parse_ba_def_enum() {
        let input = b"BO_ \"VFrameFormat\" ENUM \"StandardCAN\",\"ExtendedCAN\",\"J1939PG\" ;";
        let mut parser = Parser::new(input).unwrap();
        let def = AttributeDefinition::parse(&mut parser).unwrap();
        assert_eq!(def.name(), "VFrameFormat");
        assert_eq!(def.object_type(), AttributeObjectType::Message);
        if let AttributeValueType::Enum { values } = def.value_type() {
            assert_eq!(values.len(), 3);
            assert_eq!(values.as_slice()[0].as_str(), "StandardCAN");
            assert_eq!(values.as_slice()[1].as_str(), "ExtendedCAN");
            assert_eq!(values.as_slice()[2].as_str(), "J1939PG");
        } else {
            panic!("Expected Enum type");
        }
    }

    #[test]
    fn test_parse_ba_def_def_int() {
        let input = b"\"GenMsgCycleTime\" 100 ;";
        let mut parser = Parser::new(input).unwrap();
        let (name, value) = parse_attribute_default(&mut parser).unwrap();
        assert_eq!(name.as_str(), "GenMsgCycleTime");
        assert_eq!(value.as_int(), Some(100));
    }

    #[test]
    fn test_parse_ba_def_def_string() {
        let input = b"\"BusType\" \"CAN\" ;";
        let mut parser = Parser::new(input).unwrap();
        let (name, value) = parse_attribute_default(&mut parser).unwrap();
        assert_eq!(name.as_str(), "BusType");
        assert_eq!(value.as_string(), Some("CAN"));
    }

    #[test]
    fn test_parse_ba_message() {
        let input = b"\"GenMsgCycleTime\" BO_ 256 50 ;";
        let mut parser = Parser::new(input).unwrap();
        let (name, target, value) = parse_attribute_assignment(&mut parser).unwrap();
        assert_eq!(name.as_str(), "GenMsgCycleTime");
        assert!(matches!(target, AttributeTarget::Message(256)));
        assert_eq!(value.as_int(), Some(50));
    }

    #[test]
    fn test_parse_ba_signal() {
        let input = b"\"GenSigStartValue\" SG_ 256 RPM 0.0 ;";
        let mut parser = Parser::new(input).unwrap();
        let (name, target, value) = parse_attribute_assignment(&mut parser).unwrap();
        assert_eq!(name.as_str(), "GenSigStartValue");
        if let AttributeTarget::Signal(msg_id, sig_name) = target {
            assert_eq!(msg_id, 256);
            assert_eq!(sig_name.as_str(), "RPM");
        } else {
            panic!("Expected Signal target");
        }
        assert_eq!(value.as_float(), Some(0.0));
    }

    #[test]
    fn test_parse_ba_network() {
        let input = b"\"BusType\" \"CAN\" ;";
        let mut parser = Parser::new(input).unwrap();
        let (name, target, value) = parse_attribute_assignment(&mut parser).unwrap();
        assert_eq!(name.as_str(), "BusType");
        assert!(matches!(target, AttributeTarget::Network));
        assert_eq!(value.as_string(), Some("CAN"));
    }
}
