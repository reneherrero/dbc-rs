// Language modules - split by feature flags
// no_std constants are always available
mod en_no_std;

#[cfg(feature = "std")]
mod en_std;

// Re-export constants based on feature flags
// no_std constants are always available
pub use en_no_std::*;

// std constants are only available with std feature
#[cfg(feature = "std")]
pub use en_std::*;
