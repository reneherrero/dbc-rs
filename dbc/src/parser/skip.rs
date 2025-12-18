use super::Parser;
use crate::Error;

impl<'a> Parser<'a> {
    pub fn skip_whitespace(&mut self) -> crate::Result<&mut Self> {
        let input_len = self.input.len();
        if self.pos >= input_len {
            return Err(self.err_unexpected_eof());
        }

        if self.input[self.pos] == b' ' {
            // Skip consecutive spaces (optimize: use cached input_len)
            while self.pos + 1 < input_len && self.input[self.pos + 1] == b' ' {
                self.pos += 1;
            }
            self.pos += 1; // Skip the last space
            Ok(self)
        } else {
            Err(self.err_expected(Error::EXPECTED_WHITESPACE))
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

    pub fn skip_to_end_of_line(&mut self) {
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

    /// Skip whitespace optionally (don't error if no whitespace).
    /// Consolidates the pattern: `let _ = parser.skip_whitespace();` or `skip_whitespace().ok()`.
    #[inline]
    pub fn skip_whitespace_optional(&mut self) {
        let _ = self.skip_whitespace();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_whitespace_succeeds_with_space() {
        let input = b" test";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_ok());
        assert_eq!(parser.remaining(), b"test");
        assert_eq!(parser.pos, 1);
    }

    #[test]
    fn skip_whitespace_fails_with_tab() {
        let input = b"\ttest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected { msg, line } => {
                assert_eq!(msg, Error::EXPECTED_WHITESPACE);
                assert_eq!(line, Some(1));
            }
            _ => panic!("Expected Error::Expected"),
        }
    }

    #[test]
    fn skip_whitespace_fails_with_newline() {
        let input = b"\ntest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected { msg, line } => {
                assert_eq!(msg, Error::EXPECTED_WHITESPACE);
                assert_eq!(line, Some(1));
            }
            _ => panic!("Expected Error::Expected"),
        }
    }

    #[test]
    fn skip_whitespace_fails_with_carriage_return() {
        let input = b"\rtest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected { msg, line } => {
                assert_eq!(msg, Error::EXPECTED_WHITESPACE);
                assert_eq!(line, Some(1));
            }
            _ => panic!("Expected Error::Expected"),
        }
    }

    #[test]
    fn skip_whitespace_fails_with_form_feed() {
        let input = b"\x0ctest";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected { msg, line } => {
                assert_eq!(msg, Error::EXPECTED_WHITESPACE);
                assert_eq!(line, Some(1));
            }
            _ => panic!("Expected Error::Expected"),
        }
    }

    #[test]
    fn skip_whitespace_fails_with_non_whitespace() {
        let input = b"test";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected { msg, line } => {
                assert_eq!(msg, Error::EXPECTED_WHITESPACE);
                assert_eq!(line, Some(1));
            }
            _ => panic!("Expected Error::Expected"),
        }
        // Input should remain unchanged
        assert_eq!(parser.remaining(), b"test");
        assert_eq!(parser.pos, 0);
    }

    #[test]
    fn skip_whitespace_fails_with_empty_input() {
        let input = b" ";
        let mut parser = Parser::new(input).unwrap();
        // Skip the only character to make position at end
        parser.pos = input.len();
        let result = parser.skip_whitespace();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedEof { .. } => {}
            _ => panic!("Expected Error::UnexpectedEof"),
        }
    }

    #[test]
    fn skip_whitespace_skips_multiple_spaces() {
        let input = b"  test";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace();
        assert!(result.is_ok());
        assert_eq!(parser.remaining(), b"test");
        assert_eq!(parser.pos, 2);
    }

    #[test]
    fn skip_whitespace_chaining_stops_on_error() {
        let input = b" test";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.skip_whitespace().and_then(|p| p.skip_whitespace());
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected { msg, line } => {
                assert_eq!(msg, Error::EXPECTED_WHITESPACE);
                assert_eq!(line, Some(1));
            }
            _ => panic!("Expected Error::Expected"),
        }
    }

    #[test]
    fn line_number_increments_on_newline() {
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
    fn line_number_increments_on_carriage_return() {
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
    fn line_number_treats_crlf_as_single_newline() {
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
    fn line_number_counts_multiple_newlines() {
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
