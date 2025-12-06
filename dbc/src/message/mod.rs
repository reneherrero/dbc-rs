#[path = "message.rs"]
mod message_impl;
mod signals;

#[cfg(feature = "alloc")]
mod message_builder;

#[cfg(feature = "alloc")]
pub use message_builder::MessageBuilder;

pub use message_impl::Message;
pub use signals::Signals;
