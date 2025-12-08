#[path = "dbc.rs"]
mod dbc_impl;
mod message_list;
#[cfg(any(feature = "alloc", feature = "kernel"))]
mod value_descriptions_list;

#[cfg(any(feature = "alloc", feature = "kernel"))]
mod dbc_builder;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use dbc_builder::DbcBuilder;

pub use dbc_impl::Dbc;
pub use message_list::MessageList;
#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use value_descriptions_list::ValueDescriptionsList;
