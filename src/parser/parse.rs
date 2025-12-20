use super::Parser;
use crate::Error;
use core::str::from_utf8;

impl<'a> Parser<'a> {
    pub fn parse_u8(&mut self) -> crate::Result<u8> {
        let start_pos = self.pos;
        let start_line = self.line;
        let input_len = self.input.len();
        // Read until whitespace, colon, or end of input
        while self.pos < input_len {
            if self.input[self.pos].is_ascii_digit() {
                self.advance_one();
            } else if self.matches_any(b" \t:") || self.at_newline() {
                break;
            } else {
                self.pos = start_pos;
                return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
            }
        }

        if self.pos == start_pos {
            return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = from_utf8(num_bytes).map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_UTF8, start_line)
        })?;
        num_str.parse::<u8>().map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_NUMBER_FORMAT, start_line)
        })
    }

    pub fn parse_u32(&mut self) -> crate::Result<u32> {
        let start_pos = self.pos;
        let start_line = self.line;
        let input_len = self.input.len();
        // Read until whitespace, colon, pipe, @, or end of input
        while self.pos < input_len {
            if self.input[self.pos].is_ascii_digit() {
                self.advance_one();
            } else if self.matches_any(b" \t:|@") || self.at_newline() {
                break;
            } else {
                self.pos = start_pos;
                return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
            }
        }

        if self.pos == start_pos {
            return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = from_utf8(num_bytes).map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_UTF8, start_line)
        })?;
        num_str.parse::<u32>().map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_NUMBER_FORMAT, start_line)
        })
    }

    #[allow(dead_code)]
    pub fn parse_u64(&mut self) -> crate::Result<u64> {
        let start_pos = self.pos;
        let start_line = self.line;
        let input_len = self.input.len();
        // Read until whitespace, colon, pipe, @, or end of input
        while self.pos < input_len {
            if self.input[self.pos].is_ascii_digit() {
                self.advance_one();
            } else if self.matches_any(b" \t:|@") || self.at_newline() {
                break;
            } else {
                self.pos = start_pos;
                return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
            }
        }

        if self.pos == start_pos {
            return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = from_utf8(num_bytes).map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_UTF8, start_line)
        })?;
        num_str.parse::<u64>().map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_NUMBER_FORMAT, start_line)
        })
    }

    pub fn parse_i64(&mut self) -> crate::Result<i64> {
        let start_pos = self.pos;
        let start_line = self.line;
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
                self.pos = start_pos;
                return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
            }
        }

        // Check if we parsed anything (accounting for optional sign)
        if self.pos == start_pos || (has_sign && self.pos == start_pos + 1) {
            self.pos = start_pos;
            return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = from_utf8(num_bytes).map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_UTF8, start_line)
        })?;
        num_str.parse::<i64>().map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_NUMBER_FORMAT, start_line)
        })
    }

    pub fn parse_f64(&mut self) -> crate::Result<f64> {
        let start_pos = self.pos;
        let start_line = self.line;
        let mut has_dot = false;
        let mut has_e = false;

        // Allow leading sign (+ or -)
        if self.pos < self.input.len() && self.matches_any(b"+-") {
            self.advance_one();
        }

        // Read until whitespace, delimiter, or end of input
        let input_len = self.input.len();
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_digit() {
                self.advance_one();
            } else if byte == b'.' && !has_dot && !has_e {
                has_dot = true;
                self.advance_one();
            } else if (byte == b'e' || byte == b'E') && !has_e {
                has_e = true;
                self.advance_one();
                // Allow sign after e/E
                if let Some(next_byte) = self.current_byte() {
                    if next_byte == b'+' || next_byte == b'-' {
                        self.advance_one();
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
                self.pos = start_pos;
                return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
            }
        }

        if self.pos == start_pos {
            return Err(Error::expected_at(Error::EXPECTED_NUMBER, start_line));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = from_utf8(num_bytes).map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::INVALID_UTF8, start_line)
        })?;
        num_str.parse::<f64>().map_err(|_| {
            self.pos = start_pos;
            Error::expected_at(Error::PARSE_NUMBER_FAILED, start_line)
        })
    }

    pub fn parse_identifier(&mut self) -> crate::Result<&'a str> {
        let start_pos = self.pos;
        let start_line = self.line;
        let input_len = self.input.len();

        // First character must be alphabetic or underscore
        if self.pos >= input_len {
            return Err(Error::expected_at(Error::EXPECTED_IDENTIFIER, start_line));
        }
        let first_byte = self.input[self.pos];
        if !(first_byte.is_ascii_alphabetic() || first_byte == b'_') {
            if self.matches_any(b" \t:") || self.at_newline() {
                return Err(Error::expected_at(Error::EXPECTED_IDENTIFIER, start_line));
            }
            return Err(Error::invalid_char_at(first_byte as char, start_line));
        }
        self.advance_one();

        // Subsequent characters can be alphanumeric or underscore
        // Terminators include comma per DBC spec Section 9.5 (receivers = receiver {',' receiver})
        while self.pos < input_len {
            let byte = self.input[self.pos];
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.advance_one();
            } else if self.matches_any(b" \t:,") || self.at_newline() {
                // Comma added to support comma-separated receiver lists per DBC spec
                break;
            } else {
                return Err(Error::invalid_char_at(byte as char, start_line));
            }
        }

        let id_bytes = &self.input[start_pos..self.pos];
        from_utf8(id_bytes).map_err(|_| Error::expected_at(Error::INVALID_UTF8, start_line))
    }

    /// Parse a float value that may be empty (defaults to 0.0 if empty).
    /// This consolidates the repeated pattern of checking position before/after parse_f64.
    pub fn parse_f64_or_default(&mut self, default: f64) -> crate::Result<f64> {
        let pos_before = self.pos();
        let line = self.line;
        match self.parse_f64() {
            Ok(val) => Ok(val),
            Err(_) => {
                // If position didn't change, we're at a delimiter (empty value)
                if self.pos() == pos_before {
                    Ok(default)
                } else {
                    // Position changed but parsing failed - invalid format
                    Err(Error::expected_at(Error::EXPECTED_NUMBER, line))
                }
            }
        }
    }

    /// Parse an identifier with a custom error mapping.
    /// Consolidates the pattern: `parse_identifier().map_err(|_| Error::X(...))`.
    pub fn parse_identifier_with_error<F>(&mut self, map_error: F) -> crate::Result<&'a str>
    where
        F: FnOnce() -> Error,
    {
        let line = self.line;
        self.parse_identifier().map_err(|_| map_error().with_line(line))
    }

    /// Parse a u32 with a custom error mapping.
    /// Consolidates the pattern: `parse_u32().map_err(|_| Error::X(...))`.
    pub fn parse_u32_with_error<F>(&mut self, map_error: F) -> crate::Result<u32>
    where
        F: FnOnce() -> Error,
    {
        let line = self.line;
        self.parse_u32().map_err(|_| map_error().with_line(line))
    }

    /// Parse a u8 with a custom error mapping.
    /// Consolidates the pattern: `parse_u8().map_err(|_| Error::X(...))`.
    pub fn parse_u8_with_error<F>(&mut self, map_error: F) -> crate::Result<u8>
    where
        F: FnOnce() -> Error,
    {
        let line = self.line;
        self.parse_u8().map_err(|_| map_error().with_line(line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u8_valid() {
        let mut parser = Parser::new(b"123 ").unwrap();
        assert_eq!(parser.parse_u8().unwrap(), 123);
    }

    #[test]
    fn test_parse_u8_zero() {
        let mut parser = Parser::new(b"0 ").unwrap();
        assert_eq!(parser.parse_u8().unwrap(), 0);
    }

    #[test]
    fn test_parse_u8_max() {
        let mut parser = Parser::new(b"255 ").unwrap();
        assert_eq!(parser.parse_u8().unwrap(), 255);
    }

    #[test]
    fn test_parse_u8_overflow() {
        let mut parser = Parser::new(b"256 ").unwrap();
        assert!(parser.parse_u8().is_err());
    }

    #[test]
    fn test_parse_u8_empty() {
        let mut parser = Parser::new(b" ").unwrap();
        assert!(parser.parse_u8().is_err());
    }

    #[test]
    fn test_parse_u8_invalid_char() {
        let mut parser = Parser::new(b"12a ").unwrap();
        assert!(parser.parse_u8().is_err());
    }

    #[test]
    fn test_parse_u32_valid() {
        let mut parser = Parser::new(b"12345 ").unwrap();
        assert_eq!(parser.parse_u32().unwrap(), 12345);
    }

    #[test]
    fn test_parse_u32_zero() {
        let mut parser = Parser::new(b"0 ").unwrap();
        assert_eq!(parser.parse_u32().unwrap(), 0);
    }

    #[test]
    fn test_parse_u32_max() {
        let mut parser = Parser::new(b"4294967295 ").unwrap();
        assert_eq!(parser.parse_u32().unwrap(), u32::MAX);
    }

    #[test]
    fn test_parse_u32_with_colon_delimiter() {
        let mut parser = Parser::new(b"256:").unwrap();
        assert_eq!(parser.parse_u32().unwrap(), 256);
    }

    #[test]
    fn test_parse_u32_with_pipe_delimiter() {
        let mut parser = Parser::new(b"16|").unwrap();
        assert_eq!(parser.parse_u32().unwrap(), 16);
    }

    #[test]
    fn test_parse_u32_empty() {
        let mut parser = Parser::new(b" ").unwrap();
        assert!(parser.parse_u32().is_err());
    }

    #[test]
    fn test_parse_u64_valid() {
        let mut parser = Parser::new(b"18446744073709551615 ").unwrap();
        assert_eq!(parser.parse_u64().unwrap(), u64::MAX);
    }

    #[test]
    fn test_parse_i64_positive() {
        let mut parser = Parser::new(b"12345 ").unwrap();
        assert_eq!(parser.parse_i64().unwrap(), 12345);
    }

    #[test]
    fn test_parse_i64_negative() {
        let mut parser = Parser::new(b"-12345 ").unwrap();
        assert_eq!(parser.parse_i64().unwrap(), -12345);
    }

    #[test]
    fn test_parse_i64_zero() {
        let mut parser = Parser::new(b"0 ").unwrap();
        assert_eq!(parser.parse_i64().unwrap(), 0);
    }

    #[test]
    fn test_parse_i64_with_semicolon() {
        let mut parser = Parser::new(b"-1;").unwrap();
        assert_eq!(parser.parse_i64().unwrap(), -1);
    }

    #[test]
    fn test_parse_i64_empty() {
        let mut parser = Parser::new(b" ").unwrap();
        assert!(parser.parse_i64().is_err());
    }

    #[test]
    fn test_parse_i64_sign_only() {
        let mut parser = Parser::new(b"- ").unwrap();
        assert!(parser.parse_i64().is_err());
    }

    #[test]
    fn test_parse_f64_integer() {
        let mut parser = Parser::new(b"123 ").unwrap();
        assert_eq!(parser.parse_f64().unwrap(), 123.0);
    }

    #[test]
    fn test_parse_f64_decimal() {
        let mut parser = Parser::new(b"123.456 ").unwrap();
        assert!((parser.parse_f64().unwrap() - 123.456).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_negative() {
        let mut parser = Parser::new(b"-123.456 ").unwrap();
        assert!((parser.parse_f64().unwrap() - (-123.456)).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_positive_sign() {
        let mut parser = Parser::new(b"+123.456 ").unwrap();
        assert!((parser.parse_f64().unwrap() - 123.456).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_scientific() {
        let mut parser = Parser::new(b"1.5e10 ").unwrap();
        assert!((parser.parse_f64().unwrap() - 1.5e10).abs() < 1e5);
    }

    #[test]
    fn test_parse_f64_scientific_negative_exponent() {
        let mut parser = Parser::new(b"1.5E-5 ").unwrap();
        assert!((parser.parse_f64().unwrap() - 1.5e-5).abs() < 1e-15);
    }

    #[test]
    fn test_parse_f64_scientific_positive_exponent() {
        let mut parser = Parser::new(b"1.5e+3 ").unwrap();
        assert!((parser.parse_f64().unwrap() - 1500.0).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_empty() {
        let mut parser = Parser::new(b" ").unwrap();
        assert!(parser.parse_f64().is_err());
    }

    #[test]
    fn test_parse_f64_with_comma_delimiter() {
        let mut parser = Parser::new(b"123.5,").unwrap();
        assert!((parser.parse_f64().unwrap() - 123.5).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_with_paren_delimiter() {
        let mut parser = Parser::new(b"1.0)").unwrap();
        assert!((parser.parse_f64().unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_with_bracket_delimiter() {
        let mut parser = Parser::new(b"0.25]").unwrap();
        assert!((parser.parse_f64().unwrap() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_parse_identifier_simple() {
        let mut parser = Parser::new(b"TestSignal ").unwrap();
        assert_eq!(parser.parse_identifier().unwrap(), "TestSignal");
    }

    #[test]
    fn test_parse_identifier_underscore_start() {
        let mut parser = Parser::new(b"_PrivateSignal ").unwrap();
        assert_eq!(parser.parse_identifier().unwrap(), "_PrivateSignal");
    }

    #[test]
    fn test_parse_identifier_with_numbers() {
        let mut parser = Parser::new(b"Signal_123 ").unwrap();
        assert_eq!(parser.parse_identifier().unwrap(), "Signal_123");
    }

    #[test]
    fn test_parse_identifier_with_colon_delimiter() {
        let mut parser = Parser::new(b"Signal:").unwrap();
        assert_eq!(parser.parse_identifier().unwrap(), "Signal");
    }

    #[test]
    fn test_parse_identifier_with_comma_delimiter() {
        let mut parser = Parser::new(b"Receiver1,Receiver2").unwrap();
        assert_eq!(parser.parse_identifier().unwrap(), "Receiver1");
    }

    #[test]
    fn test_parse_identifier_number_start() {
        let mut parser = Parser::new(b"123Signal ").unwrap();
        assert!(parser.parse_identifier().is_err());
    }

    #[test]
    fn test_parse_identifier_empty() {
        let mut parser = Parser::new(b" ").unwrap();
        assert!(parser.parse_identifier().is_err());
    }

    #[test]
    fn test_parse_f64_or_default_with_value() {
        let mut parser = Parser::new(b"123.5 ").unwrap();
        assert!((parser.parse_f64_or_default(0.0).unwrap() - 123.5).abs() < 1e-10);
    }

    #[test]
    fn test_parse_f64_or_default_empty() {
        let mut parser = Parser::new(b" ").unwrap();
        assert_eq!(parser.parse_f64_or_default(42.0).unwrap(), 42.0);
    }

    #[test]
    fn test_parse_identifier_with_error() {
        let mut parser = Parser::new(b"ValidName ").unwrap();
        let result = parser.parse_identifier_with_error(|| Error::signal(Error::SIGNAL_NAME_EMPTY));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ValidName");
    }

    #[test]
    fn test_parse_identifier_with_error_invalid() {
        let mut parser = Parser::new(b"123Invalid ").unwrap();
        let result = parser.parse_identifier_with_error(|| Error::signal(Error::SIGNAL_NAME_EMPTY));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u32_with_error() {
        let mut parser = Parser::new(b"256 ").unwrap();
        let result = parser.parse_u32_with_error(|| Error::message(Error::MESSAGE_INVALID_ID));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 256);
    }

    #[test]
    fn test_parse_u32_with_error_invalid() {
        let mut parser = Parser::new(b"abc ").unwrap();
        let result = parser.parse_u32_with_error(|| Error::message(Error::MESSAGE_INVALID_ID));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_u8_with_error() {
        let mut parser = Parser::new(b"8 ").unwrap();
        let result = parser.parse_u8_with_error(|| Error::message(Error::MESSAGE_INVALID_DLC));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8);
    }

    #[test]
    fn test_parse_u8_with_error_invalid() {
        let mut parser = Parser::new(b"abc ").unwrap();
        let result = parser.parse_u8_with_error(|| Error::message(Error::MESSAGE_INVALID_DLC));
        assert!(result.is_err());
    }
}
