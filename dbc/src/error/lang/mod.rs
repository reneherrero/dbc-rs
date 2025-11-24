mod de;
mod en;
mod es;
mod fr;
mod ja;

#[cfg(feature = "lang-de")]
use de as lang;
#[cfg(not(any(
    feature = "lang-fr",
    feature = "lang-es",
    feature = "lang-de",
    feature = "lang-ja"
)))]
use en as lang; // Default: English
#[cfg(feature = "lang-es")]
use es as lang;
/// Language modules for error messages.
///
/// Language selection is done via Cargo features. English is the default.
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
#[cfg(feature = "lang-fr")]
use fr as lang;
#[cfg(feature = "lang-ja")]
use ja as lang;

// Re-export all constants from the selected language module
pub use lang::*;
