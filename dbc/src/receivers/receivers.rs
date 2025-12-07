use crate::{Parser, error::ParseError, error::ParseResult, error::messages};

/// Represents the receiver nodes for a signal in a DBC file.
///
/// A signal can have three types of receivers:
/// - **Broadcast** (`*`): The signal is broadcast to all nodes on the bus
/// - **Specific nodes**: A list of specific node names that receive this signal
/// - **None**: No explicit receivers specified (signal may be unused or receiver is implicit)
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
///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
///  SG_ Temp : 16|8@0- (1,-40) [-40|215] "°C" TCM BCM
/// "#)?;
///
/// let message = dbc.messages().at(0).unwrap();
///
/// // Broadcast receiver
/// let rpm_signal = message.signals().find("RPM").unwrap();
/// assert_eq!(rpm_signal.receivers().len(), 0); // Broadcast has no specific nodes
///
/// // Specific nodes
/// let temp_signal = message.signals().find("Temp").unwrap();
/// assert_eq!(temp_signal.receivers().len(), 2);
/// assert!(temp_signal.receivers().contains("TCM"));
/// # Ok::<(), dbc_rs::Error>(())
/// ```
///
/// # DBC Format
///
/// In DBC files, receivers are specified after the signal definition:
/// - `*` indicates broadcast
/// - Space-separated node names indicate specific receivers
/// - No receivers means `None`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::large_enum_variant)] // Nodes variant is large but necessary for no_std
pub enum Receivers<'a> {
    /// Broadcast receiver - signal is sent to all nodes on the bus.
    Broadcast,
    /// Specific receiver nodes - array of node names and count.
    ///
    /// The array can hold up to 64 nodes. The second element is the actual count.
    Nodes([Option<&'a str>; 64], usize), // Stores array and count directly
    /// No explicit receivers specified.
    None,
}

impl<'a> Receivers<'a> {
    pub(crate) fn new_broadcast() -> Self {
        Receivers::Broadcast
    }

    pub(crate) fn new_none() -> Self {
        Receivers::None
    }

    pub(crate) fn new_nodes(nodes: &[&'a str]) -> Self {
        // Validation should have been done prior (by builder or parse)
        const MAX_RECEIVER_NODES: usize = 64;
        let mut node_array: [Option<&'a str>; MAX_RECEIVER_NODES] =
            [const { None }; MAX_RECEIVER_NODES];
        let count = nodes.len();
        for (i, node) in nodes.iter().enumerate() {
            node_array[i] = Some(*node);
        }
        Receivers::Nodes(node_array, count)
    }

    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        const MAX_RECEIVER_NODES: usize = 64;

        // Skip any leading spaces (but not newlines - newlines indicate end of line)
        // If we get UnexpectedEof, we're at EOF, so return None
        match parser.skip_whitespace() {
            Ok(_) => {}
            Err(ParseError::UnexpectedEof) => return Ok(Self::new_none()),
            Err(_) => {} // Other errors (like Expected) mean there's no whitespace, continue
        }

        // Check if next character is '*' (broadcast marker)
        if parser.expect(b"*").is_ok() {
            return Ok(Self::new_broadcast());
        }

        // Check if we're at a newline (end of signal line)
        if parser.expect(b"\n").is_ok() || parser.expect(b"\r").is_ok() {
            return Ok(Self::new_none());
        }

        // Parse space-separated identifiers into fixed-size array
        let mut nodes = [None; MAX_RECEIVER_NODES];
        let mut count = 0;

        loop {
            // Skip spaces (but not newlines)
            // If we get UnexpectedEof, we're at EOF, so break
            match parser.skip_whitespace() {
                Ok(_) => {}
                Err(ParseError::UnexpectedEof) => break,
                Err(_) => {} // Other errors mean there's no whitespace, continue
            }

            // Check if we're at a newline (end of signal line)
            if parser.expect(b"\n").is_ok() || parser.expect(b"\r").is_ok() {
                break;
            }

            // Try to parse an identifier
            // parse_identifier() stops at newlines without consuming them
            let pos_before = parser.pos();
            match parser.parse_identifier() {
                Ok(node) => {
                    if count >= MAX_RECEIVER_NODES {
                        return Err(ParseError::Version(messages::SIGNAL_RECEIVERS_TOO_MANY));
                    }
                    nodes[count] = Some(node);
                    count += 1;
                }
                Err(ParseError::UnexpectedEof) => break,
                Err(_) => {
                    // Failed to parse - if position didn't change, we're at newline or invalid char
                    if parser.pos() == pos_before {
                        break;
                    }
                    // Position changed but parsing failed - invalid character, also break
                    break;
                }
            }
        }

        if count == 0 {
            Ok(Self::new_none())
        } else {
            // Collect node names into a slice for new_nodes
            let mut node_refs: [&'b str; 64] = [""; 64];
            for i in 0..count {
                if let Some(node) = nodes[i] {
                    node_refs[i] = node;
                }
            }
            // Validate before construction
            const MAX_RECEIVER_NODES: usize = 64;
            if count > MAX_RECEIVER_NODES {
                return Err(ParseError::Version(messages::SIGNAL_RECEIVERS_TOO_MANY));
            }
            // Construct directly (validation already done)
            Ok(Self::new_nodes(&node_refs[..count]))
        }
    }

    /// Returns an iterator over the receiver node names.
    ///
    /// For `Receivers::Broadcast` and `Receivers::None`, the iterator will be empty.
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
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "°C" TCM BCM
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
    /// # Broadcast and None
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM
    ///
    /// BO_ 256 Engine : 8 ECM
    ///  SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
    /// "#)?;
    ///
    /// let message = dbc.messages().at(0).unwrap();
    /// let signal = message.signals().at(0).unwrap();
    ///
    /// // Broadcast receivers return empty iterator
    /// assert_eq!(signal.receivers().iter().count(), 0);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use = "iterator is lazy and does nothing unless consumed"]
    pub fn iter(&self) -> impl Iterator<Item = &'a str> + '_ {
        struct NodeIter<'a> {
            arr: [Option<&'a str>; 64],
            count: usize,
            pos: usize,
        }
        impl<'a> Iterator for NodeIter<'a> {
            type Item = &'a str;
            fn next(&mut self) -> Option<Self::Item> {
                while self.pos < self.count {
                    let result = self.arr[self.pos];
                    self.pos += 1;
                    if let Some(node) = result {
                        return Some(node);
                    }
                }
                None
            }
        }

        match self {
            Receivers::Nodes(arr, count) => NodeIter {
                arr: *arr,
                count: *count,
                pos: 0,
            },
            _ => NodeIter {
                arr: [None; 64],
                count: 0,
                pos: 0,
            },
        }
    }

    /// Returns the number of receiver nodes.
    ///
    /// - For `Receivers::Nodes`: Returns the count of specific receiver nodes
    /// - For `Receivers::Broadcast` and `Receivers::None`: Returns `0`
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
    /// assert_eq!(signal.receivers().len(), 2);
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Receivers::Nodes(_, count) => *count,
            Receivers::Broadcast | Receivers::None => 0,
        }
    }

    /// Returns `true` if there are no specific receiver nodes.
    ///
    /// This returns `true` for both `Receivers::Broadcast` and `Receivers::None`,
    /// as neither has specific node names.
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
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if a node name is in the receivers list.
    ///
    /// For `Receivers::Broadcast` and `Receivers::None`, this always returns `false`.
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
    #[must_use]
    pub fn contains(&self, node: &str) -> bool {
        self.iter().any(|n| n == node)
    }

    /// Gets a receiver node by index.
    ///
    /// Returns `None` if:
    /// - The index is out of bounds
    /// - The receiver is `Broadcast` or `None`
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
    ///  SG_ Temp : 0|8@1+ (1,0) [0|255] "°C" TCM BCM
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
    #[must_use]
    pub fn at(&self, index: usize) -> Option<&'a str> {
        match self {
            Receivers::Nodes(arr, count) => {
                if index >= *count {
                    return None;
                }
                arr[index]
            }
            Receivers::Broadcast | Receivers::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    use crate::error::{ParseError, lang};
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    use alloc::format;

    #[test]
    fn test_parse_receivers_broadcast() {
        let input = "*";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        assert_eq!(result, Receivers::Broadcast);
    }

    #[test]
    fn test_parse_receivers_none_empty() {
        // Parser::new returns error for empty input, so use a single space instead
        // Empty receivers should be handled by Receivers::parse when called from Signal::parse
        // For this test, we'll test with whitespace-only input
        let input = " ";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        assert_eq!(result, Receivers::None);
    }

    #[test]
    fn test_parse_receivers_single_node() {
        let input = "TCM";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        match result {
            Receivers::Nodes(_, count) => {
                assert_eq!(count, 1);
                let node_count = result.len();
                assert_eq!(node_count, 1);
                let first_node = result.iter().next().unwrap();
                assert_eq!(first_node, "TCM");
            }
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_parse_receivers_multiple_nodes() {
        let input = "TCM BCM ECM";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        {
            let node_count = result.len();
            assert_eq!(node_count, 3);
            let mut iter = result.iter();
            assert_eq!(iter.next().unwrap(), "TCM");
            assert_eq!(iter.next().unwrap(), "BCM");
            assert_eq!(iter.next().unwrap(), "ECM");
            assert!(iter.next().is_none());
        }
    }

    #[test]
    fn test_parse_receivers_whitespace_only() {
        let input = "   ";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        assert_eq!(result, Receivers::None);
    }

    #[test]
    fn test_parse_receivers_with_extra_whitespace() {
        let input = "  TCM   BCM  ";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        let node_count = result.len();
        assert_eq!(node_count, 2);
        let mut iter = result.iter();
        assert_eq!(iter.next(), Some("TCM"));
        assert_eq!(iter.next(), Some("BCM"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    fn test_parse_receivers_too_many() {
        // Create a string with 65 receiver nodes (exceeds limit of 64)
        // Use a simple approach: create byte array directly
        use crate::compat::Vec;
        use alloc::format;
        let mut receivers_bytes = Vec::new();
        for i in 0..65 {
            if i > 0 {
                receivers_bytes.push(b' ');
            }
            let node_str = format!("Node{i}");
            receivers_bytes.extend_from_slice(node_str.as_bytes());
        }
        let mut parser = Parser::new(&receivers_bytes).unwrap();
        let result = Receivers::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::Version(msg) => {
                assert!(msg.contains(lang::SIGNAL_RECEIVERS_TOO_MANY));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "kernel"))]
    fn test_parse_receivers_at_limit() {
        // Create a string with exactly 64 receiver nodes (at the limit)
        // Use a simple approach: create byte array directly
        use crate::compat::Vec;
        use alloc::format;
        let mut receivers_bytes = Vec::new();
        for i in 0..64 {
            if i > 0 {
                receivers_bytes.push(b' ');
            }
            let node_str = format!("Node{i}");
            receivers_bytes.extend_from_slice(node_str.as_bytes());
        }
        let mut parser = Parser::new(&receivers_bytes).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        let node_count = result.len();
        assert_eq!(node_count, 64);
    }
}
