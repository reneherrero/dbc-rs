#[path = "version.rs"]
mod version_impl;

#[cfg(feature = "alloc")]
mod version_builder;

#[cfg(feature = "alloc")]
pub use version_builder::VersionBuilder;

pub use version_impl::Version;
