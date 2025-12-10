#[path = "message.rs"]
mod message_impl;
mod signal_list;

#[cfg(feature = "std")]
mod message_builder;

#[cfg(feature = "std")]
pub use message_builder::MessageBuilder;

pub use message_impl::Message;
pub use signal_list::SignalList;
