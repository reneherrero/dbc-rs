use crate::{Error, Result, Version, error::messages};

#[derive(Debug, Default)]
pub struct VersionBuilder {
    version: Option<String>,
}

impl VersionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    #[must_use]
    pub fn major(mut self, major: u8) -> Self {
        self.version = Some(major.to_string());
        self
    }

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

    pub fn build(self) -> Result<Version<'static>> {
        let version = self
            .version
            .ok_or_else(|| Error::Version(messages::VERSION_EMPTY.to_string()))?;

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
}
