use super::Parser;
use crate::Error;
use core::str::from_utf8;

impl<'a> Parser<'a> {
    pub fn parse_u8(&mut self) -> crate::Result<u8> {
        let start_pos = self.pos;
        let input_len = self.input.len();
        // Read until whitespace, colon, or end of input
        while self.pos < input_len {
            if self.input[self.pos].is_ascii_digit() {
                self.advance_one();
            } else if self.matches_any(b" \t:") || self.at_newline() {
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
        let num_str = from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<u8>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    pub fn parse_u32(&mut self) -> crate::Result<u32> {
        let start_pos = self.pos;
        let input_len = self.input.len();
        // Read until whitespace, colon, pipe, @, or end of input
        while self.pos < input_len {
            if self.input[self.pos].is_ascii_digit() {
                self.advance_one();
            } else if self.matches_any(b" \t:|@") || self.at_newline() {
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
        let num_str = from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<u32>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    #[allow(dead_code)]
    pub fn parse_u64(&mut self) -> crate::Result<u64> {
        let start_pos = self.pos;
        let input_len = self.input.len();
        // Read until whitespace, colon, pipe, @, or end of input
        while self.pos < input_len {
            if self.input[self.pos].is_ascii_digit() {
                self.advance_one();
            } else if self.matches_any(b" \t:|@") || self.at_newline() {
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
        let num_str = from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<u64>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    pub fn parse_i64(&mut self) -> crate::Result<i64> {
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
        let num_str = from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<i64>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_NUMBER_FORMAT))
        })
    }

    pub fn parse_f64(&mut self) -> crate::Result<f64> {
        let start_pos = self.pos;
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
                return Err(
                    self.restore_pos_err(start_pos, Error::Expected(Error::EXPECTED_NUMBER))
                );
            }
        }

        if self.pos == start_pos {
            return Err(Error::Expected(Error::EXPECTED_NUMBER));
        }

        let num_bytes = &self.input[start_pos..self.pos];
        let num_str = from_utf8(num_bytes)
            .map_err(|_| self.restore_pos_err(start_pos, Error::Expected(Error::INVALID_UTF8)))?;
        num_str.parse::<f64>().map_err(|_| {
            self.restore_pos_err(start_pos, Error::Expected(Error::PARSE_NUMBER_FAILED))
        })
    }

    pub fn parse_identifier(&mut self) -> crate::Result<&'a str> {
        let start_pos = self.pos;
        let input_len = self.input.len();

        // First character must be alphabetic or underscore
        if self.pos >= input_len {
            return Err(Error::Expected(Error::EXPECTED_IDENTIFIER));
        }
        let first_byte = self.input[self.pos];
        if !(first_byte.is_ascii_alphabetic() || first_byte == b'_') {
            if self.matches_any(b" \t:") || self.at_newline() {
                return Err(Error::Expected(Error::EXPECTED_IDENTIFIER));
            }
            return Err(Error::InvalidChar(first_byte as char));
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
                return Err(Error::InvalidChar(byte as char));
            }
        }

        let id_bytes = &self.input[start_pos..self.pos];
        from_utf8(id_bytes).map_err(|_| Error::Expected(Error::INVALID_UTF8))
    }

    /// Parse a float value that may be empty (defaults to 0.0 if empty).
    /// This consolidates the repeated pattern of checking position before/after parse_f64.
    pub fn parse_f64_or_default(&mut self, default: f64) -> crate::Result<f64> {
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

    /// Parse an identifier with a custom error mapping.
    /// Consolidates the pattern: `parse_identifier().map_err(|_| Error::X(...))`.
    pub fn parse_identifier_with_error<F>(&mut self, map_error: F) -> crate::Result<&'a str>
    where
        F: FnOnce() -> Error,
    {
        self.parse_identifier().map_err(|_| map_error())
    }

    /// Parse a u32 with a custom error mapping.
    /// Consolidates the pattern: `parse_u32().map_err(|_| Error::X(...))`.
    pub fn parse_u32_with_error<F>(&mut self, map_error: F) -> crate::Result<u32>
    where
        F: FnOnce() -> Error,
    {
        self.parse_u32().map_err(|_| map_error())
    }

    /// Parse a u8 with a custom error mapping.
    /// Consolidates the pattern: `parse_u8().map_err(|_| Error::X(...))`.
    pub fn parse_u8_with_error<F>(&mut self, map_error: F) -> crate::Result<u8>
    where
        F: FnOnce() -> Error,
    {
        self.parse_u8().map_err(|_| map_error())
    }
}
