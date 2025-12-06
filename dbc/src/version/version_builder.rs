use crate::{Error, Result, Version, error::messages};

/// Builder for creating `Version` programmatically.
///
/// This builder allows you to construct version strings when building DBC files
/// programmatically. You can either specify a complete version string or build
/// it incrementally using semantic version components.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::VersionBuilder;
///
/// // Direct version string
/// let version = VersionBuilder::new().version("1.0").build()?;
///
/// // Semantic versioning
/// let version2 = VersionBuilder::new()
///     .major(1)
///     .minor(2)
///     .patch(3)
///     .build()?;
/// assert_eq!(version2.as_str(), "1.2.3");
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `alloc` feature to be enabled.
#[derive(Debug, Default)]
pub struct VersionBuilder {
    version: Option<String>,
}

impl VersionBuilder {
    /// Creates a new `VersionBuilder` with no version set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// let builder = VersionBuilder::new();
    /// // Must set version before building
    /// let version = builder.version("1.0").build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the complete version string.
    ///
    /// This method overrides any previously set version components (major, minor, patch).
    ///
    /// # Arguments
    ///
    /// * `version` - The complete version string (e.g., "1.0", "1.2.3", "1.0-beta")
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// let version = VersionBuilder::new()
    ///     .version("1.2.3")
    ///     .build()?;
    /// assert_eq!(version.as_str(), "1.2.3");
    ///
    /// // Overrides previous components
    /// let version2 = VersionBuilder::new()
    ///     .major(1)
    ///     .minor(2)
    ///     .version("3.0") // Overrides previous
    ///     .build()?;
    /// assert_eq!(version2.as_str(), "3.0");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Sets the major version number.
    ///
    /// If `minor()` or `patch()` are called after this, they will append to the version string.
    /// If `version()` is called after this, it will override this value.
    ///
    /// # Arguments
    ///
    /// * `major` - The major version number (0-255)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// let version = VersionBuilder::new()
    ///     .major(1)
    ///     .build()?;
    /// assert_eq!(version.as_str(), "1");
    ///
    /// // Combine with minor and patch
    /// let version2 = VersionBuilder::new()
    ///     .major(1)
    ///     .minor(2)
    ///     .patch(3)
    ///     .build()?;
    /// assert_eq!(version2.as_str(), "1.2.3");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn major(mut self, major: u8) -> Self {
        self.version = Some(major.to_string());
        self
    }

    /// Sets the minor version number.
    ///
    /// If a version string already exists (from `major()` or `version()`), this appends
    /// the minor version. Otherwise, it creates a new version string with just the minor number.
    ///
    /// # Arguments
    ///
    /// * `minor` - The minor version number (0-255)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// // With major
    /// let version = VersionBuilder::new()
    ///     .major(1)
    ///     .minor(2)
    ///     .build()?;
    /// assert_eq!(version.as_str(), "1.2");
    ///
    /// // Without major (creates version "2")
    /// let version2 = VersionBuilder::new()
    ///     .minor(2)
    ///     .build()?;
    /// assert_eq!(version2.as_str(), "2");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn minor(mut self, minor: u8) -> Self {
        if let Some(ref mut v) = self.version {
            *v = format!("{}.{}", v, minor);
        } else {
            // If major wasn't called, treat this as just the version string
            self.version = Some(minor.to_string());
        }
        self
    }

    /// Sets the patch version number.
    ///
    /// If a version string already exists (from `major()`, `minor()`, or `version()`),
    /// this appends the patch version. Otherwise, it creates a new version string with
    /// just the patch number.
    ///
    /// # Arguments
    ///
    /// * `patch` - The patch version number (0-255)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// // Full semantic version
    /// let version = VersionBuilder::new()
    ///     .major(1)
    ///     .minor(2)
    ///     .patch(3)
    ///     .build()?;
    /// assert_eq!(version.as_str(), "1.2.3");
    ///
    /// // Without major/minor (creates version "3")
    /// let version2 = VersionBuilder::new()
    ///     .patch(3)
    ///     .build()?;
    /// assert_eq!(version2.as_str(), "3");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn patch(mut self, patch: u8) -> Self {
        if let Some(ref mut v) = self.version {
            *v = format!("{}.{}", v, patch);
        } else {
            // If major/minor weren't called, treat this as just the version string
            self.version = Some(patch.to_string());
        }
        self
    }

    /// Builds the `Version` from the builder configuration.
    ///
    /// This validates that a version has been set and constructs a `Version` instance
    /// with static lifetime.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Version)` if successful, or `Err(Error::Version)` if:
    /// - No version has been set (empty version)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// // Build with version string
    /// let version = VersionBuilder::new()
    ///     .version("1.0")
    ///     .build()?;
    /// assert_eq!(version.as_str(), "1.0");
    ///
    /// // Build with semantic versioning
    /// let version2 = VersionBuilder::new()
    ///     .major(1)
    ///     .minor(2)
    ///     .patch(3)
    ///     .build()?;
    /// assert_eq!(version2.as_str(), "1.2.3");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// ```rust,no_run
    /// use dbc_rs::VersionBuilder;
    ///
    /// // Missing version
    /// let result = VersionBuilder::new().build();
    /// assert!(result.is_err());
    ///
    /// // Empty string is allowed
    /// let version = VersionBuilder::new().version("").build()?;
    /// assert_eq!(version.as_str(), "");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<Version<'static>> {
        let version = self.version.ok_or_else(|| {
            Error::Version(crate::error::str_to_error_string(messages::VERSION_EMPTY))
        })?;

        // Convert owned String to static reference by leaking Box<str>
        let boxed: Box<str> = version.into_boxed_str();
        let static_ref: &'static str = Box::leak(boxed);
        Ok(Version::new(static_ref))
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::VersionBuilder;
    use crate::error::lang;

    #[test]
    fn test_version_builder_version_string() {
        let version = VersionBuilder::new().version("1.0").build().unwrap();
        assert_eq!(version.to_string(), "1.0");
    }

    #[test]
    fn test_version_builder_major_only() {
        let version = VersionBuilder::new().major(1).build().unwrap();
        assert_eq!(version.to_string(), "1");
    }

    #[test]
    fn test_version_builder_major_minor() {
        let version = VersionBuilder::new().major(1).minor(0).build().unwrap();
        assert_eq!(version.to_string(), "1.0");
    }

    #[test]
    fn test_version_builder_full() {
        let version = VersionBuilder::new().major(1).minor(2).patch(3).build().unwrap();
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_version_builder_missing_version() {
        use crate::Error;

        let result = VersionBuilder::new().build();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Version(msg) => assert!(msg.contains(lang::VERSION_EMPTY)),
            _ => panic!("Expected Version error"),
        }
    }

    #[test]
    fn test_version_builder_with_special_chars() {
        let version = VersionBuilder::new().version("1.0-beta").build().unwrap();
        assert_eq!(version.to_string(), "1.0-beta");
    }

    #[test]
    fn test_version_builder_minor_without_major() {
        let version = VersionBuilder::new().minor(2).build().unwrap();
        assert_eq!(version.to_string(), "2");
    }

    #[test]
    fn test_version_builder_patch_without_major_minor() {
        let version = VersionBuilder::new().patch(3).build().unwrap();
        assert_eq!(version.to_string(), "3");
    }

    #[test]
    fn test_version_builder_patch_without_major() {
        let version = VersionBuilder::new().minor(2).patch(3).build().unwrap();
        assert_eq!(version.to_string(), "2.3");
    }

    #[test]
    fn test_version_builder_version_overrides_major() {
        let version = VersionBuilder::new().major(1).version("2.0").build().unwrap();
        assert_eq!(version.to_string(), "2.0");
    }

    #[test]
    fn test_version_builder_version_overrides_major_minor() {
        let version = VersionBuilder::new().major(1).minor(2).version("3.0").build().unwrap();
        assert_eq!(version.to_string(), "3.0");
    }

    #[test]
    fn test_version_builder_major_minor_patch_sequence() {
        let version = VersionBuilder::new().major(1).minor(2).patch(3).build().unwrap();
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_builder_empty_string() {
        let version = VersionBuilder::new().version("").build().unwrap();
        assert_eq!(version.to_string(), "");
    }

    #[test]
    fn test_version_builder_long_version() {
        let long_version = "1.2.3.4.5.6.7.8.9.10";
        let version = VersionBuilder::new().version(long_version).build().unwrap();
        assert_eq!(version.to_string(), long_version);
    }
}
