#[cfg(feature = "attributes")]
use super::{AttributeDefaultsMap, AttributeDefinitionsMap, AttributeValuesMap};
use super::{ExtMuxIndex, ExtendedMultiplexings, Messages, ValueDescriptionsMap};
#[cfg(feature = "attributes")]
use crate::{AttributeDefinition, AttributeValue};
use crate::{
    BitTiming, Dbc, ExtendedMultiplexing, Nodes, ValueDescriptions, Version, compat::Comment,
};

impl Dbc {
    #[cfg(feature = "attributes")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        version: Option<Version>,
        bit_timing: Option<BitTiming>,
        nodes: Nodes,
        messages: Messages,
        value_descriptions: ValueDescriptionsMap,
        extended_multiplexing: ExtendedMultiplexings,
        comment: Option<Comment>,
        attribute_definitions: AttributeDefinitionsMap,
        attribute_defaults: AttributeDefaultsMap,
        attribute_values: AttributeValuesMap,
    ) -> Self {
        // Build index for fast extended multiplexing lookup
        let ext_mux_index = ExtMuxIndex::build(extended_multiplexing.as_slice());

        Self {
            version,
            bit_timing,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            ext_mux_index,
            comment,
            attribute_definitions,
            attribute_defaults,
            attribute_values,
        }
    }

    #[cfg(not(feature = "attributes"))]
    pub(crate) fn new(
        version: Option<Version>,
        bit_timing: Option<BitTiming>,
        nodes: Nodes,
        messages: Messages,
        value_descriptions: ValueDescriptionsMap,
        extended_multiplexing: ExtendedMultiplexings,
        comment: Option<Comment>,
    ) -> Self {
        // Build index for fast extended multiplexing lookup
        let ext_mux_index = ExtMuxIndex::build(extended_multiplexing.as_slice());

        Self {
            version,
            bit_timing,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            ext_mux_index,
            comment,
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
    #[must_use = "return value should be used"]
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    /// Get the bit timing configuration
    ///
    /// The BS_ section in DBC files specifies CAN bus timing parameters.
    /// Returns `None` if the BS_ section was empty or not present.
    /// Returns `Some(&BitTiming)` if timing parameters were specified.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBS_: 500000 : 1,2\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(bit_timing) = dbc.bit_timing() {
    ///     println!("Baudrate: {:?}", bit_timing.baudrate());
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn bit_timing(&self) -> Option<&BitTiming> {
        self.bit_timing.as_ref()
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
    #[must_use = "return value should be used"]
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
    #[must_use = "return value should be used"]
    pub fn messages(&self) -> &Messages {
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
    #[inline]
    #[must_use = "return value should be used"]
    pub fn value_descriptions(&self) -> &ValueDescriptionsMap {
        &self.value_descriptions
    }

    #[must_use = "return value should be used"]
    pub fn value_descriptions_for_signal(
        &self,
        message_id: u32,
        signal_name: &str,
    ) -> Option<&ValueDescriptions> {
        self.value_descriptions.for_signal(message_id, signal_name)
    }

    /// Get all extended multiplexing entries
    ///
    /// Returns a reference to all extended multiplexing (SG_MUL_VAL_) entries
    /// in the DBC file.
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
    /// BO_ 500 MuxMessage : 8 ECM
    ///  SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
    ///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] ""
    ///
    /// SG_MUL_VAL_ 500 Signal_A Mux1 0-5 ;
    /// "#)?;
    ///
    /// let ext_mux = dbc.extended_multiplexing();
    /// assert_eq!(ext_mux.len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn extended_multiplexing(&self) -> &[ExtendedMultiplexing] {
        self.extended_multiplexing.as_slice()
    }

    /// Get extended multiplexing entries for a specific message
    ///
    /// Extended multiplexing (SG_MUL_VAL_) entries define which multiplexer switch values
    /// activate specific multiplexed signals. This method returns an iterator over
    /// references to extended multiplexing entries for the given message ID.
    ///
    /// # Performance
    ///
    /// Returns an iterator of references (zero allocation) instead of cloning entries.
    /// This is optimized for the decode hot path where extended multiplexing is checked
    /// on every CAN frame.
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
    /// BO_ 500 ComplexMux : 8 ECM
    ///  SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
    ///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] ""
    ///
    /// SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
    /// "#)?;
    /// let extended: Vec<_> = dbc.extended_multiplexing_for_message(500).collect();
    /// assert_eq!(extended.len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn extended_multiplexing_for_message(
        &self,
        message_id: u32,
    ) -> impl Iterator<Item = &ExtendedMultiplexing> + '_ {
        self.extended_multiplexing
            .iter()
            .filter(move |ext_mux| ext_mux.message_id() == message_id)
    }

    /// Returns the database-level comment from CM_ (general comment), if present.
    ///
    /// This is the general comment for the entire DBC file, not associated with
    /// any specific node, message, or signal.
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
    ///
    /// CM_ "CAN database for powertrain";"#)?;
    /// assert_eq!(dbc.comment(), Some("CAN database for powertrain"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|c| c.as_ref())
    }

    /// Returns the comment for a specific node from CM_ BU_ entry, if present.
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
    ///
    /// CM_ BU_ ECM "Engine Control Module";"#)?;
    /// assert_eq!(dbc.node_comment("ECM"), Some("Engine Control Module"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn node_comment(&self, node_name: &str) -> Option<&str> {
        self.nodes.node_comment(node_name)
    }
}

// ============================================================================
// Attribute Access Methods (feature-gated)
// ============================================================================

