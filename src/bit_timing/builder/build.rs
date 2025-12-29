use super::BitTimingBuilder;
use crate::{BitTiming, Result};

impl BitTimingBuilder {
    /// Builds the `BitTiming` from this builder.
    ///
    /// # Returns
    ///
    /// Returns `Ok(BitTiming)` with the configured values.
    /// Returns an empty `BitTiming` if no values were set.
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
    ///
    /// assert_eq!(bt.baudrate(), Some(500000));
    /// assert_eq!(bt.btr1(), Some(1));
    /// assert_eq!(bt.btr2(), Some(2));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<BitTiming> {
        match (self.baudrate, self.btr1, self.btr2) {
            (Some(baudrate), Some(btr1), Some(btr2)) => {
                Ok(BitTiming::with_btr(baudrate, btr1, btr2))
            }
            (Some(baudrate), _, _) => Ok(BitTiming::with_baudrate(baudrate)),
            _ => Ok(BitTiming::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BitTimingBuilder;

    #[test]
    fn test_build_empty() {
        let bt = BitTimingBuilder::new().build().unwrap();
        assert!(bt.is_empty());
    }

    #[test]
    fn test_build_with_baudrate() {
        let bt = BitTimingBuilder::new().baudrate(250000).build().unwrap();
        assert_eq!(bt.baudrate(), Some(250000));
        assert_eq!(bt.btr1(), None);
    }

    #[test]
    fn test_build_with_all_values() {
        let bt = BitTimingBuilder::new().baudrate(1000000).btr1(5).btr2(10).build().unwrap();
        assert_eq!(bt.baudrate(), Some(1000000));
        assert_eq!(bt.btr1(), Some(5));
        assert_eq!(bt.btr2(), Some(10));
    }

    #[test]
    fn test_build_btr_without_baudrate_ignored() {
        // BTR values without baudrate result in empty BitTiming
        let bt = BitTimingBuilder::new().btr1(1).btr2(2).build().unwrap();
        assert!(bt.is_empty());
    }

    #[test]
    fn test_build_partial_btr() {
        // Only BTR1 without BTR2 - baudrate only is preserved
        let bt = BitTimingBuilder::new().baudrate(500000).btr1(1).build().unwrap();
        assert_eq!(bt.baudrate(), Some(500000));
        // BTR values are ignored if not both present
        assert_eq!(bt.btr1(), None);
    }
}
