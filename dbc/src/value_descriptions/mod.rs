mod value_descriptions_builder;
#[path = "value_descriptions.rs"]
mod value_descriptions_impl;

pub use value_descriptions_builder::ValueDescriptionsBuilder;
pub use value_descriptions_impl::ValueDescriptions;

// Re-export MAX_VALUE_DESCRIPTIONS for builder use
pub(crate) use value_descriptions_impl::MAX_VALUE_DESCRIPTIONS;
