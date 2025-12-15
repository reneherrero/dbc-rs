use super::parse_state::ParseState;
use crate::{Nodes, Parser, Result};

pub fn handle_bu(
    parser: &mut Parser,
    state: &mut ParseState,
    pos_at_keyword: usize,
    data: &str,
) -> Result<()> {
    // Nodes::parse expects BU_ keyword and will parse the entire line
    // Extract just the BU_ line by finding the newline position
    let data_bytes = data.as_bytes();
    let line_end = {
        // Find newline starting from pos_at_keyword
        let slice = &data_bytes[pos_at_keyword..];
        slice
            .iter()
            .position(|&b| b == b'\n' || b == b'\r')
            .map(|pos| pos_at_keyword + pos)
            .unwrap_or(data_bytes.len())
    };

    // Create a parser from just the BU_ line (without newline)
    let bu_input = &data_bytes[pos_at_keyword..line_end];
    let mut bu_parser = Parser::new(bu_input)?;
    state.nodes = Some(Nodes::parse(&mut bu_parser)?);

    // Skip the main parser past the line (including newline) for next keyword
    parser.skip_to_end_of_line();
    Ok(())
}
