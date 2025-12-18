mod core;
mod parse;
#[cfg(feature = "std")]
mod std;

#[cfg(feature = "std")]
mod builder;

use crate::compat::Name;

/// Represents a version string from a DBC file.
///
/// The `VERSION` statement in a DBC file specifies the database version.
/// This struct stores the version string as a borrowed reference.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 256 Engine : 8 ECM
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// if let Some(version) = dbc.version() {
///     // Access the raw string
///     assert_eq!(version.as_str(), "1.0");
///     // Display trait is available with std feature
///     #[cfg(feature = "std")]
///     {
///         println!("DBC version: {}", version);
///     }
/// }
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Format
///
/// The version string can be any sequence of printable characters enclosed in quotes.
/// Common formats include:
/// - `"1.0"` - Simple semantic version
/// - `"1.2.3"` - Full semantic version
/// - `"1.0-beta"` - Version with suffix
/// - `""` - Empty version string (allowed)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    version: Name,
}

#[cfg(feature = "std")]
pub use builder::VersionBuilder;
