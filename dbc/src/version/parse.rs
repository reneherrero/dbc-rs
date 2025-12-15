use super::Version;
use crate::{
    Error, MAX_NAME_SIZE, Parser, Result, VERSION,
    compat::{String, Vec},
};

impl Version {
    /// Parses a `VERSION` statement from a DBC file.
    ///
    /// This method expects the parser to be positioned at or after the `VERSION` keyword.
    /// It will parse the version string enclosed in quotes.
    ///
    /// # Format
    ///
    /// The expected format is: `VERSION "version_string"`
    ///
    /// # Arguments
    ///
    /// * `parser` - The parser positioned at the VERSION statement
    ///
    /// # Returns
    ///
    /// Returns `Ok(Version)` if parsing succeeds, or `Err(Error)` if:
    /// - The opening quote is missing
    /// - The closing quote is missing
    /// - The version string exceeds the maximum length (255 characters)
    /// - The version string contains invalid UTF-8
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// // Version is typically accessed from a parsed DBC
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    /// "#)?;
    ///
    /// if let Some(version) = dbc.version() {
    ///     assert_eq!(version.as_str(), "1.0");
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "parse result should be checked"]
    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Version parsing must always start with "VERSION" keyword
        parser
            .expect(VERSION.as_bytes())
            .map_err(|_| Error::Expected("Expected 'VERSION' keyword"))?;

        // Skip whitespace and expect quote (whitespace is required)
        parser
            .skip_whitespace()?
            .expect(b"\"")
            .map_err(|_| Error::Expected("Expected opening quote after VERSION"))?;

        let version_bytes = parser.take_until_quote(false, MAX_NAME_SIZE)?;

        // Convert bytes to string slice using the parser's input
        let v = Vec::<u8, { MAX_NAME_SIZE }>::from_slice(version_bytes)
            .map_err(|_| Error::Expected(Error::INVALID_UTF8))?;
        let version_str = String::<{ MAX_NAME_SIZE }>::from_utf8(v)
            .map_err(|_| Error::Version(Error::MAX_NAME_SIZE_EXCEEDED))?;

        // Construct directly (validation already done during parsing)
        Ok(Version::new(version_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_read_version() {
        let line = b"VERSION \"1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        assert_eq!(version.as_str(), "1.0");
    }

    #[test]
    fn test_read_version_invalid() {
        let line = b"VERSION 1.0";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap_err();
        match version {
            Error::Expected(_) => {}
            _ => panic!("Expected Expected error, got {:?}", version),
        }
    }

    #[test]
    fn test_version_parse_empty() {
        let line = b"";
        let result = Parser::new(line);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedEof => {}
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
            Error::Expected(_) => {}
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
            Error::Expected(_) => {}
            _ => panic!("Expected Expected error"),
        }
    }

    #[test]
    fn test_version_parse_missing_closing_quote() {
        let line = b"VERSION \"1.0";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedEof => {}
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
            Error::Expected(_) => {}
            _ => panic!("Expected Expected error"),
        }
    }
}
