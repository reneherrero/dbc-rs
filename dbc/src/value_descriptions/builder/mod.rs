use crate::{Error, MAX_VALUE_DESCRIPTIONS, Result, ValueDescriptions, error::check_max_limit};

/// Builder for creating `ValueDescriptions` programmatically.
///
/// This builder allows you to construct value descriptions when building DBC files
/// programmatically. It validates that entries are within limits.
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::ValueDescriptionsBuilder;
///
/// let value_descriptions = ValueDescriptionsBuilder::new()
///     .add_entry(0, "Park")
///     .add_entry(1, "Reverse")
///     .add_entry(2, "Neutral")
///     .add_entry(3, "Drive")
///     .build()?;
///
/// assert_eq!(value_descriptions.get(0), Some("Park"));
/// assert_eq!(value_descriptions.get(1), Some("Reverse"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # Validation
///
/// The builder validates:
/// - Maximum of 64 value descriptions (MAX_VALUE_DESCRIPTIONS)
///
/// # Feature Requirements
///
/// This builder requires the `std` feature to be enabled.
#[derive(Debug)]
pub struct ValueDescriptionsBuilder {
    entries: Vec<(u64, String)>,
}

impl ValueDescriptionsBuilder {
    /// Creates a new `ValueDescriptionsBuilder` with an empty entry list.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::ValueDescriptionsBuilder;
    ///
    /// let builder = ValueDescriptionsBuilder::new();
    /// let value_descriptions = builder.build()?;
    /// assert!(value_descriptions.is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
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
    #[must_use]
    pub fn add_entry(mut self, value: u64, description: impl AsRef<str>) -> Self {
        if self.entries.len() < MAX_VALUE_DESCRIPTIONS {
            self.entries.push((value, description.as_ref().to_string()));
        }
        self
    }

    /// Builds the `ValueDescriptions` from the builder.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of entries exceeds the maximum allowed.
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
        if let Some(err) = check_max_limit(
            self.entries.len(),
            MAX_VALUE_DESCRIPTIONS,
            Error::Decoding(Error::VALUE_DESCRIPTIONS_TOO_MANY),
        ) {
            return Err(err);
        }

        // Use Cow::Owned for owned strings (no leak needed)
        let cow_entries: Vec<(u64, String)> = self.entries.into_iter().collect();

        Ok(ValueDescriptions::from_slice(&cow_entries))
    }
}

impl Default for ValueDescriptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
