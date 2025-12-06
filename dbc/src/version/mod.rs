#[path = "version.rs"]
mod version_impl;

#[cfg(feature = "std")]
mod version_builder;

#[cfg(feature = "std")]
pub use version_builder::VersionBuilder;

pub use version_impl::Version;
