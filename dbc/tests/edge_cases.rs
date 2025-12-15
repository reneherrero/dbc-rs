//! Edge case tests for DBC parsing and validation.
//!
//! Note: Most edge case tests have been moved to their respective implementation files.
//! This file now only contains tests that don't fit into a specific module category.

#![cfg(feature = "std")]

use dbc_rs::Dbc;

#[test]
fn test_unicode_in_names() {
    // Test Unicode characters in names
    // Note: The parser currently only accepts ASCII identifiers (C-style)
    // This test documents the current limitation
    let dbc_str = r#"VERSION "1.0"

BS_:

BU_: ECU1

BO_ 256 Message : 8 ECU1
 SG_ Signal : 0|8@1+ (1,0) [0|255] ""
"#;
    let dbc = Dbc::parse(dbc_str).expect("Should handle ASCII identifiers");
    assert!(dbc.nodes().contains("ECU1"));
    assert_eq!(dbc.messages().at(0).unwrap().name(), "Message");
    // TODO: Unicode support in identifiers is a future enhancement
}
