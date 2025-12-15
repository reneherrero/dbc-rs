use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::SignalGroup;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_sig_group(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Parse SIG_GROUP_ message_id signal_group_name repetitions signal_name1 signal_name2 ... ;
        if parser.expect(crate::SIG_GROUP_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse message_id
        let message_id = match parser.parse_u32() {
            Ok(id) => id,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();

        // Parse signal_group_name
        let signal_group_name = match parser.parse_identifier() {
            Ok(name) => match crate::validate_name(name) {
                Ok(valid_name) => valid_name.as_str().to_string(),
                Err(_) => {
                    parser.skip_to_end_of_line();
                    return Ok(());
                }
            },
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();

        // Parse repetitions
        let repetitions = match parser.parse_u32() {
            Ok(rep) => rep,
            Err(_) => {
                parser.skip_to_end_of_line();
                return Ok(());
            }
        };
        parser.skip_newlines_and_spaces();

        // Optional colon after repetitions (some tools use it, spec doesn't require it)
        if parser.starts_with(b":") {
            parser.expect(b":").ok();
            parser.skip_newlines_and_spaces();
        }

        // Parse signal names (space-separated list)
        let mut signal_names = std::vec::Vec::new();
        loop {
            // Check if we've reached semicolon or end of input
            if parser.starts_with(b";") {
                break;
            }
            // Check if we've reached end of input
            if parser.peek_byte_at(0).is_none() {
                break;
            }
            match parser.parse_identifier() {
                Ok(name) => match crate::validate_name(name) {
                    Ok(valid_name) => {
                        signal_names.push(valid_name.as_str().to_string());
                        parser.skip_newlines_and_spaces();
                    }
                    Err(_) => break,
                },
                Err(_) => break,
            }
        }

        // Optional semicolon
        if parser.starts_with(b";") {
            parser.expect(b";").ok();
        }

        state.signal_groups_buffer.push(SignalGroup::new(
            message_id,
            signal_group_name,
            repetitions,
            signal_names,
        ));
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume SIG_GROUP_ keyword and skip the rest
        let _ = parser.expect(crate::SIG_GROUP_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
