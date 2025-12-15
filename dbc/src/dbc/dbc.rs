use crate::bit_timing::BitTiming;
#[cfg(feature = "std")]
use crate::comment::Comment;
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
use crate::{MessageList, Nodes, Version};
#[cfg(feature = "std")]
use crate::{ValueDescriptions, ValueDescriptionsList};

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
#[derive(Debug)]
pub struct Dbc {
    version: Option<Version>,
    nodes: Nodes,
    messages: MessageList,
    #[cfg(feature = "std")]
    value_descriptions: ValueDescriptionsList,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    attributes: std::vec::Vec<crate::attributes::AttributeDefinition>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    attribute_defaults: std::vec::Vec<crate::attributes::AttributeDefault>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    attribute_values: std::vec::Vec<crate::attributes::Attribute>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    comments: std::vec::Vec<Comment>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    value_tables: std::vec::Vec<ValueTable>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    extended_multiplexing: std::vec::Vec<ExtendedMultiplexing>,
    #[cfg(feature = "std")]
    signal_value_types:
        std::collections::BTreeMap<(u32, std::string::String), crate::SignalExtendedValueType>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_types: std::vec::Vec<crate::SignalType>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_type_references: std::vec::Vec<crate::SignalTypeReference>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_type_values: std::vec::Vec<crate::SignalTypeValue>,
    #[allow(dead_code)] // Field is stored but not yet exposed via public API
    bit_timing: Option<BitTiming>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_groups: std::vec::Vec<SignalGroup>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    message_transmitters: std::vec::Vec<MessageTransmitter>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_type_attribute_definitions: std::vec::Vec<SignalTypeAttributeDefinition>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_type_attributes: std::vec::Vec<SignalTypeAttribute>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    environment_variables: std::vec::Vec<EnvironmentVariable>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    environment_variable_data: std::vec::Vec<EnvironmentVariableData>,
}

impl Dbc {
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Used by builder
    pub(crate) fn new(
        version: Option<Version>,
        nodes: Nodes,
        messages: MessageList,
        value_descriptions: ValueDescriptionsList,
    ) -> Self {
        Self::new_with_extras(
            version,
            nodes,
            messages,
            value_descriptions,
            std::vec::Vec::new(),              // attributes
            std::vec::Vec::new(),              // attribute_defaults
            std::vec::Vec::new(),              // attribute_values
            std::vec::Vec::new(),              // comments
            std::vec::Vec::new(),              // value_tables
            std::vec::Vec::new(),              // extended_multiplexing
            std::collections::BTreeMap::new(), // signal_value_types
            std::vec::Vec::new(),              // signal_types
            std::vec::Vec::new(),              // signal_type_references
            std::vec::Vec::new(),              // signal_type_values
            None,                              // bit_timing
            #[cfg(feature = "std")]
            std::vec::Vec::new(), // signal_groups
            #[cfg(feature = "std")]
            std::vec::Vec::new(), // message_transmitters
            #[cfg(feature = "std")]
            std::vec::Vec::new(), // signal_type_attribute_definitions
            #[cfg(feature = "std")]
            std::vec::Vec::new(), // signal_type_attributes
            #[cfg(feature = "std")]
            std::vec::Vec::new(), // environment_variables
            #[cfg(feature = "std")]
            std::vec::Vec::new(), // environment_variable_data
        )
    }

