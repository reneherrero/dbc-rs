use crate::{
    Error, MAX_NAME_SIZE, Message, Result, Signal, SignalBuilder, compat, message::Signals,
};
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
        let id = self.id.ok_or_else(|| Error::message(Error::MESSAGE_ID_REQUIRED))?;
        let name = self.name.ok_or_else(|| Error::message(Error::MESSAGE_NAME_EMPTY))?;
        let dlc = self.dlc.ok_or_else(|| Error::message(Error::MESSAGE_DLC_REQUIRED))?;
        let sender = self.sender.ok_or_else(|| Error::message(Error::MESSAGE_SENDER_EMPTY))?;
        Ok((id, name, dlc, sender, self.signals))
    }

    /// Validates the builder configuration without building the final `Message`.
    ///
    /// This performs full validation of all fields:
    /// - Checks that required fields (id, name, dlc, sender) are set
    /// - Validates message-level constraints (DLC range, name not empty)
    /// - Builds and validates all signals (overlap detection, bounds checking)
    ///
    /// # Note
    ///
    /// This method clones and builds all signals internally for validation.
    /// If you only need the final `Message`, call `build()` directly.
    pub fn validate(&self) -> Result<()> {
        // Validate required fields are present
        let id = self.id.ok_or_else(|| Error::message(Error::MESSAGE_ID_REQUIRED))?;
        let name = self.name.as_ref().ok_or_else(|| Error::message(Error::MESSAGE_NAME_EMPTY))?;
        let dlc = self.dlc.ok_or_else(|| Error::message(Error::MESSAGE_DLC_REQUIRED))?;
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| Error::message(Error::MESSAGE_SENDER_EMPTY))?;

        // Build all signals for validation
        let built_signals: Vec<Signal> = self
            .signals
            .iter()
            .cloned()
            .map(|sig_builder| sig_builder.build())
            .collect::<Result<Vec<_>>>()?;

        // Validate message with signals
        Message::validate(id, name, dlc, sender, &built_signals)?;

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
        Message::validate(id, &name, dlc, &sender, &built_signals)?;

        // Convert to owned compat types (validation passed, so these should succeed)
        let name_str: compat::String<{ MAX_NAME_SIZE }> = compat::validate_name(&name)?;
        let sender_str: compat::String<{ MAX_NAME_SIZE }> = compat::validate_name(&sender)?;
        let signals_collection = Signals::from_slice(&built_signals);

        Ok(Message::new(
            id,
            name_str,
            dlc,
            sender_str,
            signals_collection,
        ))
    }
}
