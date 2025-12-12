#[path = "signal.rs"]
mod signal_impl;

#[cfg(feature = "std")]
pub mod signal_builder;

#[cfg(feature = "std")]
pub use signal_builder::SignalBuilder;

pub use signal_impl::{MultiplexerIndicator, Signal};
