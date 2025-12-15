use super::parse_state::ParseState;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_sgtype(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume SGTYPE_ keyword
        if parser.expect(crate::SGTYPE_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: SGTYPE_ type_name : size ;
        // Example: SGTYPE_ SignalType1 : 16;
        if let Ok(ext_type) = crate::SignalType::parse(parser) {
            state.signal_types_buffer.push(ext_type);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        let _ = parser.expect(crate::SGTYPE_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
