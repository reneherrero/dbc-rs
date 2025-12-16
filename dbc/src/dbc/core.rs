#[cfg(feature = "std")]
use super::ValueDescriptionsMap;
use crate::{
    Dbc, ExtendedMultiplexing, MAX_SIGNALS_PER_MESSAGE, Nodes, Version, compat::Vec, dbc::Messages,
};

impl Dbc {
    pub(crate) fn new(
        version: Option<Version>,
        nodes: Nodes,
        messages: Messages,
        #[cfg(feature = "std")] value_descriptions: ValueDescriptionsMap,
        extended_multiplexing: Vec<ExtendedMultiplexing, { MAX_SIGNALS_PER_MESSAGE }>,
    ) -> Self {
        // Validation should have been done prior (by builder)
        Self {
            version,
            nodes,
            messages,
            #[cfg(feature = "std")]
            value_descriptions,
            extended_multiplexing,
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
    #[cfg(feature = "std")]
    #[inline]
    #[must_use]
    pub fn value_descriptions(&self) -> &ValueDescriptionsMap {
        &self.value_descriptions
    }

    #[cfg(feature = "std")]
    #[must_use]
    pub fn value_descriptions_for_signal(
        &self,
        message_id: u32,
        signal_name: &str,
    ) -> Option<&crate::value_descriptions::ValueDescriptions> {
        self.value_descriptions.for_signal(message_id, signal_name)
    }

    /// Get extended multiplexing entries for a specific message
    ///
    /// Extended multiplexing (SG_MUL_VAL_) entries define which multiplexer switch values
    /// activate specific multiplexed signals. This method returns all extended multiplexing
    /// entries for the given message ID.
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
    /// let extended = dbc.extended_multiplexing_for_message(500);
    /// assert_eq!(extended.len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn extended_multiplexing_for_message(
        &self,
        message_id: u32,
    ) -> Vec<ExtendedMultiplexing, { MAX_SIGNALS_PER_MESSAGE }> {
        self.extended_multiplexing
            .iter()
            .filter(|ext_mux| ext_mux.message_id() == message_id)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::Dbc;

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
