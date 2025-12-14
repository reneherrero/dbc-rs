use crate::{DBC_KEYWORDS, Error, Result};

#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::UnexpectedEof);
        }
        Ok(Self {
            input,
            pos: 0,
            line: 1,
        })
    }

    #[inline]
    #[must_use]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)] // Public API for error reporting - will be used in future error messages
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn skip_whitespace(&mut self) -> Result<&mut Self> {
        let input_len = self.input.len();
        if self.pos >= input_len {
            return Err(Error::UnexpectedEof);
        }

        if self.input[self.pos] == b' ' {
            // Skip consecutive spaces (optimize: use cached input_len)
            while self.pos + 1 < input_len && self.input[self.pos + 1] == b' ' {
                self.pos += 1;
            }
            self.pos += 1; // Skip the last space
            Ok(self)
        } else {
            Err(Error::Expected(Error::EXPECTED_WHITESPACE))
        }
    }

    pub fn skip_newlines_and_spaces(&mut self) {
        let input_len = self.input.len();
        while self.pos < input_len {
            match self.input[self.pos] {
                b'\n' => {
                    self.pos += 1;
                    self.line += 1;
                }
                b'\r' => {
                    self.pos += 1;
                    // Handle \r\n as a single newline
                    if self.pos < input_len && self.input[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                    self.line += 1;
                }
                b' ' | b'\t' => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    pub fn peek_next_keyword(&mut self) -> Result<&'a str> {
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

    pub fn expect(&mut self, expected: &[u8]) -> Result<&mut Self> {
        if expected.is_empty() {
            return Ok(self);
        }

        // Optimize: cache input_len to avoid repeated calls
        let input_len = self.input.len();
        // Check if we have enough remaining bytes
        if input_len - self.pos < expected.len() {
            return Err(Error::Expected(Error::EXPECTED_PATTERN));
        }

        if self.starts_with(expected) {
            // Count newlines in the bytes we're about to skip
            // Optimize: cache scan_end and use single-pass algorithm
            let end_pos = self.pos + expected.len();
            let scan_end = end_pos.min(input_len);
            let mut i = self.pos;
            while i < scan_end {
                match self.input[i] {
                    b'\n' => {
                        self.line += 1;
                        i += 1;
                    }
                    b'\r' => {
                        // Check if followed by \n within the range
                        if i + 1 < scan_end && self.input[i + 1] == b'\n' {
                            // \r\n sequence - count as one newline, skip both
                            i += 2;
                            self.line += 1;
                        } else {
                            // Standalone \r
                            self.line += 1;
                            i += 1;
                        }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            self.pos = end_pos;
            Ok(self)
        } else {
            Err(Error::Expected(Error::EXPECTED_PATTERN))
        }
    }

    pub fn take_until_quote(
        &mut self,
        c_identifier: bool,
        max_str_length: usize,
    ) -> Result<&'a [u8]> {
        let start_pos = self.pos;
        let input_len = self.input.len();
        let max_pos = start_pos.saturating_add(max_str_length + 1); // +1 to account for quote
        let mut is_first_char = true;

        while self.pos < input_len {
            // Check length before processing byte (optimize: check max_pos instead of calculating length)
            if self.pos >= max_pos {
                return Err(Error::MaxStrLength(max_str_length));
            }

            let byte = self.input[self.pos];

            match byte {
                b'"' => {
                    // Found the quote, return slice up to (but not including) the quote
                    let slice = &self.input[start_pos..self.pos];
                    // Advance position past the quote
                    self.pos += 1;
                    return Ok(slice);
                }
                b'\\' | b'\t' | b'\n' | b'\r' => {
                    return Err(Error::InvalidChar(byte as char));
                }
                _ => {
                    if c_identifier {
                        if is_first_char {
                            // First char must be alpha or underscore
                            if !(byte.is_ascii_alphabetic() || byte == b'_') {
                                return Err(Error::InvalidChar(byte as char));
                            }
                            is_first_char = false;
                        } else {
                            // Subsequent chars must be alphanumeric or underscore
                            if !(byte.is_ascii_alphanumeric() || byte == b'_') {
                                return Err(Error::InvalidChar(byte as char));
                            }
                        }
                    } else {
                        // For non-c_identifier, allow any byte except control characters and quote
                        // This allows UTF-8 multi-byte sequences
                        // Only reject control characters (0-31) and DEL (127)
                        // Note: We can't validate complete UTF-8 sequences here, but we allow
                        // any byte that's not a control character, quote, or backslash
                        if byte < 32 || byte == 127 {
                            // Control character or DEL - reject
                            return Err(Error::InvalidChar(byte as char));
                        }
                        // Allow all other bytes (including UTF-8 continuation bytes)
                    }
                    self.pos += 1;
                }
            }
        }

        // Reached EOF without finding quote
        Err(Error::UnexpectedEof)
    }

    #[inline]
    #[must_use]
    fn remaining(&self) -> &'a [u8] {
        &self.input[self.pos..]
    }

    #[inline]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.remaining().is_empty()
    }

    #[inline]
    #[must_use]
    pub(crate) fn starts_with(&self, pattern: &[u8]) -> bool {
        self.remaining().starts_with(pattern)
    }

    #[inline]
    #[must_use]
    pub(crate) fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    pub(crate) fn skip_to_end_of_line(&mut self) {
        let input_len = self.input.len();
        while self.pos < input_len {
            let byte = self.input[self.pos];
            match byte {
                b'\n' => {
                    self.pos += 1;
                    self.line += 1;
                    break;
                }
                b'\r' => {
                    self.pos += 1;
                    // Handle \r\n
                    if self.pos < input_len && self.input[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                    self.line += 1;
                    break;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    pub(crate) fn parse_u8(&mut self) -> Result<u8> {
        let start_pos = self.pos;
        let input_len = self.input.len();
        // Read until whitespace, colon, or end of input
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.pos += 1;
            } else if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':') {
                break;
            } else {
                return Err(
                    self.restore_pos_err(start_pos, Error::Expected(Error::EXPECTED_NUMBER))
                );
            }
        }

        if self.pos == start_pos {
            return Err(Error::Expected(Error::EXPECTED_NUMBER));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<u8>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    /// Helper to restore position and return an error.
    /// Used to avoid duplicating the pattern of restoring position on error.
    #[inline]
    fn restore_pos_err(&mut self, pos: usize, err: Error) -> Error {
        self.pos = pos;
        err
    }

    pub(crate) fn parse_u32(&mut self) -> Result<u32> {
        let start_pos = self.pos;
        let input_len = self.input.len();
        // Read until whitespace, colon, pipe, @, semicolon, or end of input
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.pos += 1;
            } else if matches!(
                byte,
                b' ' | b'\t' | b'\n' | b'\r' | b':' | b'|' | b'@' | b';'
            ) {
                break;
            } else {
                return Err(
                    self.restore_pos_err(start_pos, Error::Expected(Error::EXPECTED_NUMBER))
                );
            }
        }

        if self.pos == start_pos {
            return Err(Error::Expected(Error::EXPECTED_NUMBER));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<u32>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    #[cfg(feature = "std")]
    pub(crate) fn parse_i64(&mut self) -> Result<i64> {
        let start_pos = self.pos;
        let mut has_sign = false;

        // Check for optional sign
        if self.pos < self.input.len() && self.input[self.pos] == b'-' {
            has_sign = true;
            self.pos += 1;
        }

        // Read digits
        let input_len = self.input.len();
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.pos += 1;
            } else if matches!(
                byte,
                b' ' | b'\t' | b'\n' | b'\r' | b':' | b'|' | b'@' | b';'
            ) {
                break;
            } else {
                return Err(
                    self.restore_pos_err(start_pos, Error::Expected(Error::EXPECTED_NUMBER))
                );
            }
        }

        // Check if we parsed anything (accounting for optional sign)
        if self.pos == start_pos || (has_sign && self.pos == start_pos + 1) {
            return Err(self.restore_pos_err(start_pos, Error::Expected(Error::EXPECTED_NUMBER)));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<i64>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    pub(crate) fn parse_f64(&mut self) -> Result<f64> {
        let start_pos = self.pos;
        let mut has_dot = false;
        let mut has_e = false;

        // Allow leading sign (+ or -)
        if self.pos < self.input.len() {
            let byte = self.input[self.pos];
            if byte == b'+' || byte == b'-' {
                self.pos += 1;
            }
        }

        // Read until whitespace, delimiter, or end of input
        let input_len = self.input.len();
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.pos += 1;
            } else if byte == b'.' && !has_dot && !has_e {
                has_dot = true;
                self.pos += 1;
            } else if (byte == b'e' || byte == b'E') && !has_e {
                has_e = true;
                self.pos += 1;
                // Allow sign after e/E
                if self.pos < input_len {
                    let next_byte = self.input[self.pos];
                    if next_byte == b'+' || next_byte == b'-' {
                        self.pos += 1;
                    }
                }
            } else if matches!(
                byte,
                b' ' | b'\t' | b'\n' | b'\r' | b':' | b',' | b')' | b']' | b'|'
            ) {
                break;
            } else {
                // Restore position before returning error to avoid corrupting parser state
                // This is critical because consumers check if position changed to detect empty values
                return Err(
                    self.restore_pos_err(start_pos, Error::Expected(Error::EXPECTED_NUMBER))
                );
            }
        }

        if self.pos == start_pos {
            return Err(Error::Expected(Error::EXPECTED_NUMBER));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<f64>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::PARSE_NUMBER_FAILED))
        })
    }

    pub(crate) fn parse_identifier(&mut self) -> Result<&'a str> {
        let start_pos = self.pos;
        let input_len = self.input.len();

        // First character must be alphabetic or underscore
        if self.pos >= input_len {
            return Err(Error::Expected(Error::EXPECTED_IDENTIFIER));
        }
        let first_byte = self.input[self.pos];
        if !(first_byte.is_ascii_alphabetic() || first_byte == b'_') {
            if matches!(first_byte, b' ' | b'\t' | b'\n' | b'\r' | b':' | b';') {
                return Err(Error::Expected(Error::EXPECTED_IDENTIFIER));
            }
            return Err(Error::InvalidChar(first_byte as char));
        }
        self.pos += 1;

        // Subsequent characters can be alphanumeric or underscore
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.pos += 1;
            } else if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':' | b';') {
                break;
            } else {
                return Err(Error::InvalidChar(byte as char));
            }
        }

        let id_bytes = &self.input[start_pos..self.pos];
        core::str::from_utf8(id_bytes).map_err(|_| Error::Expected(Error::INVALID_UTF8))
    }

    /// Expect a pattern, skip whitespace/newlines, then parse a value.
    /// This is a common pattern: `expect(b",")` followed by `skip_newlines_and_spaces()`.
    pub(crate) fn expect_then_skip(&mut self, expected: &[u8]) -> Result<&mut Self> {
        self.expect(expected)?;
        self.skip_newlines_and_spaces();
        Ok(self)
    }

    /// Parse a float value that may be empty (defaults to 0.0 if empty).
    /// This consolidates the repeated pattern of checking position before/after parse_f64.
    pub(crate) fn parse_f64_or_default(&mut self, default: f64) -> Result<f64> {
        let pos_before = self.pos();
        match self.parse_f64() {
            Ok(val) => Ok(val),
            Err(_) => {
                // If position didn't change, we're at a delimiter (empty value)
                if self.pos() == pos_before {
                    Ok(default)
                } else {
                    // Position changed but parsing failed - invalid format
                    Err(Error::Expected(Error::EXPECTED_NUMBER))
                }
            }
        }
    }

    /// Expect a pattern with a custom error message.
    /// Consolidates the common pattern: `expect(...).map_err(|_| Error::Expected(msg))`.
    pub(crate) fn expect_with_msg(
        &mut self,
        expected: &[u8],
        msg: &'static str,
    ) -> Result<&mut Self> {
        self.expect(expected).map_err(|_| Error::Expected(msg))
    }

    /// Expect a keyword, map to a custom error, then skip newlines and spaces.
    /// Consolidates the common pattern: `expect(keyword).map_err(...)?; skip_newlines_and_spaces()`.
    pub(crate) fn expect_keyword_then_skip(
        &mut self,
        keyword: &[u8],
        error_msg: &'static str,
    ) -> Result<&mut Self> {
        self.expect(keyword).map_err(|_| Error::Expected(error_msg))?;
        self.skip_newlines_and_spaces();
        Ok(self)
    }

    /// Skip whitespace optionally (don't error if no whitespace).
    /// Consolidates the pattern: `let _ = parser.skip_whitespace();` or `skip_whitespace().ok()`.
    #[inline]
    pub(crate) fn skip_whitespace_optional(&mut self) {
        let _ = self.skip_whitespace();
    }

    /// Parse an identifier with a custom error mapping.
    /// Consolidates the pattern: `parse_identifier().map_err(|_| Error::X(...))`.
    pub(crate) fn parse_identifier_with_error<F>(&mut self, map_error: F) -> Result<&'a str>
    where
        F: FnOnce() -> Error,
    {
        self.parse_identifier().map_err(|_| map_error())
    }

    /// Parse a u32 with a custom error mapping.
    /// Consolidates the pattern: `parse_u32().map_err(|_| Error::X(...))`.
    pub(crate) fn parse_u32_with_error<F>(&mut self, map_error: F) -> Result<u32>
    where
        F: FnOnce() -> Error,
    {
        self.parse_u32().map_err(|_| map_error())
    }

    /// Parse a u8 with a custom error mapping.
    /// Consolidates the pattern: `parse_u8().map_err(|_| Error::X(...))`.
    pub(crate) fn parse_u8_with_error<F>(&mut self, map_error: F) -> Result<u8>
    where
        F: FnOnce() -> Error,
    {
        self.parse_u8().map_err(|_| map_error())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod new {
        use super::*;

        #[test]
        fn succeeds_with_non_empty_input() {
            let input = b"test";
            let result = Parser::new(input);
            assert!(result.is_ok());
            let parser = result.unwrap();
            assert_eq!(parser.remaining(), b"test");
            assert_eq!(parser.pos, 0);
        }

        #[test]
        fn fails_with_empty_input() {
            let input = b"";
            let result = Parser::new(input);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::UnexpectedEof => {}
                _ => panic!("Expected Error::UnexpectedEof"),
            }
        }

        #[test]
        fn succeeds_with_single_byte() {
            let input = b"a";
            let result = Parser::new(input);
            assert!(result.is_ok());
        }

        #[test]
        fn succeeds_with_whitespace_only() {
            let input = b" ";
            let result = Parser::new(input);
            assert!(result.is_ok());
        }

        #[test]
        fn initializes_line_to_one() {
            let input = b"test";
            let parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);
        }
    }

    mod line_number {
        use super::*;

        #[test]
        fn increments_on_newline() {
            let input = b"line1\nline2";
            let mut parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);

            // Advance past "line1" to reach the newline
            parser.expect(b"line1").unwrap();
            assert_eq!(parser.line(), 1);

            // Now skip the newline
            parser.skip_newlines_and_spaces();
            assert_eq!(parser.line(), 2);
        }

        #[test]
        fn increments_on_carriage_return() {
            let input = b"line1\rline2";
            let mut parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);

            // Advance past "line1" to reach the carriage return
            parser.expect(b"line1").unwrap();
            assert_eq!(parser.line(), 1);

            parser.skip_newlines_and_spaces();
            assert_eq!(parser.line(), 2);
        }

        #[test]
        fn treats_crlf_as_single_newline() {
            let input = b"line1\r\nline2";
            let mut parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);

            // Advance past "line1" to reach the \r\n
            parser.expect(b"line1").unwrap();
            assert_eq!(parser.line(), 1);

            parser.skip_newlines_and_spaces();
            assert_eq!(parser.line(), 2);
        }

        #[test]
        fn counts_multiple_newlines() {
            let input = b"line1\n\n\nline4";
            let mut parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);

            // Advance past "line1" to reach the newlines
            parser.expect(b"line1").unwrap();
            assert_eq!(parser.line(), 1);

            parser.skip_newlines_and_spaces();
            assert_eq!(parser.line(), 4);
        }

        #[test]
        fn skip_to_end_of_line_increments_line() {
            let input = b"line1\nline2";
            let mut parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);

            parser.skip_to_end_of_line();
            assert_eq!(parser.line(), 2);
        }

        #[test]
        fn expect_increments_line_when_skipping_newlines() {
            let input = b"test\nrest";
            let mut parser = Parser::new(input).unwrap();
            assert_eq!(parser.line(), 1);

            // expect will skip "test\n" which contains a newline
            parser.expect(b"test\n").unwrap();
            assert_eq!(parser.line(), 2);
        }
    }

    mod skip_whitespace {
        use super::*;

        #[test]
        fn succeeds_with_space() {
            let input = b" test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_ok());
            assert_eq!(parser.remaining(), b"test");
            assert_eq!(parser.pos, 1);
        }

        #[test]
        fn fails_with_tab() {
            let input = b"\ttest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(msg) => assert_eq!(msg, Error::EXPECTED_WHITESPACE),
                _ => panic!("Expected Error"),
            }
        }

        #[test]
        fn fails_with_newline() {
            let input = b"\ntest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(msg) => assert_eq!(msg, Error::EXPECTED_WHITESPACE),
                _ => panic!("Expected Error"),
            }
        }

        #[test]
        fn fails_with_carriage_return() {
            let input = b"\rtest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(msg) => assert_eq!(msg, Error::EXPECTED_WHITESPACE),
                _ => panic!("Expected Error"),
            }
        }

        #[test]
        fn fails_with_form_feed() {
            let input = b"\x0ctest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(msg) => assert_eq!(msg, Error::EXPECTED_WHITESPACE),
                _ => panic!("Expected Error"),
            }
        }

        #[test]
        fn fails_with_non_whitespace() {
            let input = b"test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(msg) => assert_eq!(msg, Error::EXPECTED_WHITESPACE),
                _ => panic!("Expected Error"),
            }
            // Input should remain unchanged
            assert_eq!(parser.remaining(), b"test");
            assert_eq!(parser.pos, 0);
        }

        #[test]
        fn fails_with_empty_input() {
            let input = b" ";
            let mut parser = Parser::new(input).unwrap();
            // Skip the only character to make position at end
            parser.pos = input.len();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::UnexpectedEof => {}
                _ => panic!("Expected Error"),
            }
        }

        #[test]
        fn skips_multiple_spaces() {
            let input = b"  test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_ok());
            assert_eq!(parser.remaining(), b"test");
            assert_eq!(parser.pos, 2);
        }

        #[test]
        fn chaining_stops_on_error() {
            let input = b" test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace().and_then(|p| p.skip_whitespace());
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(msg) => assert_eq!(msg, Error::EXPECTED_WHITESPACE),
                _ => panic!("Expected Error"),
            }
        }
    }

    mod expect {
        use super::*;

        #[test]
        fn succeeds_with_version() {
            use crate::VERSION;
            let input = b"VERSION";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.expect(VERSION.as_bytes());
            assert!(result.is_ok());
            assert_eq!(parser.pos, 7);
            assert_eq!(parser.remaining(), b"");
        }

        #[test]
        fn fails_with_different_input() {
            use crate::VERSION;
            let input = b"TEST";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.expect(VERSION.as_bytes());
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(_) => {}
                _ => panic!("Expected Error::Expected"),
            }
            // Position should remain unchanged
            assert_eq!(parser.pos, 0);
        }

        #[test]
        fn fails_with_partial_match() {
            use crate::VERSION;
            let input = b"VERSIO";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.expect(VERSION.as_bytes());
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(_) => {}
                _ => panic!("Expected Error::Expected"),
            }
        }

        #[test]
        fn fails_when_remaining_input_too_short() {
            use crate::VERSION;
            let input = b"VER";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.expect(VERSION.as_bytes());
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::Expected(_) => {}
                _ => panic!("Expected Error::Expected"),
            }
        }

        #[test]
        fn succeeds_and_advances_position() {
            use crate::VERSION;
            let input = b"VERSION test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.expect(VERSION.as_bytes());
            assert!(result.is_ok());
            assert_eq!(parser.pos, 7);
            assert_eq!(parser.remaining(), b" test");
        }

        #[test]
        fn fails_when_not_at_start() {
            use crate::VERSION;
            let input = b" VERSION";
            let mut parser = Parser::new(input).unwrap();
            parser.pos = 1; // Skip the space
            let result = parser.expect(VERSION.as_bytes());
            assert!(result.is_ok());
            assert_eq!(parser.pos, 8);
        }
    }

    mod take_until_quote {
        use super::*;

        #[test]
        fn succeeds_with_quote_c_identifier_false() {
            let input = b"test\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"test");
            assert_eq!(parser.pos, 5); // position after the quote
            assert_eq!(parser.remaining(), b"rest");
        }

        #[test]
        fn succeeds_with_c_identifier_true() {
            let input = b"test_123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(true, 256);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"test_123");
            assert_eq!(parser.pos, 9); // position after the quote
        }

        #[test]
        fn succeeds_c_identifier_underscore_start() {
            let input = b"_test123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(true, 256);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"_test123");
        }

        #[test]
        fn fails_c_identifier_starts_with_digit() {
            let input = b"123test\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(true, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidChar(c) => assert_eq!(c, '1'),
                _ => panic!("Expected Error::InvalidChar"),
            }
        }

        #[test]
        fn fails_c_identifier_invalid_char() {
            let input = b"test-123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(true, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidChar(c) => assert_eq!(c, '-'),
                _ => panic!("Expected Error::InvalidChar"),
            }
        }

        #[test]
        fn succeeds_non_c_identifier_with_special_chars() {
            let input = b"test-123!@#\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"test-123!@#");
        }

        #[test]
        fn fails_with_backslash() {
            let input = b"test\\123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidChar(c) => assert_eq!(c, '\\'),
                _ => panic!("Expected Error::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_tab() {
            let input = b"test\t123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidChar(c) => assert_eq!(c, '\t'),
                _ => panic!("Expected Error::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_newline() {
            let input = b"test\n123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidChar(c) => assert_eq!(c, '\n'),
                _ => panic!("Expected Error::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_carriage_return() {
            let input = b"test\r123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::InvalidChar(c) => assert_eq!(c, '\r'),
                _ => panic!("Expected Error::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_eof() {
            let input = b"test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::UnexpectedEof => {}
                _ => panic!("Expected Error::UnexpectedEof"),
            }
        }

        #[test]
        fn succeeds_empty_string() {
            let input = b"\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"");
            assert_eq!(parser.pos, 1);
        }

        #[test]
        fn succeeds_with_printable_chars() {
            let input = b"hello world!\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"hello world!");
        }

        #[test]
        fn fails_when_exceeds_max_length() {
            let input = b"a very long string that exceeds the maximum length\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 10);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::MaxStrLength(max) => assert_eq!(max, 10),
                _ => panic!("Expected Error::MaxStrLength"),
            }
        }

        #[test]
        fn succeeds_when_at_max_length() {
            let input = b"1234567890\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 10);
            assert!(result.is_ok());
            let slice = result.unwrap();
            assert_eq!(slice, b"1234567890");
        }

        #[test]
        fn fails_when_exceeds_max_length_c_identifier() {
            let input = b"very_long_identifier_name_that_exceeds_max\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(true, 20);
            assert!(result.is_err());
            match result.unwrap_err() {
                Error::MaxStrLength(max) => assert_eq!(max, 20),
                _ => panic!("Expected Error::MaxStrLength"),
            }
        }
    }
}
