#[cfg(feature = "std")]
#[test]
fn test_ns_parse_issue() {
    use dbc_rs::Dbc;

    let content = r#"VERSION ""

NS_ :
 NS_DESC_
 CM_
 BA_DEF_
 BS_:

BU_: TEST
"#;

    match Dbc::parse(content) {
        Ok(dbc) => {
            println!("Success! Messages: {}", dbc.messages().len());
        }
        Err(e) => {
            println!("Error: {}", e);
            panic!("Parse failed: {}", e);
        }
    }
}
