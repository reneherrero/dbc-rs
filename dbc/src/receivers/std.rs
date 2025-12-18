use super::Receivers;
use std::string::String;

impl Receivers {
    /// Converts the receivers to their DBC file string representation.
    ///
    /// Per DBC spec Section 9.5:
    /// ```bnf
    /// receivers = receiver {',' receiver} ;
    /// receiver = node_name | 'Vector__XXX' ;
    /// ```
    ///
    /// - Specific nodes are comma-separated
    /// - Empty or None receivers serialize as `Vector__XXX`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dbc_rs::{ReceiversBuilder, Receivers};
    ///
    /// // No receivers -> Vector__XXX
    /// let none = ReceiversBuilder::new().none().build().unwrap();
    /// assert_eq!(none.to_dbc_string(), "Vector__XXX");
    ///
    /// // Single receiver
    /// let single = ReceiversBuilder::new().add_node("TCM").build().unwrap();
    /// assert_eq!(single.to_dbc_string(), "TCM");
    ///
    /// // Multiple receivers -> comma-separated
    /// let multi = ReceiversBuilder::new().add_node("TCM").add_node("BCM").build().unwrap();
    /// assert_eq!(multi.to_dbc_string(), "TCM,BCM");
    /// ```
    #[must_use = "return value should be used"]
    pub fn to_dbc_string(&self) -> String {
        match self {
            Receivers::Nodes(nodes) if nodes.is_empty() => crate::VECTOR_XXX.to_string(),
            Receivers::Nodes(nodes) => {
                let mut result = String::with_capacity(nodes.len() * 10);
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    result.push_str(node.as_str());
                }
                result
            }
            Receivers::None => crate::VECTOR_XXX.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ReceiversBuilder;

    #[test]
    fn test_receivers_to_dbc_string_none() {
        let receivers = ReceiversBuilder::new().none().build().unwrap();
        assert_eq!(receivers.to_dbc_string(), "Vector__XXX");
    }

    #[test]
    fn test_receivers_to_dbc_string_single() {
        let receivers = ReceiversBuilder::new().add_node("TCM").build().unwrap();
        assert_eq!(receivers.to_dbc_string(), "TCM");
    }

    #[test]
    fn test_receivers_to_dbc_string_multiple() {
        let receivers = ReceiversBuilder::new().add_node("TCM").add_node("BCM").build().unwrap();
        assert_eq!(receivers.to_dbc_string(), "TCM,BCM");
    }

    #[test]
    fn test_receivers_to_dbc_string_empty_nodes() {
        // Edge case: Nodes variant with empty list
        let receivers = ReceiversBuilder::new().build().unwrap();
        assert_eq!(receivers.to_dbc_string(), "Vector__XXX");
    }
}
