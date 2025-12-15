use super::Parser;
use crate::Error;

impl<'a> Parser<'a> {
    pub fn expect(&mut self, expected: &[u8]) -> crate::Result<&mut Self> {
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

    /// Expect a pattern, skip whitespace/newlines, then parse a value.
    /// This is a common pattern: `expect(b",")` followed by `skip_newlines_and_spaces()`.
    pub(crate) fn expect_then_skip(&mut self, expected: &[u8]) -> crate::Result<&mut Self> {
        self.expect(expected)?;
        self.skip_newlines_and_spaces();
        Ok(self)
    }

    /// Expect a pattern with a custom error message.
    /// Consolidates the common pattern: `expect(...).map_err(|_| Error::Expected(msg))`.
    pub(crate) fn expect_with_msg(
        &mut self,
        expected: &[u8],
        msg: &'static str,
    ) -> crate::Result<&mut Self> {
        self.expect(expected).map_err(|_| Error::Expected(msg))
    }

    /// Expect a keyword, map to a custom error, then skip newlines and spaces.
    /// Consolidates the common pattern: `expect(keyword).map_err(...)?; skip_newlines_and_spaces()`.
    pub(crate) fn expect_keyword_then_skip(
        &mut self,
        keyword: &[u8],
        error_msg: &'static str,
    ) -> crate::Result<&mut Self> {
        self.expect(keyword).map_err(|_| Error::Expected(error_msg))?;
        self.skip_newlines_and_spaces();
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_succeeds_with_version() {
        use crate::VERSION;
        let input = b"VERSION";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.expect(VERSION.as_bytes());
        assert!(result.is_ok());
        assert_eq!(parser.pos, 7);
        assert_eq!(parser.remaining(), b"");
    }

    #[test]
    fn expect_fails_with_different_input() {
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
    fn expect_fails_with_partial_match() {
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
    fn expect_fails_when_remaining_input_too_short() {
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
    fn expect_succeeds_and_advances_position() {
        use crate::VERSION;
        let input = b"VERSION test";
        let mut parser = Parser::new(input).unwrap();
        let result = parser.expect(VERSION.as_bytes());
        assert!(result.is_ok());
        assert_eq!(parser.pos, 7);
        assert_eq!(parser.remaining(), b" test");
    }

    #[test]
    fn expect_fails_when_not_at_start() {
        use crate::VERSION;
        let input = b" VERSION";
        let mut parser = Parser::new(input).unwrap();
        parser.pos = 1; // Skip the space
        let result = parser.expect(VERSION.as_bytes());
        assert!(result.is_ok());
        assert_eq!(parser.pos, 8);
    }
}
