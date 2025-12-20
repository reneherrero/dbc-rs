use super::{ReceiverNames, Receivers};
use crate::{Error, MAX_NODES, Parser, Result};

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

        // Check if next character is '*' (non-standard broadcast marker)
        // Per DBC spec Section 9.5, '*' is not a valid receiver format.
        // Some tools use it as an extension. We treat it as "no specific receiver" (None).
        if parser.expect(b"*").is_ok() {
            return Ok(Self::new_none());
        }

        // Per DBC spec Section 9.5: receivers = receiver {',' receiver}
        // We accept both comma-separated (per spec) and space-separated (tool extension)
        let mut nodes: ReceiverNames = ReceiverNames::new();

        loop {
            // Check if we're at a newline (end of signal line) BEFORE doing anything else
            if parser.at_newline() || parser.eof() {
                break;
            }

            // Skip whitespace and commas (spaces, tabs, and commas, but not newlines)
            while !parser.eof() && !parser.at_newline() {
                match parser.current_byte() {
                    Some(b' ') | Some(b'\t') | Some(b',') => {
                        parser.advance_one();
                    }
                    _ => break,
                }
            }

            // Check again if we're at a newline after skipping whitespace/commas
            if parser.at_newline() || parser.eof() {
                break;
            }

            // Try to parse an identifier
            // parse_identifier() stops at newlines without consuming them
            let pos_before = parser.pos();
            match parser.parse_identifier() {
                Ok(node) => {
                    // Per DBC spec Section 9.5: 'Vector__XXX' means no specific receiver
                    // If we encounter Vector__XXX as the only/first receiver, treat as None
                    if node == crate::VECTOR_XXX {
                        // Skip this "pseudo-receiver" - it represents no specific receiver
                        // Continue to see if there are more receivers (there shouldn't be)
                        // but don't add it to the nodes list
                        if parser.pos() == pos_before {
                            break;
                        }
                        if parser.at_newline() || parser.eof() {
                            break;
                        }
                        continue;
                    }

                    // Check if adding this node would exceed MAX_NODES - 1 limit
                    // Receivers can have at most MAX_NODES - 1 nodes
                    if nodes.len() >= MAX_NODES - 1 {
                        return Err(parser.err_receivers(Error::SIGNAL_RECEIVERS_TOO_MANY));
                    }
                    let node = crate::compat::validate_name(node)?;
                    nodes
                        .push(node)
                        .map_err(|_| parser.err_receivers(Error::SIGNAL_RECEIVERS_TOO_MANY))?;

                    // After parsing an identifier, check what's next
                    // parse_identifier() stops at newlines/whitespace/comma without consuming them

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
                    // Check if we're at whitespace or comma (there might be another receiver)
                    if let Some(byte) = parser.current_byte() {
                        if byte == b' ' || byte == b'\t' || byte == b',' {
                            // At separator - there might be another receiver
                            // Continue loop to skip separators and parse next receiver
                            continue;
                        }
                        // Not separator and not newline - parse_identifier() should have stopped here
                        // This indicates a bug, but break to prevent infinite loop
                        break;
                    }
                    // EOF - we're done
                    break;
                }
                Err(Error::UnexpectedEof { .. }) => break,
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
            Ok(Self::new_nodes(nodes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    #[test]
    fn test_parse_receivers_asterisk_treated_as_none() {
        // Per DBC spec Section 9.5, '*' is not a valid receiver format.
        // We treat it as "no specific receiver" (None) for compatibility.
        let input = "*";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        assert_eq!(result, Receivers::None);
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

    #[test]
    fn test_parse_receivers_too_many() {
        use crate::compat;
        use core::fmt::Write;
        // Create a string with MAX_NODES receiver nodes (exceeds limit of MAX_NODES - 1)
        // Buffer size: MAX_NODES * 8 bytes per node (e.g., "Node255 ") = 2048 bytes
        let mut receivers_bytes: compat::Vec<u8, 2560> = compat::Vec::new();
        for i in 0..MAX_NODES {
            if i > 0 {
                receivers_bytes.push(b' ').unwrap();
            }
            let mut node_str: compat::String<16> = compat::String::new();
            write!(node_str, "Node{i}").unwrap();
            receivers_bytes.extend_from_slice(node_str.as_bytes()).unwrap();
        }
        let mut parser = Parser::new(receivers_bytes.as_slice()).unwrap();
        let result = Receivers::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Receivers { msg, line } => {
                assert_eq!(msg, Error::SIGNAL_RECEIVERS_TOO_MANY);
                assert!(line.is_some());
            }
            _ => panic!("Expected Error::Receivers"),
        }
    }

    #[test]
    fn test_parse_receivers_at_limit() {
        use crate::compat;
        use core::fmt::Write;
        // Create a string with exactly MAX_NODES - 1 receiver nodes (at the limit)
        // Buffer size: (MAX_NODES - 1) * 8 bytes per node = 2040 bytes
        let mut receivers_bytes: compat::Vec<u8, 2560> = compat::Vec::new();
        for i in 0..(MAX_NODES - 1) {
            if i > 0 {
                let _ = receivers_bytes.push(b' ');
            }
            let mut node_str: compat::String<16> = compat::String::new();
            write!(node_str, "Node{i}").unwrap();
            receivers_bytes.extend_from_slice(node_str.as_bytes()).unwrap();
        }
        let mut parser = Parser::new(receivers_bytes.as_slice()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        let node_count = result.len();
        assert_eq!(node_count, MAX_NODES - 1);
    }
}
