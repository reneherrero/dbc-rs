//! Tests for features that are parsed but not yet fully implemented
//!
//! These tests verify that the parser correctly handles these DBC entries
//! without crashing, even though full functionality may not be implemented yet.

#[cfg(feature = "std")]
mod std {
    use dbc_rs::Dbc;

    // ============================================================================
    // Environment Variables Tests (Section 12)
    // ============================================================================

    #[test]
    fn test_parse_envvar_basic() {
        // EV_ with basic access type
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

EV_ EnvVar1 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse without error");
        // Verify basic structure is still parsed
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_envvar_all_access_types() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

EV_ EnvVar1 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;
EV_ EnvVar2 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR1;
EV_ EnvVar3 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR2;
EV_ EnvVar4 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR3;
EV_ EnvVar5 : 0 [0|100] "unit" 0 0 8000;
EV_ EnvVar6 : 0 [0|100] "unit" 0 0 8001;
EV_ EnvVar7 : 0 [0|100] "unit" 0 0 8002;
EV_ EnvVar8 : 0 [0|100] "unit" 0 0 8003;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse all access types");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_envvar_types() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

EV_ EnvVarInt : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;
EV_ EnvVarFloat : 1 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;
EV_ EnvVarString : 2 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse all types");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_envvar_data() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

EV_ EnvVar1 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;

ENVVAR_DATA_ EnvVar1 : 0x00 0x01 0x02 0x03;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse ENVVAR_DATA_");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_envvar_val() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

EV_ EnvVar1 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;

VAL_ EnvVar1 0 "Value0" 1 "Value1" 2 "Value2" ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse VAL_ for environment variables");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_bu_ev_rel() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM TCM

EV_ EnvVar1 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;

BU_EV_REL_ : ECM EnvVar1;
BU_EV_REL_ : TCM EnvVar1;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse BU_EV_REL_");
        assert_eq!(dbc.messages().len(), 0);
    }

    // ============================================================================
    // Signal Types Tests (Section 13.1)
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
    #[ignore = "BA_DEF_SGTYPE_ parsing not yet implemented"]
    fn test_parse_ba_def_sgtype() {
        // BA_DEF_SGTYPE_ is not yet implemented in the parser
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;

BA_DEF_SGTYPE_ "SignalTypeAttr" INT 0 100;
"#;

        // Currently fails because BA_DEF_SGTYPE_ is not in the keyword match
        let result = Dbc::parse(dbc_content);
        assert!(
            result.is_err(),
            "BA_DEF_SGTYPE_ parsing not yet implemented"
        );
    }

    #[test]
    #[ignore = "BA_SGTYPE_ parsing not yet implemented"]
    fn test_parse_ba_sgtype() {
        // BA_SGTYPE_ is not yet implemented in the parser
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

SGTYPE_ SignalType1 : 16;

BA_DEF_SGTYPE_ "SignalTypeAttr" INT 0 100;
BA_SGTYPE_ SignalType1 "SignalTypeAttr" 42;
"#;

        // Currently fails because BA_SGTYPE_ is not in the keyword match
        let result = Dbc::parse(dbc_content);
        assert!(result.is_err(), "BA_SGTYPE_ parsing not yet implemented");
    }

    // ============================================================================
    // Bit Timing Tests (Section 5)
    // ============================================================================

    #[test]
    fn test_parse_bs_empty() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BS_:
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse empty BS_");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_bs_with_baudrate() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BS_: 500
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse BS_ with baudrate");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_bs_with_btr() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BS_: 500 : 12,34
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse BS_ with BTR values");
        assert_eq!(dbc.messages().len(), 0);
    }

    // ============================================================================
    // Message Transmitters Tests (Section 11)
    // ============================================================================

    #[test]
    fn test_parse_bo_tx_bu_single() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

BO_TX_BU_ 256 : ECM;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse BO_TX_BU_ with single transmitter");
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_bo_tx_bu_multiple() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM TCM BCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

BO_TX_BU_ 256 : ECM, TCM, BCM;
"#;

        let dbc =
            Dbc::parse(dbc_content).expect("Should parse BO_TX_BU_ with multiple transmitters");
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_bo_tx_bu_multiple_messages() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

BO_ 512 TransmissionData : 8 TCM
 SG_ Gear : 0|8@1+ (1,0) [0|5] "" *

BO_TX_BU_ 256 : ECM;
BO_TX_BU_ 512 : TCM;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse BO_TX_BU_ for multiple messages");
        assert_eq!(dbc.messages().len(), 2);
    }

    // ============================================================================
    // Signal Groups Tests (Section 13.2)
    // ============================================================================

    #[test]
    fn test_parse_sig_group_basic() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@1+ (1,-40) [-40|215] "°C" *

SIG_GROUP_ 256 EngineSignals 1 : RPM Temperature;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SIG_GROUP_");
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_sig_group_multiple() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@1+ (1,-40) [-40|215] "°C" *
 SG_ Throttle : 24|8@1+ (0.392157,0) [0|100] "%" *

SIG_GROUP_ 256 EngineSignals 1 : RPM Temperature;
SIG_GROUP_ 256 ThrottleGroup 1 : Throttle;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse multiple SIG_GROUP_ entries");
        assert_eq!(dbc.messages().len(), 1);
    }

    #[test]
    fn test_parse_sig_group_with_repetitions() {
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *
 SG_ Temperature : 16|8@1+ (1,-40) [-40|215] "°C" *

SIG_GROUP_ 256 EngineSignals 5 : RPM Temperature;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse SIG_GROUP_ with repetitions");
        assert_eq!(dbc.messages().len(), 1);
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_parse_all_implemented_features_together() {
        // Test all features that are currently implemented (skipped but don't crash)
        let dbc_content = r#"
VERSION "1.0"

BU_: ECM TCM

BS_: 500

EV_ EnvVar1 : 0 [0|100] "unit" 0 0 DUMMY_NODE_VECTOR0;

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

SIG_GROUP_ 256 EngineSignals 1 : RPM;

BO_TX_BU_ 256 : ECM;

ENVVAR_DATA_ EnvVar1 : 0x00 0x01;

BU_EV_REL_ : ECM EnvVar1;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse all implemented features together");
        assert_eq!(dbc.messages().len(), 1);
        let message = dbc.messages().find_by_id(256).unwrap();
        assert_eq!(message.name(), "EngineData");
    }
}
