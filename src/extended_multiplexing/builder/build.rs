use super::ExtendedMultiplexingBuilder;
use crate::{
    Error, ExtendedMultiplexing, Result,
    compat::{Vec as CompatVec, validate_name},
};

impl ExtendedMultiplexingBuilder {
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
        let message_id =
            self.message_id.ok_or_else(|| Error::expected("message_id is required"))?;

        let signal_name_str =
            self.signal_name.ok_or_else(|| Error::expected("signal_name is required"))?;
        let signal_name = validate_name(&signal_name_str)
            .map_err(|_| Error::expected(Error::MAX_NAME_SIZE_EXCEEDED))?;

        let multiplexer_switch_str = self
            .multiplexer_switch
            .ok_or_else(|| Error::expected("multiplexer_switch is required"))?;
        let multiplexer_switch = validate_name(&multiplexer_switch_str)
            .map_err(|_| Error::expected(Error::MAX_NAME_SIZE_EXCEEDED))?;

        if self.value_ranges.is_empty() {
            return Err(Error::expected("at least one value range is required"));
        }

        // Convert std::vec::Vec to compat::Vec
        let mut value_ranges: CompatVec<(u64, u64), 64> = CompatVec::new();
        for (min, max) in self.value_ranges {
            value_ranges
                .push((min, max))
                .map_err(|_| Error::expected("too many value ranges (maximum 64)"))?;
        }

        Ok(ExtendedMultiplexing::new(
            message_id,
            signal_name,
            multiplexer_switch,
            value_ranges,
        ))
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
