use super::parse_state::ParseState;
use crate::{BitTiming, Parser, Result};

pub fn handle_bs(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    // Parse BS_: [baudrate ':' BTR1 ',' BTR2]
    if parser.expect(crate::BS_.as_bytes()).is_err() {
        parser.skip_to_end_of_line();
        return Ok(());
    }
    parser.skip_newlines_and_spaces();

    // Expect colon after BS_
    if parser.expect(b":").is_err() {
        parser.skip_to_end_of_line();
        return Ok(());
    }
    parser.skip_newlines_and_spaces();

    // Parse optional baudrate
    let baudrate = parser.parse_u32().ok();
    parser.skip_newlines_and_spaces();

    let (btr1, btr2) = if parser.expect(b":").is_ok() {
        // Parse BTR1 and BTR2
        parser.skip_newlines_and_spaces();
        let btr1_val = parser.parse_u32().ok();
        parser.skip_newlines_and_spaces();
        if parser.expect(b",").is_ok() {
            parser.skip_newlines_and_spaces();
            let btr2_val = parser.parse_u32().ok();
            (btr1_val, btr2_val)
        } else {
            (btr1_val, None)
        }
    } else {
        (None, None)
    };

    // Store bit timing (only first one if multiple)
    if state.bit_timing.is_none() {
        state.bit_timing = Some(BitTiming::new(baudrate, btr1, btr2));
    }

    // Skip to end of line in case of trailing content
    parser.skip_to_end_of_line();
    Ok(())
}
