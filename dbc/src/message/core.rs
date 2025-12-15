use super::{Message, Signals};
use crate::{Signal, compat::String};

impl Message {
    #[cfg(feature = "std")]
    pub(crate) fn new(
        id: u32,
        name: String<{ crate::MAX_NAME_SIZE }>,
        dlc: u8,
        sender: String<{ crate::MAX_NAME_SIZE }>,
        signals: impl Into<Signals>,
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
            id,
            name,
            dlc,
            sender,
            signals: signals.into(),
        }
    }

    pub(crate) fn new_from_signals(
        id: u32,
        name: &str,
        dlc: u8,
        sender: &str,
        signals: &[Signal],
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        let name_str: String<{ crate::MAX_NAME_SIZE }> = String::try_from(name)
            .map_err(|_| crate::Error::Validation(crate::Error::MAX_NAME_SIZE_EXCEEDED))
            .unwrap();
        let sender_str: String<{ crate::MAX_NAME_SIZE }> = String::try_from(sender)
            .map_err(|_| crate::Error::Validation(crate::Error::MAX_NAME_SIZE_EXCEEDED))
            .unwrap();
        Self {
            id,
            name: name_str,
            dlc,
            sender: sender_str,
            signals: Signals::from_slice(signals),
        }
    }

    /// Returns the CAN message ID.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.id(), 256);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the message name.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.name(), "EngineData");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the Data Length Code (DLC) in bytes.
    ///
    /// DLC specifies the size of the message payload. For classic CAN, this is 1-8 bytes.
    /// For CAN FD, this can be up to 64 bytes.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert_eq!(message.dlc(), 8);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn dlc(&self) -> u8 {
        self.dlc
    }

    #[inline]
    #[must_use]
    pub fn sender(&self) -> &str {
        self.sender.as_str()
    }

    /// Get a reference to the signals collection
    #[inline]
    #[must_use]
    pub fn signals(&self) -> &Signals {
        &self.signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parser, Signal};

    #[test]
    fn test_message_getters_edge_cases() {
        // Test with minimum values
        let data = b"BO_ 0 A : 1 B";
        let mut parser = Parser::new(data).unwrap();
        let signals: &[Signal] = &[];
        let message = Message::parse(&mut parser, signals).unwrap();

        assert_eq!(message.id(), 0);
        assert_eq!(message.name(), "A");
        assert_eq!(message.dlc(), 1);
        assert_eq!(message.sender(), "B");
    }
}
