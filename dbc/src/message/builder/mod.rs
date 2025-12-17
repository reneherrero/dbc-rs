use crate::{Error, Message, Result, Signal, SignalBuilder};
use std::string::String;

#[derive(Debug)]
pub struct MessageBuilder {
    id: Option<u32>,
    name: Option<String>,
    dlc: Option<u8>,
    sender: Option<String>,
    signals: Vec<SignalBuilder>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            dlc: None,
            sender: None,
            signals: Vec::new(),
        }
    }

    #[must_use = "builder method returns modified builder"]
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn dlc(mut self, dlc: u8) -> Self {
        self.dlc = Some(dlc);
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn sender(mut self, sender: impl AsRef<str>) -> Self {
        self.sender = Some(sender.as_ref().to_string());
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn add_signal(mut self, signal: SignalBuilder) -> Self {
        self.signals.push(signal);
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = SignalBuilder>) -> Self {
        self.signals.extend(signals);
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn signals(mut self, signals: Vec<SignalBuilder>) -> Self {
        self.signals = signals;
        self
    }

    #[must_use = "builder method returns modified builder"]
    pub fn clear_signals(mut self) -> Self {
        self.signals.clear();
        self
    }

    fn extract_fields(self) -> Result<(u32, String, u8, String, Vec<SignalBuilder>)> {
        let id = self.id.ok_or(Error::Message(Error::MESSAGE_ID_REQUIRED))?;
        let name = self.name.ok_or(Error::Message(Error::MESSAGE_NAME_EMPTY))?;
        let dlc = self.dlc.ok_or(Error::Message(Error::MESSAGE_DLC_REQUIRED))?;
        let sender = self.sender.ok_or(Error::Message(Error::MESSAGE_SENDER_EMPTY))?;
        Ok((id, name, dlc, sender, self.signals))
    }

    /// Validates the builder configuration without building.
    ///
    /// This performs lightweight validation of the message fields:
    /// - Checks that required fields (id, name, dlc, sender) are set
    /// - Validates message-level constraints (DLC range, name not empty)
    ///
    /// **Note:** Full signal validation (overlap detection, bounds checking)
    /// is performed during `build()`. For complete validation, call `build()`
    /// directly.
    ///
    /// # Performance
    ///
    /// This method borrows `self` and performs no allocations, making it
    /// suitable for pre-flight checks before expensive `build()` operations.
    pub fn validate(&self) -> Result<()> {
        // Validate required fields are present
        let id = self.id.ok_or(Error::Message(Error::MESSAGE_ID_REQUIRED))?;
        let name = self.name.as_ref().ok_or(Error::Message(Error::MESSAGE_NAME_EMPTY))?;
        let dlc = self.dlc.ok_or(Error::Message(Error::MESSAGE_DLC_REQUIRED))?;
        let _sender = self.sender.as_ref().ok_or(Error::Message(Error::MESSAGE_SENDER_EMPTY))?;

        // Validate message-level constraints
        Message::validate_message_fields(id, name, dlc)?;

        Ok(())
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBuilder {
    pub fn build(self) -> Result<Message> {
        let (id, name, dlc, sender, signals) = self.extract_fields()?;
        // Build all signals first
        let built_signals: Vec<Signal> = signals
            .into_iter()
            .map(|sig_builder| sig_builder.build())
            .collect::<Result<Vec<_>>>()?;
        // Validate before construction
        Message::validate_internal(id, &name, dlc, &sender, &built_signals)?;
        // Convert name and sender to String<{ MAX_NAME_SIZE }>

        Ok(Message::new(
            id,
            name.into(),
            dlc,
            sender.into(),
            built_signals,
        ))
    }
}
