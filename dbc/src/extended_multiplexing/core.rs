use super::ExtendedMultiplexing;
use crate::{MAX_NAME_SIZE, compat::String, compat::Vec};

impl ExtendedMultiplexing {
    #[allow(dead_code)] // Used by builder/parser
    pub(crate) fn new(
        message_id: u32,
        signal_name: String<{ MAX_NAME_SIZE }>,
        multiplexer_switch: String<{ MAX_NAME_SIZE }>,
        value_ranges: Vec<(u64, u64), 64>,
    ) -> Self {
        Self {
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        }
    }

    /// Returns the CAN message ID this extended multiplexing entry applies to.
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
    ///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *
    ///
    /// SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
    /// "#)?;
    ///
    /// let entry = dbc.extended_multiplexing_for_message(500).next().unwrap();
    /// assert_eq!(entry.message_id(), 500);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Returns the name of the signal this extended multiplexing entry controls.
    ///
    /// The signal will only be decoded when the multiplexer switch value falls
    /// within one of the defined value ranges.
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
    ///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *
    ///
    /// SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
    /// "#)?;
    ///
    /// let entry = dbc.extended_multiplexing_for_message(500).next().unwrap();
    /// assert_eq!(entry.signal_name(), "Signal_A");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn signal_name(&self) -> &str {
        self.signal_name.as_str()
    }

    /// Returns the name of the multiplexer switch signal that controls activation.
    ///
    /// The multiplexer switch is a signal marked with `M` in the DBC file. When
    /// decoding, the switch's current value is compared against the value ranges
    /// to determine if the controlled signal should be decoded.
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
    ///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *
    ///
    /// SG_MUL_VAL_ 500 Signal_A Mux1 5-10 ;
    /// "#)?;
    ///
    /// let entry = dbc.extended_multiplexing_for_message(500).next().unwrap();
    /// assert_eq!(entry.multiplexer_switch(), "Mux1");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn multiplexer_switch(&self) -> &str {
        self.multiplexer_switch.as_str()
    }

    /// Returns the value ranges that activate the controlled signal.
    ///
    /// Each tuple `(min, max)` defines an inclusive range. The signal is decoded
    /// when the multiplexer switch value falls within **any** of these ranges.
    ///
    /// For example, `[(0, 5), (10, 15)]` means the signal is active when the
    /// switch value is 0-5 OR 10-15.
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
    ///  SG_ Signal_A m0 : 16|16@1+ (0.1,0) [0|100] "unit" *
    ///
    /// SG_MUL_VAL_ 500 Signal_A Mux1 0-5,10-15 ;
    /// "#)?;
    ///
    /// let entry = dbc.extended_multiplexing_for_message(500).next().unwrap();
    /// let ranges = entry.value_ranges();
    ///
    /// // Check if switch value 3 activates the signal
    /// let switch_value = 3u64;
    /// let is_active = ranges.iter().any(|(min, max)| switch_value >= *min && switch_value <= *max);
    /// assert!(is_active);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn value_ranges(&self) -> &[(u64, u64)] {
        self.value_ranges.as_slice()
    }
}
