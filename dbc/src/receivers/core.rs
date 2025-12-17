use super::Receivers;
use crate::{
    MAX_NAME_SIZE, MAX_NODES,
    compat::{String, Vec},
};

impl Receivers {
    pub(crate) fn new_none() -> Self {
        Receivers::None
    }

    pub(crate) fn new_nodes(nodes: &[String<{ MAX_NAME_SIZE }>]) -> Self {
        // Validation should have been done prior (by builder or parse)
        // Receivers can have at most MAX_NODES - 1 nodes
        let vec_nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES - 1 }> =
            nodes.iter().take(MAX_NODES - 1).cloned().collect();
        Receivers::Nodes(vec_nodes)
    }

    /// Returns an iterator over the receiver node names.
    ///
    /// For `Receivers::None`, the iterator will be empty.
    /// For `Receivers::Nodes`, it iterates over the specific node names.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "°C" TCM,BCM
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// // Iterate over receiver nodes
    /// let mut iter = signal.receivers().iter();
    /// assert_eq!(iter.next(), Some("TCM"));
    /// assert_eq!(iter.next(), Some("BCM"));
    /// assert_eq!(iter.next(), None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # None Receivers
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" Vector__XXX
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// // None receivers return empty iterator
    /// assert_eq!(signal.receivers().iter().count(), 0);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        match self {
            Receivers::Nodes(nodes) => ReceiversIter {
                nodes: Some(nodes.as_slice()),
                pos: 0,
            },
            _ => ReceiversIter {
                nodes: None,
                pos: 0,
            },
        }
    }

    /// Returns the number of receiver nodes.
    ///
    /// - For `Receivers::Nodes`: Returns the count of specific receiver nodes
    /// - For `Receivers::None`: Returns `0`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "°C" TCM,BCM
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    /// assert_eq!(signal.receivers().len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn len(&self) -> usize {
        match self {
            Receivers::Nodes(nodes) => nodes.len(),
            Receivers::None => 0,
        }
    }

    /// Returns `true` if there are no specific receiver nodes.
    ///
    /// This returns `true` for `Receivers::None` and for `Receivers::Nodes` with
    /// an empty node list.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    /// assert!(signal.receivers().is_empty());
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if a node name is in the receivers list.
    ///
    /// For `Receivers::None`, this always returns `false`.
    /// For `Receivers::Nodes`, it checks if the node name is in the list.
    ///
    /// # Arguments
    ///
    /// * `node` - The node name to check
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "°C" TCM BCM
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// assert!(signal.receivers().contains("TCM"));
    /// assert!(signal.receivers().contains("BCM"));
    /// assert!(!signal.receivers().contains("ECM"));
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn contains(&self, node: &str) -> bool {
        self.iter().any(|n| n == node)
    }

    /// Gets a receiver node by index.
    ///
    /// Returns `None` if:
    /// - The index is out of bounds
    /// - The receiver is `None`
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based index of the receiver node
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "°C" TCM,BCM
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// assert_eq!(signal.receivers().at(0), Some("TCM"));
    /// assert_eq!(signal.receivers().at(1), Some("BCM"));
    /// assert_eq!(signal.receivers().at(2), None);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "return value should be used"]
    pub fn at(&self, index: usize) -> Option<&str> {
        match self {
            Receivers::Nodes(nodes) => nodes.get(index).map(|s| s.as_str()),
            Receivers::None => None,
        }
    }
}

struct ReceiversIter<'a> {
    nodes: Option<&'a [String<{ MAX_NAME_SIZE }>]>,
    pos: usize,
}

impl<'a> Iterator for ReceiversIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(nodes) = self.nodes {
            if self.pos < nodes.len() {
                let result = nodes[self.pos].as_str();
                self.pos += 1;
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Receivers;
    use crate::Parser;

    // Tests for new_none()
    mod test_new_none {
        use super::*;

        #[test]
        fn test_receivers_none() {
            let none = Receivers::new_none();
            assert_eq!(none.len(), 0);
            assert!(none.is_empty());
        }
    }

    // Tests for iter()
    mod test_iter {
        use super::*;

        #[test]
        fn test_receivers_iter() {
            let input = "TCM BCM";
            let mut parser = Parser::new(input.as_bytes()).unwrap();
            let result = Receivers::parse(&mut parser).unwrap();
            let mut iter = result.iter();
            assert_eq!(iter.next(), Some("TCM"));
            assert_eq!(iter.next(), Some("BCM"));
            assert!(iter.next().is_none());

            let none = Receivers::new_none();
            assert_eq!(none.iter().count(), 0);
        }
    }

    // Tests for len()
    mod test_len {
        use super::*;

        #[test]
        fn test_receivers_len() {
            let none = Receivers::new_none();
            assert_eq!(none.len(), 0);

            let input = "TCM BCM";
            let mut parser = Parser::new(input.as_bytes()).unwrap();
            let nodes = Receivers::parse(&mut parser).unwrap();
            assert_eq!(nodes.len(), 2);
        }
    }

    // Tests for is_empty()
    mod test_is_empty {
        use super::*;

        #[test]
        fn test_receivers_is_empty() {
            let none = Receivers::new_none();
            assert!(none.is_empty());

            let input = "TCM";
            let mut parser = Parser::new(input.as_bytes()).unwrap();
            let nodes = Receivers::parse(&mut parser).unwrap();
            assert!(!nodes.is_empty());
        }
    }

    // Tests for contains()
    mod test_contains {
        use super::*;

        #[test]
        fn test_receivers_contains() {
            let input = "TCM BCM";
            let mut parser = Parser::new(input.as_bytes()).unwrap();
            let result = Receivers::parse(&mut parser).unwrap();
            assert!(result.contains("TCM"));
            assert!(result.contains("BCM"));
            assert!(!result.contains("ECM"));

            let none = Receivers::new_none();
            assert!(!none.contains("TCM"));
        }
    }

    // Tests for at()
    mod test_at {
        use super::*;

        #[test]
        fn test_receivers_at() {
            let input = "TCM BCM";
            let mut parser = Parser::new(input.as_bytes()).unwrap();
            let result = Receivers::parse(&mut parser).unwrap();
            assert_eq!(result.at(0), Some("TCM"));
            assert_eq!(result.at(1), Some("BCM"));
            assert_eq!(result.at(2), None);

            let none = Receivers::new_none();
            assert_eq!(none.at(0), None);
        }
    }
}
