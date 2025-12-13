#[cfg(feature = "std")]
#[test]
fn test_rivian_format() {
    use dbc_rs::Dbc;

    // Test with exact format from rivian_primary_actuator.dbc (tabs in NS_)
    let content = r#"VERSION "PrimaryActuatorCAN"


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

BU_: ACM CGM EPAS_P ESP IBM OCS RCM SAS TestTool VDM Vector_XXX


BO_ 64 SAS_Status: 8 SAS
 SG_ SAS_Status_Checksum : 7|8@0+ (1,0) [0|255] "Unitless" ACM,EPAS_P,ESP,RCM,VDM
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
