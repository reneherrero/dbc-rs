#[cfg(feature = "std")]
#[test]
fn test_tesla_bu_format() {
    use dbc_rs::Dbc;

    // Test with exact format from tesla_powertrain.dbc (nodes on separate lines with tabs)
    let content = r#"VERSION ""

NS_ :
	NS_DESC_
	CM_
	BA_DEF_
BS_:

BU_:
	NEO
	MCU
	GTW
	EPAS
	DI
	ESP
	SBW
	STW
	APP
	DAS
	XXX

BO_ 262 DI_torque1: 8 DI
 SG_ DI_torqueDriver : 0|13@1- (0.25,0) [-750|750] "Nm"  NEO
"#;

    match Dbc::parse(content) {
        Ok(dbc) => {
            println!(
                "Success! Nodes: {}, Messages: {}",
                dbc.nodes().len(),
                dbc.messages().len()
            );
            assert_eq!(dbc.nodes().len(), 11);
            assert!(dbc.nodes().contains("NEO"));
            assert!(dbc.nodes().contains("MCU"));
            assert!(true);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            panic!("Parse failed: {}", e);
        }
    }
}
