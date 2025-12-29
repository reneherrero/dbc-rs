use crate::SignalBuilder;

/// Builder for creating CAN messages programmatically.
///
/// Use this builder to construct [`Message`](crate::Message) instances with validated
/// properties. All required fields must be set before calling [`build()`](Self::build).
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::MessageBuilder;
///
/// let message = MessageBuilder::new()
///     .id(0x100)
///     .name("EngineData")
///     .dlc(8)
///     .sender("ECM")
///     .build()?;
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct MessageBuilder {
    id: Option<u32>,
    name: Option<String>,
    dlc: Option<u8>,
    sender: Option<String>,
    signals: Vec<SignalBuilder>,
    comment: Option<String>,
}

mod build;
mod impls;
