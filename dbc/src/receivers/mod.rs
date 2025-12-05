use crate::{Parser, error::ParseError, error::ParseResult, error::messages};

#[cfg(feature = "std")]
mod receivers_builder;

#[cfg(feature = "std")]
pub use receivers_builder::ReceiversBuilder;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // Nodes variant is large but necessary for no_std
pub enum Receivers<'a> {
    Broadcast,
    Nodes([Option<&'a str>; 64], usize), // Stores array and count directly
    None,
}

impl<'a> Receivers<'a> {
    fn new_broadcast() -> Self {
        Receivers::Broadcast
    }

    fn new_none() -> Self {
        Receivers::None
    }

    fn new_nodes(nodes: &[&'a str]) -> Self {
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

    #[allow(dead_code)] // Used by Signal::parse, which is reserved for future use
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

    pub fn iter_nodes(&self) -> impl Iterator<Item = &'a str> + '_ {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parser, error::ParseError, error::lang};

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
                let node_count = result.iter_nodes().count();
                assert_eq!(node_count, 1);
                let first_node = result.iter_nodes().next().unwrap();
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
            let node_count = result.iter_nodes().count();
            assert_eq!(node_count, 3);
            let nodes: Vec<&str> = result.iter_nodes().collect();
            assert_eq!(nodes[0], "TCM");
            assert_eq!(nodes[1], "BCM");
            assert_eq!(nodes[2], "ECM");
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
        {
            let node_count = result.iter_nodes().count();
            assert_eq!(node_count, 2);
            let nodes: Vec<&str> = result.iter_nodes().collect();
            assert_eq!(nodes[0], "TCM");
            assert_eq!(nodes[1], "BCM");
        }
    }

    #[test]
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
            ParseError::Version(msg) => {
                assert!(msg.contains(lang::SIGNAL_RECEIVERS_TOO_MANY));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
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
        let node_count = result.iter_nodes().count();
        assert_eq!(node_count, 64);
    }
}
