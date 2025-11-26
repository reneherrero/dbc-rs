//! # dbc-rs
//!
//! A clean, zero-dependency DBC (CAN Database) file parser and editor for Rust.
//!
//! This library provides a complete solution for parsing, validating, and writing DBC files
//! in both `std` and `no_std` environments. It supports the core DBC file format features
//! including messages, signals, nodes, and version information.
//!
//! ## Features
//!
//! - **Zero dependencies** - Pure Rust implementation
//! - **no_std + alloc support** - Works on embedded targets without the standard library
//! - **Full parsing and writing** - Parse DBC files and save them back
//! - **Comprehensive validation** - CAN ID range validation, signal overlap detection, and more
//! - **Internationalization** - Error messages in multiple languages (build-time selection)
//!
//! ## Quick Start
//!
//! ```rust
//! use dbc_rs::Dbc;
//!
//! let content = "VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM\n SG_ RPM : 0|16@1+ (0.25,0) [0|8000] \"rpm\"";
//! let dbc = Dbc::parse(content)?;
//!
//! println!("Version: {}", dbc.version().to_string());
//! println!("Messages: {}", dbc.messages().len());
//! # Ok::<(), dbc_rs::Error>(())
//! ```
//!
//! ## Core Types
//!
//! - [`Dbc`] - The main structure representing a complete DBC file
//! - [`Message`] - Represents a CAN message with ID, name, DLC, sender, and signals
//! - [`Signal`] - Represents a signal within a message with scaling, offset, min/max, etc.
//! - [`Nodes`] - Represents the list of ECUs/nodes on the CAN bus
//! - [`Version`] - Represents the DBC file version (major.minor.patch)
//! - [`Error`] - Error type for parsing and validation failures
//!
//! ## Module Structure
//!
//! The library is organized into modules:
//! - Main DBC file structure and parsing logic (see [`Dbc`])
//! - CAN message definitions (see [`Message`])
//! - Signal definitions with validation (see [`Signal`])
//! - Node/ECU management (see [`Nodes`])
//! - Version string parsing and validation (see [`Version`])
//! - Error types and internationalized error messages (see [`Error`])
//!
//! ## See Also
//!
//! - [README.md](../README.md) - Comprehensive documentation and examples
//! - [Examples](../examples/) - Usage examples

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod byte_order;
mod dbc;
mod error;
mod message;
mod nodes;
mod receivers;
mod signal;
mod version;

pub use byte_order::ByteOrder;
pub use dbc::Dbc;
pub use error::{Error, Result};
pub use message::Message;
pub use nodes::Nodes;
pub use receivers::Receivers;
pub use signal::Signal;
pub use version::Version;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
