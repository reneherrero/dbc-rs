use super::VersionBuilder;
use crate::{Result, Version};

impl VersionBuilder {
    /// Builds the `Version` from this builder.
    ///
    /// # Errors
    ///
    /// This method currently always succeeds (returns empty version if not set).
    pub fn build(self) -> Result<Version> {
        match self.version {
            Some(v) => Ok(Version::new(v.into())),
            None => Ok(Version::new("".to_string().into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VersionBuilder;

    #[test]
    fn test_version_builder_missing_version() {
        let result = VersionBuilder::new().build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_builder_empty_string() {
        let version = VersionBuilder::new().version("").build().unwrap();
        assert_eq!(version.as_str(), "");
    }
}
