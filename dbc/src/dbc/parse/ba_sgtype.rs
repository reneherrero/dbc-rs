use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::{Error, SignalTypeAttribute};
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_ba_sgtype(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume BA_SGTYPE_ keyword
        if parser.expect(crate::BA_SGTYPE_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: BA_SGTYPE_ "attribute_name" signal_type_name value ;
        if let Ok(attr) = (|| -> Result<SignalTypeAttribute> {
            // Parse attribute name (quoted string)
            parser.expect(b"\"").map_err(|_| Error::Expected("Expected opening quote"))?;
            let name_bytes = parser
                .take_until_quote(false, crate::MAX_NAME_SIZE)
                .map_err(|_| Error::Expected("Expected closing quote"))?;
            let name_str = core::str::from_utf8(name_bytes)
                .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
            let name = name_str.to_string();
            parser.skip_newlines_and_spaces();

            // Parse signal_type_name (identifier)
            let signal_type_name = parser.parse_identifier()?;
            let signal_type_name = crate::validate_name(signal_type_name)?;
            let signal_type_name = signal_type_name.as_str().to_string();
            parser.skip_newlines_and_spaces();

            // Parse value (can be integer, float, or string)
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
                        let float_val = parser.parse_f64().map_err(|_| {
                            Error::Expected("Expected attribute value (integer, float, or string)")
                        })?;
                        crate::attributes::AttributeValue::Float(float_val)
                    }
                }
            };

            parser.skip_newlines_and_spaces();
            parser.expect(b";").ok(); // Semicolon is optional but common

            Ok(SignalTypeAttribute::new(name, signal_type_name, value))
        })() {
            state.signal_type_attributes_buffer.push(attr);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume BA_SGTYPE_ keyword and skip the rest
        let _ = parser.expect(crate::BA_SGTYPE_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
