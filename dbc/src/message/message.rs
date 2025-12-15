use crate::compat::String;
use crate::{Error, MAX_NAME_SIZE, Signal};

use super::SignalList;

/// Represents a CAN message in a DBC file.
///
/// A `Message` contains:
/// - A unique ID (CAN identifier)
/// - A name
/// - A DLC (Data Length Code) specifying the message size in bytes
/// - A sender node (ECU that transmits this message)
/// - A collection of signals
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 256 EngineData : 8 ECM
///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// let message = dbc.messages().at(0).unwrap();
/// println!("Message: {} (ID: {})", message.name(), message.id());
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Message {
    id: u32,
    name: String<{ MAX_NAME_SIZE }>,
    dlc: u8,
    sender: String<{ MAX_NAME_SIZE }>,
    signals: SignalList,
}

impl Message {
    #[cfg(feature = "std")]
    pub(crate) fn new(
        id: u32,
        name: String<{ crate::MAX_NAME_SIZE }>,
        dlc: u8,
        sender: String<{ crate::MAX_NAME_SIZE }>,
        signals: impl Into<SignalList>,
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
            .map_err(|_| Error::Validation(Error::MAX_NAME_SIZE_EXCEEDED))
            .unwrap();
        let sender_str: String<{ crate::MAX_NAME_SIZE }> = String::try_from(sender)
            .map_err(|_| Error::Validation(Error::MAX_NAME_SIZE_EXCEEDED))
            .unwrap();
        Self {
            id,
            name: name_str,
            dlc,
            sender: sender_str,
            signals: SignalList::from_slice(signals),
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
    pub fn signals(&self) -> &SignalList {
        &self.signals
    }
}
