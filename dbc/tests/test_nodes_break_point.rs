#[cfg(test)]
mod tests {
    use dbc_rs::Dbc;

    /// Test that Nodes::parse() correctly positions the parser at BO_ when it breaks
    #[test]
    fn test_nodes_break_at_bo() {
        let dbc_content = r#"VERSION "1.0"

BU_:
NEO
IMCU
IGTW
IEPAS
IDI
IESP
ISBW
ISTW
IAPP
IDAS
IXXX

BO_ 262 DI_torque1: 8 DI
 SG_ DI_torqueDriver : 0|13@1- (0.25,0) [-750|750] "Nm"  NEO
"#;

        let result = Dbc::parse(dbc_content);
        assert!(
            result.is_ok(),
            "Parse should succeed, got: {:?}",
            result.err()
        );
    }

    /// Test that Nodes::parse() correctly positions the parser at BS_ when it breaks
    #[test]
    fn test_nodes_break_at_bs() {
        let dbc_content = r#"VERSION "1.0"

BU_:
NEO
IMCU

BS_:

BO_ 262 DI_torque1: 8 DI
 SG_ DI_torqueDriver : 0|13@1- (0.25,0) [-750|750] "Nm"  NEO
"#;

        let result = Dbc::parse(dbc_content);
        assert!(
            result.is_ok(),
            "Parse should succeed, got: {:?}",
            result.err()
        );
    }

    /// Test that Nodes::parse() correctly positions the parser at CM_ when it breaks
    #[test]
    fn test_nodes_break_at_cm() {
        let dbc_content = r#"VERSION "1.0"

BU_:
NEO
IMCU

CM_ "Comment"

BO_ 262 DI_torque1: 8 DI
 SG_ DI_torqueDriver : 0|13@1- (0.25,0) [-750|750] "Nm"  NEO
"#;

        let result = Dbc::parse(dbc_content);
        assert!(
            result.is_ok(),
            "Parse should succeed, got: {:?}",
            result.err()
        );
    }

    /// Test that Nodes::parse() correctly handles EOF after nodes
    #[test]
    fn test_nodes_break_at_eof() {
        let dbc_content = r#"VERSION "1.0"

BU_:
NEO
IMCU
"#;

        let result = Dbc::parse(dbc_content);
        assert!(
            result.is_ok(),
            "Parse should succeed, got: {:?}",
            result.err()
        );
    }

    /// Test that Nodes::parse() correctly handles nodes on same line
    #[test]
    fn test_nodes_same_line() {
        let dbc_content = r#"VERSION "1.0"

BU_: NEO IMCU IGTW

BO_ 262 DI_torque1: 8 DI
 SG_ DI_torqueDriver : 0|13@1- (0.25,0) [-750|750] "Nm"  NEO
"#;

        let result = Dbc::parse(dbc_content);
        assert!(
            result.is_ok(),
            "Parse should succeed, got: {:?}",
            result.err()
        );
    }
}
