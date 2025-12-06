#[path = "signal.rs"]
mod signal_impl;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub mod signal_builder;

#[cfg(any(feature = "alloc", feature = "kernel"))]
pub use signal_builder::SignalBuilder;

pub use signal_impl::Signal;
