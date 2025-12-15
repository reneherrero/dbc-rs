use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::Error;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_ba(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume BA_ keyword
        if parser.expect(crate::BA_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: BA_ "attribute_name" [object_type object] value ;
        if let Ok(attr) = (|| -> Result<crate::attributes::Attribute> {
            // Parse attribute name (quoted string)
            parser.expect(b"\"").map_err(|_| Error::Expected("Expected opening quote"))?;
            let name_bytes = parser
                .take_until_quote(false, crate::MAX_NAME_SIZE)
                .map_err(|_| Error::Expected("Expected closing quote"))?;
            let name_str = core::str::from_utf8(name_bytes)
                .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
            let name = name_str.to_string();
            parser.skip_newlines_and_spaces();

            // Parse optional object type and identifier
            let (object_type, object_name, object_id) = if parser.starts_with(b"BU_") {
                parser.expect(b"BU_")?;
                parser.skip_newlines_and_spaces();
                let obj_name = parser.parse_identifier().ok().map(|n| n.to_string());
                (crate::attributes::AttributeObjectType::Node, obj_name, None)
            } else if parser.starts_with(b"BO_") {
                parser.expect(b"BO_")?;
                parser.skip_newlines_and_spaces();
                let msg_id = parser.parse_u32().ok();
                (
                    crate::attributes::AttributeObjectType::Message,
                    None,
                    msg_id,
                )
            } else if parser.starts_with(b"SG_") {
                parser.expect(b"SG_")?;
                parser.skip_newlines_and_spaces();
                let msg_id = parser.parse_u32().ok();
                parser.skip_newlines_and_spaces();
                let sig_name = parser.parse_identifier().ok().map(|n| n.to_string());
                (
                    crate::attributes::AttributeObjectType::Signal,
                    sig_name,
                    msg_id,
                )
            } else if parser.starts_with(b"EV_") {
                parser.expect(b"EV_")?;
                parser.skip_newlines_and_spaces();
                let env_name = parser.parse_identifier().ok().map(|n| n.to_string());
                (
                    crate::attributes::AttributeObjectType::EnvironmentVariable,
                    env_name,
                    None,
                )
            } else {
                // Network/global attribute
                (crate::attributes::AttributeObjectType::Network, None, None)
            };

            parser.skip_newlines_and_spaces();

            // Parse value (can be integer, hex, float, or string)
            let value = if parser.starts_with(b"\"") {
                // String value
                parser.expect(b"\"")?;
                let value_bytes = parser
                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                let value_str = core::str::from_utf8(value_bytes)
                    .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
                crate::attributes::AttributeValue::String(value_str.to_string())
            } else {
                // Try to parse as integer first
                match parser.parse_i64() {
                    Ok(int_val) => crate::attributes::AttributeValue::Int(int_val),
                    Err(_) => {
                        // If int fails, try float
                        // Note: parse_i64 may have advanced position, but parse_f64 should handle it
                        let float_val = parser.parse_f64().map_err(|_| {
                            Error::Expected("Expected attribute value (integer, float, or string)")
                        })?;
                        crate::attributes::AttributeValue::Float(float_val)
                    }
                }
            };

            parser.skip_newlines_and_spaces();
            parser.expect(b";").ok(); // Semicolon is optional but common

            Ok(crate::attributes::Attribute::new(
                name,
                object_type,
                object_name,
                object_id,
                value,
            ))
        })() {
            state.attribute_values_buffer.push(attr);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume BA_ keyword and skip the rest
        let _ = parser.expect(crate::BA_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
