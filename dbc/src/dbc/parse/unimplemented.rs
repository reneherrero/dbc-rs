use crate::Parser;

pub fn handle_unimplemented(parser: &mut Parser, keyword: &str) -> crate::Result<()> {
    // Relationship attribute definition (not yet implemented)
    let _ = parser.expect(keyword.as_bytes()).ok();
    parser.skip_to_end_of_line();
    Ok(())
}
