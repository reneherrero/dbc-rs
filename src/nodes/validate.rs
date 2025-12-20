use super::Nodes;
use crate::{Error, MAX_NAME_SIZE, MAX_NODES, Result, compat::String, error::check_max_limit};

impl Nodes {
    // Shared validation function
    pub(crate) fn validate(nodes: &[String<{ MAX_NAME_SIZE }>]) -> Result<()> {
        // Check for too many nodes (DoS protection)
        if let Some(err) = check_max_limit(
            nodes.len(),
            MAX_NODES,
            Error::Validation(Error::NODES_TOO_MANY),
        ) {
            return Err(err);
        }

        // Check for duplicate node names (case-sensitive)
        for (i, node1) in nodes.iter().enumerate() {
            for node2 in nodes.iter().skip(i + 1) {
                if node1.as_str() == node2.as_str() {
                    return Err(Error::Validation(Error::NODES_DUPLICATE_NAME));
                }
            }
        }
        Ok(())
    }
}
