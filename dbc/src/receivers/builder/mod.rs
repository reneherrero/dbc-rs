use super::Receivers;
use std::string::String;

/// Builder for creating `Receivers` programmatically.
///
/// This builder allows you to construct receiver configurations for signals
/// when building DBC files programmatically.
///
/// Per DBC specification Section 9.5, valid receivers are:
/// - Specific node names (comma-separated in DBC output)
/// - `Vector__XXX` for no specific receiver (represented as `Receivers::None`)
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::{ReceiversBuilder, SignalBuilder, ByteOrder};
///
/// // Specific nodes
/// let specific = ReceiversBuilder::new()
///     .add_node("TCM")
///     .add_node("BCM")
///     .build()?;
///
/// // No receivers (serializes as Vector__XXX)
/// let none = ReceiversBuilder::new().none().build()?;
///
/// // Use with signal builder
/// let signal = SignalBuilder::new()
///     .name("RPM")
///     .start_bit(0)
///     .length(16)
///     .byte_order(ByteOrder::BigEndian)
///     .unsigned(true)
///     .factor(0.25)
///     .offset(0.0)
///     .min(0.0)
///     .max(8000.0)
///     .receivers(ReceiversBuilder::new().add_node("TCM").add_node("BCM"))
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct ReceiversBuilder {
    pub(crate) nodes: Vec<String>,
}

mod build;
mod impls;
