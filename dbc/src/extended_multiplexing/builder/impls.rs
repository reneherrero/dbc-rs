use super::ExtendedMultiplexingBuilder;
use std::vec::Vec;

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
            value_ranges: Vec::new(),
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
    #[must_use = "builder method returns modified builder"]
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
    #[must_use = "builder method returns modified builder"]
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
    #[must_use = "builder method returns modified builder"]
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
    #[must_use = "builder method returns modified builder"]
    pub fn add_value_range(mut self, min: u64, max: u64) -> Self {
        self.value_ranges.push((min, max));
        self
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
}
