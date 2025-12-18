use super::{ExtMuxIndex, ExtendedMultiplexings, Messages, ValueDescriptionsMap};
use crate::{Dbc, ExtendedMultiplexing, Nodes, ValueDescriptions, Version};

impl Dbc {
    pub(crate) fn new(
        version: Option<Version>,
        nodes: Nodes,
        messages: Messages,
        value_descriptions: ValueDescriptionsMap,
        extended_multiplexing: ExtendedMultiplexings,
    ) -> Self {
        // Build index for fast extended multiplexing lookup
        let ext_mux_index = ExtMuxIndex::build(extended_multiplexing.as_slice());

        // Validation should have been done prior (by builder)
        Self {
            version,
            nodes,
            messages,
            value_descriptions,
            extended_multiplexing,
            ext_mux_index,
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
