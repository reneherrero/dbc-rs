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
    pub(crate) version: Option<String>,
}

mod build;
mod impls;
