use crate::error::{ParseError, ParseResult};

// Error message constants - grouped together for easy maintenance
mod err {
    #[allow(dead_code)] // Reserved for future use
    pub const EXPECTED_WHITESPACE: &str = "Expected whitespace";
    #[allow(dead_code)] // Reserved for future use
    pub const EXPECTED_PATTERN: &str = "Expected pattern";
}

#[derive(Debug)]
pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> ParseResult<Self> {
        if input.is_empty() {
            return Err(ParseError::UnexpectedEof);
        }
        Ok(Self { input, pos: 0 })
    }

    #[inline]
    #[must_use]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    #[must_use]
    pub(crate) fn is_at_start(&self) -> bool {
        self.pos == 0
    }

    pub fn skip_whitespace(&mut self) -> ParseResult<&mut Self> {
        if self.pos >= self.input.len() {
            return Err(ParseError::UnexpectedEof);
        }

        if self.input[self.pos] == b' ' {
            while self.pos < self.input.len() && self.input[self.pos] == b' ' {
                self.pos += 1;
            }
            Ok(self)
        } else {
            Err(ParseError::Expected(err::EXPECTED_WHITESPACE))
        }
    }

    pub fn skip_newlines_and_spaces(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b'\n' | b'\r' | b' ' | b'\t' => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    pub fn find_next_keyword(&mut self) -> ParseResult<&'a str> {
        // Skip newlines and spaces to find the next keyword
        self.skip_newlines_and_spaces();

        // Check if we're at EOF
        if self.pos >= self.input.len() {
            return Err(ParseError::UnexpectedEof);
        }

        // Try to match each keyword (checking longer ones first)
        for keyword in crate::DBC_KEYWORDS {
            let keyword_bytes = keyword.as_bytes();
            if self.starts_with(keyword_bytes) {
                // Check if the next character after the keyword is a valid delimiter
                let next_pos = self.pos + keyword_bytes.len();
                if next_pos >= self.input.len() {
                    // End of input, keyword is valid
                    self.pos = next_pos;
                    return Ok(keyword);
                }

                let next_byte = self.input[next_pos];
                // Valid delimiters: whitespace, newline, colon (for keywords like "BU_:")
                // Note: underscore is allowed as it may be part of compound keywords
                if matches!(next_byte, b' ' | b'\n' | b'\r' | b'\t' | b':') {
                    self.pos = next_pos;
                    return Ok(keyword);
                }
            }
        }

        // No keyword matched
        Err(ParseError::Expected("Expected keyword"))
    }

    pub fn expect(&mut self, expected: &[u8]) -> ParseResult<&mut Self> {
        if expected.is_empty() {
            return Ok(self);
        }

        // Check if we have enough remaining bytes
        if self.input.len() - self.pos < expected.len() {
            return Err(ParseError::Expected(err::EXPECTED_PATTERN));
        }

        if self.starts_with(expected) {
            self.pos += expected.len();
            Ok(self)
        } else {
            Err(ParseError::Expected(err::EXPECTED_PATTERN))
        }
    }

    pub fn take_until_quote(
        &mut self,
        c_identifier: bool,
        max_str_length: u16,
    ) -> ParseResult<&'a [u8]> {
        let start_pos = self.pos;
        let mut is_first_char = true;

        while self.pos < self.input.len() {
            let byte = self.input[self.pos];

            // Check if we've exceeded max length before processing the byte
            let current_length = (self.pos - start_pos) as u16;
            if current_length > max_str_length {
                return Err(ParseError::MaxStrLength(max_str_length));
            }

            match byte {
                b'"' => {
                    // Found the quote, return slice up to (but not including) the quote
                    let slice = &self.input[start_pos..self.pos];
                    // Advance position past the quote
                    self.pos += 1;
                    return Ok(slice);
                }
                b'\\' | b'\t' | b'\n' | b'\r' => {
                    return Err(ParseError::InvalidChar(byte as char));
                }
                _ => {
                    if c_identifier {
                        if is_first_char {
                            // First char must be alpha or underscore
                            if !(byte.is_ascii_alphabetic() || byte == b'_') {
                                return Err(ParseError::InvalidChar(byte as char));
                            }
                            is_first_char = false;
                        } else {
                            // Subsequent chars must be alphanumeric or underscore
                            if !(byte.is_ascii_alphanumeric() || byte == b'_') {
                                return Err(ParseError::InvalidChar(byte as char));
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
                            return Err(ParseError::InvalidChar(byte as char));
                        }
                        // Allow all other bytes (including UTF-8 continuation bytes)
                    }
                    self.pos += 1;
                }
            }
        }

        // Reached EOF without finding quote
        Err(ParseError::UnexpectedEof)
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
        while self.pos < self.input.len() {
            let byte = self.input[self.pos];
            if byte == b'\n' {
                self.pos += 1;
                break;
            } else if byte == b'\r' {
                self.pos += 1;
                // Handle \r\n
                if self.pos < self.input.len() && self.input[self.pos] == b'\n' {
                    self.pos += 1;
                }
                break;
            }
            self.pos += 1;
        }
    }

    pub(crate) fn parse_u32(&mut self) -> ParseResult<u32> {
        let start_pos = self.pos;
        // Read until whitespace, colon, pipe, @, or end of input
        while self.pos < self.input.len() {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.pos += 1;
            } else if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':' | b'|' | b'@') {
                break;
            } else {
                return Err(ParseError::Expected("Expected number"));
            }
        }

        if self.pos == start_pos {
            return Err(ParseError::Expected("Expected number"));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| ParseError::Expected("Invalid UTF-8 in number"))?;
        num_str.parse().map_err(|_| ParseError::Expected("Invalid number format"))
    }

    pub(crate) fn parse_u8(&mut self) -> ParseResult<u8> {
        let start_pos = self.pos;
        // Read until whitespace, colon, or end of input
        while self.pos < self.input.len() {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.pos += 1;
            } else if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':') {
                break;
            } else {
                return Err(ParseError::Expected("Expected number"));
            }
        }

        if self.pos == start_pos {
            return Err(ParseError::Expected("Expected number"));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| ParseError::Expected("Invalid UTF-8 in number"))?;
        num_str.parse().map_err(|_| ParseError::Expected("Invalid number format"))
    }

    pub(crate) fn parse_f64(&mut self) -> ParseResult<f64> {
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
        while self.pos < self.input.len() {
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
                if self.pos < self.input.len()
                    && (self.input[self.pos] == b'+' || self.input[self.pos] == b'-')
                {
                    self.pos += 1;
                }
            } else if matches!(
                byte,
                b' ' | b'\t' | b'\n' | b'\r' | b':' | b',' | b')' | b']' | b'|'
            ) {
                break;
            } else {
                return Err(ParseError::Expected("Expected number"));
            }
        }

        if self.pos == start_pos {
            return Err(ParseError::Expected("Expected number"));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = core::str::from_utf8(num_bytes)
            .map_err(|_| ParseError::Expected("Invalid UTF-8 in number"))?;
        num_str.parse().map_err(|_| ParseError::Expected("Invalid number format"))
    }

    pub(crate) fn parse_identifier(&mut self) -> ParseResult<&'a str> {
        let start_pos = self.pos;
        let mut is_first_char = true;

        while self.pos < self.input.len() {
            let byte = self.input[self.pos];
            if is_first_char {
                if byte.is_ascii_alphabetic() || byte == b'_' {
                    self.pos += 1;
                    is_first_char = false;
                } else if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':') {
                    break;
                } else {
                    return Err(ParseError::InvalidChar(byte as char));
                }
            } else if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.pos += 1;
            } else if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b':') {
                break;
            } else {
                return Err(ParseError::InvalidChar(byte as char));
            }
        }

        if self.pos == start_pos {
            return Err(ParseError::Expected("Expected identifier"));
        }

        let id_bytes = &self.input[start_pos..self.pos];
        core::str::from_utf8(id_bytes)
            .map_err(|_| ParseError::Expected("Invalid UTF-8 in identifier"))
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
                ParseError::UnexpectedEof => {}
                _ => panic!("Expected ParseError::UnexpectedEof"),
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
                ParseError::Expected(msg) => assert_eq!(msg, err::EXPECTED_WHITESPACE),
                _ => panic!("Expected ParseError"),
            }
        }

        #[test]
        fn fails_with_newline() {
            let input = b"\ntest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::Expected(msg) => assert_eq!(msg, err::EXPECTED_WHITESPACE),
                _ => panic!("Expected ParseError"),
            }
        }

        #[test]
        fn fails_with_carriage_return() {
            let input = b"\rtest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::Expected(msg) => assert_eq!(msg, err::EXPECTED_WHITESPACE),
                _ => panic!("Expected ParseError"),
            }
        }

        #[test]
        fn fails_with_form_feed() {
            let input = b"\x0ctest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::Expected(msg) => assert_eq!(msg, err::EXPECTED_WHITESPACE),
                _ => panic!("Expected ParseError"),
            }
        }

        #[test]
        fn fails_with_non_whitespace() {
            let input = b"test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.skip_whitespace();
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::Expected(msg) => assert_eq!(msg, err::EXPECTED_WHITESPACE),
                _ => panic!("Expected ParseError"),
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
                ParseError::UnexpectedEof => {}
                _ => panic!("Expected ParseError"),
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
                ParseError::Expected(msg) => assert_eq!(msg, err::EXPECTED_WHITESPACE),
                _ => panic!("Expected ParseError"),
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
                ParseError::Expected(_) => {}
                _ => panic!("Expected ParseError::Expected"),
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
                ParseError::Expected(_) => {}
                _ => panic!("Expected ParseError::Expected"),
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
                ParseError::Expected(_) => {}
                _ => panic!("Expected ParseError::Expected"),
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
                ParseError::InvalidChar(c) => assert_eq!(c, '1'),
                _ => panic!("Expected ParseError::InvalidChar"),
            }
        }

        #[test]
        fn fails_c_identifier_invalid_char() {
            let input = b"test-123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(true, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidChar(c) => assert_eq!(c, '-'),
                _ => panic!("Expected ParseError::InvalidChar"),
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
                ParseError::InvalidChar(c) => assert_eq!(c, '\\'),
                _ => panic!("Expected ParseError::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_tab() {
            let input = b"test\t123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidChar(c) => assert_eq!(c, '\t'),
                _ => panic!("Expected ParseError::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_newline() {
            let input = b"test\n123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidChar(c) => assert_eq!(c, '\n'),
                _ => panic!("Expected ParseError::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_carriage_return() {
            let input = b"test\r123\"rest";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::InvalidChar(c) => assert_eq!(c, '\r'),
                _ => panic!("Expected ParseError::InvalidChar"),
            }
        }

        #[test]
        fn fails_with_eof() {
            let input = b"test";
            let mut parser = Parser::new(input).unwrap();
            let result = parser.take_until_quote(false, 256);
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::UnexpectedEof => {}
                _ => panic!("Expected ParseError::UnexpectedEof"),
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
                ParseError::MaxStrLength(max) => assert_eq!(max, 10),
                _ => panic!("Expected ParseError::MaxStrLength"),
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
                ParseError::MaxStrLength(max) => assert_eq!(max, 20),
                _ => panic!("Expected ParseError::MaxStrLength"),
            }
        }
    }
}
