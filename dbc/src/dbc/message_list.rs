use crate::compat::Vec;
use crate::error::lang;
use crate::{Error, MAX_MESSAGES, Message, Result};

/// Encapsulates the messages array and count for a DBC
///
/// Uses `Vec<Message>` for dynamic sizing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageList {
    messages: Vec<Message, { MAX_MESSAGES }>,
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
        self.iter().find(|m| m.id() == id)
    }
}
