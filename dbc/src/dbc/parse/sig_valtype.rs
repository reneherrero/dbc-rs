use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::Error;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_sig_valtype(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Parse SIG_VALTYPE_ message_id signal_name : value_type ;
        if parser.expect(crate::SIG_VALTYPE_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        if let Ok((message_id, signal_name, value_type)) =
            (|| -> Result<(u32, std::string::String, crate::SignalExtendedValueType)> {
                let message_id = parser.parse_u32()?;
                parser.skip_newlines_and_spaces();
                let signal_name = parser.parse_identifier()?;
                let signal_name = signal_name.to_string();
                parser.skip_newlines_and_spaces();

                // Expect colon
                parser
                    .expect(b":")
                    .map_err(|_| Error::Expected("Expected ':' after signal name"))?;
                parser.skip_newlines_and_spaces();

                // Parse value type (0, 1, or 2)
                let value_type_num = parser.parse_u32()?;
                if value_type_num > 2 {
                    return Err(Error::Validation(Error::INVALID_SIGNAL_VALUE_TYPE));
                }
                let value_type = crate::SignalExtendedValueType::from_u8(value_type_num as u8)?;

                parser.skip_newlines_and_spaces();
                parser.expect(b";").ok(); // Semicolon is optional

                Ok((message_id, signal_name, value_type))
            })()
        {
            state.signal_value_types_buffer.insert((message_id, signal_name), value_type);
        } else {
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        let _ = parser.expect(crate::SIG_VALTYPE_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
