#[path = "message.rs"]
mod message_impl;
mod signals;

#[cfg(feature = "std")]
mod message_builder;

#[cfg(feature = "std")]
pub use message_builder::MessageBuilder;

pub use message_impl::Message;
pub use signals::Signals;
