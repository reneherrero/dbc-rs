use super::parse_state::ParseState;
use crate::{Parser, Result, Version};

pub fn handle_version(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    // Version::parse expects VERSION keyword, don't consume it here
    state.version = Some(Version::parse(parser)?);
    Ok(())
}
