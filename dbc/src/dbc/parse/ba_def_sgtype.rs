use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::{Error, SignalTypeAttributeDefinition};
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_ba_def_sgtype(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume BA_DEF_SGTYPE_ keyword
        if parser.expect(crate::BA_DEF_SGTYPE_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: BA_DEF_SGTYPE_ "attribute_name" value_type ;
        // Similar to BA_DEF_ but without object_type (always for signal types)
        if let Ok(attr_def) = (|| -> Result<SignalTypeAttributeDefinition> {
            // Parse attribute name (quoted string)
            parser.expect(b"\"").map_err(|_| Error::Expected("Expected opening quote"))?;
            let name_bytes = parser
                .take_until_quote(false, crate::MAX_NAME_SIZE)
                .map_err(|_| Error::Expected("Expected closing quote"))?;
            let name_str = core::str::from_utf8(name_bytes)
                .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
            let name = name_str.to_string();
            parser.skip_newlines_and_spaces();

            // Parse value type (same as BA_DEF_)
            let value_type = if parser.starts_with(b"INT") {
                parser.expect(b"INT")?;
                parser.skip_newlines_and_spaces();
                let min = parser
                    .parse_i64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
                parser.skip_newlines_and_spaces();
                let max = parser
                    .parse_i64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
                crate::attributes::AttributeValueType::Int(min, max)
            } else if parser.starts_with(b"HEX") {
                parser.expect(b"HEX")?;
                parser.skip_newlines_and_spaces();
                let min = parser
                    .parse_i64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
                parser.skip_newlines_and_spaces();
                let max = parser
                    .parse_i64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
                crate::attributes::AttributeValueType::Hex(min, max)
            } else if parser.starts_with(b"FLOAT") {
                parser.expect(b"FLOAT")?;
                parser.skip_newlines_and_spaces();
                let min = parser
                    .parse_f64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
                parser.skip_newlines_and_spaces();
                let max = parser
                    .parse_f64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
                crate::attributes::AttributeValueType::Float(min, max)
            } else if parser.starts_with(b"STRING") {
                parser.expect(b"STRING")?;
                crate::attributes::AttributeValueType::String
            } else if parser.starts_with(b"ENUM") {
                parser.expect(b"ENUM")?;
                parser.skip_newlines_and_spaces();
                let mut enum_values = std::vec::Vec::<std::string::String>::new();
                loop {
                    parser.skip_newlines_and_spaces();
                    if parser.starts_with(b";") {
                        break;
                    }
                    parser.expect(b"\"").map_err(|_| Error::Expected("Expected opening quote"))?;
                    let enum_bytes = parser
                        .take_until_quote(false, crate::MAX_NAME_SIZE)
                        .map_err(|_| Error::Expected("Expected closing quote"))?;
                    let enum_str = core::str::from_utf8(enum_bytes)
                        .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
                    enum_values.push(enum_str.to_string());
                    parser.skip_newlines_and_spaces();
                    if parser.starts_with(b",") {
                        parser.expect(b",").ok();
                        // Continue to next enum value
                    } else {
                        // End of enum values (semicolon or end of input)
                        break;
                    }
                }
                crate::attributes::AttributeValueType::Enum(enum_values)
            } else {
                return Err(Error::Expected(
                    "Expected attribute value type (INT, HEX, FLOAT, STRING, or ENUM)",
                ));
            };

            parser.skip_newlines_and_spaces();
            parser.expect(b";").ok(); // Semicolon is optional but common

            Ok(SignalTypeAttributeDefinition::new(name, value_type))
        })() {
            state.signal_type_attribute_definitions_buffer.push(attr_def);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume BA_DEF_SGTYPE_ keyword and skip the rest
        let _ = parser.expect(crate::BA_DEF_SGTYPE_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
