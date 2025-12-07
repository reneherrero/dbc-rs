// Language modules - all are compiled but only one is selected via features
mod de;
mod en;
mod es;
mod fr;
mod ja;

// Select language module based on feature flags
// Default to English if no language feature is specified
#[cfg(any(
    feature = "lang-en",
    not(any(
        feature = "lang-de",
        feature = "lang-es",
        feature = "lang-fr",
        feature = "lang-ja"
    ))
))]
use en as lang;

#[cfg(feature = "lang-de")]
use de as lang;

#[cfg(feature = "lang-es")]
use es as lang;

#[cfg(feature = "lang-fr")]
use fr as lang;

#[cfg(feature = "lang-ja")]
use ja as lang;

// Re-export all constants from the selected language module
pub use lang::*;
