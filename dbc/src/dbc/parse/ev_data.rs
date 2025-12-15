use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::{EnvironmentVariableData, Error};
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_ev_data(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Parse EV_DATA_ env_var_name : data_size ;
        // Same format as ENVVAR_DATA_
        if parser.expect(crate::EV_DATA_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        if let Ok(env_var_data) = (|| -> Result<EnvironmentVariableData> {
            // Parse env_var_name (identifier)
            let name = parser.parse_identifier()?;
            let name = crate::validate_name(name)?;
            let name = name.as_str().to_string();
            parser.skip_newlines_and_spaces();

            // Expect colon
            parser
                .expect(b":")
                .map_err(|_| Error::Expected("Expected ':' after env var name"))?;
            parser.skip_newlines_and_spaces();

            // Parse data_size (unsigned_integer)
            let data_size = parser
                .parse_u32()
                .map_err(|_| Error::Expected(crate::error::Error::EXPECTED_NUMBER))?;
            parser.skip_newlines_and_spaces();

            // Optional semicolon
            if parser.starts_with(b";") {
                parser.expect(b";").ok();
            }

            Ok(EnvironmentVariableData::new(name, data_size))
        })() {
            state.environment_variable_data_buffer.push(env_var_data);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume keyword and skip the rest
        let _ = parser.expect(crate::EV_DATA_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
