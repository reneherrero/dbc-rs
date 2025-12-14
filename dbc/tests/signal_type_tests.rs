//! Tests for Signal Types (SGTYPE_, SIG_TYPE_REF_, SGTYPE_VAL_)

#[cfg(feature = "std")]
mod std {
    use dbc_rs::{Dbc, SignalType, SignalTypeReference, SignalTypeValue};

    // ============================================================================
    // Signal Type Definition Tests (SGTYPE_)
    // ============================================================================

    #[test]
    fn test_parse_sgtype_definition() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SGTYPE_");
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 1);
        assert_eq!(signal_types[0].name(), "SignalType1");
        assert_eq!(signal_types[0].size(), 16);
    }

    #[test]
    fn test_parse_sgtype_multiple() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;
SGTYPE_ SignalType2 : 32;
SGTYPE_ SignalType3 : 8;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse multiple SGTYPE_");
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 3);
        assert_eq!(signal_types[0].name(), "SignalType1");
        assert_eq!(signal_types[0].size(), 16);
        assert_eq!(signal_types[1].name(), "SignalType2");
        assert_eq!(signal_types[1].size(), 32);
        assert_eq!(signal_types[2].name(), "SignalType3");
        assert_eq!(signal_types[2].size(), 8);
    }

    #[test]
    fn test_parse_sgtype_without_semicolon() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SGTYPE_ without semicolon");
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 1);
        assert_eq!(signal_types[0].name(), "SignalType1");
        assert_eq!(signal_types[0].size(), 16);
    }

    // ============================================================================
    // Signal Type Reference Tests (SIG_TYPE_REF_)
    // ============================================================================

    #[test]
    fn test_parse_sig_type_ref() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

SIG_TYPE_REF_ 256 RPM : SignalType1;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SIG_TYPE_REF_");
        let refs = dbc.signal_type_references();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].message_id(), 256);
        assert_eq!(refs[0].signal_name(), "RPM");
        assert_eq!(refs[0].type_name(), "SignalType1");
    }

    #[test]
    fn test_parse_sig_type_ref_multiple() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;
SGTYPE_ SignalType2 : 8;

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@1+ (1,-40) [-40|215] "°C" *

SIG_TYPE_REF_ 256 RPM : SignalType1;
SIG_TYPE_REF_ 256 Temperature : SignalType2;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse multiple SIG_TYPE_REF_");
        let refs = dbc.signal_type_references();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].message_id(), 256);
        assert_eq!(refs[0].signal_name(), "RPM");
        assert_eq!(refs[0].type_name(), "SignalType1");
        assert_eq!(refs[1].message_id(), 256);
        assert_eq!(refs[1].signal_name(), "Temperature");
        assert_eq!(refs[1].type_name(), "SignalType2");
    }

    #[test]
    fn test_parse_sig_type_ref_without_semicolon() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

SIG_TYPE_REF_ 256 RPM : SignalType1
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SIG_TYPE_REF_ without semicolon");
        let refs = dbc.signal_type_references();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].message_id(), 256);
        assert_eq!(refs[0].signal_name(), "RPM");
        assert_eq!(refs[0].type_name(), "SignalType1");
    }

    // ============================================================================
    // Signal Type Value Tests (SGTYPE_VAL_)
    // ============================================================================

    #[test]
    fn test_parse_sgtype_val() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;

SGTYPE_VAL_ SignalType1 0 "Value0" 1 "Value1" 2 "Value2" ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SGTYPE_VAL_");
        let values = dbc.signal_type_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0].type_name(), "SignalType1");
        assert_eq!(values[0].value(), 0);
        assert_eq!(values[0].description(), "Value0");
        assert_eq!(values[1].value(), 1);
        assert_eq!(values[1].description(), "Value1");
        assert_eq!(values[2].value(), 2);
        assert_eq!(values[2].description(), "Value2");
    }

    #[test]
    fn test_parse_sgtype_val_multiple_types() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;
SGTYPE_ SignalType2 : 8;

SGTYPE_VAL_ SignalType1 0 "Zero" 1 "One" ;
SGTYPE_VAL_ SignalType2 0 "Off" 1 "On" 2 "Error" ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse multiple SGTYPE_VAL_");
        let values = dbc.signal_type_values();
        assert_eq!(values.len(), 5);

        // Check SignalType1 values
        let type1_values: Vec<&SignalTypeValue> =
            values.iter().filter(|v| v.type_name() == "SignalType1").collect();
        assert_eq!(type1_values.len(), 2);
        assert_eq!(type1_values[0].value(), 0);
        assert_eq!(type1_values[0].description(), "Zero");
        assert_eq!(type1_values[1].value(), 1);
        assert_eq!(type1_values[1].description(), "One");

        // Check SignalType2 values
        let type2_values: Vec<&SignalTypeValue> =
            values.iter().filter(|v| v.type_name() == "SignalType2").collect();
        assert_eq!(type2_values.len(), 3);
        assert_eq!(type2_values[0].value(), 0);
        assert_eq!(type2_values[0].description(), "Off");
        assert_eq!(type2_values[1].value(), 1);
        assert_eq!(type2_values[1].description(), "On");
        assert_eq!(type2_values[2].value(), 2);
        assert_eq!(type2_values[2].description(), "Error");
    }

    #[test]
    fn test_parse_sgtype_val_single_value() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;

SGTYPE_VAL_ SignalType1 0 "Zero" ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SGTYPE_VAL_ with single value");
        let values = dbc.signal_type_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].type_name(), "SignalType1");
        assert_eq!(values[0].value(), 0);
        assert_eq!(values[0].description(), "Zero");
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_parse_all_signal_types_together() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;
SGTYPE_ SignalType2 : 8;

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Status : 16|8@1+ (1,0) [0|255] "" *

SIG_TYPE_REF_ 256 RPM : SignalType1;
SIG_TYPE_REF_ 256 Status : SignalType2;

SGTYPE_VAL_ SignalType1 0 "Idle" 1000 "Running" 2000 "Max" ;
SGTYPE_VAL_ SignalType2 0 "Off" 1 "On" 2 "Error" ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse all signal types together");

        // Check signal types
        assert_eq!(dbc.signal_types().len(), 2);

        // Check signal type references
        assert_eq!(dbc.signal_type_references().len(), 2);

        // Check signal type values
        assert_eq!(dbc.signal_type_values().len(), 5);

        // Verify message exists
        let message = dbc.messages().find_by_id(256).unwrap();
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.signals().len(), 2);
    }
}
