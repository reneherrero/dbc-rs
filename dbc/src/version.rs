use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::{Error, error::messages};

/// Represents the version string from a DBC file.
///
/// DBC files can specify a version in the format "major.minor.patch",
/// where minor and patch are optional. This struct stores the parsed
/// version components.
///
/// # Examples
///
/// ```rust
/// use dbc_rs::Version;
///
/// let version = Version::builder()
///     .major(1)
///     .minor(0)
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct Version {
    major: u8,
    minor: Option<u8>,
    patch: Option<u8>,
}

impl Version {
    /// Create a new builder for constructing a `Version`
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Version;
    ///
    /// let v1 = Version::builder().major(1).build()?;
    /// let v2 = Version::builder().major(1).minor(0).build()?;
    /// let v3 = Version::builder().major(1).minor(2).patch(3).build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn builder() -> VersionBuilder {
        VersionBuilder::new()
    }

    /// This is an internal constructor. For public API usage, use [`Version::builder()`] instead.
    pub(crate) fn new(major: u8, minor: Option<u8>, patch: Option<u8>) -> Result<Self, Error> {
        if minor.is_none() && patch.is_some() {
            return Err(Error::Version(
                messages::VERSION_PATCH_REQUIRES_MINOR.to_string(),
            ));
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub(super) fn parse(version: &str) -> Result<Self, Error> {
        // Remove "VERSION " prefix
        let version = if let Some(v) = version.strip_prefix("VERSION") {
            v
        } else {
            return Err(Error::Version(messages::VERSION_EMPTY.to_string()));
        }
        .trim();

        if version.is_empty() {
            return Err(Error::Version(messages::VERSION_EMPTY.to_string()));
        }

        // Must be enclosed in double quotes
        if !version.starts_with('"') || !version.ends_with('"') {
            return Err(Error::Version(messages::VERSION_INVALID.to_string()));
        }

        let parts: Vec<&str> = version[1..version.len() - 1].split('.').collect();

        // Min 1 and maximum 3 parts
        if parts.is_empty() || parts.len() > 3 {
            return Err(Error::Version(messages::VERSION_INVALID.to_string()));
        }

        // Parse parts
        let major =
            parts[0].parse().map_err(|e| Error::Version(messages::parse_number_failed(e)))?;
        let minor: Option<u8> = if parts.len() > 1 {
            Some(parts[1].parse().map_err(|e| Error::Version(messages::parse_number_failed(e)))?)
        } else {
            None
        };
        let patch: Option<u8> = if parts.len() > 2 {
            Some(parts[2].parse().map_err(|e| Error::Version(messages::parse_number_failed(e)))?)
        } else {
            None
        };

        Ok(Version {
            major,
            minor,
            patch,
        })
    }

    /// Get the major version number
    #[inline]
    pub fn major(&self) -> u8 {
        self.major
    }

    /// Get the minor version number, if present
    #[inline]
    pub fn minor(&self) -> Option<u8> {
        self.minor
    }

    /// Get the patch version number, if present
    #[inline]
    pub fn patch(&self) -> Option<u8> {
        self.patch
    }

    /// Format version as a string (e.g., "1.2.3" or "1.0")
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match (self.minor, self.patch) {
            (Some(minor), Some(patch)) => format!("{}.{}.{}", self.major, minor, patch),
            (Some(minor), None) => format!("{}.{}", self.major, minor),
            (None, _) => format!("{}", self.major),
        }
    }

    /// Format version in DBC file format (e.g., `VERSION "1.0"`)
    ///
    /// Useful for debugging and visualization of the version in DBC format.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc_rs::Version;
    ///
    /// let version = Version::builder().major(1).minor(0).build()?;
    /// assert_eq!(version.to_dbc_string(), "VERSION \"1.0\"");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn to_dbc_string(&self) -> String {
        format!("VERSION \"{}\"", self.to_string())
    }
}

/// Builder for constructing a `Version` with a fluent API
///
/// This builder provides a more ergonomic way to construct `Version` instances.
///
/// # Examples
///
/// ```
/// use dbc_rs::Version;
///
/// // Major version only
/// let v1 = Version::builder().major(1).build()?;
///
/// // Major and minor
/// let v2 = Version::builder().major(1).minor(0).build()?;
///
/// // Full version (major.minor.patch)
/// let v3 = Version::builder().major(1).minor(2).patch(3).build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct VersionBuilder {
    major: Option<u8>,
    minor: Option<u8>,
    patch: Option<u8>,
}

impl VersionBuilder {
    fn new() -> Self {
        Self {
            major: None,
            minor: None,
            patch: None,
        }
    }

    /// Set the major version number (required)
    pub fn major(mut self, major: u8) -> Self {
        self.major = Some(major);
        self
    }

