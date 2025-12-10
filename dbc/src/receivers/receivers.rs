use crate::{
    Cow, MAX_RECEIVER_NODES, Parser,
    error::{self, ParseError, ParseResult},
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

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
pub enum Receivers<'a> {
    /// Broadcast receiver - signal is sent to all nodes on the bus.
    Broadcast,
    /// Specific receiver nodes - vector of node names.
    Nodes(Vec<Cow<'a, str>>),
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

    #[cfg(feature = "std")]
    pub(crate) fn new_nodes(nodes: &[impl Into<Cow<'a, str>> + Clone]) -> Self {
        // Validation should have been done prior (by builder or parse)
        let vec_nodes: Vec<Cow<'a, str>> =
            nodes.iter().take(MAX_RECEIVER_NODES).map(|node| node.clone().into()).collect();
        Receivers::Nodes(vec_nodes)
    }

    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
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

        // Parse space-separated identifiers into Vec
        let mut nodes: Vec<Cow<'a, str>> = Vec::new();

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
                    if nodes.len() >= MAX_RECEIVER_NODES {
                        return Err(ParseError::Receivers(
                            error::lang::SIGNAL_RECEIVERS_TOO_MANY,
                        ));
                    }
                    nodes.push(Cow::Borrowed(node));
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

        if nodes.is_empty() {
            Ok(Self::new_none())
        } else {
            Ok(Receivers::Nodes(nodes))
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
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        struct ReceiversIter<'a> {
            nodes: Option<&'a Vec<Cow<'a, str>>>,
            pos: usize,
        }

        impl<'a> Iterator for ReceiversIter<'a> {
            type Item = &'a str;
            fn next(&mut self) -> Option<Self::Item> {
                if let Some(nodes) = self.nodes {
                    if self.pos < nodes.len() {
                        let result = nodes[self.pos].as_ref();
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

        match self {
            Receivers::Nodes(nodes) => ReceiversIter {
                nodes: Some(nodes),
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
            Receivers::Nodes(nodes) => nodes.len(),
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
    pub fn at(&self, index: usize) -> Option<&str> {
        match self {
            Receivers::Nodes(nodes) => nodes.get(index).map(|cow| cow.as_ref()),
            Receivers::Broadcast | Receivers::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

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
        match &result {
            Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 1);
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
    #[cfg(feature = "std")]
    fn test_parse_receivers_too_many() {
        // Create a string with 65 receiver nodes (exceeds limit of 64)
        // Use a simple approach: create byte array directly
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
            ParseError::Receivers(msg) => {
                assert!(msg.contains(crate::error::lang::SIGNAL_RECEIVERS_TOO_MANY));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_parse_receivers_at_limit() {
        // Create a string with exactly 64 receiver nodes (at the limit)
        // Use a simple approach: create byte array directly
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
