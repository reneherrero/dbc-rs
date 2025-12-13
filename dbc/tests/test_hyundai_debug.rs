#[cfg(feature = "std")]
#[test]
fn test_hyundai_debug() {
    use dbc_rs::Dbc;

    // Test with just the beginning of the file
    let content = r#"VERSION ""

NS_ :
 NS_DESC_
 CM_
 BA_DEF_
 BA_
 VAL_
 CAT_DEF_
 CAT_
 FILTER
 BA_DEF_DEF_
 EV_DATA_
 ENVVAR_DATA_
 SGTYPE_
 SGTYPE_VAL_
 BA_DEF_SGTYPE_
 BA_SGTYPE_
 SIG_TYPE_REF_
 VAL_TABLE_
 SIG_GROUP_
 SIG_VALTYPE_
 SIGTYPE_VALTYPE_
 BO_TX_BU_
 BA_DEF_REL_
 BA_REL_
 BA_DEF_DEF_REL_
 BU_SG_REL_
 BU_EV_REL_
 BU_BO_REL_
 SG_MUL_VAL_

BS_:

BU_: TEST
"#;

    match Dbc::parse(content) {
        Ok(dbc) => {
            println!("Success! Messages: {}", dbc.messages().len());
            assert!(true);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            panic!("Parse failed: {}", e);
        }
    }
}
