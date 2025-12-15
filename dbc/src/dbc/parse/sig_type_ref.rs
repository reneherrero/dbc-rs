use super::parse_state::ParseState;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_sig_type_ref(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Consume SIG_TYPE_REF_ keyword
        if parser.expect(crate::SIG_TYPE_REF_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse: SIG_TYPE_REF_ message_id signal_name : type_name ;
        // Example: SIG_TYPE_REF_ 256 RPM : SignalType1;
        if let Ok(ext_ref) = crate::SignalTypeReference::parse(parser) {
            state.signal_type_references_buffer.push(ext_ref);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        let _ = parser.expect(crate::SIG_TYPE_REF_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
