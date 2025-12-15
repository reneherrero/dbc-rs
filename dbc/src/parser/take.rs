use super::Parser;
use crate::Error;

impl<'a> Parser<'a> {
    pub fn take_until_quote(
        &mut self,
        c_identifier: bool,
        max_str_length: usize,
    ) -> crate::Result<&'a [u8]> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_until_quote_succeeds_with_quote_c_identifier_false() {
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
    fn take_until_quote_succeeds_with_c_identifier_true() {
        let input = b"test_123\"rest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.take_until_quote(true, 256);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, b"test_123");
        assert_eq!(parser.pos, 9); // position after the quote
    }

    #[test]
    fn take_until_quote_succeeds_c_identifier_underscore_start() {
        let input = b"_test123\"rest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.take_until_quote(true, 256);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, b"_test123");
    }

    #[test]
    fn take_until_quote_fails_c_identifier_starts_with_digit() {
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
    fn take_until_quote_fails_c_identifier_invalid_char() {
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
    fn take_until_quote_succeeds_non_c_identifier_with_special_chars() {
        let input = b"test-123!@#\"rest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.take_until_quote(false, 256);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, b"test-123!@#");
    }

    #[test]
    fn take_until_quote_fails_with_backslash() {
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
    fn take_until_quote_fails_with_tab() {
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
    fn take_until_quote_fails_with_newline() {
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
    fn take_until_quote_fails_with_carriage_return() {
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
    fn take_until_quote_fails_with_eof() {
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
    fn take_until_quote_succeeds_empty_string() {
        let input = b"\"rest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.take_until_quote(false, 256);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, b"");
        assert_eq!(parser.pos, 1);
    }

    #[test]
    fn take_until_quote_succeeds_with_printable_chars() {
        let input = b"hello world!\"rest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.take_until_quote(false, 256);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, b"hello world!");
    }

    #[test]
    fn take_until_quote_fails_when_exceeds_max_length() {
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
    fn take_until_quote_succeeds_when_at_max_length() {
        let input = b"1234567890\"rest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.take_until_quote(false, 10);
        assert!(result.is_ok());
        let slice = result.unwrap();
        assert_eq!(slice, b"1234567890");
    }

    #[test]
    fn take_until_quote_fails_when_exceeds_max_length_c_identifier() {
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
