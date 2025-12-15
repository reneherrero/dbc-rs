#[cfg(feature = "std")]
use crate::comment::Comment;
use crate::compat::Vec;
#[cfg(feature = "std")]
use crate::environment_variable::EnvironmentVariable;
#[cfg(feature = "std")]
use crate::environment_variable_data::EnvironmentVariableData;
#[cfg(feature = "std")]
use crate::extended_multiplexing::ExtendedMultiplexing;
#[cfg(feature = "std")]
use crate::message_transmitter::MessageTransmitter;
#[cfg(feature = "std")]
use crate::signal_group::SignalGroup;
#[cfg(feature = "std")]
use crate::signal_type_attribute::SignalTypeAttribute;
#[cfg(feature = "std")]
use crate::signal_type_attribute_definition::SignalTypeAttributeDefinition;
#[cfg(feature = "std")]
use crate::value_table::ValueTable;
use crate::{BitTiming, Error, MAX_MESSAGES, Message, MessageList, Nodes, Result, Version};
#[cfg(feature = "std")]
use crate::{ValueDescriptions, ValueDescriptionsList};
#[cfg(feature = "std")]
use std::collections::BTreeMap;

use super::super::super::Dbc;
use crate::dbc::validation::Validation;

/// State accumulated during parsing
pub struct ParseState {
    pub version: Option<Version>,
    pub nodes: Option<Nodes>,
    pub bit_timing: Option<BitTiming>,
    pub messages_buffer: Vec<Message, { MAX_MESSAGES }>,
    pub message_count_actual: usize,
    #[cfg(feature = "std")]
    #[allow(clippy::type_complexity)]
    pub value_descriptions_buffer: std::vec::Vec<(
        Option<u32>,
        std::string::String,
        std::vec::Vec<(u64, std::string::String)>,
    )>,
    #[cfg(feature = "std")]
    pub attributes_buffer: std::vec::Vec<crate::attributes::AttributeDefinition>,
    #[cfg(feature = "std")]
    pub attribute_defaults_buffer: std::vec::Vec<crate::attributes::AttributeDefault>,
    #[cfg(feature = "std")]
    pub attribute_values_buffer: std::vec::Vec<crate::attributes::Attribute>,
    #[cfg(feature = "std")]
    pub comments_buffer: std::vec::Vec<Comment>,
    #[cfg(feature = "std")]
    pub value_tables_buffer: std::vec::Vec<ValueTable>,
    #[cfg(feature = "std")]
    pub extended_multiplexing_buffer: std::vec::Vec<ExtendedMultiplexing>,
    #[cfg(feature = "std")]
    pub signal_value_types_buffer:
        BTreeMap<(u32, std::string::String), crate::SignalExtendedValueType>,
    #[cfg(feature = "std")]
    pub signal_types_buffer: std::vec::Vec<crate::SignalType>,
    #[cfg(feature = "std")]
    pub signal_type_references_buffer: std::vec::Vec<crate::SignalTypeReference>,
    #[cfg(feature = "std")]
    pub signal_type_values_buffer: std::vec::Vec<crate::SignalTypeValue>,
    #[cfg(feature = "std")]
    pub signal_groups_buffer: std::vec::Vec<SignalGroup>,
    #[cfg(feature = "std")]
    pub message_transmitters_buffer: std::vec::Vec<MessageTransmitter>,
    #[cfg(feature = "std")]
    pub signal_type_attribute_definitions_buffer: std::vec::Vec<SignalTypeAttributeDefinition>,
    #[cfg(feature = "std")]
    pub signal_type_attributes_buffer: std::vec::Vec<SignalTypeAttribute>,
    #[cfg(feature = "std")]
    pub environment_variables_buffer: std::vec::Vec<EnvironmentVariable>,
    #[cfg(feature = "std")]
    pub environment_variable_data_buffer: std::vec::Vec<EnvironmentVariableData>,
}

impl ParseState {
    pub fn new() -> Self {
        Self {
            version: None,
            nodes: None,
            bit_timing: None,
            messages_buffer: Vec::new(),
            message_count_actual: 0,
            #[cfg(feature = "std")]
            value_descriptions_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            attributes_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            attribute_defaults_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            attribute_values_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            comments_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            value_tables_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            extended_multiplexing_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            signal_value_types_buffer: BTreeMap::new(),
            #[cfg(feature = "std")]
            signal_types_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            signal_type_references_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            signal_type_values_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            signal_groups_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            message_transmitters_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            signal_type_attribute_definitions_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            signal_type_attributes_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            environment_variables_buffer: std::vec::Vec::new(),
            #[cfg(feature = "std")]
            environment_variable_data_buffer: std::vec::Vec::new(),
        }
    }

    pub fn build_dbc(self) -> Result<Dbc> {
        // Allow empty nodes (DBC spec allows empty BU_: line)
        let nodes = self.nodes.unwrap_or_default();

        // If no version was parsed, default to empty version
        let version = self.version.or_else(|| {
            static EMPTY_VERSION: &[u8] = b"VERSION \"\"";
            let mut parser = crate::Parser::new(EMPTY_VERSION).ok()?;
            Version::parse(&mut parser).ok()
        });

        // Build value descriptions map for storage in Dbc
        #[cfg(feature = "std")]
        let value_descriptions_list = {
            let mut map: BTreeMap<(Option<u32>, std::string::String), ValueDescriptions> =
                BTreeMap::new();
            for (message_id, signal_name, entries) in self.value_descriptions_buffer {
                let key = (message_id, signal_name);
                let value_descriptions = ValueDescriptions::from_slice(&entries);
                map.insert(key, value_descriptions);
            }
            ValueDescriptionsList::from_map(map)
        };

        // Convert messages buffer to slice for validation and construction
        let messages_slice: &[Message] = self.messages_buffer.as_slice();

        // Validate messages (duplicate IDs, sender in nodes, etc.)
        #[cfg(feature = "std")]
        Validation::validate(&nodes, messages_slice, Some(&value_descriptions_list)).map_err(
            |e| {
                crate::error::map_val_error(e, Error::Message, || {
                    Error::Message(Error::MESSAGE_ERROR_PREFIX)
                })
            },
        )?;
        #[cfg(not(feature = "std"))]
        Validation::validate(&nodes, messages_slice).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(Error::MESSAGE_ERROR_PREFIX)
            })
        })?;

        // Construct directly (validation already done)
        let messages = MessageList::new(messages_slice)?;
        #[cfg(feature = "std")]
        {
            Ok(Dbc::new_with_extras(
                version,
                nodes,
                messages,
                value_descriptions_list,
                self.attributes_buffer,
                self.attribute_defaults_buffer,
                self.attribute_values_buffer,
                self.comments_buffer,
                self.value_tables_buffer,
                self.extended_multiplexing_buffer,
                self.signal_value_types_buffer,
                self.signal_types_buffer,
                self.signal_type_references_buffer,
                self.signal_type_values_buffer,
                self.bit_timing,
                self.signal_groups_buffer,
                self.message_transmitters_buffer,
                self.signal_type_attribute_definitions_buffer,
                self.signal_type_attributes_buffer,
                self.environment_variables_buffer,
                self.environment_variable_data_buffer,
            ))
        }
        #[cfg(not(feature = "std"))]
        {
            Ok(Dbc::new_no_std(version, nodes, messages, self.bit_timing))
        }
    }
}
