mod de;
mod en;
mod es;
mod fr;
mod ja;

/// Language modules for error messages.
///
/// Language selection is done via Cargo features. English is the default.
/// Language features are mutually exclusive - only one can be enabled at a time.
///
/// # Usage
///
/// In your `Cargo.toml`:
/// ```toml
/// [dependencies]
/// dbc-rs = { version = "1", features = ["lang-fr"] }  # French
/// ```
///
/// Available languages:
/// - **Default**: English (no feature needed)
/// - `lang-fr`: French
/// - `lang-es`: Spanish
/// - `lang-de`: German
/// - `lang-ja`: Japanese

// Language selection: English is default, others require feature flags
// Features are mutually exclusive - prioritize in order: de, es, fr, ja, then default to en
#[cfg(feature = "lang-de")]
use de as lang;

#[cfg(all(feature = "lang-es", not(feature = "lang-de")))]
use es as lang;

#[cfg(all(
    feature = "lang-fr",
    not(any(feature = "lang-de", feature = "lang-es"))
))]
use fr as lang;

#[cfg(all(
    feature = "lang-ja",
    not(any(feature = "lang-de", feature = "lang-es", feature = "lang-fr"))
))]
use ja as lang;

// Default: English (when no language feature is enabled, or as fallback)
#[cfg(not(any(
    feature = "lang-de",
    feature = "lang-es",
    feature = "lang-fr",
    feature = "lang-ja"
)))]
use en as lang;

// Re-export all constants from the selected language module
pub use lang::*;
