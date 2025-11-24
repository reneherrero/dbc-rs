mod de;
/// Language modules for error messages.
///
/// To select a language at build time, use feature flags:
/// - Default: English (en)
/// - `lang-fr`: French
/// - `lang-es`: Spanish
/// - `lang-de`: German
/// - `lang-ja`: Japanese
///
/// Example in Cargo.toml:
/// ```toml
/// [features]
/// default = ["std"]
/// lang-fr = []  # Enable French
/// ```
mod en;
mod es;
mod fr;
mod ja;

// Select the language module to use based on feature flags
#[cfg(feature = "lang-de")]
use de as lang;
#[cfg(not(any(
    feature = "lang-fr",
    feature = "lang-es",
    feature = "lang-de",
    feature = "lang-ja"
)))]
use en as lang;
#[cfg(feature = "lang-es")]
use es as lang;
#[cfg(feature = "lang-fr")]
use fr as lang;
#[cfg(feature = "lang-ja")]
use ja as lang;

// Re-export all constants from the selected language module
pub use lang::*;
