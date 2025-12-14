use std::string::String;

use crate::{Error, Message, Result, Signal, SignalBuilder};

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

    #[must_use]
    pub fn id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn dlc(mut self, dlc: u8) -> Self {
        self.dlc = Some(dlc);
        self
    }

    #[must_use]
    pub fn sender(mut self, sender: impl AsRef<str>) -> Self {
        self.sender = Some(sender.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn add_signal(mut self, signal: SignalBuilder) -> Self {
        self.signals.push(signal);
        self
    }

    #[must_use]
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = SignalBuilder>) -> Self {
        self.signals.extend(signals);
        self
    }

    #[must_use]
    pub fn signals(mut self, signals: Vec<SignalBuilder>) -> Self {
        self.signals = signals;
        self
    }

    #[must_use]
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

    #[must_use = "validation result should be checked"]
    pub fn validate(mut self) -> Result<Self> {
        // Extract fields (this consumes signals, but we'll reconstruct)
        let signals_clone = self.signals.clone();
        let id = self.id.ok_or(Error::Message(Error::MESSAGE_ID_REQUIRED))?;
        let name = self.name.ok_or(Error::Message(Error::MESSAGE_NAME_EMPTY))?;
        let dlc = self.dlc.ok_or(Error::Message(Error::MESSAGE_DLC_REQUIRED))?;
        let sender = self.sender.ok_or(Error::Message(Error::MESSAGE_SENDER_EMPTY))?;
        // Build signals for validation using cloned signals
        let built_signals: Vec<Signal> = signals_clone
            .into_iter()
            .map(|sig_builder| sig_builder.build())
            .collect::<Result<Vec<_>>>()?;
        // Validate signals directly
        Message::validate_internal(id, &name, dlc, &sender, &built_signals)?;
        // Reconstruct from original data (signals were cloned before, so use original)
        self.id = Some(id);
        self.name = Some(name);
        self.dlc = Some(dlc);
        self.sender = Some(sender);
        // signals are already set (we cloned before, so self.signals is still there)
        Ok(self)
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
