#[path = "receivers.rs"]
mod receivers_impl;

#[cfg(feature = "alloc")]
mod receivers_builder;

#[cfg(feature = "alloc")]
pub use receivers_builder::ReceiversBuilder;

pub use receivers_impl::Receivers;
