use crate::{ExtendedMultiplexing, MAX_EXTENDED_MULTIPLEXING, Nodes, Version, compat::Vec};

type ExtendedMultiplexings = Vec<ExtendedMultiplexing, { MAX_EXTENDED_MULTIPLEXING }>;

// Index for fast extended multiplexing lookup by (message_id, signal_name)
// Maps to indices into the extended_multiplexing vec
mod ext_mux_index;
use ext_mux_index::ExtMuxIndex;

// Module declarations
mod messages;
mod value_descriptions_map;

// Include modules for additional functionality
#[cfg(feature = "std")]
mod builder;
mod decode;
mod impls;
mod parse;
#[cfg(feature = "std")]
mod std;
mod validate;

// Re-exports
use messages::Messages;

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
    value_descriptions: ValueDescriptionsMap,
    extended_multiplexing: ExtendedMultiplexings,
    /// Index for O(1) extended multiplexing lookup by (message_id, signal_name)
    ext_mux_index: ExtMuxIndex,
}
