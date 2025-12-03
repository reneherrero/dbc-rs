use crate::{
    error::{Error, Result, messages},
    message::Message,
    signal::Signal,
};

#[cfg(feature = "std")]
#[derive(Debug, Clone, Default)]
pub struct MessageBuilder {
    id: Option<u32>,
    name: Option<String>,
    dlc: Option<u8>,
    sender: Option<String>,
    signals: Vec<Signal>,
}

#[cfg(feature = "std")]
impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
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
    pub fn add_signal(mut self, signal: Signal) -> Self {
        self.signals.push(signal);
        self
    }

    #[must_use]
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = Signal>) -> Self {
        self.signals.extend(signals);
        self
    }

    #[must_use]
    pub fn signals(mut self, signals: Vec<Signal>) -> Self {
        self.signals = signals;
        self
    }

    #[must_use]
    pub fn clear_signals(mut self) -> Self {
        self.signals.clear();
        self
    }

    fn extract_fields(self) -> Result<(u32, String, u8, String, Vec<Signal>)> {
        let id = self
            .id
            .ok_or_else(|| Error::Message(messages::MESSAGE_ID_REQUIRED.to_string()))?;
        let name = self
            .name
            .ok_or_else(|| Error::Message(messages::MESSAGE_NAME_EMPTY.to_string()))?;
        let dlc = self
            .dlc
            .ok_or_else(|| Error::Message(messages::MESSAGE_DLC_REQUIRED.to_string()))?;
        let sender = self
            .sender
            .ok_or_else(|| Error::Message(messages::MESSAGE_SENDER_EMPTY.to_string()))?;
        Ok((id, name, dlc, sender, self.signals))
    }

    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<Self> {
        let (id, name, dlc, sender, signals) = self.extract_fields()?;
        Message::validate(id, &name, dlc, &sender, &signals).map_err(Error::from)?;
        Ok(Self {
            id: Some(id),
            name: Some(name),
            dlc: Some(dlc),
            sender: Some(sender),
            signals,
        })
    }

    pub fn build(self) -> Result<Message> {
        let (id, name, dlc, sender, signals) = self.extract_fields()?;
        Message::new(id, &name, dlc, &sender, signals)
    }
}
