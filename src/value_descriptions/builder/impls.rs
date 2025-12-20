use super::ValueDescriptionsBuilder;
use crate::MAX_VALUE_DESCRIPTIONS;
use std::vec::Vec;

impl ValueDescriptionsBuilder {
    /// Creates a new `ValueDescriptionsBuilder` with an empty entry list.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ValueDescriptionsBuilder;
    ///
    /// let builder = ValueDescriptionsBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds a value-description pair to the builder.
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value (u64)
    /// * `description` - The human-readable description
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ValueDescriptionsBuilder;
    ///
    /// let builder = ValueDescriptionsBuilder::new()
    ///     .add_entry(0, "Off")
    ///     .add_entry(1, "On");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "builder method returns modified builder"]
    pub fn add_entry(mut self, value: u64, description: impl AsRef<str>) -> Self {
        if self.entries.len() < MAX_VALUE_DESCRIPTIONS {
            self.entries.push((value, description.as_ref().to_string()));
        }
        self
    }
}

impl Default for ValueDescriptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_add_entry() {
        let vd = ValueDescriptionsBuilder::new()
            .add_entry(0, "Off")
            .add_entry(1, "On")
            .build()
            .unwrap();

        assert_eq!(vd.len(), 2);
        assert_eq!(vd.get(0), Some("Off"));
        assert_eq!(vd.get(1), Some("On"));
    }

    #[test]
    fn test_builder_multiple_entries() {
        let vd = ValueDescriptionsBuilder::new()
            .add_entry(0, "Park")
            .add_entry(1, "Reverse")
            .add_entry(2, "Neutral")
            .add_entry(3, "Drive")
            .add_entry(4, "Low")
            .build()
            .unwrap();

        assert_eq!(vd.len(), 5);
        assert_eq!(vd.get(0), Some("Park"));
        assert_eq!(vd.get(4), Some("Low"));
    }

    #[test]
    fn test_builder_non_sequential_values() {
        let vd = ValueDescriptionsBuilder::new()
            .add_entry(0, "Zero")
            .add_entry(10, "Ten")
            .add_entry(100, "Hundred")
            .build()
            .unwrap();

        assert_eq!(vd.len(), 3);
        assert_eq!(vd.get(0), Some("Zero"));
        assert_eq!(vd.get(10), Some("Ten"));
        assert_eq!(vd.get(100), Some("Hundred"));
        assert_eq!(vd.get(5), None);
    }

    #[test]
    fn test_builder_large_values() {
        let vd = ValueDescriptionsBuilder::new()
            .add_entry(u64::MAX, "Max")
            .add_entry(0, "Min")
            .build()
            .unwrap();

        assert_eq!(vd.len(), 2);
        assert_eq!(vd.get(u64::MAX), Some("Max"));
        assert_eq!(vd.get(0), Some("Min"));
    }
}
