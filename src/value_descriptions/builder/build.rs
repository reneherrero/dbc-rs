use super::ValueDescriptionsBuilder;
use crate::{
    Error, MAX_NAME_SIZE, MAX_VALUE_DESCRIPTIONS, Result, ValueDescriptions, compat,
    error::check_max_limit,
};

impl ValueDescriptionsBuilder {
    /// Extracts and validates the required fields from the builder.
    fn extract_fields(&self) -> Result<&[(u64, String)]> {
        if self.entries.is_empty() {
            return Err(Error::Validation(Error::VALUE_DESCRIPTIONS_EMPTY));
        }

        if let Some(err) = check_max_limit(
            self.entries.len(),
            MAX_VALUE_DESCRIPTIONS,
            Error::Decoding(Error::VALUE_DESCRIPTIONS_TOO_MANY),
        ) {
            return Err(err);
        }

        Ok(&self.entries)
    }

    /// Validates the builder configuration without consuming it.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The entry list is empty
    /// - The number of entries exceeds the maximum allowed
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ValueDescriptionsBuilder;
    ///
    /// let builder = ValueDescriptionsBuilder::new()
    ///     .add_entry(0, "Off")
    ///     .add_entry(1, "On")
    ///     .validate()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "validation result should be checked"]
    pub fn validate(self) -> Result<Self> {
        self.extract_fields()?;
        Ok(self)
    }

    /// Builds the `ValueDescriptions` from the builder.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The entry list is empty
    /// - The number of entries exceeds the maximum allowed
    /// - Any description exceeds MAX_NAME_SIZE
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ValueDescriptionsBuilder;
    ///
    /// let value_descriptions = ValueDescriptionsBuilder::new()
    ///     .add_entry(0, "Park")
    ///     .add_entry(1, "Drive")
    ///     .build()?;
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<ValueDescriptions> {
        let entries = self.extract_fields()?;

        // Convert std types to compat types
        let mut compat_entries: compat::Vec<
            (u64, compat::String<{ MAX_NAME_SIZE }>),
            { MAX_VALUE_DESCRIPTIONS },
        > = compat::Vec::new();

        for (value, desc) in entries {
            let compat_desc = compat::String::try_from(desc.as_str())
                .map_err(|_| Error::Validation(Error::MAX_NAME_SIZE_EXCEEDED))?;
            compat_entries
                .push((*value, compat_desc))
                .map_err(|_| Error::Validation(Error::VALUE_DESCRIPTIONS_TOO_MANY))?;
        }

        Ok(ValueDescriptions::new(compat_entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = ValueDescriptionsBuilder::new();
        // Empty builder should fail validation
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_builder_default() {
        let builder = ValueDescriptionsBuilder::default();
        // Empty builder should fail validation
        assert!(builder.build().is_err());
    }

    #[test]
    fn test_builder_empty_fails() {
        let result = ValueDescriptionsBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validate_empty_fails() {
        let result = ValueDescriptionsBuilder::new().validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validate_valid() {
        let result = ValueDescriptionsBuilder::new().add_entry(0, "Off").validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_at_max_limit() {
        let mut builder = ValueDescriptionsBuilder::new();
        for i in 0..MAX_VALUE_DESCRIPTIONS {
            builder = builder.add_entry(i as u64, format!("Value{}", i));
        }

        let vd = builder.build().unwrap();
        assert_eq!(vd.len(), MAX_VALUE_DESCRIPTIONS);
    }

    #[test]
    fn test_builder_ignores_entries_over_limit() {
        let mut builder = ValueDescriptionsBuilder::new();
        // Add exactly MAX_VALUE_DESCRIPTIONS + 1 entries
        for i in 0..=MAX_VALUE_DESCRIPTIONS {
            builder = builder.add_entry(i as u64, format!("Value{}", i));
        }

        // Builder should silently ignore entries over the limit
        let vd = builder.build().unwrap();
        assert_eq!(vd.len(), MAX_VALUE_DESCRIPTIONS);
    }

    #[test]
    fn test_builder_description_name_too_long() {
        // Create a description that exceeds MAX_NAME_SIZE
        let long_desc = "A".repeat(MAX_NAME_SIZE + 1);
        let result = ValueDescriptionsBuilder::new().add_entry(0, long_desc).build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_single_entry() {
        let vd = ValueDescriptionsBuilder::new().add_entry(42, "Answer").build().unwrap();

        assert_eq!(vd.len(), 1);
        assert_eq!(vd.get(42), Some("Answer"));
    }

    #[test]
    fn test_builder_clone() {
        let builder = ValueDescriptionsBuilder::new().add_entry(0, "Off").add_entry(1, "On");

        let cloned = builder.clone();
        let vd1 = builder.build().unwrap();
        let vd2 = cloned.build().unwrap();

        assert_eq!(vd1.len(), vd2.len());
    }
}
