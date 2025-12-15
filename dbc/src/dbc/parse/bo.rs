use super::parse_state::ParseState;
use crate::compat::Vec;
use crate::{Error, MAX_MESSAGES, MAX_SIGNALS_PER_MESSAGE, Message, Parser, Result, Signal};

pub fn handle_bo(
    parser: &mut Parser,
    state: &mut ParseState,
    pos_at_keyword: usize,
    data: &str,
) -> Result<()> {
    // Check limit using MAX_MESSAGES constant
    if state.message_count_actual >= MAX_MESSAGES {
        return Err(Error::Nodes(Error::NODES_TOO_MANY));
    }

    // Save parser position (at BO_ keyword, so Message::parse can consume it)
    let message_start_pos = pos_at_keyword;

    // Don't manually parse - just find where the header ends by looking for the colon and sender
    // We need to find the end of the header line to separate it from signals
    let header_line_end = {
        // Skip to end of line to find where header ends
        let mut temp_parser = Parser::new(&data.as_bytes()[pos_at_keyword..])?;
        // Skip BO_ keyword
        temp_parser.expect(crate::BO_.as_bytes()).ok();
        temp_parser.skip_whitespace().ok();
        temp_parser.parse_u32().ok(); // ID
        temp_parser.skip_whitespace().ok();
        temp_parser.parse_identifier().ok(); // name
        temp_parser.skip_whitespace().ok();
        temp_parser.expect(b":").ok(); // colon
        temp_parser.skip_whitespace().ok();
        temp_parser.parse_u8().ok(); // DLC
        temp_parser.skip_whitespace().ok();
        temp_parser.parse_identifier().ok(); // sender
        pos_at_keyword + temp_parser.pos()
    };

    // Now parse signals from the original parser
    parser.skip_to_end_of_line(); // Skip past header line

    let mut signals_array: Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }> = Vec::new();

    loop {
        parser.skip_newlines_and_spaces();
        if parser.starts_with(crate::SG_.as_bytes()) {
            if let Some(next_byte) = parser.peek_byte_at(3) {
                if matches!(next_byte, b' ' | b'\n' | b'\r' | b'\t') {
                    if signals_array.len() >= MAX_SIGNALS_PER_MESSAGE {
                        return Err(Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY));
                    }
                    // Signal::parse expects SG_ keyword, which we've already verified with starts_with
                    let signal = Signal::parse(parser)?;
                    signals_array
                        .push(signal)
                        .map_err(|_| Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY))?;
                    continue;
                }
            }
        }
        break;
    }

    // Restore parser to start of message line and use Message::parse
    // Create a new parser from the original input, but only up to the end of the header
    // (not including signals, so Message::parse doesn't complain about extra content)
    let message_input = &data.as_bytes()[message_start_pos..header_line_end];
    let mut message_parser = Parser::new(message_input)?;

    // Use Message::parse which will parse the header and use our signals
    let message = Message::parse(&mut message_parser, signals_array.as_slice())?;

    state
        .messages_buffer
        .push(message)
        .map_err(|_| Error::Message(Error::NODES_TOO_MANY))?;
    state.message_count_actual += 1;
    Ok(())
}
