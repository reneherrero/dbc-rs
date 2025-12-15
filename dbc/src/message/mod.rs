mod core;
mod parse;
#[cfg(feature = "std")]
mod serialize;
mod signals;
mod validate;

#[cfg(feature = "std")]
mod builder;

use crate::{MAX_NAME_SIZE, compat::String};
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
    name: String<{ MAX_NAME_SIZE }>,
    dlc: u8,
    sender: String<{ MAX_NAME_SIZE }>,
    signals: Signals,
}
