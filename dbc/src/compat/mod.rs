//! Compatibility layer for `alloc` and `heapless` abstractions.
//!
//! This module provides `String<{N}>` and `Vec<T, {N}>` types that abstract over
//! `alloc::string::String`/`alloc::vec::Vec` (with `alloc` feature) and
//! `heapless::String<N>`/`heapless::Vec<T, N>` (with `heapless` feature).

#[cfg(all(not(feature = "alloc"), not(feature = "heapless")))]
compile_error!("Either the `alloc` or `heapless` feature must be enabled");

#[cfg(feature = "alloc")]
extern crate alloc;

mod string;
mod vec;

pub use string::String;
pub use vec::Vec;
