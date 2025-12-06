use crate::{
    Parser,
    error::{ParseError, ParseResult, lang},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version<'a> {
    version: &'a str,
}

impl<'a> Version<'a> {
    pub(crate) fn new(version: &'a str) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self { version }
    }

    #[must_use = "parse result should be checked"]
    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        use crate::VERSION;
        // Note: When called from Dbc::parse, find_next_keyword already consumed "VERSION"
        // So we try to expect "VERSION" first, and if that fails, we're already past it
        if parser.expect(VERSION.as_bytes()).is_ok() {
            // Successfully consumed "VERSION", now skip whitespace and expect quote
            parser
                .skip_whitespace()?
                .expect(b"\"")
                .map_err(|_| ParseError::Expected("Expected opening quote after VERSION"))?;
        } else {
            // Check if we're at the start of input and input doesn't start with "VERSION"
            // If so, "VERSION" is required
            // Note: expect() doesn't change position on failure, so we can check is_at_start() here
            if parser.is_at_start() && !parser.starts_with(VERSION.as_bytes()) {
                // Use the constant in the error message (VERSION is "VERSION")
                return Err(ParseError::Expected("Expected 'VERSION' keyword"));
            }
            // Already past "VERSION" from find_next_keyword
            // find_next_keyword advances to right after "VERSION", which should be at whitespace or quote
            // Skip whitespace if present, then expect quote
            let _ = parser.skip_whitespace().ok(); // Ignore error if no whitespace
            parser
                .expect(b"\"")
                .map_err(|_| ParseError::Expected("Expected opening quote after VERSION"))?;
        }

        // Read version content until closing quote (allow any printable characters)
        // Use a reasonable max length for version strings (e.g., 255 characters)
        // Note: take_until_quote already advances past the closing quote
        const MAX_VERSION_LENGTH: u16 = 255;
        let version_bytes = parser.take_until_quote(false, MAX_VERSION_LENGTH)?;

        // Convert bytes to string slice using the parser's input
        let version_str = core::str::from_utf8(version_bytes)
            .map_err(|_| ParseError::Version(lang::VERSION_INVALID))?;

        // Construct directly (validation already done during parsing)
        Ok(Version::new(version_str))
    }

    pub fn as_str(&self) -> &'a str {
        self.version
    }

    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn to_dbc_string(&self) -> String {
        use crate::VERSION;
        if self.version.is_empty() {
            format!("{} \"\"", VERSION)
        } else {
            format!("{} \"{}\"", VERSION, self.version)
        }
    }
}

// Display implementation
impl<'a> core::fmt::Display for Version<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use crate::Parser;
    use crate::error::ParseError;

    #[test]
    fn test_read_version() {
        let line = b"VERSION \"1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        #[cfg(feature = "alloc")]
        assert_eq!(version.to_string(), "1.0");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "1.0");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "1.0");
    }

    #[test]
    fn test_read_version_invalid() {
        let line = b"VERSION 1.0";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap_err();
        // When there's no quote after VERSION, we get Expected error
        match version {
            ParseError::Expected(_) => {
                // This is expected - we're looking for a quote but found space/1
            }
            _ => panic!("Expected Expected error, got {:?}", version),
        }
    }

    #[test]
    fn test_version_parse_empty() {
        let line = b"";
        let result = Parser::new(line);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnexpectedEof => {
                // Empty input results in UnexpectedEof
            }
            _ => panic!("Expected UnexpectedEof"),
        }
    }

    #[test]
    fn test_version_parse_no_version_prefix() {
        let line = b"\"1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Expected(_) => {
                // Expected "VERSION" but got quote
            }
            _ => panic!("Expected Expected error"),
        }
    }

    #[test]
    fn test_version_parse_no_quotes() {
        let line = b"VERSION 1.0";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Expected(_) => {
                // Expected quote but got space/1
            }
            _ => panic!("Expected Expected error"),
        }
    }

    #[test]
    fn test_version_parse_major_only() {
        let line = b"VERSION \"1\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_ok());
        let version = result.unwrap();
        #[cfg(feature = "alloc")]
        assert_eq!(version.to_string(), "1");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "1");
    }

    #[test]
    fn test_version_parse_full_version() {
        let line = b"VERSION \"1.2.3\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_ok());
        let version = result.unwrap();
        #[cfg(feature = "std")]
        assert_eq!(version.to_string(), "1.2.3");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "1.2.3");
    }

    #[test]
    fn test_version_parse_with_whitespace() {
        let line = b"VERSION  \"1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_ok());
        let version = result.unwrap();
        #[cfg(feature = "std")]
        assert_eq!(version.to_string(), "1.0");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "1.0");
    }

    #[test]
    fn test_version_parse_empty_quotes() {
        let line = b"VERSION \"\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        #[cfg(feature = "std")]
        assert_eq!(version.to_string(), "");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "");
    }

    #[test]
    fn test_version_parse_missing_closing_quote() {
        let line = b"VERSION \"1.0";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnexpectedEof => {
                // Reached EOF without finding closing quote
            }
            _ => panic!("Expected UnexpectedEof"),
        }
    }

    #[test]
    fn test_version_parse_missing_opening_quote() {
        let line = b"VERSION 1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Expected(_) => {
                // Expected quote but got space/1
            }
            _ => panic!("Expected Expected error"),
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_version_to_dbc_string() {
        let line1 = b"VERSION \"1\"";
        let mut parser1 = Parser::new(line1).unwrap();
        let v1 = Version::parse(&mut parser1).unwrap();
        assert_eq!(v1.to_dbc_string(), "VERSION \"1\"");

        let line2 = b"VERSION \"1.0\"";
        let mut parser2 = Parser::new(line2).unwrap();
        let v2 = Version::parse(&mut parser2).unwrap();
        assert_eq!(v2.to_dbc_string(), "VERSION \"1.0\"");

        let line3 = b"VERSION \"2.3.4\"";
        let mut parser3 = Parser::new(line3).unwrap();
        let v3 = Version::parse(&mut parser3).unwrap();
        assert_eq!(v3.to_dbc_string(), "VERSION \"2.3.4\"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_version_empty_round_trip() {
        let line = b"VERSION \"\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        assert_eq!(version.to_dbc_string(), "VERSION \"\"");
    }

    #[test]
    fn test_version_with_special_chars() {
        let line = b"VERSION \"1.0-beta\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        #[cfg(feature = "std")]
        assert_eq!(version.to_string(), "1.0-beta");
        #[cfg(not(feature = "alloc"))]
        assert_eq!(version.as_str(), "1.0-beta");
    }
}
