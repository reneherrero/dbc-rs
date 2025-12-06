#[path = "message.rs"]
mod message_impl;
mod signals;

#[cfg(any(feature = "alloc", feature = "kernel"))]
mod message_builder;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use message_builder::MessageBuilder;

pub use message_impl::Message;
pub use signals::Signals;
