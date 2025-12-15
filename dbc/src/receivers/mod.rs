mod core;
mod parse;

#[cfg(feature = "std")]
mod builder;

use crate::{
    MAX_NAME_SIZE, MAX_RECEIVER_NODES,
    compat::{String, Vec},
};

/// Represents the receiver nodes for a signal in a DBC file.
///
/// A signal can have three types of receivers:
/// - **Broadcast** (`*`): The signal is broadcast to all nodes on the bus
/// - **Specific nodes**: A list of specific node names that receive this signal
/// - **None**: No explicit receivers specified (signal may be unused or receiver is implicit)
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc = Dbc::parse(r#"VERSION "1.0"
///
/// BU_: ECM TCM BCM
///
/// BO_ 256 Engine : 8 ECM
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
///  SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C" TCM BCM
/// "#)?;
///
/// let message = dbc.messages().at(0).unwrap();
///
/// // Broadcast receiver
/// let rpm_signal = message.signals().find("RPM").unwrap();
/// assert_eq!(rpm_signal.receivers().len(), 0); // Broadcast has no specific nodes
///
/// // Specific nodes
/// let temp_signal = message.signals().find("Temp").unwrap();
/// assert_eq!(temp_signal.receivers().len(), 2);
/// assert!(temp_signal.receivers().contains("TCM"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # DBC Format
///
/// In DBC files, receivers are specified after the signal definition:
/// - `*` indicates broadcast
/// - Space-separated node names indicate specific receivers
/// - No receivers means `None`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::large_enum_variant)]
pub enum Receivers {
    /// Broadcast receiver - signal is sent to all nodes on the bus.
    Broadcast,
    /// Specific receiver nodes - vector of node names.
    Nodes(Vec<String<{ MAX_NAME_SIZE }>, { MAX_RECEIVER_NODES }>),
    /// No explicit receivers specified.
    None,
}

#[cfg(feature = "std")]
pub use builder::ReceiversBuilder;
