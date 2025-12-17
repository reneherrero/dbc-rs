mod core;
mod parse;

#[cfg(feature = "std")]
mod builder;

use crate::{
    MAX_NAME_SIZE, MAX_NODES,
    compat::{String, Vec},
};

/// Represents the receiver nodes for a signal in a DBC file.
///
/// Per the DBC specification (Section 9.5), receivers are defined as:
/// ```bnf
/// receiver = node_name | 'Vector__XXX' ;
/// receivers = receiver {',' receiver} ;
/// ```
///
/// A signal can have two types of receivers:
/// - **Specific nodes**: A list of specific node names that receive this signal
/// - **None**: No explicit receivers specified (use `Vector__XXX` in DBC output)
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
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" Vector__XXX
///  SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C" TCM,BCM
/// "#)?;
///
/// let message = dbc.messages().at(0).unwrap();
///
/// // No specific receiver (Vector__XXX)
/// let rpm_signal = message.signals().find("RPM").unwrap();
/// assert_eq!(rpm_signal.receivers().len(), 0);
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
/// In DBC files, receivers are specified after the signal unit:
/// - Comma-separated node names indicate specific receivers (per spec)
/// - `Vector__XXX` indicates no specific receiver
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::large_enum_variant)]
pub enum Receivers {
    /// Specific receiver nodes - vector of node names.
    Nodes(Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES - 1 }>),
    /// No explicit receivers specified (serializes as `Vector__XXX`).
    None,
}

#[cfg(feature = "std")]
pub use builder::ReceiversBuilder;
