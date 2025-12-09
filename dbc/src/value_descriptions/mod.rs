#[cfg(any(feature = "alloc", feature = "kernel"))]
mod value_descriptions_builder;

#[path = "value_descriptions.rs"]
#[cfg(any(feature = "alloc", feature = "kernel"))]
mod value_descriptions_impl;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use value_descriptions_builder::ValueDescriptionsBuilder;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use value_descriptions_impl::ValueDescriptions;

// Re-export MAX_VALUE_DESCRIPTIONS for builder use
#[cfg(any(feature = "alloc", feature = "kernel"))]
pub(crate) use value_descriptions_impl::MAX_VALUE_DESCRIPTIONS;
