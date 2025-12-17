use super::Nodes;
use crate::BU_;
use std::fmt::{Display, Formatter, Result};

impl Display for Nodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.nodes.is_empty() {
            return Ok(());
        }
        for (i, node) in self.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", node)?;
        }
        Ok(())
    }
}

impl Nodes {
    /// Converts the nodes to their DBC file representation.
    ///
    /// Returns a string in the format: `BU_: Node1 Node2 Node3 ...`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_: ECM TCM BCM
    /// "#)?;
    ///
    /// let dbc_string = dbc.nodes().to_dbc_string();
    /// assert_eq!(dbc_string, "BU_: ECM TCM BCM");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Empty Nodes
    ///
    /// Empty node lists are represented as `BU_:`:
    ///
    /// ```rust,no_run
    /// use dbc_rs::Dbc;
    ///
    /// let dbc = Dbc::parse(r#"VERSION "1.0"
    ///
    /// BU_:
    /// "#)?;
    ///
    /// assert_eq!(dbc.nodes().to_dbc_string(), "BU_:");
    /// # Ok::<(), dbc_rs::Error>(())
    /// ```
    ///
    /// # Feature Requirements
    ///
    /// This method requires the `std` feature to be enabled.
    #[must_use = "return value should be used"]
    pub fn to_dbc_string(&self) -> std::string::String {
        let mut result = format!("{}:", BU_);
        let nodes_str = format!("{}", self);
        if !nodes_str.is_empty() {
            result.push(' ');
            result.push_str(&nodes_str);
        }
        result
    }
}
