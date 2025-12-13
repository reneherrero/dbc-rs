use crate::compat::Vec;
use crate::error::lang;
use crate::{Error, MAX_MESSAGES, Message, Result};

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::OnceLock;

/// Encapsulates the messages array and count for a DBC
///
/// Uses `Vec<Message>` for dynamic sizing.
/// With the `std` feature, provides O(1) lookup via lazy-initialized HashMap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageList {
    messages: Vec<Message, { MAX_MESSAGES }>,
    #[cfg(feature = "std")]
    id_index: OnceLock<HashMap<u32, usize>>,
}

#[cfg(feature = "std")]
impl std::hash::Hash for MessageList {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash only the messages, not the lazy-initialized index
        self.messages.hash(state);
    }
}

#[cfg(not(feature = "std"))]
impl core::hash::Hash for MessageList {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        // Hash only the messages
        self.messages.hash(state);
    }
}

impl MessageList {
    /// Create MessageList from a slice of messages by cloning them
    pub(crate) fn new(messages: &[Message]) -> Result<Self> {
        if let Some(err) = crate::check_max_limit(
            messages.len(),
            MAX_MESSAGES,
            Error::Message(lang::NODES_TOO_MANY),
        ) {
            return Err(err);
        }
        let messages_vec: Vec<Message, { MAX_MESSAGES }> = messages.iter().cloned().collect();
        Ok(Self {
            messages: messages_vec,
            #[cfg(feature = "std")]
            id_index: OnceLock::new(),
        })
    }

    #[cfg(feature = "std")]
    /// Build or retrieve the ID-to-index HashMap (lazy initialization)
    fn id_index(&self) -> &HashMap<u32, usize> {
        self.id_index.get_or_init(|| {
            let mut map = HashMap::with_capacity(self.messages.len());
            for (index, message) in self.messages.iter().enumerate() {
                map.insert(message.id(), index);
            }
            map
        })
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Message> {
        self.iter().find(|m| m.name() == name)
    }

    /// Find a message by CAN ID, or None if not found
    ///
    /// **Performance:** With the `std` feature, this uses O(1) HashMap lookup.
    /// Without `std`, falls back to O(n) linear search.
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
    #[must_use]
    pub fn find_by_id(&self, id: u32) -> Option<&Message> {
        #[cfg(feature = "std")]
        {
            // O(1) lookup using HashMap
            self.id_index()
                .get(&id)
                .and_then(|&index| self.messages.get(index))
        }
        #[cfg(not(feature = "std"))]
        {
            // O(n) linear search fallback for no_std
            self.iter().find(|m| m.id() == id)
        }
    }
}
