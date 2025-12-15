#[path = "message.rs"]
mod message_impl;
mod message_parse;
mod message_serialize;
mod message_validation;
mod signal_list;

#[cfg(feature = "std")]
mod message_builder;

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
pub use message_builder::MessageBuilder;

pub use message_impl::Message;
pub use signal_list::SignalList;
