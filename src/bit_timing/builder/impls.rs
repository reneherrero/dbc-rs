use super::BitTimingBuilder;

impl BitTimingBuilder {
    /// Creates a new `BitTimingBuilder` with no values set.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::BitTimingBuilder;
    ///
    /// let builder = BitTimingBuilder::new();
    /// let bt = builder.build()?;
    /// assert!(bt.is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn new() -> Self {
        Self {
            baudrate: None,
            btr1: None,
            btr2: None,
        }
    }

    /// Sets the baudrate in bits per second.
    ///
    /// # Arguments
    ///
    /// * `baudrate` - The CAN bus baudrate (e.g., 500000 for 500 kbps)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::BitTimingBuilder;
    ///
    /// let bt = BitTimingBuilder::new()
    ///     .baudrate(500000)
    ///     .build()?;
    /// assert_eq!(bt.baudrate(), Some(500000));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn baudrate(mut self, baudrate: u32) -> Self {
        self.baudrate = Some(baudrate);
        self
    }

    /// Sets the Bus Timing Register 1 (BTR1) value.
    ///
    /// BTR1 is typically only meaningful when baudrate is also set.
    ///
    /// # Arguments
    ///
    /// * `btr1` - The BTR1 register value
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::BitTimingBuilder;
    ///
    /// let bt = BitTimingBuilder::new()
    ///     .baudrate(500000)
    ///     .btr1(1)
    ///     .build()?;
    /// assert_eq!(bt.btr1(), Some(1));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn btr1(mut self, btr1: u32) -> Self {
        self.btr1 = Some(btr1);
        self
    }

    /// Sets the Bus Timing Register 2 (BTR2) value.
    ///
    /// BTR2 is typically only meaningful when baudrate and BTR1 are also set.
    ///
    /// # Arguments
    ///
    /// * `btr2` - The BTR2 register value
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::BitTimingBuilder;
    ///
    /// let bt = BitTimingBuilder::new()
    ///     .baudrate(500000)
    ///     .btr1(1)
    ///     .btr2(2)
    ///     .build()?;
    /// assert_eq!(bt.btr2(), Some(2));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn btr2(mut self, btr2: u32) -> Self {
        self.btr2 = Some(btr2);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::BitTimingBuilder;

    #[test]
    fn test_bit_timing_builder_empty() {
        let bt = BitTimingBuilder::new().build().unwrap();
        assert!(bt.is_empty());
        assert_eq!(bt.baudrate(), None);
        assert_eq!(bt.btr1(), None);
        assert_eq!(bt.btr2(), None);
    }

    #[test]
    fn test_bit_timing_builder_baudrate_only() {
        let bt = BitTimingBuilder::new().baudrate(500000).build().unwrap();
        assert!(!bt.is_empty());
        assert_eq!(bt.baudrate(), Some(500000));
        assert_eq!(bt.btr1(), None);
        assert_eq!(bt.btr2(), None);
    }

    #[test]
    fn test_bit_timing_builder_full() {
        let bt = BitTimingBuilder::new().baudrate(500000).btr1(1).btr2(2).build().unwrap();
        assert!(!bt.is_empty());
        assert_eq!(bt.baudrate(), Some(500000));
        assert_eq!(bt.btr1(), Some(1));
        assert_eq!(bt.btr2(), Some(2));
    }

    #[test]
    fn test_bit_timing_builder_default() {
        let bt = BitTimingBuilder::default().build().unwrap();
        assert!(bt.is_empty());
    }
}

impl Default for BitTimingBuilder {
    fn default() -> Self {
        Self::new()
    }
}
