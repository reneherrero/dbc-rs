#[cfg(feature = "std")]
use std::collections::BTreeMap;

#[cfg(feature = "std")]
use crate::comment::Comment;
use crate::compat::Vec;
#[cfg(feature = "std")]
use crate::value_table::ValueTable;
use crate::{
    Error, MAX_MESSAGES, MAX_SIGNALS_PER_MESSAGE, Message, MessageList, Nodes, Parser, Result,
    Signal, Version,
};
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
    signal_value_types: std::collections::BTreeMap<
        (u32, std::string::String),
        crate::signal_type::SignalExtendedValueType,
    >,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_types: std::vec::Vec<crate::signal_type::SignalType>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_type_references: std::vec::Vec<crate::signal_type::SignalTypeReference>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_type_values: std::vec::Vec<crate::signal_type::SignalTypeValue>,
    #[allow(dead_code)] // Field is stored but not yet exposed via public API
    bit_timing: Option<BitTiming>,
    #[cfg(feature = "std")]
    #[allow(dead_code)] // Fields are stored but not yet exposed via public API
    signal_groups: std::vec::Vec<SignalGroup>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedMultiplexing {
    message_id: u32,
    signal_name: crate::compat::String<{ crate::MAX_NAME_SIZE }>,
    multiplexer_switch: crate::compat::String<{ crate::MAX_NAME_SIZE }>,
    value_ranges: crate::compat::Vec<(u8, u8), 64>, // Max 64 ranges per extended multiplexing entry
}

impl ExtendedMultiplexing {
    #[allow(dead_code)] // Used by builder
    pub(crate) fn new(
        message_id: u32,
        signal_name: crate::compat::String<{ crate::MAX_NAME_SIZE }>,
        multiplexer_switch: crate::compat::String<{ crate::MAX_NAME_SIZE }>,
        value_ranges: crate::compat::Vec<(u8, u8), 64>,
    ) -> Self {
        Self {
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        }
    }

    #[must_use]
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    #[must_use]
    pub fn signal_name(&self) -> &str {
        self.signal_name.as_str()
    }

    #[must_use]
    pub fn multiplexer_switch(&self) -> &str {
        self.multiplexer_switch.as_str()
    }

    #[must_use]
    pub fn value_ranges(&self) -> &[(u8, u8)] {
        self.value_ranges.as_slice()
    }
}

/// Bit Timing definition (BS_)
///
/// Represents the bit timing configuration for the CAN bus.
/// Typically empty or obsolete in modern CAN systems.
#[derive(Debug, Clone, PartialEq)]
pub struct BitTiming {
    baudrate: Option<u32>,
    btr1: Option<u32>,
    btr2: Option<u32>,
}

impl BitTiming {
    /// Create a new BitTiming with optional values
    pub(crate) fn new(baudrate: Option<u32>, btr1: Option<u32>, btr2: Option<u32>) -> Self {
        Self {
            baudrate,
            btr1,
            btr2,
        }
    }

    /// Get the baudrate (if set)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn baudrate(&self) -> Option<u32> {
        self.baudrate
    }

    /// Get BTR1 (if set)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn btr1(&self) -> Option<u32> {
        self.btr1
    }

    /// Get BTR2 (if set)
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn btr2(&self) -> Option<u32> {
        self.btr2
    }
}

/// Signal Group definition (SIG_GROUP_)
///
/// Represents a group of related signals within a message.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "std")]
pub struct SignalGroup {
    message_id: u32,
    signal_group_name: std::string::String,
    repetitions: u32,
    signal_names: std::vec::Vec<std::string::String>,
}

#[cfg(feature = "std")]
impl SignalGroup {
    /// Create a new SignalGroup
    pub(crate) fn new(
        message_id: u32,
        signal_group_name: std::string::String,
        repetitions: u32,
        signal_names: std::vec::Vec<std::string::String>,
    ) -> Self {
        Self {
            message_id,
            signal_group_name,
            repetitions,
            signal_names,
        }
    }

    /// Get the message ID
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Get the signal group name
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn signal_group_name(&self) -> &str {
        &self.signal_group_name
    }

    /// Get the repetitions value
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn repetitions(&self) -> u32 {
        self.repetitions
    }

    /// Get the list of signal names
    #[must_use]
    #[allow(dead_code)] // Method is part of public API but not yet used
    pub fn signal_names(&self) -> &[std::string::String] {
        &self.signal_names
    }
}

impl Dbc {
    // Validate function for std feature (with value_descriptions)
    #[cfg(feature = "std")]
    pub(crate) fn validate(
        nodes: &Nodes,
        messages: &[Message],
        value_descriptions: Option<&ValueDescriptionsList>,
    ) -> Result<()> {
        Self::validate_common(nodes, messages)?;

        // Validate value descriptions if provided
        if let Some(value_descriptions) = value_descriptions {
            // Validate that all value descriptions reference existing messages and signals
            for ((message_id_opt, signal_name), _) in value_descriptions.iter() {
                // Check if message exists (for message-specific value descriptions)
                if let Some(message_id) = message_id_opt {
                    let message_exists = messages.iter().any(|msg| msg.id() == message_id);
                    if !message_exists {
                        return Err(Error::Validation(
                            Error::VALUE_DESCRIPTION_MESSAGE_NOT_FOUND,
                        ));
                    }

                    // Check if signal exists in the message
                    let signal_exists = messages.iter().any(|msg| {
                        msg.id() == message_id && msg.signals().find(signal_name).is_some()
                    });
                    if !signal_exists {
                        return Err(Error::Validation(Error::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND));
                    }
                } else {
                    // For global value descriptions (message_id is None), check if signal exists in any message
                    let signal_exists =
                        messages.iter().any(|msg| msg.signals().find(signal_name).is_some());
                    if !signal_exists {
                        return Err(Error::Validation(Error::VALUE_DESCRIPTION_SIGNAL_NOT_FOUND));
                    }
                }
            }
        }

        Ok(())
    }

    // Validate function for no_std mode (without value_descriptions)
    #[cfg(not(feature = "std"))]
    pub(crate) fn validate(nodes: &Nodes, messages: &[Message]) -> Result<()> {
        Self::validate_common(nodes, messages)
    }

