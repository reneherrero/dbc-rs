use super::Receivers;
use crate::{
    Error, MAX_NAME_SIZE, MAX_RECEIVER_NODES, Parser, Result,
    compat::{String, Vec},
};

impl Receivers {
    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Skip any leading spaces (but not newlines - newlines indicate end of line)
        // If we get UnexpectedEof, we're at EOF, so return None
        if let Err(Error::UnexpectedEof) = parser.skip_whitespace() {
            return Ok(Self::new_none());
        }
        // Other errors (like Expected) mean there's no whitespace, continue

        // Check if next character is '*' (broadcast marker)
        if parser.expect(b"*").is_ok() {
            return Ok(Self::new_broadcast());
        }

        // Check if we're at a newline (end of signal line)
        if parser.expect(b"\n").is_ok() || parser.expect(b"\r").is_ok() {
            return Ok(Self::new_none());
        }

        // Parse space-separated identifiers into Vec
        let mut nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_RECEIVER_NODES }> = Vec::new();

        loop {
            // Skip spaces (but not newlines)
            // If we get UnexpectedEof, we're at EOF, so break
            if let Err(Error::UnexpectedEof) = parser.skip_whitespace() {
                break;
            }
            // Other errors mean there's no whitespace, continue

            // Check if we're at a newline (end of signal line)
            if parser.expect(b"\n").is_ok() || parser.expect(b"\r").is_ok() {
                break;
            }

            // Try to parse an identifier
            // parse_identifier() stops at newlines without consuming them
            let pos_before = parser.pos();
            match parser.parse_identifier() {
                Ok(node) => {
                    if let Some(err) = crate::check_max_limit(
                        nodes.len(),
                        MAX_RECEIVER_NODES - 1,
                        Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY),
                    ) {
                        return Err(err);
                    }
                    let node = crate::validate_name(node)?;
                    nodes
                        .push(node)
                        .map_err(|_| Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY))?;
                }
                Err(Error::UnexpectedEof) => break,
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
                assert_eq!(first_node.as_str(), "TCM");
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
            assert_eq!(iter.next().unwrap().as_str(), "TCM");
            assert_eq!(iter.next().unwrap().as_str(), "BCM");
            assert_eq!(iter.next().unwrap().as_str(), "ECM");
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
        let node1 = iter.next().unwrap();
        assert_eq!(node1.as_str(), "TCM");
        let node2 = iter.next().unwrap();
        assert_eq!(node2.as_str(), "BCM");
        assert!(iter.next().is_none());
    }

    // Tests that require std (for format! macro)
    #[cfg(feature = "std")]
    mod tests_std {
        use super::*;
        use crate::Error;

        #[test]
        fn test_parse_receivers_too_many() {
            // Create a string with 65 receiver nodes (exceeds limit of 64)
            // Use std::vec::Vec since we need more than 64 bytes
            let mut receivers_bytes = std::vec::Vec::new();
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
                Error::Receivers(msg) => {
                    assert_eq!(msg, Error::SIGNAL_RECEIVERS_TOO_MANY);
                }
                _ => panic!("Expected Error::Receivers"),
            }
        }

        #[test]
        fn test_parse_receivers_at_limit() {
            // Create a string with exactly 64 receiver nodes (at the limit)
            // Use std::vec::Vec since we need more than 64 bytes
            let mut receivers_bytes = std::vec::Vec::new();
            for i in 0..MAX_RECEIVER_NODES {
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
}
