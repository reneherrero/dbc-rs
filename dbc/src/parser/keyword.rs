use super::Parser;
use crate::{DBC_KEYWORDS, Error};

impl<'a> Parser<'a> {
    pub fn peek_next_keyword(&mut self) -> crate::Result<&'a str> {
        // Skip newlines and spaces to find the next keyword
        self.skip_newlines_and_spaces();

        // Optimize: cache input_len to avoid repeated calls
        let input_len = self.input.len();
        // Check if we're at EOF
        if self.pos >= input_len {
            return Err(Error::UnexpectedEof);
        }

        // Try to match each keyword (checking longer ones first)
        // Note: This function does NOT advance the position - it only peeks at the keyword
        // Callers must consume the keyword themselves using expect()
        for keyword in DBC_KEYWORDS {
            let keyword_bytes = keyword.as_bytes();
            if self.starts_with(keyword_bytes) {
                // Check if the next character after the keyword is a valid delimiter
                let next_pos = self.pos + keyword_bytes.len();
                if next_pos >= input_len {
                    // End of input, keyword is valid (but don't advance position)
                    return Ok(keyword);
                }

                let next_byte = self.input[next_pos];
                // Valid delimiters: whitespace, newline, colon (for keywords like "BU_:")
                // Note: underscore is allowed as it may be part of compound keywords
                if matches!(next_byte, b' ' | b'\n' | b'\r' | b'\t' | b':') {
                    // Keyword found (but don't advance position)
                    return Ok(keyword);
                }
            }
        }

        // No keyword matched
        Err(Error::Expected(Error::EXPECTED_KEYWORD))
    }
}
