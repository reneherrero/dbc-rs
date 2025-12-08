#[cfg(any(feature = "alloc", feature = "kernel"))]
use crate::compat::{Box, String, Vec, str_to_string};
use crate::{
    error, {Error, Message, ParseOptions, Result, Signal},
};

#[cfg(any(feature = "alloc", feature = "kernel"))]
#[derive(Debug, Clone, Default)]
pub struct MessageBuilder {
    id: Option<u32>,
    name: Option<String>,
    dlc: Option<u8>,
    sender: Option<String>,
    signals: Vec<Signal<'static>>,
}

#[cfg(any(feature = "alloc", feature = "kernel"))]
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
        self.name = Some(str_to_string(name));
        self
    }

    #[must_use]
    pub fn dlc(mut self, dlc: u8) -> Self {
        self.dlc = Some(dlc);
        self
    }

    #[must_use]
    pub fn sender(mut self, sender: impl AsRef<str>) -> Self {
        self.sender = Some(str_to_string(sender));
        self
    }

    #[must_use]
    pub fn add_signal(mut self, signal: Signal<'static>) -> Self {
        self.signals.push(signal);
        self
    }

    #[must_use]
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = Signal<'static>>) -> Self {
        self.signals.extend(signals);
        self
    }

    #[must_use]
    pub fn signals(mut self, signals: Vec<Signal<'static>>) -> Self {
        self.signals = signals;
        self
    }

    #[must_use]
    pub fn clear_signals(mut self) -> Self {
        self.signals.clear();
        self
    }

    fn extract_fields(self) -> Result<(u32, String, u8, String, Vec<Signal<'static>>)> {
        let id = self.id.ok_or(Error::message(error::lang::MESSAGE_ID_REQUIRED))?;
        let name = self.name.ok_or(Error::message(error::lang::MESSAGE_NAME_EMPTY))?;
        let dlc = self.dlc.ok_or(Error::message(error::lang::MESSAGE_DLC_REQUIRED))?;
        let sender = self.sender.ok_or(Error::message(error::lang::MESSAGE_SENDER_EMPTY))?;
        Ok((id, name, dlc, sender, self.signals))
    }

    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<Self> {
        let (id, name, dlc, sender, signals) = self.extract_fields()?;
        // Convert Vec to Option array for validation (all Some)
        let signals_options: Vec<Option<Signal<'static>>> =
            signals.iter().cloned().map(Some).collect();
        let signals_options_slice: &[Option<Signal<'static>>] = &signals_options;
        Message::validate_internal(
            id,
            &name,
            dlc,
            &sender,
            signals_options_slice,
            signals_options_slice.len(),
            ParseOptions::new(), // Builder always uses strict mode
        )
        .map_err(Error::from)?;
        Ok(Self {
            id: Some(id),
            name: Some(name),
            dlc: Some(dlc),
            sender: Some(sender),
            signals,
        })
    }

    pub fn build(self) -> Result<Message<'static>> {
        let (id, name, dlc, sender, signals) = self.extract_fields()?;
        // Convert Vec to Option array for validation (all Some)
        let signals_options: Vec<Option<Signal<'static>>> =
            signals.iter().cloned().map(Some).collect();
        let signals_options_slice: &[Option<Signal<'static>>] = &signals_options;
        // Validate before construction
        Message::validate_internal(
            id,
            &name,
            dlc,
            &sender,
            signals_options_slice,
            signals_options_slice.len(),
            ParseOptions::new(), // Builder always uses strict mode
        )
        .map_err(|e| match e {
            crate::error::ParseError::Message(msg) => Error::dbc(msg),
            _ => Error::ParseError(e),
        })?;
        // Convert owned strings to static references by leaking Box<str>
        // name and sender are already String types, so convert directly
        let name_boxed: Box<str> = name.into_boxed_str();
        let name_static: &'static str = Box::leak(name_boxed);
        let sender_boxed: Box<str> = sender.into_boxed_str();
        let sender_static: &'static str = Box::leak(sender_boxed);
        // Convert Vec to slice and leak to get static lifetime
        let signals_boxed: Box<[Signal<'static>]> = signals.into_boxed_slice();
        let signals_static: &'static [Signal<'static>] = Box::leak(signals_boxed);
        Ok(Message::new(
            id,
            name_static,
            dlc,
            sender_static,
            signals_static,
        ))
    }
}
