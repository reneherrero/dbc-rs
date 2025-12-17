use super::Parser;
use crate::{DBC_KEYWORDS, Error};

impl<'a> Parser<'a> {
    pub fn peek_next_keyword(&mut self) -> crate::Result<&'a str> {
        // Skip newlines and spaces to find the next keyword
        self.skip_newlines_and_spaces();

        // Check if we're at EOF
        if self.eof() {
            return Err(Error::UnexpectedEof);
        }

        // Try to match each keyword (checking longer ones first)
        // Note: This function does NOT advance the position - it only peeks at the keyword
        // Callers must consume the keyword themselves using expect()
        for keyword in DBC_KEYWORDS {
            let keyword_bytes = keyword.as_bytes();
            if self.starts_with(keyword_bytes) {
                // Check if the character after the keyword is a valid delimiter
                // Valid delimiters: whitespace, newline, colon (for keywords like "BU_:")
                // Note: underscore is allowed as it may be part of compound keywords
                let next_byte = self.peek_byte_at(keyword_bytes.len());
                let is_valid_delimiter = next_byte
                    .map(|b| matches!(b, b' ' | b'\t' | b':' | b'\n' | b'\r'))
                    .unwrap_or(true); // EOF is valid (end of input)

                if is_valid_delimiter {
                    // Keyword found (but don't advance position)
                    return Ok(keyword);
                }
            }
        }

        // No keyword matched
        Err(Error::Expected(Error::EXPECTED_KEYWORD))
    }
}
