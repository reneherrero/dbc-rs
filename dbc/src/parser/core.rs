use super::Parser;
use crate::Error;

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> crate::Result<Self> {
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

    #[inline]
    #[must_use]
    pub fn remaining(&self) -> &'a [u8] {
        &self.input[self.pos..]
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.remaining().is_empty()
    }

    /// Check if we're at the end of the file.
    /// Returns `true` if the current position is at or beyond the end of the input.
    #[inline]
    #[must_use]
    pub fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Check if the current byte at the parser's position matches any of the bytes in the provided slice.
    /// Returns `true` if the current byte is found in the slice, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn matches_any(&self, matches: &[u8]) -> bool {
        if self.pos < self.input.len() {
            let byte = self.input[self.pos];
            matches.contains(&byte)
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    pub fn starts_with(&self, pattern: &[u8]) -> bool {
        self.remaining().starts_with(pattern)
    }

    #[inline]
    #[must_use]
    pub fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    /// Helper to restore position and return an error.
    /// Used to avoid duplicating the pattern of restoring position on error.
    #[inline]
    pub fn restore_pos_err(&mut self, pos: usize, err: Error) -> Error {
        self.pos = pos;
        err
    }

    /// Get the byte at the current position (internal access).
    #[inline]
    pub fn current_byte(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    /// Advance the position by the given amount (internal access).
    /// Increment the position by 1 (internal access).
    #[inline]
    pub fn advance_one(&mut self) {
        self.pos += 1;
    }

    /// Check if we're at a newline (without consuming it).
    /// Handles both Unix (\n) and Windows (\r\n) newlines.
    /// Returns `true` if the next character(s) form a newline, `false` otherwise.
    #[inline]
    pub fn at_newline(&self) -> bool {
        match self.peek_byte_at(0) {
            Some(b'\n') => true,
            Some(b'\r') => {
                // Check if it's \r\n (Windows) or just \r (old Mac)
                // For our purposes, we consider both as newlines
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_succeeds_with_non_empty_input() {
        let input = b"test";
        let result = Parser::new(input);
        assert!(result.is_ok());
        let parser = result.unwrap();
        assert_eq!(parser.remaining(), b"test");
        assert_eq!(parser.pos, 0);
    }

    #[test]
    fn new_fails_with_empty_input() {
        let input = b"";
        let result = Parser::new(input);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedEof => {}
            _ => panic!("Expected Error::UnexpectedEof"),
        }
    }

    #[test]
    fn new_succeeds_with_single_byte() {
        let input = b"a";
        let result = Parser::new(input);
        assert!(result.is_ok());
    }

    #[test]
    fn new_succeeds_with_whitespace_only() {
        let input = b" ";
        let result = Parser::new(input);
        assert!(result.is_ok());
    }

    #[test]
    fn new_initializes_line_to_one() {
        let input = b"test";
        let parser = Parser::new(input).unwrap();
        assert_eq!(parser.line(), 1);
    }

    #[test]
    fn eof_returns_false_when_not_at_end() {
        let input = b"test";
        let parser = Parser::new(input).unwrap();
        assert!(!parser.eof());
    }

    #[test]
    fn eof_returns_true_when_at_end() {
        let input = b"test";
        let mut parser = Parser::new(input).unwrap();
        parser.pos = input.len();
        assert!(parser.eof());
    }

    #[test]
    fn eof_returns_true_when_past_end() {
        let input = b"test";
        let mut parser = Parser::new(input).unwrap();
        parser.pos = input.len() + 1;
        assert!(parser.eof());
    }

    #[test]
    fn eof_returns_true_after_consuming_all_input() {
        let input = b"test";
        let mut parser = Parser::new(input).unwrap();
        parser.pos = 4;
        assert!(parser.eof());
    }

    #[test]
    fn is_newline_byte_returns_true_for_newline() {
        let parser = Parser::new(b"\ntest").unwrap();
        assert!(parser.at_newline());
    }

    #[test]
    fn is_newline_byte_returns_true_for_carriage_return() {
        let parser = Parser::new(b"\rtest").unwrap();
        assert!(parser.at_newline());
    }

    #[test]
    fn is_newline_byte_returns_false_for_space() {
        let parser = Parser::new(b" test").unwrap();
        assert!(!parser.at_newline());
    }

    #[test]
    fn is_newline_byte_returns_false_for_tab() {
        let parser = Parser::new(b"\ttest").unwrap();
        assert!(!parser.at_newline());
    }

    #[test]
    fn is_newline_byte_returns_false_for_other_chars() {
        let parser_a = Parser::new(b"atest").unwrap();
        assert!(!parser_a.at_newline());
        let parser_0 = Parser::new(b"0test").unwrap();
        assert!(!parser_0.at_newline());
        let parser_colon = Parser::new(b":test").unwrap();
        assert!(!parser_colon.at_newline());
    }

    #[test]
    fn matches_any_returns_true_when_byte_in_slice() {
        let parser_space = Parser::new(b" test").unwrap();
        assert!(parser_space.matches_any(b" \t-"));
        let parser_tab = Parser::new(b"\ttest").unwrap();
        assert!(parser_tab.matches_any(b" \t-"));
        let parser_dash = Parser::new(b"-test").unwrap();
        assert!(parser_dash.matches_any(b" \t-"));
    }

    #[test]
    fn matches_any_returns_false_when_byte_not_in_slice() {
        let parser_a = Parser::new(b"atest").unwrap();
        assert!(!parser_a.matches_any(b" \t-"));
        let parser_0 = Parser::new(b"0test").unwrap();
        assert!(!parser_0.matches_any(b" \t-"));
    }

    #[test]
    fn matches_any_works_with_empty_slice() {
        let parser = Parser::new(b" test").unwrap();
        assert!(!parser.matches_any(&[]));
    }

    #[test]
    fn matches_any_works_with_single_byte() {
        let parser_x = Parser::new(b"xtest").unwrap();
        assert!(parser_x.matches_any(b"x"));
        let parser_y = Parser::new(b"ytest").unwrap();
        assert!(!parser_y.matches_any(b"x"));
    }
}