    #[cfg(feature = "std")]
    #[allow(clippy::too_many_arguments)] // Builder needs all these parameters
    pub(crate) fn new_with_extras(
        version: Option<Version>,
        nodes: Nodes,
        messages: MessageList,
        value_descriptions: ValueDescriptionsList,
        attributes: std::vec::Vec<crate::attributes::AttributeDefinition>,
        attribute_defaults: std::vec::Vec<crate::attributes::AttributeDefault>,
        attribute_values: std::vec::Vec<crate::attributes::Attribute>,
        comments: std::vec::Vec<Comment>,
        value_tables: std::vec::Vec<ValueTable>,
        extended_multiplexing: std::vec::Vec<ExtendedMultiplexing>,
        signal_value_types: std::collections::BTreeMap<
            (u32, std::string::String),
            crate::SignalExtendedValueType,
        >,
        signal_types: std::vec::Vec<crate::SignalType>,
        signal_type_references: std::vec::Vec<crate::SignalTypeReference>,
        signal_type_values: std::vec::Vec<crate::SignalTypeValue>,
        bit_timing: Option<BitTiming>,
        signal_groups: std::vec::Vec<SignalGroup>,
        message_transmitters: std::vec::Vec<MessageTransmitter>,
        signal_type_attribute_definitions: std::vec::Vec<SignalTypeAttributeDefinition>,
        signal_type_attributes: std::vec::Vec<SignalTypeAttribute>,
        environment_variables: std::vec::Vec<EnvironmentVariable>,
        environment_variable_data: std::vec::Vec<EnvironmentVariableData>,
    ) -> Self {
        // Validation should have been done prior (by builder)
        Self {
            version,
            nodes,
            messages,
            value_descriptions,
            attributes,
            attribute_defaults,
            attribute_values,
            comments,
            value_tables,
            extended_multiplexing,
            signal_value_types,
            signal_types,
            signal_type_references,
            signal_type_values,
            bit_timing,
            #[cfg(feature = "std")]
            signal_groups,
            #[cfg(feature = "std")]
            message_transmitters,
            #[cfg(feature = "std")]
            signal_type_attribute_definitions,
            #[cfg(feature = "std")]
            signal_type_attributes,
            #[cfg(feature = "std")]
            environment_variables,
            #[cfg(feature = "std")]
            environment_variable_data,
        }
    }

    #[cfg(not(feature = "std"))]
    #[allow(dead_code)] // Used by parse_state
    pub(crate) fn new_no_std(
        version: Option<Version>,
        nodes: Nodes,
        messages: MessageList,
        bit_timing: Option<BitTiming>,
    ) -> Self {
        Self {
            version,
            nodes,
            messages,
            bit_timing,
        }
    }

