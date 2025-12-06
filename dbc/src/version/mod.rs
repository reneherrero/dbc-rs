#[path = "version.rs"]
mod version_impl;

#[cfg(any(feature = "alloc", feature = "kernel"))]
mod version_builder;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use version_builder::VersionBuilder;

pub use version_impl::Version;
