mod decode;
mod impls;
mod parse;
mod signals;
#[cfg(feature = "std")]
mod std;
mod validate;

#[cfg(feature = "std")]
mod builder;

use crate::compat::{Comment, Name};
#[cfg(feature = "std")]
pub use builder::MessageBuilder;
pub use signals::Signals;

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
    name: Name,
    dlc: u8,
    sender: Name,
    signals: Signals,
    /// Comment text from CM_ BO_ entry
    comment: Option<Comment>,
}

impl Message {
    // CAN ID validation constants (per DBC spec Section 8.1)
    // - Standard CAN ID: 0 to 2047 (0x7FF) - 11-bit identifier
    // - Extended CAN ID: Set bit 31 (0x80000000) and use bits 0-28
    // Valid ID ranges:
    // - 0x00000000 to 0x1FFFFFFF (standard or extended without flag)
    // - 0x80000000 to 0x9FFFFFFF (extended with bit 31 flag)
    // - 0xC0000000 (special pseudo-message ID per Section 8.6)
    // Invalid: 0x20000000 to 0x7FFFFFFF and 0xA0000000+ (except 0xC0000000)

    /// Maximum 29-bit extended CAN ID value
    const MAX_EXTENDED_ID: u32 = 0x1FFF_FFFF;
    /// Bit 31 flag indicating extended CAN ID format
    pub(crate) const EXTENDED_ID_FLAG: u32 = 0x8000_0000;
    /// Maximum extended CAN ID with bit 31 flag set
    const MAX_EXTENDED_ID_WITH_FLAG: u32 = Self::EXTENDED_ID_FLAG | Self::MAX_EXTENDED_ID;
    /// Special pseudo-message ID for VECTOR__INDEPENDENT_SIG_MSG (per spec Section 8.6)
    const PSEUDO_MESSAGE_ID: u32 = 0xC000_0000;
}
