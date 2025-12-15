use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::{Error, value_table::ValueTable};
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_val_table(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume VAL_TABLE_ keyword
        if parser.expect(crate::VAL_TABLE_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: VAL_TABLE_ table_name value1 "desc1" value2 "desc2" ... ;
        if let Ok(value_table) = (|| -> Result<ValueTable> {
            let table_name = parser.parse_identifier()?;
            let table_name_validated = crate::validate_name(table_name)?;
            let table_name = table_name_validated.as_str().to_string();
            parser.skip_newlines_and_spaces();

            let mut entries = std::vec::Vec::<(u64, std::string::String)>::new();

            loop {
                parser.skip_newlines_and_spaces();
                // Check for semicolon (end of VAL_TABLE_ statement)
                if parser.starts_with(b";") {
                    parser.expect(b";").ok();
                    break;
                }

                // Parse value
                let value = parser
                    .parse_i64()
                    .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?
                    as u64;
                parser.skip_newlines_and_spaces();

                // Parse description string
                parser.expect(b"\"").map_err(|_| Error::Expected("Expected opening quote"))?;
                let desc_bytes = parser
                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                let desc_str = core::str::from_utf8(desc_bytes)
                    .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
                let desc = desc_str.to_string();

                entries.push((value, desc));
            }

            Ok(ValueTable::new(table_name, entries))
        })() {
            state.value_tables_buffer.push(value_table);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume VAL_TABLE_ keyword and skip the rest
        let _ = parser.expect(crate::VAL_TABLE_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
