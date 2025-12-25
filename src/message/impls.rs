use super::{Message, Signals};
use crate::compat::{Comment, Name};

impl Message {
    pub(crate) fn new(
        id: u32,
        name: Name,
        dlc: u8,
        sender: Name,
        signals: Signals,
        comment: Option<Comment>,
    ) -> Self {
        // Validation should have been done prior (by builder or parse)
        Self {
            id,
            name,
            dlc,
            sender,
            signals,
            comment,
        }
    }

    /// Returns the CAN message ID.
    ///
    /// This returns the raw CAN ID as it would appear on the bus (11-bit or 29-bit).
    /// For extended (29-bit) IDs, the internal flag bit is stripped.
    /// Use [`is_extended()`](Self::is_extended) to check if this is an extended ID.
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
    #[must_use = "return value should be used"]
    pub fn id(&self) -> u32 {
        self.id & Self::MAX_EXTENDED_ID
    }

    /// Returns the raw internal ID including any extended ID flag.
    ///
    /// This is primarily for internal use. Most users should use [`id()`](Self::id) instead.
    #[inline]
    #[must_use = "return value should be used"]
    pub(crate) fn id_with_flag(&self) -> u32 {
        self.id
    }

    /// Returns `true` if this message uses an extended (29-bit) CAN ID.
    ///
    /// Standard CAN uses 11-bit identifiers (0-2047), while extended CAN uses 29-bit
    /// identifiers (0-536870911).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// // Standard 11-bit ID
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 256 EngineData : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert!(!message.is_extended());
    ///
    /// // Extended 29-bit ID (with flag bit set: 0x80000000 | 0x18DAF115)
    /// let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 2564485397 OBD2 : 8 ECM"#)?;
    /// let message = dbc.messages().at(0).unwrap();
    /// assert!(message.is_extended());
    /// assert_eq!(message.id(), 0x18DAF115);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_extended(&self) -> bool {
        (self.id & Self::EXTENDED_ID_FLAG) != 0
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
    #[must_use = "return value should be used"]
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
    #[must_use = "return value should be used"]
    pub fn dlc(&self) -> u8 {
        self.dlc
    }

    /// Get the sender node name for this message.
    ///
    /// The sender is the node that transmits this message on the CAN bus.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
    /// "#)?;
    ///
    /// let message = dbc.messages().iter().next().unwrap();
    /// assert_eq!(message.sender(), "ECM");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn sender(&self) -> &str {
        self.sender.as_str()
    }

    /// Returns a reference to the signals collection for this message.
    ///
    /// The [`Signals`] collection provides methods to iterate, search, and access signals by index.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" ECM
    ///  SG_ Torque : 16|16@1+ (0.1,0) [0|500] "Nm" ECM
    /// "#)?;
    ///
    /// let message = dbc.messages().find("Engine").unwrap();
    /// let signals = message.signals();
    /// assert_eq!(signals.len(), 2);
    /// assert!(signals.find("RPM").is_some());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    /// Returns the minimum number of bytes required to decode all signals in this message.
    ///
    /// This calculates the actual byte coverage of all signals, which may be less than
    /// the declared DLC. Use this when validating frame payloads for decoding - the
    /// payload must have at least this many bytes to decode all signals successfully.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "" ECM
    ///  SG_ Pressure : 8|8@1+ (1,0) [0|255] "" ECM
    /// "#)?;
    ///
    /// let message = dbc.messages().find("Engine").unwrap();
    /// assert_eq!(message.dlc(), 8);              // Declared DLC is 8
    /// assert_eq!(message.min_bytes_required(), 2); // But signals only need 2 bytes
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "return value should be used"]
    pub fn min_bytes_required(&self) -> u8 {
        if self.signals.is_empty() {
            return 0;
        }

        let mut max_bit: u16 = 0;
        for signal in self.signals.iter() {
            let (_lsb, msb) =
                Self::bit_range(signal.start_bit(), signal.length(), signal.byte_order());
            if msb > max_bit {
                max_bit = msb;
            }
        }

        // Convert max bit position to bytes: (max_bit / 8) + 1
        ((max_bit / 8) + 1) as u8
    }

    /// Returns the message comment from CM_ BO_ entry, if present.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|c| c.as_ref())
    }

    /// Sets the message comment (from CM_ BO_ entry).
    /// Used internally during parsing when CM_ entries are processed after messages.
    #[inline]
    pub(crate) fn set_comment(&mut self, comment: Comment) {
        self.comment = Some(comment);
    }

    /// Returns a mutable reference to the signals collection.
    /// Used internally during parsing when CM_ entries are processed after signals.
    #[inline]
    pub(crate) fn signals_mut(&mut self) -> &mut Signals {
        &mut self.signals
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
