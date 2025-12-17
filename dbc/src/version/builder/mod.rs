use crate::{Result, Version};

/// Builder for creating `Version` programmatically.
///
/// This builder allows you to construct version strings when building DBC files
/// programmatically by specifying a complete version string.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::VersionBuilder;
///
/// // Direct version string
/// let version = VersionBuilder::new().version("1.0").build()?;
/// assert_eq!(version.as_str(), "1.0");
///
/// // Semantic versioning (as a string)
/// let version2 = VersionBuilder::new()
///     .version("1.2.3")
///     .build()?;
/// assert_eq!(version2.as_str(), "1.2.3");
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug)]
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
        Self { version: None }
    }

    /// Sets the complete version string.
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
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn version(mut self, version: impl AsRef<str>) -> Self {
        self.version = Some(version.as_ref().to_string());
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
    pub fn build(self) -> Result<Version> {
        match self.version {
            Some(v) => Ok(Version::new(v.into())),
            None => Ok(Version::new("".to_string().into())),
        }

        // Use Cow::Owned for owned strings (no leak needed)
    }
}

impl Default for VersionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::VersionBuilder;

    #[test]
    fn test_version_builder_version_string() {
        let version = VersionBuilder::new().version("1.0").build().unwrap();
        assert_eq!(version.as_str(), "1.0");
    }

    #[test]
    fn test_version_builder_missing_version() {
        let result = VersionBuilder::new().build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_builder_with_special_chars() {
        let version = VersionBuilder::new().version("1.0-beta").build().unwrap();
        assert_eq!(version.as_str(), "1.0-beta");
    }

    #[test]
    fn test_version_builder_empty_string() {
        let version = VersionBuilder::new().version("").build().unwrap();
        assert_eq!(version.as_str(), "");
    }

    #[test]
    fn test_version_builder_long_version() {
        let long_version = "1.2.3.4.5.6.7.8.9.10";
        let version = VersionBuilder::new().version(long_version).build().unwrap();
        assert_eq!(version.as_str(), long_version);
    }
}
