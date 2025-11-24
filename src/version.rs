use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::{Error, error::messages};

#[derive(Debug)]
pub struct Version {
    major: u8,
    minor: Option<u8>,
    patch: Option<u8>,
}

impl Version {
    /// Create a new Version with the given major, minor, and patch numbers
    ///
    /// # Errors
    ///
    /// Returns an error if `minor` is `None` but `patch` is `Some`, as a patch
    /// version requires a minor version.
    ///
    /// # Examples
    ///
    /// ```
    /// use dbc::Version;
    ///
    /// let v1 = Version::new(1, None, None)?;
    /// let v2 = Version::new(1, Some(0), None)?;
    /// let v3 = Version::new(1, Some(2), Some(3))?;
    /// # Ok::<(), dbc::Error>(())
    /// ```
    pub fn new(major: u8, minor: Option<u8>, patch: Option<u8>) -> Result<Self, Error> {
        if minor.is_none() && patch.is_some() {
            return Err(Error::InvalidData(
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
        let version = if version.starts_with("VERSION") {
            &version[7..]
        } else {
            return Err(Error::InvalidData(messages::VERSION_EMPTY.to_string()));
        }
        .trim();

        if version.is_empty() {
            return Err(Error::InvalidData(messages::VERSION_EMPTY.to_string()));
        }

        // Must be enclosed in double quotes
        if !version.starts_with('"') || !version.ends_with('"') {
            return Err(Error::InvalidData(messages::VERSION_INVALID.to_string()));
        }

        let parts: Vec<&str> = version[1..version.len() - 1].split('.').collect();

        // Min 1 and aximum 3 parts
        if parts.len() < 1 || parts.len() > 3 {
            return Err(Error::InvalidData(messages::VERSION_INVALID.to_string()));
        }

        // Parse parts
        let major = parts[0].parse()?;
        let minor: Option<u8> = if parts.len() > 1 {
            Some(parts[1].parse()?)
        } else {
            None
        };
        let patch: Option<u8> = if parts.len() > 2 {
            Some(parts[2].parse()?)
        } else {
            None
        };

        return Ok(Version {
            major,
            minor,
            patch,
        });
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
    /// use dbc::Version;
    ///
    /// let version = Version::new(1, Some(0), None)?;
    /// assert_eq!(version.to_dbc_string(), "VERSION \"1.0\"");
    /// # Ok::<(), dbc::Error>(())
    /// ```
    pub fn to_dbc_string(&self) -> String {
        format!("VERSION \"{}\"", self.to_string())
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
            Error::InvalidData(messages::VERSION_INVALID.to_string())
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
            Error::InvalidData(messages::VERSION_PATCH_REQUIRES_MINOR.to_string())
        );
    }

    #[test]
    fn test_version_parse_empty() {
        let result = Version::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Empty version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_no_version_prefix() {
        let result = Version::parse("\"1.0\"");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Empty version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_no_quotes() {
        let result = Version::parse("VERSION 1.0");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Invalid version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_too_many_parts() {
        let result = Version::parse("VERSION \"1.2.3.4\"");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Invalid version string")),
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_version_parse_invalid_number() {
        let result = Version::parse("VERSION \"abc\"");
        assert!(result.is_err());
        // This should trigger ParseIntError conversion
        match result.unwrap_err() {
            Error::InvalidData(msg) => assert!(msg.contains("Failed to parse number")),
            _ => panic!("Expected InvalidData error from ParseIntError"),
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
