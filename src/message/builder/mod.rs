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
#[derive(Debug)]
pub struct MessageBuilder {
    pub(crate) id: Option<u32>,
    pub(crate) name: Option<String>,
    pub(crate) dlc: Option<u8>,
    pub(crate) sender: Option<String>,
    pub(crate) signals: Vec<SignalBuilder>,
    pub(crate) comment: Option<String>,
}

mod build;
mod impls;