    /// Set the minor version number (optional)
    pub fn minor(mut self, minor: u8) -> Self {
        self.minor = Some(minor);
        self
    }

    /// Set the patch version number (optional, requires minor to be set)
    pub fn patch(mut self, patch: u8) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Validate the current builder state
    ///
    /// This method performs the same validation as `Version::validate()` but on the
    /// builder's current state. Useful for checking validity before calling `build()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required field (`major`) is missing
    /// - `patch` is set but `minor` is not (patch requires minor)
    #[must_use]
    pub fn validate(&self) -> Result<(), Error> {
        let _major = self
            .major
            .ok_or_else(|| Error::Version(messages::VERSION_MAJOR_REQUIRED.to_string()))?;

        if self.minor.is_none() && self.patch.is_some() {
            return Err(Error::Version(
                messages::VERSION_PATCH_REQUIRES_MINOR.to_string(),
            ));
        }

        Ok(())
    }

    /// Build the `Version` from the builder
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required field (`major`) is missing
    /// - `patch` is set but `minor` is not (patch requires minor)
    #[must_use]
    pub fn build(self) -> Result<Version, Error> {
        let major = self
            .major
            .ok_or_else(|| Error::Version(messages::VERSION_MAJOR_REQUIRED.to_string()))?;

        Version::new(major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use crate::{Error, error::messages};

    #[test]
    fn test_read_version() {
        let line = "VERSION \"1.0\"";
        let version = Version::parse(line).unwrap();
        assert_eq!(version.major(), 1);
        assert_eq!(version.minor(), Some(0));
        assert_eq!(version.patch(), None);
    }

    #[test]
    fn test_read_version_invalid() {
        let line = "VERSION 1.0";
        let version = Version::parse(line).unwrap_err();
        assert_eq!(
            version,
            Error::Version(messages::VERSION_INVALID.to_string())
        );
    }

    #[test]
    fn test_version_new() {
        let v1 = Version::new(1, None, None).unwrap();
        assert_eq!(v1.major(), 1);
        assert_eq!(v1.minor(), None);
        assert_eq!(v1.patch(), None);
        assert_eq!(v1.to_string(), "1");

        let v2 = Version::new(2, Some(3), None).unwrap();
        assert_eq!(v2.major(), 2);
        assert_eq!(v2.minor(), Some(3));
        assert_eq!(v2.patch(), None);
        assert_eq!(v2.to_string(), "2.3");

        let v3 = Version::new(4, Some(5), Some(6)).unwrap();
        assert_eq!(v3.major(), 4);
        assert_eq!(v3.minor(), Some(5));
        assert_eq!(v3.patch(), Some(6));
        assert_eq!(v3.to_string(), "4.5.6");
    }

    #[test]
    fn test_version_new_invalid_patch_without_minor() {
        let result = Version::new(1, None, Some(2));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Error::Version(messages::VERSION_PATCH_REQUIRES_MINOR.to_string())
        );
    }

    #[test]
    fn test_version_parse_empty() {
        let result = Version::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Version(msg) => assert!(msg.contains("Empty version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_no_version_prefix() {
        let result = Version::parse("\"1.0\"");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Version(msg) => assert!(msg.contains("Empty version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_no_quotes() {
        let result = Version::parse("VERSION 1.0");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Version(msg) => assert!(msg.contains("Invalid version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_too_many_parts() {
        let result = Version::parse("VERSION \"1.2.3.4\"");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Version(msg) => assert!(msg.contains("Invalid version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_invalid_number() {
        let result = Version::parse("VERSION \"abc\"");
        assert!(result.is_err());
        // This should trigger ParseIntError conversion
        match result.unwrap_err() {
            Error::Version(msg) => assert!(msg.contains("Failed to parse number")),
            _ => panic!("Expected Version error from ParseIntError"),
        }
    }

    #[test]
    fn test_version_to_string_all_variants() {
        let v1 = Version::new(1, None, None).unwrap();
        assert_eq!(v1.to_string(), "1");

        let v2 = Version::new(2, Some(3), None).unwrap();
        assert_eq!(v2.to_string(), "2.3");

        let v3 = Version::new(4, Some(5), Some(6)).unwrap();
        assert_eq!(v3.to_string(), "4.5.6");
    }

    #[test]
    fn test_version_to_dbc_string() {
        let v1 = Version::new(1, None, None).unwrap();
        assert_eq!(v1.to_dbc_string(), "VERSION \"1\"");

        let v2 = Version::new(1, Some(0), None).unwrap();
        assert_eq!(v2.to_dbc_string(), "VERSION \"1.0\"");

        let v3 = Version::new(2, Some(3), Some(4)).unwrap();
        assert_eq!(v3.to_dbc_string(), "VERSION \"2.3.4\"");
    }
}