#[cfg(feature = "attributes")]
impl Dbc {
    /// Get all attribute definitions (BA_DEF_ entries).
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
    ///
    /// BA_DEF_ BO_ "GenMsgCycleTime" INT 0 10000;"#)?;
    /// assert_eq!(dbc.attribute_definitions().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn attribute_definitions(&self) -> &AttributeDefinitionsMap {
        &self.attribute_definitions
    }

    /// Get an attribute definition by name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn attribute_definition(&self, name: &str) -> Option<&AttributeDefinition> {
        self.attribute_definitions.get(name)
    }

    /// Get all attribute defaults (BA_DEF_DEF_ entries).
    #[inline]
    #[must_use = "return value should be used"]
    pub fn attribute_defaults(&self) -> &AttributeDefaultsMap {
        &self.attribute_defaults
    }

    /// Get the default value for an attribute by name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn attribute_default(&self, name: &str) -> Option<&AttributeValue> {
        self.attribute_defaults.get(name)
    }

    /// Get all attribute values (BA_ entries).
    #[inline]
    #[must_use = "return value should be used"]
    pub fn attribute_values(&self) -> &AttributeValuesMap {
        &self.attribute_values
    }

    /// Get a network-level attribute value by name.
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
    ///
    /// BA_DEF_ "BusType" STRING;
    /// BA_DEF_DEF_ "BusType" "";
    /// BA_ "BusType" "CAN";"#)?;
    /// if let Some(value) = dbc.network_attribute("BusType") {
    ///     assert_eq!(value.as_string(), Some("CAN"));
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn network_attribute(&self, name: &str) -> Option<&AttributeValue> {
        self.attribute_values.get_network(name)
    }

    /// Get a node attribute value by node name and attribute name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn node_attribute(&self, node_name: &str, attr_name: &str) -> Option<&AttributeValue> {
        self.attribute_values.get_node(node_name, attr_name)
    }

    /// Get a message attribute value by message ID and attribute name.
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
    ///
    /// BA_DEF_ BO_ "GenMsgCycleTime" INT 0 10000;
    /// BA_DEF_DEF_ "GenMsgCycleTime" 0;
    /// BA_ "GenMsgCycleTime" BO_ 256 100;"#)?;
    /// if let Some(value) = dbc.message_attribute(256, "GenMsgCycleTime") {
    ///     assert_eq!(value.as_int(), Some(100));
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn message_attribute(&self, message_id: u32, attr_name: &str) -> Option<&AttributeValue> {
        self.attribute_values.get_message(message_id, attr_name)
    }

    /// Get a signal attribute value by message ID, signal name, and attribute name.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn signal_attribute(
        &self,
        message_id: u32,
        signal_name: &str,
        attr_name: &str,
    ) -> Option<&AttributeValue> {
        self.attribute_values.get_signal(message_id, signal_name, attr_name)
    }

    /// Get a network attribute value with fallback to default.
    ///
    /// First checks for a specific value assignment, then falls back to the
    /// attribute's default value if no specific assignment exists.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn network_attribute_or_default(&self, name: &str) -> Option<&AttributeValue> {
        self.network_attribute(name).or_else(|| self.attribute_default(name))
    }

    /// Get a node attribute value with fallback to default.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn node_attribute_or_default(
        &self,
        node_name: &str,
        attr_name: &str,
    ) -> Option<&AttributeValue> {
        self.node_attribute(node_name, attr_name)
            .or_else(|| self.attribute_default(attr_name))
    }

    /// Get a message attribute value with fallback to default.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn message_attribute_or_default(
        &self,
        message_id: u32,
        attr_name: &str,
    ) -> Option<&AttributeValue> {
        self.message_attribute(message_id, attr_name)
            .or_else(|| self.attribute_default(attr_name))
    }

    /// Get a signal attribute value with fallback to default.
    #[inline]
    #[must_use = "return value should be used"]
    pub fn signal_attribute_or_default(
        &self,
        message_id: u32,
        signal_name: &str,
        attr_name: &str,
    ) -> Option<&AttributeValue> {
        self.signal_attribute(message_id, signal_name, attr_name)
            .or_else(|| self.attribute_default(attr_name))
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

    #[test]
    fn test_parse_extended_multiplexing() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 500 ComplexMux : 8 ECM
 SG_ Mux1 M : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *

SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
"#,
        )
        .unwrap();

        let ext_entries: crate::compat::Vec<_, { crate::MAX_EXTENDED_MULTIPLEXING }> =
            dbc.extended_multiplexing_for_message(500).collect();
        assert_eq!(
            ext_entries.len(),
            1,
            "Extended multiplexing entry should be parsed"
        );
        assert_eq!(ext_entries[0].signal_name(), "Signal_A");
        assert_eq!(ext_entries[0].multiplexer_switch(), "Mux1");
        assert_eq!(ext_entries[0].value_ranges(), [(5, 10)]);
    }

    #[test]
    fn test_version() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();
        assert_eq!(dbc.version().map(|v| v.as_str()), Some("1.0"));
    }

    #[test]
    fn test_nodes() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();
        assert_eq!(dbc.nodes().len(), 2);
        assert!(dbc.nodes().contains("ECM"));
        assert!(dbc.nodes().contains("TCM"));
    }

    #[test]
    fn test_messages() {
        let dbc = Dbc::parse(
            r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
"#,
        )
        .unwrap();
        assert_eq!(dbc.messages().len(), 1);
        let message = dbc.messages().at(0).unwrap();
        assert_eq!(message.name(), "Engine");
        assert_eq!(message.id(), 256);
    }
}
