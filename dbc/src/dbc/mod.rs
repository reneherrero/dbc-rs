use crate::{ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, Nodes, Version, compat::Vec};

// Module declarations
mod messages;
#[cfg(feature = "std")]
mod value_descriptions_map;

// Include modules for additional functionality
#[cfg(feature = "std")]
mod builder;
mod core;
mod decode;
mod parse;
#[cfg(feature = "std")]
mod serialize;
mod validate;

// Re-exports
use messages::Messages;

#[cfg(feature = "std")]
use value_descriptions_map::ValueDescriptionsMap;

#[cfg(feature = "std")]
pub use builder::DbcBuilder;

use validate::Validate;

/// Represents a complete DBC (CAN database) file.
///
/// A `Dbc` contains:
/// - An optional version string
/// - A list of nodes (ECUs)
/// - A collection of messages with their signals
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM TCM
///
/// BO_ 256 EngineData : 8 ECM
///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" TCM
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// println!("Parsed {} messages", dbc.messages().len());
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Dbc {
    version: Option<Version>,
    nodes: Nodes,
    messages: Messages,
    #[cfg(feature = "std")]
    value_descriptions: ValueDescriptionsMap,
    extended_multiplexing: Vec<ExtendedMultiplexing, { MAX_EXTENDED_MULTIPLEXING }>,
}
