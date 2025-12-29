use crate::{ByteOrder, ReceiversBuilder};

/// Builder for creating CAN signals programmatically.
///
/// Use this builder to construct [`Signal`](crate::Signal) instances with validated
/// properties. Required fields (name, start_bit, length) must be set before calling
/// [`build()`](Self::build).
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::{SignalBuilder, ByteOrder};
///
/// let signal = SignalBuilder::new()
///     .name("EngineRPM")
///     .start_bit(0)
///     .length(16)
///     .byte_order(ByteOrder::LittleEndian)
///     .unsigned(true)
///     .factor(0.25)
///     .offset(0.0)
///     .min(0.0)
///     .max(8000.0)
///     .unit("rpm")
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct SignalBuilder {
    name: Option<String>,
    start_bit: Option<u16>,
    length: Option<u16>,
    byte_order: Option<ByteOrder>,
    unsigned: Option<bool>,
    factor: Option<f64>,
    offset: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    unit: Option<String>,
    receivers: ReceiversBuilder,
    comment: Option<String>,
}

mod build;
mod impls;
