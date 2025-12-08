/// Value descriptions for a signal.
///
/// Maps numeric signal values to human-readable text descriptions.
/// For example, a gear position signal might map:
/// - 0 -> "Park"
/// - 1 -> "Reverse"
/// - 2 -> "Neutral"
/// - 3 -> "Drive"
///
/// # Examples
///
/// ```rust,no_run
/// use dbc_rs::Dbc;
///
/// let dbc_content = r#"VERSION "1.0"
///
/// BU_: ECM
///
/// BO_ 100 EngineData : 8 ECM
///  SG_ GearPosition : 0|8@1+ (1,0) [0|5] "" *
///
/// VAL_ 100 GearPosition 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" ;
/// "#;
///
/// let dbc = Dbc::parse(dbc_content)?;
/// let message = dbc.messages().iter().find(|m| m.id() == 100).unwrap();
/// let signal = message.signals().find("GearPosition").unwrap();
///
/// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
///     if let Some(description) = value_descriptions.get(0) {
///         println!("Value 0 means: {}", description);
///     }
/// }
/// # Ok::<(), dbc_rs::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueDescriptions<'a> {
    entries: alloc::boxed::Box<[Option<(u64, &'a str)>]>,
    count: usize,
}

// Maximum value descriptions per signal
// Most signals have 2-10 value descriptions, but some can have more
pub(crate) const MAX_VALUE_DESCRIPTIONS: usize = 64;

impl<'a> ValueDescriptions<'a> {
    /// Create ValueDescriptions from a slice of (value, description) pairs
    pub(crate) fn from_slice(entries: &[(u64, &'a str)]) -> Self {
        use alloc::vec;
        let count = entries.len().min(MAX_VALUE_DESCRIPTIONS);
        let mut vec_entries: alloc::vec::Vec<Option<(u64, &'a str)>> =
            vec![None; MAX_VALUE_DESCRIPTIONS];
        for (i, (value, desc)) in entries.iter().take(count).enumerate() {
            vec_entries[i] = Some((*value, *desc));
        }
        Self {
            entries: vec_entries.into_boxed_slice(),
            count,
        }
    }

    /// Get the description for a numeric value
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     if let Some(desc) = value_descriptions.get(0) {
    ///         println!("Value 0: {}", desc);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use]
    pub fn get(&self, value: u64) -> Option<&'a str> {
        for (v, desc) in self.entries.iter().take(self.count).flatten() {
            if *v == value {
                return Some(desc);
            }
        }
        None
    }

    /// Get the number of value descriptions
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if there are any value descriptions
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Iterate over all value descriptions
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dbc_rs::Dbc;
    /// # let dbc = Dbc::parse(r#"VERSION "1.0"\n\nBU_: ECM\n\nBO_ 100 Engine : 8 ECM\n SG_ Gear : 0|8@1+ (1,0) [0|5] "" *\n\nVAL_ 100 Gear 0 "Park" 1 "Reverse" ;"#)?;
    /// # let message = dbc.messages().iter().next().unwrap();
    /// # let signal = message.signals().iter().next().unwrap();
    /// if let Some(value_descriptions) = dbc.value_descriptions_for_signal(message.id(), signal.name()) {
    ///     for (value, description) in value_descriptions.iter() {
    ///         println!("{} -> {}", value, description);
    ///     }
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    pub fn iter(&self) -> ValueDescriptionsIter<'_> {
        ValueDescriptionsIter {
            entries: &self.entries,
            count: self.count,
            pos: 0,
        }
    }
}

/// Iterator over value descriptions
pub struct ValueDescriptionsIter<'a> {
    entries: &'a [Option<(u64, &'a str)>],
    count: usize,
    pos: usize,
}

impl<'a> Iterator for ValueDescriptionsIter<'a> {
    type Item = (u64, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.count {
            if let Some(entry) = &self.entries[self.pos] {
                let result = *entry;
                self.pos += 1;
                return Some(result);
            }
            self.pos += 1;
        }
        None
    }
}
