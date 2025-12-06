#[path = "signal.rs"]
mod signal_impl;

#[cfg(feature = "alloc")]
pub mod signal_builder;

#[cfg(feature = "alloc")]
pub use signal_builder::SignalBuilder;

pub use signal_impl::Signal;
