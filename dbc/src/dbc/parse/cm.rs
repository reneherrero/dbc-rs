use super::parse_state::ParseState;
#[cfg(feature = "std")]
use crate::Error;
use crate::{Parser, Result};

#[allow(unused_variables)]
pub fn handle_cm(parser: &mut Parser, state: &mut ParseState) -> Result<()> {
    #[cfg(feature = "std")]
    {
        use crate::comment::Comment;
        // Consume CM_ keyword
        if parser.expect(crate::CM_.as_bytes()).is_err() {
            parser.skip_to_end_of_line();
            return Ok(());
        }
        parser.skip_newlines_and_spaces();

        // Parse comment: CM_ [object_type object] "text" ;
        // object_type can be: BU_, BO_, SG_, EV_, or empty (general comment)
        if let Ok(comment) = (|| -> Result<Comment> {
            let comment = if parser.starts_with(b"BU_") {
                // Node comment: CM_ BU_ node_name "text" ;
                parser.expect(b"BU_")?;
                parser.skip_newlines_and_spaces();
                let node_name = parser
                    .parse_identifier()
                    .ok()
                    .and_then(|n| crate::validate_name(n).ok())
                    .map(|s| s.as_str().to_string());
                parser.skip_newlines_and_spaces();
                let text = if parser.expect(b"\"").is_ok() {
                    let text_bytes = parser.take_until_quote(false, 1024)?;
                    core::str::from_utf8(text_bytes)
                        .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?
                        .to_string()
                } else {
                    String::new()
                };
                Comment::new(
                    crate::comment::CommentObjectType::Node,
                    node_name,
                    None,
                    text,
                )
            } else if parser.starts_with(b"BO_") {
                // Message comment: CM_ BO_ message_id "text" ;
                parser.expect(b"BO_")?;
                parser.skip_newlines_and_spaces();
                let message_id = parser.parse_u32().ok();
                parser.skip_newlines_and_spaces();
                let text = if parser.expect(b"\"").is_ok() {
                    let text_bytes = parser.take_until_quote(false, 1024)?;
                    core::str::from_utf8(text_bytes)
                        .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?
                        .to_string()
                } else {
                    String::new()
                };
                Comment::new(
                    crate::comment::CommentObjectType::Message,
                    None,
                    message_id,
                    text,
                )
            } else if parser.starts_with(b"SG_") {
                // Signal comment: CM_ SG_ message_id signal_name "text" ;
                parser.expect(b"SG_")?;
                parser.skip_newlines_and_spaces();
                let message_id = parser.parse_u32().ok();
                parser.skip_newlines_and_spaces();
                let signal_name = parser
                    .parse_identifier()
                    .ok()
                    .and_then(|n| crate::validate_name(n).ok())
                    .map(|s| s.as_str().to_string());
                parser.skip_newlines_and_spaces();
                let text = if parser.expect(b"\"").is_ok() {
                    let text_bytes = parser.take_until_quote(false, 1024)?;
                    core::str::from_utf8(text_bytes)
                        .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?
                        .to_string()
                } else {
                    String::new()
                };
                Comment::new(
                    crate::comment::CommentObjectType::Signal,
                    signal_name,
                    message_id,
                    text,
                )
            } else if parser.starts_with(b"EV_") {
                // Environment variable comment: CM_ EV_ env_var_name "text" ;
                parser.expect(b"EV_")?;
                parser.skip_newlines_and_spaces();
                let env_var_name = parser
                    .parse_identifier()
                    .ok()
                    .and_then(|n| crate::validate_name(n).ok())
                    .map(|s| s.as_str().to_string());
                parser.skip_newlines_and_spaces();
                let text = if parser.expect(b"\"").is_ok() {
                    let text_bytes = parser.take_until_quote(false, 1024)?;
                    core::str::from_utf8(text_bytes)
                        .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?
                        .to_string()
                } else {
                    String::new()
                };
                Comment::new(
                    crate::comment::CommentObjectType::EnvironmentVariable,
                    env_var_name,
                    None,
                    text,
                )
            } else {
                // General comment: CM_ "text" ;
                let text = if parser.expect(b"\"").is_ok() {
                    let text_bytes = parser.take_until_quote(false, 1024)?;
                    core::str::from_utf8(text_bytes)
                        .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?
                        .to_string()
                } else {
                    String::new()
                };
                Comment::new(crate::comment::CommentObjectType::General, None, None, text)
            };
            parser.skip_newlines_and_spaces();
            parser.expect(b";").ok(); // Semicolon is optional but common
            Ok(comment)
        })() {
            state.comments_buffer.push(comment);
        } else {
            // If parsing fails, just skip the line
            parser.skip_to_end_of_line();
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // In no_std mode, consume CM_ keyword and skip the rest
        let _ = parser.expect(crate::CM_.as_bytes()).ok();
        parser.skip_to_end_of_line();
    }
    Ok(())
}
