/// Options for configuring DBC parsing behavior.
///
/// # Examples
///
/// ```
/// use dbc_rs::{Dbc, ParseOptions};
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 256 Test : 8 ECM
///  SG_ Signal1 : 0|8@1+ (1,0) [0|255] ""
/// "#;
///
/// // Use lenient mode to allow signals that extend beyond message boundaries
/// let options = ParseOptions::lenient();
/// let dbc = Dbc::parse_with_options(dbc_content, options)?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseOptions {
    /// If `true`, signals that extend beyond message boundaries will cause a parse error.
    /// If `false`, such signals will be allowed (lenient mode).
    ///
    /// Default: `true` (strict mode)
    ///
    /// # Note
    ///
    /// Many real-world DBC files have signals that technically extend beyond message
    /// boundaries but are still valid in practice. Setting this to `false` allows
    /// parsing such files.
    pub strict_boundary_check: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            strict_boundary_check: true,
        }
    }
}

impl ParseOptions {
    /// Creates a new `ParseOptions` with default settings (strict mode).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            strict_boundary_check: true,
        }
    }

    /// Creates a new `ParseOptions` with lenient boundary checking enabled.
    ///
    /// This allows signals that extend beyond message boundaries, which is useful
    /// for parsing real-world DBC files that may have technically invalid but
    /// commonly used signal definitions.
    #[must_use]
    pub const fn lenient() -> Self {
        Self {
            strict_boundary_check: false,
        }
    }
}
