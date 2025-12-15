use super::Version;
use crate::VERSION;
use std::fmt::{Display, Formatter, Result};

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
impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.version)
    }
}

impl Version {
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
    pub fn to_dbc_string(&self) -> String {
        if self.version.is_empty() {
            format!("{} \"\"", VERSION)
        } else {
            format!("{} \"{}\"", VERSION, &self.version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

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
