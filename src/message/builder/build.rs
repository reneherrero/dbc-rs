use super::MessageBuilder;
use crate::{
    Error, MAX_NAME_SIZE, Message, Result, Signal, SignalBuilder, compat, message::Signals,
    required_field,
};

impl MessageBuilder {
    fn extract_fields(self) -> Result<(u32, String, u8, String, Vec<SignalBuilder>)> {
        let id = required_field!(self.id, Error::message(Error::MESSAGE_ID_REQUIRED))?;
        let name = required_field!(self.name, Error::message(Error::MESSAGE_NAME_EMPTY))?;
        let dlc = required_field!(self.dlc, Error::message(Error::MESSAGE_DLC_REQUIRED))?;
        let sender = required_field!(self.sender, Error::message(Error::MESSAGE_SENDER_EMPTY))?;
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
        let id = required_field!(self.id, Error::message(Error::MESSAGE_ID_REQUIRED))?;
        let name = required_field!(
            self.name.as_ref(),
            Error::message(Error::MESSAGE_NAME_EMPTY)
        )?;
        let dlc = required_field!(self.dlc, Error::message(Error::MESSAGE_DLC_REQUIRED))?;
        let sender = required_field!(
            self.sender.as_ref(),
            Error::message(Error::MESSAGE_SENDER_EMPTY)
        )?;

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

    /// Builds and validates the `Message`.
    ///
    /// Consumes the builder and returns a fully constructed and validated [`Message`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any required field (id, name, dlc, sender) is missing
    /// - Name exceeds maximum length (32 characters)
    /// - Any signal fails to build or validate
    /// - Signals overlap in the message payload
    /// - Signal extends beyond DLC bounds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::{MessageBuilder, SignalBuilder, ByteOrder, ReceiversBuilder};
    ///
    /// let signal = SignalBuilder::new()
    ///     .name("RPM")
    ///     .start_bit(0)
    ///     .length(16)
    ///     .byte_order(ByteOrder::LittleEndian)
    ///     .unsigned(true)
    ///     .factor(1.0)
    ///     .offset(0.0)
    ///     .min(0.0)
    ///     .max(8000.0)
    ///     .receivers(ReceiversBuilder::new().none());
    ///
    /// let message = MessageBuilder::new()
    ///     .id(0x100)
    ///     .name("EngineData")
    ///     .dlc(8)
    ///     .sender("ECM")
    ///     .add_signal(signal)
    ///     .build()?;
    ///
    /// assert_eq!(message.name(), "EngineData");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ByteOrder, ReceiversBuilder};

    fn minimal_signal() -> SignalBuilder {
        SignalBuilder::new()
            .name("TestSignal")
            .start_bit(0)
            .length(8)
            .byte_order(ByteOrder::LittleEndian)
            .unsigned(true)
            .factor(1.0)
            .offset(0.0)
            .min(0.0)
            .max(255.0)
            .receivers(ReceiversBuilder::new().none())
    }

    fn minimal_message() -> MessageBuilder {
        MessageBuilder::new().id(256).name("TestMessage").dlc(8).sender("ECM")
    }

    #[test]
    fn test_message_builder_new() {
        let builder = MessageBuilder::new();
        // Default builder should fail to build
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_message_builder_default() {
        let builder = MessageBuilder::default();
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_message_builder_minimal() {
        let message = minimal_message().build().unwrap();

        assert_eq!(message.id(), 256);
        assert_eq!(message.name(), "TestMessage");
        assert_eq!(message.dlc(), 8);
        assert_eq!(message.sender(), "ECM");
        assert_eq!(message.signals().len(), 0);
    }

    #[test]
    fn test_message_builder_missing_id() {
        let result = MessageBuilder::new().name("Test").dlc(8).sender("ECM").build();

        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_missing_name() {
        let result = MessageBuilder::new().id(256).dlc(8).sender("ECM").build();

        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_missing_dlc() {
        let result = MessageBuilder::new().id(256).name("Test").sender("ECM").build();

        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_missing_sender() {
        let result = MessageBuilder::new().id(256).name("Test").dlc(8).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_validate_valid() {
        let builder = minimal_message().add_signal(minimal_signal());
        let result = builder.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_message_builder_validate_missing_id() {
        let builder = MessageBuilder::new().name("Test").dlc(8).sender("ECM");
        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_validate_missing_name() {
        let builder = MessageBuilder::new().id(256).dlc(8).sender("ECM");
        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_validate_missing_dlc() {
        let builder = MessageBuilder::new().id(256).name("Test").sender("ECM");
        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_validate_missing_sender() {
        let builder = MessageBuilder::new().id(256).name("Test").dlc(8);
        let result = builder.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_builder_extended_id() {
        // Extended CAN ID (29-bit, with flag)
        let message = MessageBuilder::new()
            .id(0x80000100) // Extended ID flag + ID
            .name("ExtendedMsg")
            .dlc(8)
            .sender("ECM")
            .build()
            .unwrap();

        // id() returns the raw CAN ID without the extended flag
        assert_eq!(message.id(), 0x100);
        assert!(message.is_extended());
    }

    #[test]
    fn test_message_builder_dlc_zero() {
        let message = minimal_message().dlc(0).build().unwrap();
        assert_eq!(message.dlc(), 0);
    }

    #[test]
    fn test_message_builder_dlc_max() {
        // CAN FD supports up to 64 bytes
        let message = minimal_message().dlc(64).build().unwrap();
        assert_eq!(message.dlc(), 64);
    }
}