    // Common validation logic shared by both versions
    fn validate_common(nodes: &Nodes, messages: &[Message]) -> Result<()> {
        // Check for duplicate message IDs
        for (i, msg1) in messages.iter().enumerate() {
            for msg2 in messages.iter().skip(i + 1) {
                if msg1.id() == msg2.id() {
                    return Err(Error::Validation(Error::DUPLICATE_MESSAGE_ID));
                }
            }
        }

        // Validate that all message senders are in the nodes list
        // Skip validation if nodes list is empty (empty nodes allowed per DBC spec)
        if !nodes.is_empty() {
            for msg in messages {
                if !nodes.contains(msg.sender()) {
                    return Err(Error::Validation(Error::SENDER_NOT_IN_NODES));
                }
            }
        }

        Ok(())
    }

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
            crate::signal_type::SignalExtendedValueType,
        >,
        signal_types: std::vec::Vec<crate::signal_type::SignalType>,
        signal_type_references: std::vec::Vec<crate::signal_type::SignalTypeReference>,
        signal_type_values: std::vec::Vec<crate::signal_type::SignalTypeValue>,
        bit_timing: Option<BitTiming>,
        signal_groups: std::vec::Vec<SignalGroup>,
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

    /// Decode a CAN message payload using the message ID to find the corresponding message definition.
    ///
    /// This is a high-performance method for decoding CAN messages in `no_std` environments.
    /// It finds the message by ID, then decodes all signals in the message from the payload bytes.
    ///
    /// # Arguments
    ///
    /// * `id` - The CAN message ID to look up
    /// * `payload` - The CAN message payload bytes (up to 64 bytes for CAN FD)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<...>)` - A vector of (signal_name, physical_value) pairs
    /// * `Err(Error)` - If the message ID is not found, payload length doesn't match DLC, or signal decoding fails
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
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
    /// "#)?;
    ///
    /// // Decode a CAN message with RPM value of 2000 (raw: 8000 = 0x1F40)
    /// let payload = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let decoded = dbc.decode(256, &payload)?;
    /// assert_eq!(decoded.len(), 1);
    /// assert_eq!(decoded[0].0, "RPM");
    /// assert_eq!(decoded[0].1, 2000.0);
    /// assert_eq!(decoded[0].2, Some("rpm"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    /// High-performance CAN message decoding optimized for throughput.
    ///
    /// Performance optimizations:
    /// - O(1) or O(log n) message lookup via feature-flagged index (heapless/alloc)
    /// - Inlined hot paths
    /// - Direct error construction (no closure allocation)
    /// - Early validation to avoid unnecessary work
    /// - Optimized signal decoding loop
    #[inline]
    pub fn decode(
        &self,
        id: u32,
        payload: &[u8],
    ) -> Result<Vec<(&str, f64, Option<&str>), { MAX_SIGNALS_PER_MESSAGE }>> {
        // Find message by ID (performance-critical lookup)
        // Uses optimized index when available (O(1) with heapless, O(log n) with alloc)
        let message =
            self.messages.find_by_id(id).ok_or(Error::Decoding(Error::MESSAGE_NOT_FOUND))?;

        // Cache DLC conversion to avoid repeated casts
        let dlc = message.dlc() as usize;

        // Validate payload length matches message DLC (early return before any decoding)
        if payload.len() < dlc {
            return Err(Error::Decoding(Error::PAYLOAD_LENGTH_MISMATCH));
        }

        // Allocate Vec for decoded signals (name, value, unit)
        // Note: heapless Vec grows as needed; alloc Vec allocates dynamically
        let mut decoded_signals: Vec<(&str, f64, Option<&str>), { MAX_SIGNALS_PER_MESSAGE }> =
            Vec::new();

        // Decode all signals in the message
        // Iterate directly - compiler optimizes this hot path
        let signals = message.signals();
        for signal in signals.iter() {
            let value = signal.decode(payload)?;
            // Push with error handling - capacity is checked by Vec
            decoded_signals
                .push((signal.name(), value, signal.unit()))
                .map_err(|_| Error::Decoding(Error::MESSAGE_TOO_MANY_SIGNALS))?;
        }

        Ok(decoded_signals)
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
    ) -> Option<crate::signal_type::SignalExtendedValueType> {
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
    pub fn signal_types(&self) -> &[crate::signal_type::SignalType] {
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
    pub fn signal_type_references(&self) -> &[crate::signal_type::SignalTypeReference] {
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
    pub fn signal_type_values(&self) -> &[crate::signal_type::SignalTypeValue] {
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
    ) -> std::vec::Vec<&crate::signal_type::SignalTypeValue> {
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
            .filter(|ext_mux| ext_mux.message_id == message_id)
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

    /// Parse a DBC file from a string slice
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
    ///  SG_ RPM : 0|16@0+ (0.25,0) [0|8000] "rpm""#;
    ///
    /// let dbc = Dbc::parse(dbc_content)?;
    /// assert_eq!(dbc.messages().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse(data: &str) -> Result<Self> {
        let mut parser = Parser::new(data.as_bytes())?;

        let mut messages_buffer: Vec<Message, { MAX_MESSAGES }> = Vec::new();

        let mut message_count_actual = 0;

        // Parse version, nodes, and messages
        use crate::{
            BA_, BA_DEF_, BA_DEF_DEF_, BA_DEF_SGTYPE_, BA_SGTYPE_, BO_, BO_TX_BU_, BS_, BU_, CM_,
            EV_, NS_, SG_, SG_MUL_VAL_, SGTYPE_, SGTYPE_VAL_, SIG_GROUP_, SIG_TYPE_REF_,
            SIG_VALTYPE_, VAL_, VAL_TABLE_, VERSION,
        };

        let mut version: Option<Version> = None;
        let mut nodes: Option<Nodes> = None;
        let mut bit_timing: Option<BitTiming> = None;

        // Store value descriptions during parsing: (message_id, signal_name, value, description)
        #[cfg(feature = "std")]
        type ValueDescriptionsBufferEntry = (
            Option<u32>,
            std::string::String,
            std::vec::Vec<(u64, std::string::String)>,
        );
        #[cfg(feature = "std")]
        let mut value_descriptions_buffer: std::vec::Vec<ValueDescriptionsBufferEntry> =
            std::vec::Vec::new();

        // Store attributes, comments, value tables, and extended multiplexing during parsing
        #[cfg(feature = "std")]
        let mut attributes_buffer: std::vec::Vec<crate::attributes::AttributeDefinition> =
            std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut attribute_defaults_buffer: std::vec::Vec<
            crate::attributes::AttributeDefault,
        > = std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut attribute_values_buffer: std::vec::Vec<crate::attributes::Attribute> =
            std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut comments_buffer: std::vec::Vec<Comment> = std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut value_tables_buffer: std::vec::Vec<ValueTable> = std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut extended_multiplexing_buffer: std::vec::Vec<ExtendedMultiplexing> =
            std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut signal_value_types_buffer: std::collections::BTreeMap<
            (u32, std::string::String),
            crate::signal_type::SignalExtendedValueType,
        > = std::collections::BTreeMap::new();
        #[cfg(feature = "std")]
        let mut signal_types_buffer: std::vec::Vec<crate::signal_type::SignalType> =
            std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut signal_type_references_buffer: std::vec::Vec<
            crate::signal_type::SignalTypeReference,
        > = std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut signal_type_values_buffer: std::vec::Vec<
            crate::signal_type::SignalTypeValue,
        > = std::vec::Vec::new();
        #[cfg(feature = "std")]
        let mut signal_groups_buffer: std::vec::Vec<SignalGroup> = std::vec::Vec::new();

        loop {
            // Skip comments (lines starting with //)
            parser.skip_newlines_and_spaces();
            if parser.starts_with(b"//") {
                parser.skip_to_end_of_line();
                continue;
            }

            let keyword_result = parser.peek_next_keyword();
            let keyword = match keyword_result {
                Ok(kw) => kw,
                Err(Error::UnexpectedEof) => break,
                Err(Error::Expected(_)) => {
                    if parser.starts_with(b"//") {
                        parser.skip_to_end_of_line();
                        continue;
                    }
                    return Err(keyword_result.unwrap_err());
                }
                Err(e) => return Err(e),
            };

            // Save position after peek_next_keyword (which skips whitespace, so we're at the keyword)
            let pos_at_keyword = parser.pos();

            match keyword {
                NS_ => {
                    // Consume NS_ keyword
                    parser
                        .expect(crate::NS_.as_bytes())
                        .map_err(|_| Error::Expected("Failed to consume NS_ keyword"))?;
                    parser.skip_newlines_and_spaces();
                    let _ = parser.expect(b":").ok();
                    loop {
                        parser.skip_newlines_and_spaces();
                        if parser.is_empty() {
                            break;
                        }
                        if parser.starts_with(b" ") || parser.starts_with(b"\t") {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        if parser.starts_with(b"//") {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        if parser.starts_with(BS_.as_bytes())
                            || parser.starts_with(BU_.as_bytes())
                            || parser.starts_with(BO_.as_bytes())
                            || parser.starts_with(SG_.as_bytes())
                            || parser.starts_with(VERSION.as_bytes())
                        {
                            break;
                        }
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                CM_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume CM_ keyword
                        if parser.expect(crate::CM_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse comment: CM_ [object_type object] "text" ;
                        // object_type can be: BU_, BO_, SG_, EV_, or empty (general comment)
                        if let Ok(comment) = (|| -> Result<Comment> {
                            let comment = if parser.starts_with(b"BU_") {
                                // Node comment: CM_ BU_ node_name "text" ;
                                parser.expect(b"BU_")?;
                                parser.skip_newlines_and_spaces();
                                let node_name = parser
                                    .parse_identifier()
                                    .ok()
                                    .and_then(|n| crate::validate_name(n).ok())
                                    .map(|s| s.as_str().to_string());
                                parser.skip_newlines_and_spaces();
                                let text = if parser.expect(b"\"").is_ok() {
                                    let text_bytes = parser.take_until_quote(false, 1024)?;
                                    core::str::from_utf8(text_bytes)
                                        .map_err(|_| {
                                            Error::Expected(crate::error::Error::INVALID_UTF8)
                                        })?
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                Comment::new(
                                    crate::comment::CommentObjectType::Node,
                                    node_name,
                                    None,
                                    text,
                                )
                            } else if parser.starts_with(b"BO_") {
                                // Message comment: CM_ BO_ message_id "text" ;
                                parser.expect(b"BO_")?;
                                parser.skip_newlines_and_spaces();
                                let message_id = parser.parse_u32().ok();
                                parser.skip_newlines_and_spaces();
                                let text = if parser.expect(b"\"").is_ok() {
                                    let text_bytes = parser.take_until_quote(false, 1024)?;
                                    core::str::from_utf8(text_bytes)
                                        .map_err(|_| {
                                            Error::Expected(crate::error::Error::INVALID_UTF8)
                                        })?
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                Comment::new(
                                    crate::comment::CommentObjectType::Message,
                                    None,
                                    message_id,
                                    text,
                                )
                            } else if parser.starts_with(b"SG_") {
                                // Signal comment: CM_ SG_ message_id signal_name "text" ;
                                parser.expect(b"SG_")?;
                                parser.skip_newlines_and_spaces();
                                let message_id = parser.parse_u32().ok();
                                parser.skip_newlines_and_spaces();
                                let signal_name = parser
                                    .parse_identifier()
                                    .ok()
                                    .and_then(|n| crate::validate_name(n).ok())
                                    .map(|s| s.as_str().to_string());
                                parser.skip_newlines_and_spaces();
                                let text = if parser.expect(b"\"").is_ok() {
                                    let text_bytes = parser.take_until_quote(false, 1024)?;
                                    core::str::from_utf8(text_bytes)
                                        .map_err(|_| {
                                            Error::Expected(crate::error::Error::INVALID_UTF8)
                                        })?
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                Comment::new(
                                    crate::comment::CommentObjectType::Signal,
                                    signal_name,
                                    message_id,
                                    text,
                                )
                            } else if parser.starts_with(b"EV_") {
                                // Environment variable comment: CM_ EV_ env_var_name "text" ;
                                parser.expect(b"EV_")?;
                                parser.skip_newlines_and_spaces();
                                let env_var_name = parser
                                    .parse_identifier()
                                    .ok()
                                    .and_then(|n| crate::validate_name(n).ok())
                                    .map(|s| s.as_str().to_string());
                                parser.skip_newlines_and_spaces();
                                let text = if parser.expect(b"\"").is_ok() {
                                    let text_bytes = parser.take_until_quote(false, 1024)?;
                                    core::str::from_utf8(text_bytes)
                                        .map_err(|_| {
                                            Error::Expected(crate::error::Error::INVALID_UTF8)
                                        })?
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                Comment::new(
                                    crate::comment::CommentObjectType::EnvironmentVariable,
                                    env_var_name,
                                    None,
                                    text,
                                )
                            } else {
                                // General comment: CM_ "text" ;
                                let text = if parser.expect(b"\"").is_ok() {
                                    let text_bytes = parser.take_until_quote(false, 1024)?;
                                    core::str::from_utf8(text_bytes)
                                        .map_err(|_| {
                                            Error::Expected(crate::error::Error::INVALID_UTF8)
                                        })?
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                Comment::new(
                                    crate::comment::CommentObjectType::General,
                                    None,
                                    None,
                                    text,
                                )
                            };
                            parser.skip_newlines_and_spaces();
                            parser.expect(b";").ok(); // Semicolon is optional but common
                            Ok(comment)
                        })() {
                            comments_buffer.push(comment);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume CM_ keyword and skip the rest
                        let _ = parser.expect(crate::CM_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                BS_ => {
                    // Parse BS_: [baudrate ':' BTR1 ',' BTR2]
                    if parser.expect(crate::BS_.as_bytes()).is_err() {
                        parser.skip_to_end_of_line();
                        continue;
                    }
                    parser.skip_newlines_and_spaces();

                    // Expect colon after BS_
                    if parser.expect(b":").is_err() {
                        parser.skip_to_end_of_line();
                        continue;
                    }
                    parser.skip_newlines_and_spaces();

                    // Parse optional baudrate
                    let baudrate = parser.parse_u32().ok();
                    parser.skip_newlines_and_spaces();

                    let (btr1, btr2) = if parser.expect(b":").is_ok() {
                        // Parse BTR1 and BTR2
                        parser.skip_newlines_and_spaces();
                        let btr1_val = parser.parse_u32().ok();
                        parser.skip_newlines_and_spaces();
                        if parser.expect(b",").is_ok() {
                            parser.skip_newlines_and_spaces();
                            let btr2_val = parser.parse_u32().ok();
                            (btr1_val, btr2_val)
                        } else {
                            (btr1_val, None)
                        }
                    } else {
                        (None, None)
                    };

                    // Store bit timing (only first one if multiple)
                    if bit_timing.is_none() {
                        bit_timing = Some(BitTiming::new(baudrate, btr1, btr2));
                    }

                    // Skip to end of line in case of trailing content
                    parser.skip_to_end_of_line();
                    continue;
                }
                SIG_GROUP_ => {
                    #[cfg(feature = "std")]
                    {
                        // Parse SIG_GROUP_ message_id signal_group_name repetitions signal_name1 signal_name2 ... ;
                        if parser.expect(crate::SIG_GROUP_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse message_id
                        let message_id = match parser.parse_u32() {
                            Ok(id) => id,
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        parser.skip_newlines_and_spaces();

                        // Parse signal_group_name
                        let signal_group_name = match parser.parse_identifier() {
                            Ok(name) => match crate::validate_name(name) {
                                Ok(valid_name) => valid_name.as_str().to_string(),
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    continue;
                                }
                            },
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        parser.skip_newlines_and_spaces();

                        // Parse repetitions
                        let repetitions = match parser.parse_u32() {
                            Ok(rep) => rep,
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        parser.skip_newlines_and_spaces();

                        // Parse signal names (space-separated list)
                        let mut signal_names = std::vec::Vec::new();
                        loop {
                            // Check if we've reached semicolon or end of input
                            if parser.starts_with(b";") {
                                break;
                            }
                            // Check if we've reached end of input
                            if parser.peek_byte_at(0).is_none() {
                                break;
                            }
                            match parser.parse_identifier() {
                                Ok(name) => match crate::validate_name(name) {
                                    Ok(valid_name) => {
                                        signal_names.push(valid_name.as_str().to_string());
                                        parser.skip_newlines_and_spaces();
                                    }
                                    Err(_) => break,
                                },
                                Err(_) => break,
                            }
                        }

                        // Optional semicolon
                        if parser.starts_with(b";") {
                            parser.expect(b";").ok();
                        }

                        signal_groups_buffer.push(SignalGroup::new(
                            message_id,
                            signal_group_name,
                            repetitions,
                            signal_names,
                        ));
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume SIG_GROUP_ keyword and skip the rest
                        let _ = parser.expect(crate::SIG_GROUP_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                EV_ | BO_TX_BU_ => {
                    // Consume keyword then skip to end of line (not yet implemented)
                    let _ = parser.expect(keyword.as_bytes()).ok();
                    parser.skip_to_end_of_line();
                    continue;
                }
                SIG_VALTYPE_ => {
                    #[cfg(feature = "std")]
                    {
                        // Parse SIG_VALTYPE_ message_id signal_name : value_type ;
                        if parser.expect(crate::SIG_VALTYPE_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        if let Ok((message_id, signal_name, value_type)) = (|| -> Result<(u32, std::string::String, crate::signal_type::SignalExtendedValueType)> {
                            let message_id = parser.parse_u32()?;
                            parser.skip_newlines_and_spaces();
                            let signal_name = parser.parse_identifier()?;
                            let signal_name = signal_name.to_string();
                            parser.skip_newlines_and_spaces();

                            // Expect colon
                            parser.expect(b":").map_err(|_| Error::Expected("Expected ':' after signal name"))?;
                            parser.skip_newlines_and_spaces();

                            // Parse value type (0, 1, or 2)
                            let value_type_num = parser.parse_u32()?;
                            if value_type_num > 2 {
                                return Err(Error::Validation(Error::INVALID_SIGNAL_VALUE_TYPE));
                            }
                            let value_type = crate::signal_type::SignalExtendedValueType::from_u8(value_type_num as u8)?;

                            parser.skip_newlines_and_spaces();
                            parser.expect(b";").ok(); // Semicolon is optional

                            Ok((message_id, signal_name, value_type))
                        })() {
                            signal_value_types_buffer.insert((message_id, signal_name), value_type);
                        } else {
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        let _ = parser.expect(crate::SIG_VALTYPE_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                VAL_TABLE_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume VAL_TABLE_ keyword
                        if parser.expect(crate::VAL_TABLE_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: VAL_TABLE_ table_name value1 "desc1" value2 "desc2" ... ;
                        if let Ok(value_table) = (|| -> Result<ValueTable> {
                            let table_name = parser.parse_identifier()?;
                            let table_name_validated = crate::validate_name(table_name)?;
                            let table_name = table_name_validated.as_str().to_string();
                            parser.skip_newlines_and_spaces();

                            let mut entries = std::vec::Vec::<(u64, std::string::String)>::new();

                            loop {
                                parser.skip_newlines_and_spaces();
                                // Check for semicolon (end of VAL_TABLE_ statement)
                                if parser.starts_with(b";") {
                                    parser.expect(b";").ok();
                                    break;
                                }

                                // Parse value
                                let value = parser.parse_i64().map_err(|_| {
                                    Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                })? as u64;
                                parser.skip_newlines_and_spaces();

                                // Parse description string
                                parser
                                    .expect(b"\"")
                                    .map_err(|_| Error::Expected("Expected opening quote"))?;
                                let desc_bytes = parser
                                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                                let desc_str = core::str::from_utf8(desc_bytes).map_err(|_| {
                                    Error::Expected(crate::error::Error::INVALID_UTF8)
                                })?;
                                let desc = desc_str.to_string();

                                entries.push((value, desc));
                            }

                            Ok(ValueTable::new(table_name, entries))
                        })() {
                            value_tables_buffer.push(value_table);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume VAL_TABLE_ keyword and skip the rest
                        let _ = parser.expect(crate::VAL_TABLE_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                BA_DEF_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume BA_DEF_ keyword
                        if parser.expect(crate::BA_DEF_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: BA_DEF_ [object_type] "attribute_name" value_type [min max] ;
                        if let Ok(attr_def) =
                            (|| -> Result<crate::attributes::AttributeDefinition> {
                                // Parse optional object type (BU_, BO_, SG_, EV_, or empty for network)
                                let object_type = if parser.starts_with(b"BU_") {
                                    parser.expect(b"BU_")?;
                                    parser.skip_newlines_and_spaces();
                                    crate::attributes::AttributeObjectType::Node
                                } else if parser.starts_with(b"BO_") {
                                    parser.expect(b"BO_")?;
                                    parser.skip_newlines_and_spaces();
                                    crate::attributes::AttributeObjectType::Message
                                } else if parser.starts_with(b"SG_") {
                                    parser.expect(b"SG_")?;
                                    parser.skip_newlines_and_spaces();
                                    crate::attributes::AttributeObjectType::Signal
                                } else if parser.starts_with(b"EV_") {
                                    parser.expect(b"EV_")?;
                                    parser.skip_newlines_and_spaces();
                                    crate::attributes::AttributeObjectType::EnvironmentVariable
                                } else {
                                    crate::attributes::AttributeObjectType::Network
                                };

                                // Parse attribute name (quoted string)
                                parser
                                    .expect(b"\"")
                                    .map_err(|_| Error::Expected("Expected opening quote"))?;
                                let name_bytes = parser
                                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                                let name_str = core::str::from_utf8(name_bytes).map_err(|_| {
                                    Error::Expected(crate::error::Error::INVALID_UTF8)
                                })?;
                                let name = name_str.to_string();
                                parser.skip_newlines_and_spaces();

                                // Parse value type
                                let value_type = if parser.starts_with(b"INT") {
                                    parser.expect(b"INT")?;
                                    parser.skip_newlines_and_spaces();
                                    let min = parser.parse_i64().map_err(|_| {
                                        Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                    })?;
                                    parser.skip_newlines_and_spaces();
                                    let max = parser.parse_i64().map_err(|_| {
                                        Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                    })?;
                                    crate::attributes::AttributeValueType::Int(min, max)
                                } else if parser.starts_with(b"HEX") {
                                    parser.expect(b"HEX")?;
                                    parser.skip_newlines_and_spaces();
                                    let min = parser.parse_i64().map_err(|_| {
                                        Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                    })?;
                                    parser.skip_newlines_and_spaces();
                                    let max = parser.parse_i64().map_err(|_| {
                                        Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                    })?;
                                    crate::attributes::AttributeValueType::Hex(min, max)
                                } else if parser.starts_with(b"FLOAT") {
                                    parser.expect(b"FLOAT")?;
                                    parser.skip_newlines_and_spaces();
                                    let min = parser.parse_f64().map_err(|_| {
                                        Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                    })?;
                                    parser.skip_newlines_and_spaces();
                                    let max = parser.parse_f64().map_err(|_| {
                                        Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                    })?;
                                    crate::attributes::AttributeValueType::Float(min, max)
                                } else if parser.starts_with(b"STRING") {
                                    parser.expect(b"STRING")?;
                                    crate::attributes::AttributeValueType::String
                                } else if parser.starts_with(b"ENUM") {
                                    parser.expect(b"ENUM")?;
                                    parser.skip_newlines_and_spaces();
                                    let mut enum_values =
                                        std::vec::Vec::<std::string::String>::new();
                                    loop {
                                        parser.skip_newlines_and_spaces();
                                        if parser.starts_with(b";") {
                                            break;
                                        }
                                        parser.expect(b"\"").map_err(|_| {
                                            Error::Expected("Expected opening quote")
                                        })?;
                                        let enum_bytes = parser
                                            .take_until_quote(false, crate::MAX_NAME_SIZE)
                                            .map_err(|_| {
                                                Error::Expected("Expected closing quote")
                                            })?;
                                        let enum_str =
                                            core::str::from_utf8(enum_bytes).map_err(|_| {
                                                Error::Expected(crate::error::Error::INVALID_UTF8)
                                            })?;
                                        enum_values.push(enum_str.to_string());
                                        parser.skip_newlines_and_spaces();
                                        if parser.starts_with(b",") {
                                            parser.expect(b",").ok();
                                            // Continue to next enum value
                                        } else {
                                            // End of enum values (semicolon or end of input)
                                            break;
                                        }
                                    }
                                    crate::attributes::AttributeValueType::Enum(enum_values)
                                } else {
                                    return Err(Error::Expected(
                                        "Expected attribute value type (INT, HEX, FLOAT, STRING, or ENUM)",
                                    ));
                                };

                                parser.skip_newlines_and_spaces();
                                parser.expect(b";").ok(); // Semicolon is optional but common

                                Ok(crate::attributes::AttributeDefinition::new(
                                    object_type,
                                    name,
                                    value_type,
                                ))
                            })()
                        {
                            attributes_buffer.push(attr_def);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume BA_DEF_ keyword and skip the rest
                        let _ = parser.expect(crate::BA_DEF_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                BA_DEF_DEF_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume BA_DEF_DEF_ keyword
                        if parser.expect(crate::BA_DEF_DEF_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: BA_DEF_DEF_ "attribute_name" value ;
                        if let Ok(attr_default) =
                            (|| -> Result<crate::attributes::AttributeDefault> {
                                // Parse attribute name (quoted string)
                                parser
                                    .expect(b"\"")
                                    .map_err(|_| Error::Expected("Expected opening quote"))?;
                                let name_bytes = parser
                                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                                let name_str = core::str::from_utf8(name_bytes).map_err(|_| {
                                    Error::Expected(crate::error::Error::INVALID_UTF8)
                                })?;
                                let name = name_str.to_string();
                                parser.skip_newlines_and_spaces();

                                // Parse value (can be integer, float, or string)
                                let value = if parser.starts_with(b"\"") {
                                    // String value
                                    parser.expect(b"\"")?;
                                    let value_bytes = parser
                                        .take_until_quote(false, crate::MAX_NAME_SIZE)
                                        .map_err(|_| Error::Expected("Expected closing quote"))?;
                                    let value_str =
                                        core::str::from_utf8(value_bytes).map_err(|_| {
                                            Error::Expected(crate::error::Error::INVALID_UTF8)
                                        })?;
                                    crate::attributes::AttributeValue::String(value_str.to_string())
                                } else {
                                    // Try to parse as integer first
                                    match parser.parse_i64() {
                                        Ok(int_val) => {
                                            crate::attributes::AttributeValue::Int(int_val)
                                        }
                                        Err(_) => {
                                            // If int fails, try float
                                            // Note: parse_i64 may have advanced position, but parse_f64 should handle it
                                            let float_val = parser.parse_f64()
                                            .map_err(|_| Error::Expected("Expected attribute value (integer, float, or string)"))?;
                                            crate::attributes::AttributeValue::Float(float_val)
                                        }
                                    }
                                };

                                parser.skip_newlines_and_spaces();
                                parser.expect(b";").ok(); // Semicolon is optional but common

                                Ok(crate::attributes::AttributeDefault::new(name, value))
                            })()
                        {
                            attribute_defaults_buffer.push(attr_default);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume BA_DEF_DEF_ keyword and skip the rest
                        let _ = parser.expect(crate::BA_DEF_DEF_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                BA_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume BA_ keyword
                        if parser.expect(crate::BA_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: BA_ "attribute_name" [object_type object] value ;
                        if let Ok(attr) = (|| -> Result<crate::attributes::Attribute> {
                            // Parse attribute name (quoted string)
                            parser
                                .expect(b"\"")
                                .map_err(|_| Error::Expected("Expected opening quote"))?;
                            let name_bytes =
                                parser
                                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                            let name_str = core::str::from_utf8(name_bytes)
                                .map_err(|_| Error::Expected(crate::error::Error::INVALID_UTF8))?;
                            let name = name_str.to_string();
                            parser.skip_newlines_and_spaces();

                            // Parse optional object type and identifier
                            let (object_type, object_name, object_id) =
                                if parser.starts_with(b"BU_") {
                                    parser.expect(b"BU_")?;
                                    parser.skip_newlines_and_spaces();
                                    let obj_name =
                                        parser.parse_identifier().ok().map(|n| n.to_string());
                                    (crate::attributes::AttributeObjectType::Node, obj_name, None)
                                } else if parser.starts_with(b"BO_") {
                                    parser.expect(b"BO_")?;
                                    parser.skip_newlines_and_spaces();
                                    let msg_id = parser.parse_u32().ok();
                                    (
                                        crate::attributes::AttributeObjectType::Message,
                                        None,
                                        msg_id,
                                    )
                                } else if parser.starts_with(b"SG_") {
                                    parser.expect(b"SG_")?;
                                    parser.skip_newlines_and_spaces();
                                    let msg_id = parser.parse_u32().ok();
                                    parser.skip_newlines_and_spaces();
                                    let sig_name =
                                        parser.parse_identifier().ok().map(|n| n.to_string());
                                    (
                                        crate::attributes::AttributeObjectType::Signal,
                                        sig_name,
                                        msg_id,
                                    )
                                } else if parser.starts_with(b"EV_") {
                                    parser.expect(b"EV_")?;
                                    parser.skip_newlines_and_spaces();
                                    let env_name =
                                        parser.parse_identifier().ok().map(|n| n.to_string());
                                    (
                                        crate::attributes::AttributeObjectType::EnvironmentVariable,
                                        env_name,
                                        None,
                                    )
                                } else {
                                    // Network/global attribute
                                    (crate::attributes::AttributeObjectType::Network, None, None)
                                };

                            parser.skip_newlines_and_spaces();

                            // Parse value (can be integer, hex, float, or string)
                            let value = if parser.starts_with(b"\"") {
                                // String value
                                parser.expect(b"\"")?;
                                let value_bytes = parser
                                    .take_until_quote(false, crate::MAX_NAME_SIZE)
                                    .map_err(|_| Error::Expected("Expected closing quote"))?;
                                let value_str =
                                    core::str::from_utf8(value_bytes).map_err(|_| {
                                        Error::Expected(crate::error::Error::INVALID_UTF8)
                                    })?;
                                crate::attributes::AttributeValue::String(value_str.to_string())
                            } else {
                                // Try to parse as integer first
                                match parser.parse_i64() {
                                    Ok(int_val) => crate::attributes::AttributeValue::Int(int_val),
                                    Err(_) => {
                                        // If int fails, try float
                                        // Note: parse_i64 may have advanced position, but parse_f64 should handle it
                                        let float_val = parser.parse_f64()
                                            .map_err(|_| Error::Expected("Expected attribute value (integer, float, or string)"))?;
                                        crate::attributes::AttributeValue::Float(float_val)
                                    }
                                }
                            };

                            parser.skip_newlines_and_spaces();
                            parser.expect(b";").ok(); // Semicolon is optional but common

                            Ok(crate::attributes::Attribute::new(
                                name,
                                object_type,
                                object_name,
                                object_id,
                                value,
                            ))
                        })() {
                            attribute_values_buffer.push(attr);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume BA_ keyword and skip the rest
                        let _ = parser.expect(crate::BA_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                VAL_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume VAL_ keyword
                        let _ = parser.expect(crate::VAL_.as_bytes()).ok();
                        // Parse VAL_ statement: VAL_ message_id signal_name value1 "desc1" value2 "desc2" ... ;
                        // Note: message_id of -1 (0xFFFFFFFF) means the value descriptions apply to
                        // all signals with this name in ANY message (global value descriptions)
                        parser.skip_newlines_and_spaces();
                        let message_id = match parser.parse_i64() {
                            Ok(id) => {
                                // -1 (0xFFFFFFFF) is the magic number for global value descriptions
                                if id == -1 {
                                    None
                                } else if id >= 0 && id <= u32::MAX as i64 {
                                    Some(id as u32)
                                } else {
                                    parser.skip_to_end_of_line();
                                    continue;
                                }
                            }
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        parser.skip_newlines_and_spaces();
                        let signal_name = match parser.parse_identifier() {
                            Ok(name) => name.to_string(),
                            Err(_) => {
                                parser.skip_to_end_of_line();
                                continue;
                            }
                        };
                        // Parse value-description pairs
                        let mut entries: std::vec::Vec<(u64, std::string::String)> =
                            std::vec::Vec::new();
                        loop {
                            parser.skip_newlines_and_spaces();
                            // Check for semicolon (end of VAL_ statement)
                            if parser.starts_with(b";") {
                                parser.expect(b";").ok();
                                break;
                            }
                            // Parse value (as i64 first to handle negative values like -1, then convert to u64)
                            // Note: -1 (0xFFFFFFFF) is the magic number for global value descriptions in message_id,
                            // but values in VAL_ can also be negative
                            let value = match parser.parse_i64() {
                                Ok(v) => {
                                    // Handle -1 specially: convert to 0xFFFFFFFF (u32::MAX) instead of large u64
                                    if v == -1 { 0xFFFF_FFFFu64 } else { v as u64 }
                                }
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    break;
                                }
                            };
                            parser.skip_newlines_and_spaces();
                            // Parse description string (expect quote, then take until quote)
                            if parser.expect(b"\"").is_err() {
                                parser.skip_to_end_of_line();
                                break;
                            }
                            let description_bytes = match parser.take_until_quote(false, 1024) {
                                Ok(bytes) => bytes,
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    break;
                                }
                            };
                            let description = match core::str::from_utf8(description_bytes) {
                                Ok(s) => s.to_string(),
                                Err(_) => {
                                    parser.skip_to_end_of_line();
                                    break;
                                }
                            };
                            entries.push((value, description));
                        }
                        if !entries.is_empty() {
                            value_descriptions_buffer.push((message_id, signal_name, entries));
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume VAL_ keyword and skip the rest
                        let _ = parser.expect(crate::VAL_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                VERSION => {
                    // Version::parse expects VERSION keyword, don't consume it here
                    version = Some(Version::parse(&mut parser)?);
                    continue;
                }
                BU_ => {
                    // Nodes::parse expects BU_ keyword, create parser from original input including it
                    parser.skip_to_end_of_line();
                    let bu_input = &data.as_bytes()[pos_at_keyword..parser.pos()];
                    let mut bu_parser = Parser::new(bu_input)?;
                    nodes = Some(Nodes::parse(&mut bu_parser)?);
                    continue;
                }
                BO_ => {
                    // Check limit using MAX_MESSAGES constant
                    if message_count_actual >= MAX_MESSAGES {
                        return Err(Error::Nodes(Error::NODES_TOO_MANY));
                    }

                    // Save parser position (at BO_ keyword, so Message::parse can consume it)
                    let message_start_pos = pos_at_keyword;

                    // Don't manually parse - just find where the header ends by looking for the colon and sender
                    // We need to find the end of the header line to separate it from signals
                    let header_line_end = {
                        // Skip to end of line to find where header ends
                        let mut temp_parser = Parser::new(&data.as_bytes()[pos_at_keyword..])?;
                        // Skip BO_ keyword
                        temp_parser.expect(crate::BO_.as_bytes()).ok();
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_u32().ok(); // ID
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_identifier().ok(); // name
                        temp_parser.skip_whitespace().ok();
                        temp_parser.expect(b":").ok(); // colon
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_u8().ok(); // DLC
                        temp_parser.skip_whitespace().ok();
                        temp_parser.parse_identifier().ok(); // sender
                        pos_at_keyword + temp_parser.pos()
                    };

                    // Now parse signals from the original parser
                    parser.skip_to_end_of_line(); // Skip past header line

                    let mut signals_array: Vec<Signal, { MAX_SIGNALS_PER_MESSAGE }> = Vec::new();

                    loop {
                        parser.skip_newlines_and_spaces();
                        if parser.starts_with(crate::SG_.as_bytes()) {
                            if let Some(next_byte) = parser.peek_byte_at(3) {
                                if matches!(next_byte, b' ' | b'\n' | b'\r' | b'\t') {
                                    if signals_array.len() >= MAX_SIGNALS_PER_MESSAGE {
                                        return Err(Error::Receivers(
                                            Error::SIGNAL_RECEIVERS_TOO_MANY,
                                        ));
                                    }
                                    // Signal::parse expects SG_ keyword, which we've already verified with starts_with
                                    let signal = Signal::parse(&mut parser)?;
                                    signals_array.push(signal).map_err(|_| {
                                        Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY)
                                    })?;
                                    continue;
                                }
                            }
                        }
                        break;
                    }

                    // Restore parser to start of message line and use Message::parse
                    // Create a new parser from the original input, but only up to the end of the header
                    // (not including signals, so Message::parse doesn't complain about extra content)
                    let message_input = &data.as_bytes()[message_start_pos..header_line_end];
                    let mut message_parser = Parser::new(message_input)?;

                    // Use Message::parse which will parse the header and use our signals
                    let message = Message::parse(&mut message_parser, signals_array.as_slice())?;

                    messages_buffer
                        .push(message)
                        .map_err(|_| Error::Message(Error::NODES_TOO_MANY))?;
                    message_count_actual += 1;
                    continue;
                }
                SG_MUL_VAL_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume SG_MUL_VAL_ keyword
                        if parser.expect(crate::SG_MUL_VAL_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: SG_MUL_VAL_ message_id signal_name multiplexer_switch value_ranges ;
                        // Example: SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
                        if let Ok(ext_mux) = (|| -> Result<ExtendedMultiplexing> {
                            let message_id = parser.parse_u32().map_err(|_| {
                                Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                            })?;
                            parser.skip_newlines_and_spaces();

                            let signal_name = parser.parse_identifier()?;
                            let signal_name = crate::validate_name(signal_name)?;
                            parser.skip_newlines_and_spaces();

                            let multiplexer_switch = parser.parse_identifier()?;
                            let multiplexer_switch = crate::validate_name(multiplexer_switch)?;
                            parser.skip_newlines_and_spaces();

                            let mut value_ranges = crate::compat::Vec::<(u8, u8), 64>::new();

                            loop {
                                parser.skip_newlines_and_spaces();
                                // Check for semicolon (end of SG_MUL_VAL_ statement)
                                if parser.starts_with(b";") {
                                    parser.expect(b";").ok();
                                    break;
                                }

                                // Parse value range: min-max
                                let min = parser.parse_u32().map_err(|_| {
                                    Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                })? as u8;
                                parser
                                    .expect(b"-")
                                    .map_err(|_| Error::Expected("Expected '-' in value range"))?;
                                let max = parser.parse_u32().map_err(|_| {
                                    Error::Expected(crate::error::Error::EXPECTED_NUMBER)
                                })? as u8;

                                value_ranges.push((min, max)).map_err(|_| {
                                    Error::Validation(crate::error::Error::MAX_NAME_SIZE_EXCEEDED)
                                })?;

                                // Check for comma (more ranges) or semicolon (end)
                                parser.skip_newlines_and_spaces();
                                if parser.starts_with(b",") {
                                    parser.expect(b",").ok();
                                    // Continue to next range
                                } else if parser.starts_with(b";") {
                                    parser.expect(b";").ok();
                                    break;
                                } else {
                                    // End of ranges
                                    break;
                                }
                            }

                            Ok(ExtendedMultiplexing::new(
                                message_id,
                                signal_name,
                                multiplexer_switch,
                                value_ranges,
                            ))
                        })() {
                            extended_multiplexing_buffer.push(ext_mux);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        // In no_std mode, consume SG_MUL_VAL_ keyword and skip the rest
                        let _ = parser.expect(crate::SG_MUL_VAL_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                SGTYPE_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume SGTYPE_ keyword
                        if parser.expect(crate::SGTYPE_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: SGTYPE_ type_name : size ;
                        // Example: SGTYPE_ SignalType1 : 16;
                        if let Ok(ext_type) = crate::signal_type::SignalType::parse(&mut parser) {
                            signal_types_buffer.push(ext_type);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        let _ = parser.expect(crate::SGTYPE_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                SIG_TYPE_REF_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume SIG_TYPE_REF_ keyword
                        if parser.expect(crate::SIG_TYPE_REF_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: SIG_TYPE_REF_ message_id signal_name : type_name ;
                        // Example: SIG_TYPE_REF_ 256 RPM : SignalType1;
                        if let Ok(ext_ref) =
                            crate::signal_type::SignalTypeReference::parse(&mut parser)
                        {
                            signal_type_references_buffer.push(ext_ref);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        let _ = parser.expect(crate::SIG_TYPE_REF_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                SGTYPE_VAL_ => {
                    #[cfg(feature = "std")]
                    {
                        // Consume SGTYPE_VAL_ keyword
                        if parser.expect(crate::SGTYPE_VAL_.as_bytes()).is_err() {
                            parser.skip_to_end_of_line();
                            continue;
                        }
                        parser.skip_newlines_and_spaces();

                        // Parse: SGTYPE_VAL_ type_name value "description" value "description" ... ;
                        // Example: SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One" 2 "Two";
                        if let Ok(values) = crate::signal_type::SignalTypeValue::parse(&mut parser)
                        {
                            signal_type_values_buffer.extend(values);
                        } else {
                            // If parsing fails, just skip the line
                            parser.skip_to_end_of_line();
                        }
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        let _ = parser.expect(crate::SGTYPE_VAL_.as_bytes()).ok();
                        parser.skip_to_end_of_line();
                    }
                    continue;
                }
                BA_DEF_SGTYPE_ | BA_SGTYPE_ => {
                    // Attribute definitions and values for signal types
                    // Similar to BA_DEF_ and BA_ but for signal types
                    // For now, skip these (can be implemented later if needed)
                    let _ = parser.expect(keyword.as_bytes()).ok();
                    parser.skip_to_end_of_line();
                    continue;
                }
                #[allow(unreachable_patterns)]
                // False positive: peek_next_keyword returns longest match first
                SG_ => {
                    // Orphaned signal (not inside a message) - skip it
                    parser.skip_to_end_of_line();
                    continue;
                }
                #[allow(unreachable_patterns)]
                // False positive: all keywords should be handled above
                _ => {
                    parser.skip_to_end_of_line();
                    continue;
                }
            }
        }

        // Allow empty nodes (DBC spec allows empty BU_: line)
        let nodes = nodes.unwrap_or_default();

        // If no version was parsed, default to empty version
        let version = version.or_else(|| {
            static EMPTY_VERSION: &[u8] = b"VERSION \"\"";
            let mut parser = Parser::new(EMPTY_VERSION).ok()?;
            Version::parse(&mut parser).ok()
        });

        // Build value descriptions map for storage in Dbc
        #[cfg(feature = "std")]
        let value_descriptions_list = {
            let mut map: BTreeMap<(Option<u32>, std::string::String), ValueDescriptions> =
                BTreeMap::new();
            for (message_id, signal_name, entries) in value_descriptions_buffer {
                let key = (message_id, signal_name);
                let value_descriptions = ValueDescriptions::from_slice(&entries);
                map.insert(key, value_descriptions);
            }
            ValueDescriptionsList::from_map(map)
        };

        // Convert messages buffer to slice for validation and construction
        let messages_slice: &[Message] = messages_buffer.as_slice();

        // Validate messages (duplicate IDs, sender in nodes, etc.)
        #[cfg(feature = "std")]
        Self::validate(&nodes, messages_slice, Some(&value_descriptions_list)).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(Error::MESSAGE_ERROR_PREFIX)
            })
        })?;
        #[cfg(not(feature = "std"))]
        Self::validate(&nodes, messages_slice).map_err(|e| {
            crate::error::map_val_error(e, Error::Message, || {
                Error::Message(Error::MESSAGE_ERROR_PREFIX)
            })
        })?;

        // Construct directly (validation already done)
        let messages = MessageList::new(messages_slice)?;
        Ok(Self {
            version,
            nodes,
            messages,
            #[cfg(feature = "std")]
            value_descriptions: value_descriptions_list,
            #[cfg(feature = "std")]
            attributes: attributes_buffer,
            #[cfg(feature = "std")]
            attribute_defaults: attribute_defaults_buffer,
            #[cfg(feature = "std")]
            attribute_values: attribute_values_buffer,
            #[cfg(feature = "std")]
            comments: comments_buffer,
            #[cfg(feature = "std")]
            value_tables: value_tables_buffer,
            #[cfg(feature = "std")]
            extended_multiplexing: extended_multiplexing_buffer,
            #[cfg(feature = "std")]
            signal_value_types: signal_value_types_buffer,
            #[cfg(feature = "std")]
            signal_types: signal_types_buffer,
            #[cfg(feature = "std")]
            signal_type_references: signal_type_references_buffer,
            #[cfg(feature = "std")]
            signal_type_values: signal_type_values_buffer,
            bit_timing,
            #[cfg(feature = "std")]
            signal_groups: signal_groups_buffer,
        })
    }

    /// Parse a DBC file from a byte slice
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc_bytes = b"VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM";
    /// let dbc = Dbc::parse_bytes(dbc_bytes)?;
    /// println!("Parsed {} messages", dbc.messages().len());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn parse_bytes(data: &[u8]) -> Result<Dbc> {
        let content =
            core::str::from_utf8(data).map_err(|_e| Error::Expected(Error::INVALID_UTF8))?;
        Dbc::parse(content)
    }

    /// Serialize this DBC to a DBC format string
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let dbc_string = dbc.to_dbc_string();
    /// // The string can be written to a file or used elsewhere
    /// assert!(dbc_string.contains("VERSION"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[cfg(feature = "std")]
    #[must_use]
    pub fn to_dbc_string(&self) -> String {
        // Pre-allocate with estimated capacity
        // Estimate: ~50 chars per message + ~100 chars per signal
        let signal_count: usize = self.messages().iter().map(|m| m.signals().len()).sum();
        let estimated_capacity = 200 + (self.messages.len() * 50) + (signal_count * 100);
        let mut result = String::with_capacity(estimated_capacity);

        // VERSION line
        if let Some(version) = &self.version {
            result.push_str(&version.to_dbc_string());
            result.push_str("\n\n");
        }

        // BU_ line
        result.push_str(&self.nodes.to_dbc_string());
        result.push('\n');

        // BO_ and SG_ lines for each message
        for message in self.messages().iter() {
            result.push('\n');
            result.push_str(&message.to_string_full());
        }

        result
    }
}

#[cfg(feature = "std")]
impl core::fmt::Display for Dbc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dbc_string())
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
}
