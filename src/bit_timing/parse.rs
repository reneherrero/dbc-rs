use super::BitTiming;
use crate::{BS_, Parser, Result};

impl BitTiming {
    /// Parses a `BS_:` statement from a DBC file.
    ///
    /// This method expects the parser to be positioned at the `BS_` keyword.
    ///
    /// # Format
    ///
    /// ```text
    /// BS_:                        (empty)
    /// BS_: 500                    (baudrate only)
    /// BS_: 500 : 12,34            (baudrate with BTR1, BTR2)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `parser` - The parser positioned at the BS_ statement
    ///
    /// # Returns
    ///
    /// Returns `Ok(BitTiming)` if parsing succeeds.
    #[must_use = "parse result should be checked"]
    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Consume BS_ keyword
        parser.expect(BS_.as_bytes())?;
        parser.skip_whitespace().ok();

        // Expect colon
        parser.expect(b":")?;
        parser.skip_whitespace().ok();

        // Check if we're at end of line (empty BS_:)
        if parser.at_newline() || parser.is_empty() {
            return Ok(BitTiming::new());
        }

        // Try to parse baudrate
        let baudrate = match parser.parse_u32() {
            Ok(b) => b,
            Err(_) => return Ok(BitTiming::new()), // No baudrate, empty section
        };

        parser.skip_whitespace().ok();

        // Check for BTR values (optional)
        if parser.at_newline() || parser.is_empty() {
            return Ok(BitTiming::with_baudrate(baudrate));
        }

        // Expect colon before BTR values
        if parser.expect(b":").is_err() {
            return Ok(BitTiming::with_baudrate(baudrate));
        }
        parser.skip_whitespace().ok();

        // Parse BTR1 manually (parse_u32 doesn't stop at comma)
        let mut btr1 = 0u32;
        let mut found_btr1_digits = false;
        loop {
            if parser.eof() {
                break;
            }
            let Some(b) = parser.current_byte() else {
                break;
            };
            if b.is_ascii_digit() {
                found_btr1_digits = true;
                btr1 = btr1.wrapping_mul(10).wrapping_add((b - b'0') as u32);
                parser.advance_one();
            } else {
                // Stop on any non-digit (whitespace, comma, newline, etc.)
                break;
            }
        }
        if !found_btr1_digits {
            return Ok(BitTiming::with_baudrate(baudrate));
        }

        parser.skip_whitespace().ok();

        // Expect comma between BTR1 and BTR2
        if parser.expect(b",").is_err() {
            return Ok(BitTiming::with_baudrate(baudrate));
        }
        parser.skip_whitespace().ok();

        // Parse BTR2 manually
        let mut btr2 = 0u32;
        let mut found_btr2_digits = false;
        loop {
            if parser.eof() {
                break;
            }
            let Some(b) = parser.current_byte() else {
                break;
            };
            if b.is_ascii_digit() {
                found_btr2_digits = true;
                btr2 = btr2.wrapping_mul(10).wrapping_add((b - b'0') as u32);
                parser.advance_one();
            } else {
                // Stop on any non-digit (whitespace, newline, etc.)
                break;
            }
        }
        if !found_btr2_digits {
            return Ok(BitTiming::with_baudrate(baudrate));
        }

        Ok(BitTiming::with_btr(baudrate, btr1, btr2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let mut parser = Parser::new(b"BS_:").unwrap();
        let bt = BitTiming::parse(&mut parser).unwrap();
        assert!(bt.is_empty());
        assert_eq!(bt.baudrate(), None);
        assert_eq!(bt.btr1(), None);
        assert_eq!(bt.btr2(), None);
    }

    #[test]
    fn test_parse_empty_with_newline() {
        let mut parser = Parser::new(b"BS_:\n").unwrap();
        let bt = BitTiming::parse(&mut parser).unwrap();
        assert!(bt.is_empty());
    }

    #[test]
    fn test_parse_baudrate_only() {
        let mut parser = Parser::new(b"BS_: 500").unwrap();
        let bt = BitTiming::parse(&mut parser).unwrap();
        assert!(!bt.is_empty());
        assert_eq!(bt.baudrate(), Some(500));
        assert_eq!(bt.btr1(), None);
        assert_eq!(bt.btr2(), None);
    }

    #[test]
    fn test_parse_full() {
        let mut parser = Parser::new(b"BS_: 500 : 12,34").unwrap();
        let bt = BitTiming::parse(&mut parser).unwrap();
        assert!(!bt.is_empty());
        assert_eq!(bt.baudrate(), Some(500));
        assert_eq!(bt.btr1(), Some(12));
        assert_eq!(bt.btr2(), Some(34));
    }

    #[test]
    fn test_parse_full_no_spaces() {
        let mut parser = Parser::new(b"BS_:500:12,34").unwrap();
        let bt = BitTiming::parse(&mut parser).unwrap();
        assert_eq!(bt.baudrate(), Some(500));
        assert_eq!(bt.btr1(), Some(12));
        assert_eq!(bt.btr2(), Some(34));
    }

    #[test]
    fn test_parse_full_with_newline() {
        let mut parser = Parser::new(b"BS_: 500 : 12,34\n").unwrap();
        let bt = BitTiming::parse(&mut parser).unwrap();
        assert_eq!(bt.baudrate(), Some(500));
        assert_eq!(bt.btr1(), Some(12));
        assert_eq!(bt.btr2(), Some(34));
    }
}
