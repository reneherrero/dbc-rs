use crate::{BO_, BS_, BU_, Error, Parser, Result, SG_, VERSION};

pub fn handle_ns(parser: &mut Parser) -> Result<()> {
    // Consume NS_ keyword
    parser
        .expect(crate::NS_.as_bytes())
        .map_err(|_| Error::Expected("Failed to consume NS_ keyword"))?;
    parser.skip_newlines_and_spaces();
    let _ = parser.expect(b":").ok();
    loop {
        parser.skip_newlines_and_spaces();
        if parser.is_empty() {
            break;
        }
        if parser.starts_with(b" ") || parser.starts_with(b"\t") {
            parser.skip_to_end_of_line();
            continue;
        }
        if parser.starts_with(b"//") {
            parser.skip_to_end_of_line();
            continue;
        }
        if parser.starts_with(BS_.as_bytes())
            || parser.starts_with(BU_.as_bytes())
            || parser.starts_with(BO_.as_bytes())
            || parser.starts_with(SG_.as_bytes())
            || parser.starts_with(VERSION.as_bytes())
        {
            break;
        }
        parser.skip_to_end_of_line();
    }
    Ok(())
}
