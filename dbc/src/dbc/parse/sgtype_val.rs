use super::parse_state::ParseState;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_sgtype_val(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume SGTYPE_VAL_ keyword
        if parser.expect(crate::SGTYPE_VAL_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: SGTYPE_VAL_ type_name value "description" value "description" ... ;
        // Example: SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One" 2 "Two";
        if let Ok(values) = crate::SignalTypeValue::parse(parser) {
            state.signal_type_values_buffer.extend(values);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        let _ = parser.expect(crate::SGTYPE_VAL_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
