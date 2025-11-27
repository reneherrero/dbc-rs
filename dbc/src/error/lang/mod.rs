mod de;
mod en;
mod es;
mod fr;
mod ja;

use en as lang;

// Re-export all constants from the selected language module
pub use lang::*;
