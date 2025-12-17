use super::Version;
use crate::{MAX_NAME_SIZE, compat::String};

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
    #[inline]
    #[must_use = "return value should be used"]
    pub fn as_str(&self) -> &str {
        self.version.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    // Helper function to assert version string (works in all configurations)
    fn assert_version_str(version: &Version, expected: &str) {
        assert_eq!(version.as_str(), expected);
        #[cfg(feature = "std")]
        assert_eq!(version.to_string(), expected);
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
    fn test_version_with_special_chars() {
        let line = b"VERSION \"1.0-beta\"";
        let mut parser = Parser::new(line).unwrap();
        let version = Version::parse(&mut parser).unwrap();
        assert_version_str(&version, "1.0-beta");
    }
}
