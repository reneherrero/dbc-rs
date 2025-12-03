use crate::{Parser, error::ParseError, error::ParseResult, error::messages};

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq)]
pub enum Receivers {
    Broadcast,
    Nodes(Vec<String>),
    None,
}

#[cfg(not(feature = "std"))]
#[derive(Debug, Clone, PartialEq)]
pub enum Receivers<'a> {
    Broadcast,
    Nodes(&'a [&'a str]),
    None,
}

#[cfg(feature = "std")]
impl Receivers {
    pub(crate) fn parse<'b>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        const MAX_RECEIVER_NODES: usize = 64;

        // Skip any leading whitespace
        let _ = parser.skip_whitespace();

        // Check if we're at EOF
        if parser.remaining().is_empty() {
            return Ok(Receivers::None);
        }

        // Check if next character is '*'
        let remaining = parser.remaining();
        if remaining.starts_with(b"*") {
            parser.expect(b"*")?;
            return Ok(Receivers::Broadcast);
        }

        // Parse space-separated identifiers
        use alloc::vec::Vec;
        let mut nodes: Vec<&str> = Vec::new();

        loop {
            // Skip whitespace
            let _ = parser.skip_whitespace();

            // Check if we're at EOF or end of line
            if parser.remaining().is_empty() {
                break;
            }

            // Check if we're at a newline (end of signal line)
            if parser.remaining().starts_with(b"\n") || parser.remaining().starts_with(b"\r") {
                break;
            }

            // Try to parse an identifier
            match parser.parse_identifier() {
                Ok(node) => {
                    nodes.push(node);
                    // Check for too many receiver nodes (DoS protection)
                    if nodes.len() > MAX_RECEIVER_NODES {
                        return Err(ParseError::Version(messages::SIGNAL_RECEIVERS_TOO_MANY));
                    }
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if nodes.is_empty() {
            Ok(Receivers::None)
        } else {
            // Convert to owned Vec<String>
            let nodes: Vec<String> = nodes.into_iter().map(|s| s.to_string()).collect();

            Ok(Receivers::Nodes(nodes))
        }
    }
}

#[cfg(not(feature = "std"))]
impl<'a> Receivers<'a> {
    #[allow(dead_code)] // Used by Signal::parse, which is reserved for future use
    pub(crate) fn parse<'b: 'a>(parser: &mut Parser<'b>) -> ParseResult<Self> {
        const MAX_RECEIVER_NODES: usize = 64;

        // Skip any leading whitespace
        let _ = parser.skip_whitespace();

        // Check if we're at EOF
        if parser.remaining().is_empty() {
            return Ok(Receivers::None);
        }

        // Check if next character is '*'
        let remaining = parser.remaining();
        if remaining.starts_with(b"*") {
            parser.expect(b"*")?;
            return Ok(Receivers::Broadcast);
        }

        // Parse space-separated identifiers
        use alloc::vec::Vec;
        let mut nodes: Vec<&str> = Vec::new();

        loop {
            // Skip whitespace
            let _ = parser.skip_whitespace();

            // Check if we're at EOF or end of line
            if parser.remaining().is_empty() {
                break;
            }

            // Try to parse an identifier
            match parser.parse_identifier() {
                Ok(node) => {
                    nodes.push(node);
                    // Check for too many receiver nodes (DoS protection)
                    if nodes.len() > MAX_RECEIVER_NODES {
                        return Err(ParseError::Version(messages::SIGNAL_RECEIVERS_TOO_MANY));
                    }
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if nodes.is_empty() {
            Ok(Receivers::None)
        } else {
            // For no_std, we need to leak the Vec to get a static slice
            // This is safe because the parser's input lifetime ensures the data is valid
            use alloc::boxed::Box;
            let boxed: Box<[&'b str]> = nodes.into_boxed_slice();
            let leaked = Box::leak(boxed);
            Ok(Receivers::Nodes(leaked))
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
    #[ignore]
    fn test_parse_receivers_none_empty() {
        let input = "";
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
            Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], "TCM");
            }
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_parse_receivers_multiple_nodes() {
        let input = "TCM BCM ECM";
        let mut parser = Parser::new(input.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        match result {
            Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 3);
                assert_eq!(nodes[0], "TCM");
                assert_eq!(nodes[1], "BCM");
                assert_eq!(nodes[2], "ECM");
            }
            _ => panic!("Expected Nodes variant"),
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
        match result {
            Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert_eq!(nodes[0], "TCM");
                assert_eq!(nodes[1], "BCM");
            }
            _ => panic!("Expected Nodes variant"),
        }
    }

    #[test]
    fn test_parse_receivers_too_many() {
        // Create a string with 65 receiver nodes (exceeds limit of 64)
        let mut receivers = String::new();
        for i in 0..65 {
            if i > 0 {
                receivers.push(' ');
            }
            receivers.push_str(&format!("Node{i}"));
        }
        let mut parser = Parser::new(receivers.as_bytes()).unwrap();
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
        let mut receivers = String::new();
        for i in 0..64 {
            if i > 0 {
                receivers.push(' ');
            }
            receivers.push_str(&format!("Node{i}"));
        }
        let mut parser = Parser::new(receivers.as_bytes()).unwrap();
        let result = Receivers::parse(&mut parser).unwrap();
        match result {
            Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 64);
            }
            _ => panic!("Expected Nodes variant"),
        }
    }
}