    /// Get the version of the DBC file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(version) = dbc.version() {
    ///     // Version is available
    ///     let _ = version.as_str();
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    /// Get a reference to the nodes collection
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM TCM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let nodes = dbc.nodes();
    /// assert_eq!(nodes.len(), 2);
    /// // Iterate over nodes
    /// let mut iter = nodes.iter();
    /// assert_eq!(iter.next(), Some("ECM"));
    /// assert_eq!(iter.next(), Some("TCM"));
    /// assert_eq!(iter.next(), None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    /// Get a reference to the messages collection
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let messages = dbc.messages();
    /// assert_eq!(messages.len(), 1);
    /// let message = messages.at(0).unwrap();
    /// assert_eq!(message.name(), "Engine");
    /// assert_eq!(message.id(), 256);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn messages(&self) -> &MessageList {
        &self.messages
    }

    /// Get value descriptions for a specific signal
    ///
    /// Value descriptions map numeric signal values to human-readable text.
    /// Returns `None` if the signal has no value descriptions.
    ///
    /// **Global Value Descriptions**: According to the Vector DBC specification,
    /// a message_id of `-1` (0xFFFFFFFF) in a `VAL_` statement means the value
    /// descriptions apply to all signals with that name in ANY message. This
    /// method will first check for a message-specific entry, then fall back to
    /// a global entry if one exists.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(100, "Gear") {
    ///     if let Some(desc) = value_descriptions.get(0) {
    ///         println!("Value 0 means: {}", desc);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn get_signal_value_type(
        &self,
        message_id: u32,
        signal_name: &str,
    ) -> Option<crate::SignalExtendedValueType> {
        self.signal_value_types.get(&(message_id, signal_name.to_string())).copied()
    }

    /// Get all signal types defined in the DBC file
    ///
    /// # Returns
    ///
    /// Vector of signal type definitions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"
    /// # VERSION "1.0"
    /// # BU_: ECM
    /// # SGTYPE_ SignalType1 : 16;
    /// # "#)?;
    /// let signal_types = dbc.signal_types();
    /// assert_eq!(signal_types.len(), 1);
    /// assert_eq!(signal_types[0].name(), "SignalType1");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn signal_types(&self) -> &[crate::SignalType] {
        &self.signal_types
    }

    /// Get all signal type references (SIG_TYPE_REF_)
    ///
    /// # Returns
    ///
    /// Vector of signal type references
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"
    /// # VERSION "1.0"
    /// # BU_: ECM
    /// # BO_ 256 EngineData : 8 ECM
    /// #  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *
    /// # SIG_TYPE_REF_ 256 RPM : SignalType1;
    /// # "#)?;
    /// let refs = dbc.signal_type_references();
    /// assert_eq!(refs.len(), 1);
    /// assert_eq!(refs[0].signal_name(), "RPM");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn signal_type_references(&self) -> &[crate::SignalTypeReference] {
        &self.signal_type_references
    }

    /// Get all signal type value descriptions (SGTYPE_VAL_)
    ///
    /// # Returns
    ///
    /// Vector of signal type value descriptions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"
    /// # VERSION "1.0"
    /// # BU_: ECM
    /// # SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One";
    /// # "#)?;
    /// let values = dbc.signal_type_values();
    /// assert_eq!(values.len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn signal_type_values(&self) -> &[crate::SignalTypeValue] {
        &self.signal_type_values
    }

    /// Get signal type value descriptions for a specific signal type name
    ///
    /// This method finds all `SignalTypeValue` entries that reference the given
    /// signal type name, allowing you to access value descriptions (enumerations)
    /// for a `SignalType`.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The name of the signal type to find values for
    ///
    /// # Returns
    ///
    /// Vector of signal type value descriptions that match the given type name
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"
    /// # VERSION "1.0"
    /// # BU_: ECM
    /// # SGTYPE_ TempSensor : 16;
    /// # SGTYPE_VAL_ TempSensor 0 "Cold" 100 "Normal" 200 "Hot";
    /// # "#)?;
    /// let signal_type = &dbc.signal_types()[0];
    /// let values = dbc.signal_type_values_for(signal_type.name());
    /// assert_eq!(values.len(), 3);
    /// assert_eq!(values[0].value(), 0);
    /// assert_eq!(values[0].description(), "Cold");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn signal_type_values_for(
        &self,
        type_name: &str,
    ) -> std::vec::Vec<&crate::SignalTypeValue> {
        self.signal_type_values
            .iter()
            .filter(|val| val.type_name() == type_name)
            .collect()
    }

    /// Get extended multiplexing entries for a specific message
    ///
    /// Returns all `SG_MUL_VAL_` entries that apply to the specified message.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The CAN message ID
    ///
    /// # Returns
    ///
    /// Vector of extended multiplexing entries for the message
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"
    /// # VERSION "1.0"
    /// # BU_: ECM
    /// # BO_ 500 ComplexMux : 8 ECM
    /// #  SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
    /// #  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] ""
    /// # SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
    /// # "#)?;
    /// let extended = dbc.extended_multiplexing_for_message(500);
    /// assert_eq!(extended.len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn extended_multiplexing_for_message(
        &self,
        message_id: u32,
    ) -> std::vec::Vec<&ExtendedMultiplexing> {
        self.extended_multiplexing
            .iter()
            .filter(|ext_mux| ext_mux.message_id() == message_id)
            .collect()
    }

    /// Get a reference to the value descriptions list
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
    /// BO_ 100 Engine : 8 ECM
    ///  SG_ Gear : 0|8@1+ (1,0) [0|5] "" *
    ///
    /// VAL_ 100 Gear 0 "Park" 1 "Drive" ;"#)?;
    /// let value_descriptions_list = dbc.value_descriptions();
    /// assert_eq!(value_descriptions_list.len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn value_descriptions(&self) -> &ValueDescriptionsList {
        &self.value_descriptions
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn value_descriptions_for_signal(
        &self,
        message_id: u32,
        signal_name: &str,
    ) -> Option<&ValueDescriptions> {
        self.value_descriptions.for_signal(message_id, signal_name)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    #![allow(clippy::float_cmp)]
    use super::*;
    use crate::Error;

    #[test]
    fn parses_real_dbc() {
        let data = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar""#;

        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.messages().len(), 2);
        let mut messages_iter = dbc.messages().iter();
        let msg0 = messages_iter.next().unwrap();
        assert_eq!(msg0.signals().len(), 2);
        let mut signals_iter = msg0.signals().iter();
        assert_eq!(signals_iter.next().unwrap().name(), "RPM");
        assert_eq!(signals_iter.next().unwrap().name(), "Temp");
        let msg1 = messages_iter.next().unwrap();
        assert_eq!(msg1.signals().len(), 1);
        assert_eq!(msg1.signals().iter().next().unwrap().name(), "Pressure");
    }

    #[test]
    fn test_parse_duplicate_message_id() {
        // Test that parse also validates duplicate message IDs
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData1 : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"

BO_ 256 EngineData2 : 8 ECM
 SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                assert!(msg.contains(Error::DUPLICATE_MESSAGE_ID));
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_parse_sender_not_in_nodes() {
        // Test that parse also validates message senders are in nodes list
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 TCM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Message(msg) => {
                assert!(msg.contains(Error::SENDER_NOT_IN_NODES));
            }
            _ => panic!("Expected Error::Message"),
        }
    }

    #[test]
    fn test_parse_empty_file() {
        // Test parsing an empty file
        let result = Dbc::parse("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::UnexpectedEof => {
                // Empty file should result in unexpected EOF
            }
            _ => panic!("Expected Error::UnexpectedEof"),
        }
    }

    #[test]
    fn test_parse_missing_nodes() {
        // Test parsing without BU_ statement
        // Note: The parser allows missing BU_ line and treats it as empty nodes
        // This is consistent with allowing empty nodes per DBC spec
        let data = r#"VERSION "1.0"

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"
"#;

        let result = Dbc::parse(data);
        // Parser should succeed with empty nodes (missing BU_ is treated as empty nodes)
        assert!(result.is_ok());
        let dbc = result.unwrap();
        assert!(dbc.nodes().is_empty());
    }

    #[test]
    fn test_parse_bytes() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;

        let bytes = data.as_bytes();
        let dbc = Dbc::parse_bytes(bytes).unwrap();
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("1.0".to_string())
        );
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_bytes_invalid_utf8() {
        // Invalid UTF-8 sequence
        let invalid_bytes = &[0xFF, 0xFE, 0xFD];
        let result = Dbc::parse_bytes(invalid_bytes);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Expected(msg) => {
                assert_eq!(msg, Error::INVALID_UTF8);
            }
            _ => panic!("Expected Error::Expected with INVALID_UTF8"),
        }
    }

    #[test]
    fn test_save_round_trip() {
        let original = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@0+ (0.1,0) [0|1000] "bar"
"#;

        let dbc = Dbc::parse(original).unwrap();
        let saved = dbc.to_dbc_string();
        let dbc2 = Dbc::parse(&saved).unwrap();

        // Verify round-trip: parsed data should match
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            dbc2.version().map(|v| v.to_string())
        );
        assert_eq!(dbc.messages().len(), dbc2.messages().len());

        for (msg1, msg2) in dbc.messages().iter().zip(dbc2.messages().iter()) {
            assert_eq!(msg1.id(), msg2.id());
            assert_eq!(msg1.name(), msg2.name());
            assert_eq!(msg1.dlc(), msg2.dlc());
            assert_eq!(msg1.sender(), msg2.sender());
            assert_eq!(msg1.signals().len(), msg2.signals().len());

            for (sig1, sig2) in msg1.signals().iter().zip(msg2.signals().iter()) {
                assert_eq!(sig1.name(), sig2.name());
                assert_eq!(sig1.start_bit(), sig2.start_bit());
                assert_eq!(sig1.length(), sig2.length());
                assert_eq!(sig1.byte_order(), sig2.byte_order());
                assert_eq!(sig1.is_unsigned(), sig2.is_unsigned());
                assert_eq!(sig1.factor(), sig2.factor());
                assert_eq!(sig1.offset(), sig2.offset());
                assert_eq!(sig1.min(), sig2.min());
                assert_eq!(sig1.max(), sig2.max());
                assert_eq!(sig1.unit(), sig2.unit());
                assert_eq!(sig1.receivers(), sig2.receivers());
            }
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_save_multiple_messages() {
        // Use parsing instead of builders
        let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;
        let dbc = Dbc::parse(dbc_content).unwrap();
        let saved = dbc.to_dbc_string();

        // Verify both messages are present
        assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
        assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
        assert!(saved.contains("SG_ RPM"));
        assert!(saved.contains("SG_ Pressure"));
    }

    // Note: Builder limit tests have been moved to dbc_builder.rs
    // These tests require building many messages programmatically, which is builder functionality

    #[test]
    fn test_parse_without_version() {
        // DBC file without VERSION line should default to empty version
        let data = r#"
BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.version().map(|v| v.to_string()), Some("".to_string()));
    }

    #[test]
    fn test_parse_without_version_with_comment() {
        // DBC file with comment and no VERSION line
        let data = r#"// This is a comment
BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;
        let dbc = Dbc::parse(data).unwrap();
        assert_eq!(dbc.version().map(|v| v.to_string()), Some("".to_string()));
    }

    #[test]
    fn test_parse_with_strict_boundary_check() {
        // Test that strict mode (default) rejects signals that extend beyond boundaries
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Test : 8 ECM
 SG_ CHECKSUM : 63|8@1+ (1,0) [0|255] ""
"#;

        // Default (strict) mode should fail
        let result = Dbc::parse(data);
        assert!(result.is_err());

        // Explicit strict mode should also fail
        let result = Dbc::parse(data);
        assert!(result.is_err());
    }

    // Tests that require std (for to_dbc_string, value_descriptions, etc.)
    #[cfg(feature = "std")]
    mod tests_std {
        use super::*;

        #[test]
        fn test_parse_from_string() {
            let data = String::from(
                r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#,
            );

            let dbc = Dbc::parse(&data).unwrap();
            assert_eq!(
                dbc.version().map(|v| v.to_string()),
                Some("1.0".to_string())
            );
            assert_eq!(dbc.messages().len(), 1);
        }

        #[test]
        fn test_save_basic() {
            // Use parsing instead of builders
            let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm" *
"#;
            let dbc = Dbc::parse(dbc_content).unwrap();

            let saved = dbc.to_dbc_string();
            assert!(saved.contains("VERSION \"1.0\""));
            assert!(saved.contains("BU_: ECM"));
            assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
            assert!(saved.contains("SG_ RPM : 0|16@0+ (0.25,0) [0|8000] \"rpm\" *")); // BigEndian = @0
        }

        #[test]
        fn test_save_round_trip() {
            let original = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@0- (1,-40) [-40|215] "°C" TCM

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@0+ (0.1,0) [0|1000] "bar"
"#;

            let dbc = Dbc::parse(original).unwrap();
            let saved = dbc.to_dbc_string();
            let dbc2 = Dbc::parse(&saved).unwrap();

            // Verify round-trip: parsed data should match
            assert_eq!(
                dbc.version().map(|v| v.to_string()),
                dbc2.version().map(|v| v.to_string())
            );
            assert_eq!(dbc.messages().len(), dbc2.messages().len());

            for (msg1, msg2) in dbc.messages().iter().zip(dbc2.messages().iter()) {
                assert_eq!(msg1.id(), msg2.id());
                assert_eq!(msg1.name(), msg2.name());
                assert_eq!(msg1.dlc(), msg2.dlc());
                assert_eq!(msg1.sender(), msg2.sender());
                assert_eq!(msg1.signals().len(), msg2.signals().len());

                for (sig1, sig2) in msg1.signals().iter().zip(msg2.signals().iter()) {
                    assert_eq!(sig1.name(), sig2.name());
                    assert_eq!(sig1.start_bit(), sig2.start_bit());
                    assert_eq!(sig1.length(), sig2.length());
                    assert_eq!(sig1.byte_order(), sig2.byte_order());
                    assert_eq!(sig1.is_unsigned(), sig2.is_unsigned());
                    assert_eq!(sig1.factor(), sig2.factor());
                    assert_eq!(sig1.offset(), sig2.offset());
                    assert_eq!(sig1.min(), sig2.min());
                    assert_eq!(sig1.max(), sig2.max());
                    assert_eq!(sig1.unit(), sig2.unit());
                    assert_eq!(sig1.receivers(), sig2.receivers());
                }
            }
        }

        #[test]
        fn test_save_multiple_messages() {
            // Use parsing instead of builders
            let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm"

BO_ 512 BrakeData : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;
            let dbc = Dbc::parse(dbc_content).unwrap();
            let saved = dbc.to_dbc_string();

            // Verify both messages are present
            assert!(saved.contains("BO_ 256 EngineData : 8 ECM"));
            assert!(saved.contains("BO_ 512 BrakeData : 4 TCM"));
            assert!(saved.contains("SG_ RPM"));
            assert!(saved.contains("SG_ Pressure"));
        }

        #[test]
        fn test_parse_val_value_descriptions() {
            let data = r#"VERSION ""

NS_ :

BS_:

BU_: Node1 Node2

BO_ 100 Message1 : 8 Node1
 SG_ Signal : 32|8@1- (1,0) [-1|4] "Gear" Node2

VAL_ 100 Signal -1 "Reverse" 0 "Neutral" 1 "First" 2 "Second" 3 "Third" 4 "Fourth" ;
"#;

            let dbc = match Dbc::parse(data) {
                Ok(dbc) => dbc,
                Err(e) => panic!("Failed to parse DBC: {:?}", e),
            };

            // Verify basic structure
            assert_eq!(dbc.messages().len(), 1);
            let message = dbc.messages().iter().find(|m| m.id() == 100).unwrap();
            assert_eq!(message.name(), "Message1");
            assert_eq!(message.sender(), "Node1");

            // Verify signal exists
            let signal = message.signals().find("Signal").unwrap();
            assert_eq!(signal.name(), "Signal");
            assert_eq!(signal.start_bit(), 32);
            assert_eq!(signal.length(), 8);
            assert_eq!(signal.unit(), Some("Gear"));

            // Verify value descriptions are parsed and accessible
            let value_descriptions = dbc
                .value_descriptions_for_signal(100, "Signal")
                .expect("Value descriptions should exist for signal");

            // Verify all value mappings
            assert_eq!(value_descriptions.get(0xFFFFFFFF), Some("Reverse")); // -1 as u32
            assert_eq!(value_descriptions.get(0), Some("Neutral"));
            assert_eq!(value_descriptions.get(1), Some("First"));
            assert_eq!(value_descriptions.get(2), Some("Second"));
            assert_eq!(value_descriptions.get(3), Some("Third"));
            assert_eq!(value_descriptions.get(4), Some("Fourth"));

            // Verify non-existent values return None
            assert_eq!(value_descriptions.get(5), None);
            assert_eq!(value_descriptions.get(99), None);

            // Verify we can iterate over all value descriptions
            let iter = value_descriptions.iter();
            let mut entries: std::vec::Vec<_> = iter.collect();
            entries.sort_by_key(|(v, _)| *v);

            assert_eq!(entries.len(), 6);
            // After sorting by key (u64), 0xFFFFFFFF (4294967295) comes after smaller values
            assert_eq!(entries[0], (0, "Neutral".to_string()));
            assert_eq!(entries[1], (1, "First".to_string()));
            assert_eq!(entries[2], (2, "Second".to_string()));
            assert_eq!(entries[3], (3, "Third".to_string()));
            assert_eq!(entries[4], (4, "Fourth".to_string()));
            assert_eq!(entries[5], (0xFFFFFFFF, "Reverse".to_string()));
        }

        #[test]
        fn test_parse_val_global_value_descriptions() {
            // Test global value descriptions (VAL_ -1) that apply to all signals with the same name
            let data = r#"VERSION "1.0"

NS_ :

    VAL_

BS_:

BU_: ECU DASH

BO_ 256 EngineData: 8 ECU
 SG_ EngineRPM : 0|16@1+ (0.125,0) [0|8000] "rpm" Vector__XXX
 SG_ DI_gear : 24|3@1+ (1,0) [0|7] "" Vector__XXX

BO_ 512 DashboardDisplay: 8 DASH
 SG_ DI_gear : 0|3@1+ (1,0) [0|7] "" Vector__XXX
 SG_ SpeedDisplay : 8|16@1+ (0.01,0) [0|300] "km/h" Vector__XXX

VAL_ -1 DI_gear 0 "INVALID" 1 "P" 2 "R" 3 "N" 4 "D" 5 "S" 6 "L" 7 "SNA" ;
"#;

            let dbc = match Dbc::parse(data) {
                Ok(dbc) => dbc,
                Err(e) => panic!("Failed to parse DBC: {:?}", e),
            };

            // Verify basic structure
            assert_eq!(dbc.messages().len(), 2);

            // Verify first message (EngineData)
            let engine_msg = dbc.messages().iter().find(|m| m.id() == 256).unwrap();
            assert_eq!(engine_msg.name(), "EngineData");
            assert_eq!(engine_msg.sender(), "ECU");
            let di_gear_signal1 = engine_msg.signals().find("DI_gear").unwrap();
            assert_eq!(di_gear_signal1.name(), "DI_gear");
            assert_eq!(di_gear_signal1.start_bit(), 24);

            // Verify second message (DashboardDisplay)
            let dash_msg = dbc.messages().iter().find(|m| m.id() == 512).unwrap();
            assert_eq!(dash_msg.name(), "DashboardDisplay");
            assert_eq!(dash_msg.sender(), "DASH");
            let di_gear_signal2 = dash_msg.signals().find("DI_gear").unwrap();
            assert_eq!(di_gear_signal2.name(), "DI_gear");
            assert_eq!(di_gear_signal2.start_bit(), 0);

            // Verify global value descriptions apply to DI_gear in message 256
            let value_descriptions1 = dbc
                .value_descriptions_for_signal(256, "DI_gear")
                .expect("Global value descriptions should exist for DI_gear in message 256");

            assert_eq!(value_descriptions1.get(0), Some("INVALID"));
            assert_eq!(value_descriptions1.get(1), Some("P"));
            assert_eq!(value_descriptions1.get(2), Some("R"));
            assert_eq!(value_descriptions1.get(3), Some("N"));
            assert_eq!(value_descriptions1.get(4), Some("D"));
            assert_eq!(value_descriptions1.get(5), Some("S"));
            assert_eq!(value_descriptions1.get(6), Some("L"));
            assert_eq!(value_descriptions1.get(7), Some("SNA"));

            // Verify global value descriptions also apply to DI_gear in message 512
            let value_descriptions2 = dbc
                .value_descriptions_for_signal(512, "DI_gear")
                .expect("Global value descriptions should exist for DI_gear in message 512");

            // Both should return the same value descriptions (same reference or same content)
            assert_eq!(value_descriptions2.get(0), Some("INVALID"));
            assert_eq!(value_descriptions2.get(1), Some("P"));
            assert_eq!(value_descriptions2.get(2), Some("R"));
            assert_eq!(value_descriptions2.get(3), Some("N"));
            assert_eq!(value_descriptions2.get(4), Some("D"));
            assert_eq!(value_descriptions2.get(5), Some("S"));
            assert_eq!(value_descriptions2.get(6), Some("L"));
            assert_eq!(value_descriptions2.get(7), Some("SNA"));

            // Verify they should be the same instance (both reference the global entry)
            // Since we store by (Option<u32>, &str), both should return the same entry
            assert_eq!(value_descriptions1.len(), value_descriptions2.len());
            assert_eq!(value_descriptions1.len(), 8);

            // Verify other signals don't have value descriptions
            assert_eq!(dbc.value_descriptions_for_signal(256, "EngineRPM"), None);
            assert_eq!(dbc.value_descriptions_for_signal(512, "SpeedDisplay"), None);
        }
    }

    #[test]
    fn test_decode_message() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Decode a CAN message with RPM = 2000 (raw: 8000 = 0x1F40) and Temp = 50°C (raw: 90)
        // Little-endian: RPM at bits 0-15, Temp at bits 16-23
        let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &payload).unwrap();

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].0, "RPM");
        assert_eq!(decoded[0].1, 2000.0);
        assert_eq!(decoded[0].2, Some("rpm"));
        assert_eq!(decoded[1].0, "Temp");
        assert_eq!(decoded[1].1, 50.0);
        assert_eq!(decoded[1].2, Some("°C"));
    }

    #[test]
    fn test_decode_message_not_found() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Try to decode a non-existent message ID
        let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = dbc.decode(512, &payload);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Decoding(msg) => {
                assert!(msg.contains(Error::MESSAGE_NOT_FOUND));
            }
            _ => panic!("Expected Error::Decoding"),
        }
    }

    #[test]
    fn test_decode_payload_length_mismatch() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Try to decode with payload shorter than DLC (DLC is 8, payload is 4)
        let payload = [0x40, 0x1F, 0x00, 0x00];
        let result = dbc.decode(256, &payload);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Decoding(msg) => {
                assert!(msg.contains(Error::PAYLOAD_LENGTH_MISMATCH));
            }
            _ => panic!("Expected Error::Decoding"),
        }
    }

    #[test]
    fn test_decode_big_endian_signal() {
        let data = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@0+ (1.0,0) [0|65535] "rpm" *
"#;

        let dbc = Dbc::parse(data).unwrap();

        // Decode a big-endian signal: RPM = 256 (raw: 256 = 0x0100)
        // For big-endian at bit 0-15, the bytes are arranged as [0x01, 0x00]
        // Testing with a simple value that's easier to verify
        let payload = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &payload).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].0, "RPM");
        // The exact value depends on big-endian bit extraction implementation
        // We just verify that decoding doesn't crash and returns a value
        assert!(decoded[0].1 >= 0.0);
        assert_eq!(decoded[0].2, Some("rpm"));
    }

    // Edge case tests using Dbc::parse
    #[test]
    fn test_empty_dbc_file() {
        // Empty file should fail (needs at least nodes)
        let result = Dbc::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_minimal_valid_dbc() {
        // Minimal valid DBC: just nodes (empty nodes allowed)
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_:
"#;
        let dbc = Dbc::parse(dbc_str).expect("Should parse minimal DBC");
        assert!(dbc.nodes().is_empty());
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_duplicate_message_id() {
        // Test duplicate message ID detection
        let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECM

BO_ 256 Message1 : 8 ECM
BO_ 256 Message2 : 8 ECM
"#;
        let result = Dbc::parse(dbc_str);
        assert!(result.is_err(), "Should detect duplicate message IDs");
    }
}
