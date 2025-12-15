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
    pub(crate) fn remaining(&self) -> &'a [u8] {
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

    /// Helper to restore position and return an error.
    /// Used to avoid duplicating the pattern of restoring position on error.
    #[inline]
    pub(crate) fn restore_pos_err(&mut self, pos: usize, err: Error) -> Error {
        self.pos = pos;
        err
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
}
