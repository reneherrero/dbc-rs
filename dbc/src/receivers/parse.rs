use super::Receivers;
use crate::{
    Error, MAX_NAME_SIZE, MAX_NODES, Parser, Result,
    compat::{String, Vec},
};

impl Receivers {
    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Skip any leading spaces (but not newlines - newlines indicate end of line)
        // Skip whitespace (spaces and tabs, but not newlines) before checking for broadcast/newline
        // Manually skip spaces and tabs since skip_whitespace() only handles spaces
        while !parser.eof() && !parser.at_newline() {
            match parser.current_byte() {
                Some(b' ') | Some(b'\t') => {
                    parser.advance_one();
                }
                _ => break,
            }
        }

        // Check if we're at a newline (end of signal line) - do this BEFORE checking for '*'
        if parser.at_newline() || parser.eof() {
            return Ok(Self::new_none());
        }

        // Check if next character is '*' (broadcast marker)
        if parser.expect(b"*").is_ok() {
            return Ok(Self::new_broadcast());
        }

        // Parse space-separated identifiers into Vec
        let mut nodes: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES - 1 }> = Vec::new();

        loop {
            // Check if we're at a newline (end of signal line) BEFORE doing anything else
            if parser.at_newline() || parser.eof() {
                break;
            }

            // Skip whitespace (spaces and tabs, but not newlines)
            // Manually skip spaces and tabs since skip_whitespace() only handles spaces
            while !parser.eof() && !parser.at_newline() {
                match parser.current_byte() {
                    Some(b' ') | Some(b'\t') => {
                        parser.advance_one();
                    }
                    _ => break,
                }
            }

            // Check again if we're at a newline after skipping whitespace
            if parser.at_newline() || parser.eof() {
                break;
            }

            // Try to parse an identifier
            // parse_identifier() stops at newlines without consuming them
            let pos_before = parser.pos();
            match parser.parse_identifier() {
                Ok(node) => {
                    // Check if adding this node would exceed MAX_NODES - 1 limit
                    // Receivers can have at most MAX_NODES - 1 nodes
                    if nodes.len() >= MAX_NODES - 1 {
                        return Err(Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY));
                    }
                    let node = crate::compat::validate_name(node)?;
                    nodes
                        .push(node)
                        .map_err(|_| Error::Receivers(Error::SIGNAL_RECEIVERS_TOO_MANY))?;

                    // After parsing an identifier, check what's next
                    // parse_identifier() stops at newlines/whitespace without consuming them

                    // Safety check: if position didn't advance, we're stuck - break
                    if parser.pos() == pos_before {
                        break;
                    }

                    // CRITICAL: Check for newline FIRST - parse_identifier() stops at \r/\n without consuming
                    // Check what's next after parsing the identifier
                    if parser.at_newline() || parser.eof() {
                        // At newline or EOF - we're done
                        break;
                    }
                    // Check if we're at whitespace (there might be another receiver)
                    if let Some(byte) = parser.current_byte() {
                        if byte == b' ' || byte == b'\t' {
                            // At whitespace - there might be another receiver
                            // Continue loop to skip whitespace and parse next receiver
                            continue;
                        }
                        // Not whitespace and not newline - parse_identifier() should have stopped here
                        // This indicates a bug, but break to prevent infinite loop
                        break;
                    }
                    // EOF - we're done
                    break;
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
            Ok(Self::new_nodes(&nodes))
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
                assert_eq!(result.iter().next(), Some("TCM"));
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
            assert_eq!(iter.next(), Some("TCM"));
            assert_eq!(iter.next(), Some("BCM"));
            assert_eq!(iter.next(), Some("ECM"));
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
        assert!(iter.next().is_none());
    }

    // Tests that require std (for format! macro)
    #[cfg(feature = "std")]
    mod tests_std {
        use super::*;
        use crate::Error;

        #[test]
        fn test_parse_receivers_too_many() {
            // Create a string with MAX_NODES receiver nodes (exceeds limit of MAX_NODES - 1)
            // Use std::vec::Vec since we need more than 64 bytes
            let mut receivers_bytes = std::vec::Vec::new();
            for i in 0..MAX_NODES {
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
            // Create a string with exactly MAX_NODES - 1 receiver nodes (at the limit)
            // Use std::vec::Vec since we need more than 64 bytes
            let mut receivers_bytes = std::vec::Vec::new();
            for i in 0..(MAX_NODES - 1) {
                if i > 0 {
                    receivers_bytes.push(b' ');
                }
                let node_str = format!("Node{i}");
                receivers_bytes.extend_from_slice(node_str.as_bytes());
            }
            let mut parser = Parser::new(&receivers_bytes).unwrap();
            let result = Receivers::parse(&mut parser).unwrap();
            let node_count = result.len();
            assert_eq!(node_count, MAX_NODES - 1);
        }
    }
}
