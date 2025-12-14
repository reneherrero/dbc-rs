use crate::compat::{String, Vec};
use crate::error::{Error, Result};
use crate::{MAX_NAME_SIZE, Parser, VERSION};

/// Represents a version string from a DBC file.
///
/// The `VERSION` statement in a DBC file specifies the database version.
/// This struct stores the version string as a borrowed reference.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 256 Engine : 8 ECM
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// if let Some(version) = dbc.version() {
///     println!("DBC version: {}", version);
///     // Access the raw string
///     assert_eq!(version.as_str(), "1.0");
/// }
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Format
///
/// The version string can be any sequence of printable characters enclosed in quotes.
/// Common formats include:
/// - `"1.0"` - Simple semantic version
/// - `"1.2.3"` - Full semantic version
/// - `"1.0-beta"` - Version with suffix
/// - `""` - Empty version string (allowed)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    version: String<{ MAX_NAME_SIZE }>,
}

impl Version {
    /// Creates a new `Version` from a version string.
    ///
    /// # Note
    ///
    /// This method is intended for internal use. For parsing from DBC content,
    /// use `Version::parse()`. For programmatic construction, use `VersionBuilder`
    /// (requires `std` feature).
    ///
    /// # Arguments
    ///
    /// * `version` - The version string (should be validated before calling this)
    pub(crate) fn new(version: String<{ MAX_NAME_SIZE }>) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self { version }
    }

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

    /// Returns the version string as a `&str`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.2.3"
    ///
    /// BU_: ECM
    /// "#)?;
    ///
    /// if let Some(version) = dbc.version() {
    ///     assert_eq!(version.as_str(), "1.2.3");
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.version.as_str()
    }

    /// Converts the version to its DBC file representation.
    ///
    /// Returns a string in the format: `VERSION "version_string"`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    /// "#)?;
    ///
    /// if let Some(version) = dbc.version() {
    ///     let dbc_string = version.to_dbc_string();
    ///     assert_eq!(dbc_string, "VERSION \"1.0\"");
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Empty Version
    ///
    /// Empty version strings are represented as `VERSION ""`:
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION ""
    ///
    /// BU_: ECM
    /// "#)?;
    ///
    /// if let Some(version) = dbc.version() {
    ///     assert_eq!(version.to_dbc_string(), "VERSION \"\"");
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Feature Requirements
    ///
    /// This method requires the `std` feature to be enabled.
    #[must_use]
    #[cfg(feature = "std")]
    pub fn to_dbc_string(&self) -> std::string::String {
        if self.version.is_empty() {
            format!("{} \"\"", VERSION)
        } else {
            format!("{} \"{}\"", VERSION, &self.version)
        }
    }
}

/// Display implementation for `Version`.
///
/// Formats the version as just the version string (without the `VERSION` keyword or quotes).
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc = Dbc::parse(r#"VERSION "1.2.3"
///
/// BU_: ECM
/// "#)?;
///
/// if let Some(version) = dbc.version() {
///     // Display trait formats as just the version string
///     assert_eq!(format!("{}", version), "1.2.3");
///     // Use to_dbc_string() for full DBC format (requires std feature)
///     #[cfg(feature = "std")]
///     assert_eq!(version.to_dbc_string(), "VERSION \"1.2.3\"");
/// }
/// # Ok::<(), dbc_rs::Error>(())
/// ```
impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use crate::Parser;
    use crate::error::Error;

    // Helper function to assert version string (works in all configurations)
    fn assert_version_str(version: &Version, expected: &str) {
        assert_eq!(version.as_str(), expected);
        #[cfg(feature = "std")]
        assert_eq!(version.to_string(), expected);
    }

    // Tests that work in all configurations
    #[test]
    fn test_read_version() {
        let line = b"VERSION \"1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        assert_version_str(&version, "1.0");
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
    fn test_version_parse_major_only() {
        let line = b"VERSION \"1\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_version_str(&version, "1");
    }

    #[test]
    fn test_version_parse_full_version() {
        let line = b"VERSION \"1.2.3\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_version_str(&version, "1.2.3");
    }

    #[test]
    fn test_version_parse_with_whitespace() {
        let line = b"VERSION  \"1.0\"";
        let mut parser = Parser::new(line).unwrap();
        let result = Version::parse(&mut parser);
        assert!(result.is_ok());
        let version = result.unwrap();
        assert_version_str(&version, "1.0");
    }

    #[test]
    fn test_version_parse_empty_quotes() {
        let line = b"VERSION \"\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        assert_version_str(&version, "");
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

    #[test]
    fn test_version_with_special_chars() {
        let line = b"VERSION \"1.0-beta\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        assert_version_str(&version, "1.0-beta");
    }

    // Tests that require std (to_dbc_string is only available with std)
    #[cfg(feature = "std")]
    mod tests_with_std {
        use super::*;

        #[test]
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
        fn test_version_empty_round_trip() {
            let line = b"VERSION \"\"";
            let mut parser = Parser::new(line).unwrap();
            let version = Version::parse(&mut parser).unwrap();
            assert_eq!(version.to_dbc_string(), "VERSION \"\"");
        }
    }
}
