#[cfg(feature = "std")]
#[test]
fn test_ns_with_tabs() {
    use dbc_rs::Dbc;

    // Test with tabs in NS_ section (like rivian_primary_actuator.dbc)
    let content = "VERSION \"\"\n\nNS_ :\n\tNS_DESC_\n\tCM_\n\tBA_DEF_\nBS_:\n\nBU_: TEST\n";

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
