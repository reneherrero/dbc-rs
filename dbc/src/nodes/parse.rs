use super::Nodes;
use crate::{
    BU_, Error, MAX_NAME_SIZE, MAX_NODES, Parser, Result,
    compat::{String, Vec},
};

impl Nodes {
    #[must_use = "parse result should be checked"]
    pub(crate) fn parse(parser: &mut Parser) -> Result<Self> {
        // Nodes parsing must always start with "BU_" keyword
        parser
            .expect(BU_.as_bytes())
            .map_err(|_| Error::Expected("Expected BU_ keyword"))?;

        // Expect ":" after "BU_" (no whitespace between BU_ and :)
        parser.expect_with_msg(b":", "Expected colon after BU_")?;

        // Skip optional whitespace after ":"
        parser.skip_newlines_and_spaces();

        // Parse node names into Vec
        let mut node_names: Vec<String<{ MAX_NAME_SIZE }>, { MAX_NODES }> = Vec::new();

        loop {
            // Skip whitespace before each node name
            parser.skip_whitespace_optional();

            // Try to parse an identifier (node name)
            // parse_identifier() will fail if we're at EOF
            match parser.parse_identifier() {
                Ok(node) => {
                    if let Some(err) = crate::check_max_limit(
                        node_names.len(),
                        MAX_NODES - 1,
                        Error::Nodes(Error::NODES_TOO_MANY),
                    ) {
                        return Err(err);
                    }
                    let node_str = crate::validate_name(node)?;
                    node_names.push(node_str).map_err(|_| Error::Nodes(Error::NODES_TOO_MANY))?;
                }
                Err(_) => {
                    // No more identifiers, break
                    break;
                }
            }
        }

        if node_names.is_empty() {
            return Ok(Nodes { nodes: Vec::new() });
        }

        // Validate before construction
        Self::validate(node_names.as_slice()).map_err(|e| {
            crate::error::map_val_error(e, Error::Nodes, || Error::Nodes(Error::NODES_ERROR_PREFIX))
        })?;
        // Construct directly (validation already done)
        Ok(Self { nodes: node_names })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Parser};

    #[test]
    fn test_nodes_from_valid_line() {
        let line = b"BU_: ECM TCM BCM ABS";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let mut iter = nodes.iter();
        assert_eq!(iter.next(), Some("ECM"));
        assert_eq!(iter.next(), Some("TCM"));
        assert_eq!(iter.next(), Some("BCM"));
        assert_eq!(iter.next(), Some("ABS"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_nodes_from_single_node() {
        let line = b"BU_: ONLYONE";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let mut iter = nodes.iter();
        assert_eq!(iter.next(), Some("ONLYONE"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_nodes_from_with_extra_spaces() {
        let line = b"BU_:   Node1   Node2   ";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        let mut iter = nodes.iter();
        assert_eq!(iter.next(), Some("Node1"));
        assert_eq!(iter.next(), Some("Node2"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_nodes_from_empty_list() {
        let line = b"BU_:";
        let mut parser = Parser::new(line).unwrap();
        let nodes = Nodes::parse(&mut parser).unwrap();
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_nodes_parse_duplicate() {
        let line = b"BU_: ECM TCM ECM";
        let mut parser = Parser::new(line).unwrap();
        let result = Nodes::parse(&mut parser);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Nodes(msg) => assert!(msg == Error::NODES_DUPLICATE_NAME),
            _ => panic!("Expected Error::Nodes"),
        }
    }
}
