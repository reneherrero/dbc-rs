use super::VersionBuilder;

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
    fn test_version_builder_with_special_chars() {
        let version = VersionBuilder::new().version("1.0-beta").build().unwrap();
        assert_eq!(version.as_str(), "1.0-beta");
    }

    #[test]
    fn test_version_builder_long_version() {
        let long_version = "1.2.3.4.5.6.7.8.9.10";
        let version = VersionBuilder::new().version(long_version).build().unwrap();
        assert_eq!(version.as_str(), long_version);
    }
}
