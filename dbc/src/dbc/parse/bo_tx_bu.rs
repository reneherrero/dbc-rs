use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::MessageTransmitter;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_bo_tx_bu(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        // Parse BO_TX_BU_ message_id : transmitter1 transmitter2 ... ;
        if parser.expect(crate::BO_TX_BU_.as_bytes()).is_err() {
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

        // Expect colon
        if parser.expect(b":").is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse transmitter node names (space or comma-separated list)
        let mut transmitters = std::vec::Vec::new();

        // Parse at least one transmitter or empty list (semicolon immediately)
        loop {
            parser.skip_newlines_and_spaces();

            // Check if we've reached semicolon (end of list, possibly empty)
            if parser.starts_with(b";") {
                parser.expect(b";").ok();
                break;
            }

            // Check if we've reached end of input
            if parser.peek_byte_at(0).is_none() {
                break;
            }

            // Try to parse an identifier
            match parser.parse_identifier() {
                Ok(name) => match crate::validate_name(name) {
                    Ok(valid_name) => {
                        transmitters.push(valid_name.as_str().to_string());
                        parser.skip_newlines_and_spaces();

                        // Check for comma (more transmitters coming)
                        if parser.starts_with(b",") {
                            parser.expect(b",").ok();
                            parser.skip_newlines_and_spaces();
                            // Continue to parse next transmitter
                            continue;
                        }

                        // Check for semicolon (end of list)
                        if parser.starts_with(b";") {
                            parser.expect(b";").ok();
                            break;
                        }

                        // No comma or semicolon - might be space-separated
                        // Continue loop to try parsing next identifier
                    }
                    Err(_) => {
                        // Invalid name, stop parsing
                        break;
                    }
                },
                Err(_) => {
                    // Can't parse identifier, stop parsing
                    break;
                }
            }
        }

        // Only push if we have transmitters (allow empty list)
        state
            .message_transmitters_buffer
            .push(MessageTransmitter::new(message_id, transmitters));
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume BO_TX_BU_ keyword and skip the rest
        let _ = parser.expect(crate::BO_TX_BU_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
