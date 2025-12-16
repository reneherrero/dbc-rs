use crate::{
    Error, ExtendedMultiplexing, Result,
    compat::{Vec as CompatVec, validate_name},
};

/// Builder for creating `ExtendedMultiplexing` programmatically.
///
/// This builder allows you to construct extended multiplexing entries when building DBC files
/// programmatically. Extended multiplexing (SG_MUL_VAL_) entries define which multiplexer
/// switch values activate specific multiplexed signals.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::ExtendedMultiplexingBuilder;
///
/// // Build an extended multiplexing entry
/// let ext_mux = ExtendedMultiplexingBuilder::new()
///     .message_id(500)
///     .signal_name("Signal_A")
///     .multiplexer_switch("Mux1")
///     .add_value_range(0, 5)
///     .add_value_range(10, 15)
///     .build()?;
///
/// assert_eq!(ext_mux.message_id(), 500);
/// assert_eq!(ext_mux.signal_name(), "Signal_A");
/// assert_eq!(ext_mux.multiplexer_switch(), "Mux1");
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug, Clone)]
pub struct ExtendedMultiplexingBuilder {
    message_id: Option<u32>,
    signal_name: Option<String>,
    multiplexer_switch: Option<String>,
    value_ranges: std::vec::Vec<(u64, u64)>,
}

impl ExtendedMultiplexingBuilder {
    /// Creates a new `ExtendedMultiplexingBuilder` with no fields set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// let builder = ExtendedMultiplexingBuilder::new();
    /// // Must set message_id, signal_name, multiplexer_switch, and at least one value range before building
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new() -> Self {
        Self {
            message_id: None,
            signal_name: None,
            multiplexer_switch: None,
            value_ranges: std::vec::Vec::new(),
        }
    }

    /// Sets the message ID.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The CAN message ID this extended multiplexing entry applies to
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// let builder = ExtendedMultiplexingBuilder::new()
    ///     .message_id(500);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn message_id(mut self, message_id: u32) -> Self {
        self.message_id = Some(message_id);
        self
    }

    /// Sets the signal name.
    ///
    /// # Arguments
    ///
    /// * `signal_name` - The name of the multiplexed signal
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// let builder = ExtendedMultiplexingBuilder::new()
    ///     .signal_name("Signal_A");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn signal_name(mut self, signal_name: impl AsRef<str>) -> Self {
        self.signal_name = Some(signal_name.as_ref().to_string());
        self
    }

    /// Sets the multiplexer switch name.
    ///
    /// # Arguments
    ///
    /// * `multiplexer_switch` - The name of the multiplexer switch signal
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// let builder = ExtendedMultiplexingBuilder::new()
    ///     .multiplexer_switch("Mux1");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn multiplexer_switch(mut self, multiplexer_switch: impl AsRef<str>) -> Self {
        self.multiplexer_switch = Some(multiplexer_switch.as_ref().to_string());
        self
    }

    /// Adds a value range to the extended multiplexing entry.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum switch value (inclusive)
    /// * `max` - The maximum switch value (inclusive)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// let builder = ExtendedMultiplexingBuilder::new()
    ///     .add_value_range(0, 5)
    ///     .add_value_range(10, 15);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn add_value_range(mut self, min: u64, max: u64) -> Self {
        self.value_ranges.push((min, max));
        self
    }

    /// Builds the `ExtendedMultiplexing` from the builder configuration.
    ///
    /// This validates that all required fields have been set and constructs an
    /// `ExtendedMultiplexing` instance.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ExtendedMultiplexing)` if successful, or `Err(Error)` if:
    /// - message_id is not set
    /// - signal_name is not set or invalid
    /// - multiplexer_switch is not set or invalid
    /// - No value ranges have been added
    /// - Any name exceeds MAX_NAME_SIZE
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// let ext_mux = ExtendedMultiplexingBuilder::new()
    ///     .message_id(500)
    ///     .signal_name("Signal_A")
    ///     .multiplexer_switch("Mux1")
    ///     .add_value_range(0, 5)
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// ```rust,no_run
    /// use dbc_rs::ExtendedMultiplexingBuilder;
    ///
    /// // Missing message_id
    /// let result = ExtendedMultiplexingBuilder::new()
    ///     .signal_name("Signal_A")
    ///     .multiplexer_switch("Mux1")
    ///     .add_value_range(0, 5)
    ///     .build();
    /// assert!(result.is_err());
    ///
    /// // Missing value ranges
    /// let result = ExtendedMultiplexingBuilder::new()
    ///     .message_id(500)
    ///     .signal_name("Signal_A")
    ///     .multiplexer_switch("Mux1")
    ///     .build();
    /// assert!(result.is_err());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<ExtendedMultiplexing> {
        let message_id = self.message_id.ok_or(Error::Expected("message_id is required"))?;

        let signal_name_str = self.signal_name.ok_or(Error::Expected("signal_name is required"))?;
        let signal_name = validate_name(&signal_name_str)
            .map_err(|_| Error::Expected(Error::MAX_NAME_SIZE_EXCEEDED))?;

        let multiplexer_switch_str = self
            .multiplexer_switch
            .ok_or(Error::Expected("multiplexer_switch is required"))?;
        let multiplexer_switch = validate_name(&multiplexer_switch_str)
            .map_err(|_| Error::Expected(Error::MAX_NAME_SIZE_EXCEEDED))?;

        if self.value_ranges.is_empty() {
            return Err(Error::Expected("at least one value range is required"));
        }

        // Convert std::vec::Vec to compat::Vec
        let mut value_ranges: CompatVec<(u64, u64), 64> = CompatVec::new();
        for (min, max) in self.value_ranges {
            value_ranges
                .push((min, max))
                .map_err(|_| Error::Expected("too many value ranges (maximum 64)"))?;
        }

        Ok(ExtendedMultiplexing::new(
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        ))
    }
}

impl Default for ExtendedMultiplexingBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::ExtendedMultiplexingBuilder;

    #[test]
    fn test_extended_multiplexing_builder_basic() {
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("Signal_A")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5)
            .build()
            .unwrap();

        assert_eq!(ext_mux.message_id(), 500);
        assert_eq!(ext_mux.signal_name(), "Signal_A");
        assert_eq!(ext_mux.multiplexer_switch(), "Mux1");
        assert_eq!(ext_mux.value_ranges(), &[(0, 5)]);
    }

    #[test]
    fn test_extended_multiplexing_builder_multiple_ranges() {
        let ext_mux = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("Signal_A")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5)
            .add_value_range(10, 15)
            .add_value_range(20, 25)
            .build()
            .unwrap();

        assert_eq!(ext_mux.value_ranges().len(), 3);
        assert_eq!(ext_mux.value_ranges()[0], (0, 5));
        assert_eq!(ext_mux.value_ranges()[1], (10, 15));
        assert_eq!(ext_mux.value_ranges()[2], (20, 25));
    }

    #[test]
    fn test_extended_multiplexing_builder_missing_message_id() {
        let result = ExtendedMultiplexingBuilder::new()
            .signal_name("Signal_A")
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_extended_multiplexing_builder_missing_signal_name() {
        let result = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .multiplexer_switch("Mux1")
            .add_value_range(0, 5)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_extended_multiplexing_builder_missing_multiplexer_switch() {
        let result = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("Signal_A")
            .add_value_range(0, 5)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_extended_multiplexing_builder_no_value_ranges() {
        let result = ExtendedMultiplexingBuilder::new()
            .message_id(500)
            .signal_name("Signal_A")
            .multiplexer_switch("Mux1")
            .build();
        assert!(result.is_err());
    }
}
