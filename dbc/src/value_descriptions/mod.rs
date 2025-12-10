#[cfg(feature = "std")]
mod value_descriptions_builder;

#[path = "value_descriptions.rs"]
#[cfg(feature = "std")]
mod value_descriptions_impl;

#[cfg(feature = "std")]
pub use value_descriptions_builder::ValueDescriptionsBuilder;

#[cfg(feature = "std")]
pub use value_descriptions_impl::ValueDescriptions;
