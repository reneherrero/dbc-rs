use crate::{Error, MAX_MESSAGES, Message, Result, compat::Vec};
#[cfg(feature = "heapless")]
use heapless::index_map::FnvIndexMap;

/// Encapsulates the messages array and count for a DBC
///
/// Uses `Vec<Message>` for dynamic sizing.
/// Optionally includes an index for O(1) or O(log n) message lookup by ID.
#[derive(Debug, Clone, PartialEq)]
pub struct Messages {
    messages: Vec<Message, { MAX_MESSAGES }>,
    // Optional index for fast ID lookup (feature-flagged)
    #[cfg(feature = "heapless")]
    id_index: Option<FnvIndexMap<u32, usize, { MAX_MESSAGES }>>,
    #[cfg(all(feature = "alloc", not(feature = "heapless")))]
    sorted_indices: Option<alloc::vec::Vec<(u32, usize)>>, // (id, index) pairs sorted by id
}

impl Messages {
    /// Create Messages from a slice of messages by cloning them
    #[allow(dead_code)]
    pub(crate) fn new(messages: &[Message]) -> Result<Self> {
        if let Some(err) = crate::error::check_max_limit(
            messages.len(),
            MAX_MESSAGES,
            Error::message(Error::NODES_TOO_MANY),
        ) {
            return Err(err);
        }
        let messages_vec: Vec<Message, { MAX_MESSAGES }> = messages.iter().cloned().collect();
        Self::from_vec(messages_vec)
    }

    /// Create Messages from an owned Vec (avoids cloning)
    pub(crate) fn from_vec(messages: Vec<Message, { MAX_MESSAGES }>) -> Result<Self> {
        if let Some(err) = crate::error::check_max_limit(
            messages.len(),
            MAX_MESSAGES,
            Error::message(Error::NODES_TOO_MANY),
        ) {
            return Err(err);
        }

        // Build index for fast lookup (if features allow)
        #[cfg(feature = "heapless")]
        let id_index = Self::build_heapless_index(&messages);
        #[cfg(all(feature = "alloc", not(feature = "heapless")))]
        let sorted_indices = Self::build_sorted_index(&messages);

        Ok(Self {
            messages,
            #[cfg(feature = "heapless")]
            id_index,
            #[cfg(all(feature = "alloc", not(feature = "heapless")))]
            sorted_indices,
        })
    }

    /// Build heapless index for O(1) lookup (only with heapless feature)
    #[cfg(feature = "heapless")]
    fn build_heapless_index(
        messages: &[Message],
    ) -> Option<FnvIndexMap<u32, usize, { MAX_MESSAGES }>> {
        let mut index = FnvIndexMap::new();
        for (idx, msg) in messages.iter().enumerate() {
            if index.insert(msg.id_with_flag(), idx).is_err() {
                // If we can't insert (capacity full or duplicate), return None
                // This should not happen in practice if MAX_MESSAGES is correct
                return None;
            }
        }
        Some(index)
    }

    /// Build sorted index for O(log n) lookup (only with alloc feature, no heapless)
    #[cfg(all(feature = "alloc", not(feature = "heapless")))]
    fn build_sorted_index(messages: &[Message]) -> Option<alloc::vec::Vec<(u32, usize)>> {
        let mut indices = alloc::vec::Vec::with_capacity(messages.len());
        for (idx, msg) in messages.iter().enumerate() {
            indices.push((msg.id_with_flag(), idx));
        }
        // Sort by message ID for binary search
        indices.sort_by_key(|&(id, _)| id);
        Some(indices)
    }

    /// Get an iterator over the messages
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// let mut iter = dbc.messages().iter();
    /// let message = iter.next().unwrap();
    /// assert_eq!(message.name(), "Engine");
    /// assert_eq!(message.id(), 256);
    /// assert!(iter.next().is_none());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = &Message> + '_ {
        self.messages.iter()
    }

    /// Get the number of messages
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// assert_eq!(dbc.messages().len(), 1);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Returns `true` if there are no messages
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM")?;
    /// assert!(dbc.messages().is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a message by index, or None if index is out of bounds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(message) = dbc.messages().at(0) {
    ///     assert_eq!(message.name(), "Engine");
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn at(&self, index: usize) -> Option<&Message> {
        self.messages.get(index)
    }

    /// Find a message by name, or None if not found
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(message) = dbc.messages().find("Engine") {
    ///     assert_eq!(message.name(), "Engine");
    ///     assert_eq!(message.id(), 256);
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[must_use = "return value should be used"]
    pub fn find(&self, name: &str) -> Option<&Message> {
        self.iter().find(|m| m.name() == name)
    }

    /// Find a message by CAN ID, or None if not found
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse("VERSION \"1.0\"\n\nBU_: ECM\n\nBO_ 256 Engine : 8 ECM")?;
    /// if let Some(message) = dbc.messages().find_by_id(256) {
    ///     assert_eq!(message.name(), "Engine");
    ///     assert_eq!(message.id(), 256);
    /// }
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    /// Find a message by CAN ID with optimized lookup based on available features.
    ///
    /// - With `heapless` feature: O(1) lookup using FnvIndexMap
    /// - With `alloc` feature (no heapless): O(log n) lookup using binary search on sorted indices
    /// - Otherwise: O(n) linear search
    #[inline]
    #[must_use = "return value should be used"]
    pub fn find_by_id(&self, id: u32) -> Option<&Message> {
        #[cfg(feature = "heapless")]
        {
            if let Some(ref index) = self.id_index {
                if let Some(&idx) = index.get(&id) {
                    return self.messages.get(idx);
                }
                return None;
            }
        }

        #[cfg(all(feature = "alloc", not(feature = "heapless")))]
        {
            if let Some(ref sorted) = self.sorted_indices {
                // Binary search for O(log n) lookup
                if let Ok(pos) = sorted.binary_search_by_key(&id, |&(msg_id, _)| msg_id) {
                    let &(_, idx) = sorted.get(pos)?;
                    return self.messages.get(idx);
                }
                return None;
            }
        }

        // Fallback: linear search O(n)
        // This is used when no alloc/heapless features are enabled
        self.messages.iter().find(|m| m.id_with_flag() == id)
    }
}
