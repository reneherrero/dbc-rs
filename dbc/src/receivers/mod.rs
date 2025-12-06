#[path = "receivers.rs"]
mod receivers_impl;

#[cfg(any(feature = "alloc", feature = "kernel"))]
mod receivers_builder;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use receivers_builder::ReceiversBuilder;

pub use receivers_impl::Receivers;
