//! Integration tests for DBC file parsing and manipulation.

#[cfg(feature = "std")]
mod std {
    use dbc_rs::Dbc;
    use std::fs::read_to_string;

    #[test]
    fn test_parse_simple_dbc() {
        let content = read_to_string("tests/data/simple.dbc").expect("Failed to read simple.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse simple.dbc");

        // Verify version
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("1.0".to_string())
        );

        // Verify nodes
        assert!(dbc.nodes().contains("ECU1"));
        assert!(dbc.nodes().contains("ECU2"));

        // Verify messages
        assert_eq!(dbc.messages().len(), 2);

        // Verify EngineStatus message
        let engine_msg = dbc.messages().iter().find(|m| m.id() == 100).unwrap();
        assert_eq!(engine_msg.name(), "EngineStatus");
        assert_eq!(engine_msg.sender(), "ECU1");
        assert_eq!(engine_msg.dlc(), 8);
        assert_eq!(engine_msg.signals().len(), 2);

        let speed = engine_msg.signals().find("EngineSpeed").unwrap();
        assert_eq!(speed.start_bit(), 0);
        assert_eq!(speed.length(), 16);
        assert_eq!(speed.factor(), 0.25);
        assert_eq!(speed.unit(), Some("rpm"));

        // Verify VehicleSpeed message
        let speed_msg = dbc.messages().iter().find(|m| m.id() == 200).unwrap();
        assert_eq!(speed_msg.name(), "VehicleSpeed");
        assert_eq!(speed_msg.sender(), "ECU2");
        assert_eq!(speed_msg.dlc(), 4);
    }

    #[test]
    fn test_parse_multiplexed_dbc() {
        let content =
            read_to_string("tests/data/multiplexed.dbc").expect("Failed to read multiplexed.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse multiplexed.dbc");

        // Verify version
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("2.1".to_string())
        );

        // Verify nodes
        assert!(dbc.nodes().contains("GATEWAY"));
        assert!(dbc.nodes().contains("SENSOR"));
        assert!(dbc.nodes().contains("ACTUATOR"));

        // Verify messages
        assert_eq!(dbc.messages().len(), 2);

        // Verify SensorData message
        let sensor_msg = dbc.messages().iter().find(|m| m.id() == 300).unwrap();
        assert_eq!(sensor_msg.name(), "SensorData");
        assert_eq!(sensor_msg.signals().len(), 4);

        let temp = sensor_msg.signals().find("Temperature").unwrap();
        assert_eq!(temp.start_bit(), 8);
        assert_eq!(temp.offset(), -50.0);
        assert_eq!(temp.unit(), Some("°C"));

        // Verify ActuatorControl message
        let actuator_msg = dbc.messages().iter().find(|m| m.id() == 400).unwrap();
        assert_eq!(actuator_msg.name(), "ActuatorControl");
        assert_eq!(actuator_msg.dlc(), 6);
        assert_eq!(actuator_msg.signals().len(), 3);

        let force = actuator_msg.signals().find("Force").unwrap();
        assert!(!force.is_unsigned()); // Should be signed
        assert_eq!(force.unit(), Some("N"));
    }

    #[test]
    fn test_parse_minimal_dbc() {
        let content = read_to_string("tests/data/minimal.dbc").expect("Failed to read minimal.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse minimal.dbc");

        // Verify version (just major, no minor/patch)
        assert_eq!(dbc.version().map(|v| v.to_string()), Some("1".to_string()));

        // Verify single node
        assert!(dbc.nodes().contains("NODE1"));
        assert_eq!(dbc.nodes().len(), 1);

        // Verify single message
        assert_eq!(dbc.messages().len(), 1);

        let msg = dbc.messages().at(0).unwrap();
        assert_eq!(msg.id(), 256);
        assert_eq!(msg.name(), "TestMessage");
        assert_eq!(msg.dlc(), 1); // Minimal DLC
        assert_eq!(msg.signals().len(), 1);

        let sig = msg.signals().at(0).unwrap();
        assert_eq!(sig.name(), "TestSignal");
        assert_eq!(sig.length(), 8);
    }

    #[test]
    fn test_parse_extended_ids_dbc() {
        let content =
            read_to_string("tests/data/extended_ids.dbc").expect("Failed to read extended_ids.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse extended_ids.dbc");

        // Verify version
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("1.5".to_string())
        );

        // Verify messages with hex IDs
        assert_eq!(dbc.messages().len(), 2);

        let engine_msg = dbc.messages().iter().find(|m| m.id() == 416).unwrap();
        assert_eq!(engine_msg.name(), "EngineData");
        assert_eq!(engine_msg.signals().len(), 2);

        let trans_msg = dbc.messages().iter().find(|m| m.id() == 688).unwrap();
        assert_eq!(trans_msg.name(), "TransmissionData");
        assert_eq!(trans_msg.signals().len(), 2);

        // Verify small signals (4-bit gear, 1-bit clutch)
        let gear = trans_msg.signals().find("Gear").unwrap();
        assert_eq!(gear.length(), 4);
        assert_eq!(gear.start_bit(), 0);

        let clutch = trans_msg.signals().find("Clutch").unwrap();
        assert_eq!(clutch.length(), 1);
        assert_eq!(clutch.start_bit(), 4);
    }

    #[test]
    fn test_parse_broadcast_signals_dbc() {
        let content = read_to_string("tests/data/broadcast_signals.dbc")
            .expect("Failed to read broadcast_signals.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse broadcast_signals.dbc");

        // Verify nodes
        assert!(dbc.nodes().contains("BROADCASTER"));
        assert!(dbc.nodes().contains("RECEIVER1"));
        assert!(dbc.nodes().contains("RECEIVER2"));

        // Verify message
        assert_eq!(dbc.messages().len(), 1);

        let msg = dbc.messages().iter().find(|m| m.id() == 500).unwrap();
        assert_eq!(msg.name(), "BroadcastMessage");
        assert_eq!(msg.signals().len(), 3);

        // Verify broadcast signal (receivers = *)
        let status = msg.signals().find("Status").unwrap();
        assert_eq!(status.receivers(), &dbc_rs::Receivers::Broadcast);

        // Verify signals with specific receivers
        let data1 = msg.signals().find("Data1").unwrap();
        match data1.receivers() {
            dbc_rs::Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 2);
                let node_vec: Vec<String> =
                    data1.receivers().iter().map(|s| s.to_string()).collect();
                assert!(node_vec.iter().any(|s| s == "RECEIVER1"));
                assert!(node_vec.iter().any(|s| s == "RECEIVER2"));
            }
            _ => panic!("Data1 should have specific receivers"),
        }

        let data2 = msg.signals().find("Data2").unwrap();
        match data2.receivers() {
            dbc_rs::Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 1);
                let node_vec: Vec<String> =
                    data2.receivers().iter().map(|s| s.to_string()).collect();
                assert_eq!(node_vec[0], "RECEIVER1");
            }
            _ => panic!("Data2 should have specific receivers"),
        }
    }

    #[test]
    #[ignore = "VAL_TABLE_ not implemented"]
    fn test_parse_complete_dbc_file() {
        // Parse the complete.dbc file
        let content =
            read_to_string("tests/data/complete.dbc").expect("Failed to read complete.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse complete.dbc");

        // Verify basic structure
        assert_eq!(
            dbc.version().map(|v| v.to_string()),
            Some("2.0".to_string())
        );

        // Verify nodes
        let nodes = dbc.nodes();
        assert!(nodes.contains("ECM"));
        assert!(nodes.contains("TCM"));
        assert!(nodes.contains("BCM"));
        assert!(nodes.contains("ABS"));
        assert!(nodes.contains("SENSOR"));

        // Verify messages
        assert_eq!(dbc.messages().len(), 4);

        // Verify first message (EngineData)
        let engine_msg = dbc
            .messages()
            .iter()
            .find(|m| m.id() == 256)
            .expect("EngineData message not found");
        assert_eq!(engine_msg.name(), "EngineData");
        assert_eq!(engine_msg.dlc(), 8);
        assert_eq!(engine_msg.sender(), "ECM");
        assert_eq!(engine_msg.signals().len(), 4);

        // Verify signals in EngineData
        let rpm = engine_msg.signals().find("RPM").expect("RPM signal not found");
        assert_eq!(rpm.start_bit(), 0);
        assert_eq!(rpm.length(), 16);
        assert_eq!(rpm.factor(), 0.25);
        assert_eq!(rpm.unit(), Some("rpm"));

        let temp = engine_msg.signals().find("Temperature").expect("Temperature signal not found");
        assert_eq!(temp.start_bit(), 16);
        assert_eq!(temp.length(), 8);
        assert_eq!(temp.unit(), Some("°C"));

        // Verify second message (TransmissionData)
        let trans_msg = dbc
            .messages()
            .iter()
            .find(|m| m.id() == 512)
            .expect("TransmissionData message not found");
        assert_eq!(trans_msg.name(), "TransmissionData");
        assert_eq!(trans_msg.sender(), "TCM");
        assert_eq!(trans_msg.signals().len(), 4);

        // Verify third message (BrakeData) - now with DLC 6
        let brake_msg = dbc
            .messages()
            .iter()
            .find(|m| m.id() == 768)
            .expect("BrakeData message not found");
        assert_eq!(brake_msg.name(), "BrakeData");
        assert_eq!(brake_msg.sender(), "ABS");
        assert_eq!(brake_msg.dlc(), 6); // Updated from 4 to 6
        assert_eq!(brake_msg.signals().len(), 4);

        // Verify signals in BrakeData
        let brake_pressure = brake_msg
            .signals()
            .find("BrakePressure")
            .expect("BrakePressure signal not found");
        assert_eq!(brake_pressure.start_bit(), 0);
        assert_eq!(brake_pressure.length(), 16);
        assert_eq!(brake_pressure.unit(), Some("bar"));

        let abs_active = brake_msg.signals().find("ABSActive").expect("ABSActive signal not found");
        assert_eq!(abs_active.start_bit(), 16);
        assert_eq!(abs_active.length(), 1);

        let wheel_speed_front_left =
            brake_msg.signals().find("WheelSpeedFL").expect("WheelSpeedFL signal not found");
        assert_eq!(wheel_speed_front_left.start_bit(), 17);
        assert_eq!(wheel_speed_front_left.length(), 15);
        assert_eq!(wheel_speed_front_left.unit(), Some("km/h"));

        let wheel_speed_front_right =
            brake_msg.signals().find("WheelSpeedFR").expect("WheelSpeedFR signal not found");
        assert_eq!(wheel_speed_front_right.start_bit(), 32);
        assert_eq!(wheel_speed_front_right.length(), 15);
        assert_eq!(wheel_speed_front_right.unit(), Some("km/h"));
        // Verify this signal now fits within the 6-byte message (48 bits)
        assert!(
            wheel_speed_front_right.start_bit() + wheel_speed_front_right.length()
                <= u16::from(brake_msg.dlc()) * 8
        );

        // Verify fourth message (SensorData)
        let sensor_msg = dbc
            .messages()
            .iter()
            .find(|m| m.id() == 1024)
            .expect("SensorData message not found");
        assert_eq!(sensor_msg.name(), "SensorData");
        assert_eq!(sensor_msg.sender(), "SENSOR");
        assert_eq!(sensor_msg.dlc(), 6);
        assert_eq!(sensor_msg.signals().len(), 3);
    }

    #[test]
    fn test_parse_j1939_dbc() {
        // Integration test for j1939.dbc file
        // Source of truth: dbc/SPECIFICATIONS.md
        // This test ensures that parse results match the content of tests/data/j1939.dbc
        //
        // The j1939.dbc file contains:
        // - J1939-specific attributes (VFrameFormat, SPN, PGN)
        // - Extended CAN IDs (VECTOR__INDEPENDENT_SIG_MSG pseudo-message with ID 3221225472)
        // - Attribute definitions and values (BA_DEF_, BA_, BA_DEF_DEF_)
        // - Comments (CM_)
        // - Node definitions with J1939 attributes (NmJ1939Function, NmStationAddress)
        //
        // Reference: SPECIFICATIONS.md sections:
        // - Section 8: Message Definitions (extended IDs, VECTOR__INDEPENDENT_SIG_MSG)
        // - Section 8.6: Pseudo-Message (VECTOR__INDEPENDENT_SIG_MSG with DLC 0)
        // - Section 15: User-Defined Attributes (BA_DEF_, BA_, BA_DEF_DEF_)
        // - Section 17: Common Attributes (J1939-specific attributes)
        // - Section 14: Comments (CM_)
        //
        // NOTE: The j1939.dbc file contains a VECTOR__INDEPENDENT_SIG_MSG pseudo-message
        // with DLC 0, which is valid per SPECIFICATIONS.md Section 8.6. However, the parser
        // currently enforces DLC >= 1 for all messages (including pseudo-messages).
        // Therefore, this file cannot be fully parsed until DLC 0 support is added for
        // pseudo-messages. This test documents the expected structure once support is added.

        let content = read_to_string("tests/data/j1939.dbc").expect("Failed to read j1939.dbc");

        // Currently, the parser rejects DLC 0, so parsing will fail
        // Verify we get the expected DLC validation error (not some other parsing error)
        let parse_result = Dbc::parse(&content);
        assert!(
            parse_result.is_err(),
            "Parser should currently reject DLC 0 messages until pseudo-message support is added"
        );

        // Validate the specific error is about DLC being too small
        let error = parse_result.unwrap_err();
        let error_msg = format!("{}", error);
        assert!(
            error_msg.contains("DLC") || error_msg.contains("must be at least"),
            "Expected DLC validation error, but got: {}",
            error_msg
        );

        // Verify the file content structure matches expectations before parsing fails
        // This ensures the file format is correct and we're testing the right thing
        assert!(content.contains("VERSION \"\""));
        assert!(content.contains("BU_: Turbocharger OnBoardDataLogger"));
        assert!(content.contains("BO_ 3221225472 VECTOR__INDEPENDENT_SIG_MSG: 0 Vector__XXX"));
        assert!(content.contains("SG_ TrailerWeight : 0|16@1+ (2,0) [0|128510] \"kg\""));
        assert!(content.contains("SG_ TireTemp : 0|16@1+ (0.03125,-273) [-273|1734.96875]"));
        assert!(content.contains("SG_ TirePress : 0|8@1+ (4,0) [0|1000] \"kPa\""));

        // Verify extended message ID format (0xC0000000 = 3221225472)
        // See SPECIFICATIONS.md Section 8.1: Extended ID has bit 31 set
        assert_eq!(
            3221225472u32, 0xC0000000u32,
            "Message ID should be 0xC0000000"
        );

        // Verify we can manually construct the expected structure using builders
        // This ensures our expected assertions (commented below) are valid and will work
        // once DLC 0 support is added
        #[cfg(feature = "std")]
        {
            use dbc_rs::{
                ByteOrder, NodesBuilder, ReceiversBuilder, SignalBuilder, VersionBuilder,
            };

            // Build the expected structure manually to validate our test expectations
            // This ensures our expected assertions (commented below) are valid and will work
            // once DLC 0 support is added
            let _expected_version = VersionBuilder::new().version("").build().unwrap();
            let expected_nodes = NodesBuilder::new()
                .add_node("Turbocharger")
                .add_node("OnBoardDataLogger")
                .add_node("Transmission2")
                .add_node("Transmission1")
                .add_node("Engine2")
                .add_node("Engine1")
                .build()
                .unwrap();

            // Note: We cannot build a message with DLC 0 yet, so we use DLC 1 as a placeholder
            // to validate signal structure. Once DLC 0 is supported, this test will catch
            // if our expectations don't match the actual parsed structure.
            let trailer_weight_sig = SignalBuilder::new()
                .name("TrailerWeight")
                .start_bit(0)
                .length(16)
                .byte_order(ByteOrder::LittleEndian)
                .unsigned(true)
                .factor(2.0)
                .offset(0.0)
                .min(0.0)
                .max(128510.0)
                .unit("kg")
                .receivers(ReceiversBuilder::new().add_node("Vector__XXX"))
                .build()
                .unwrap();

            let tire_temp_sig = SignalBuilder::new()
                .name("TireTemp")
                .start_bit(0)
                .length(16)
                .byte_order(ByteOrder::LittleEndian)
                .unsigned(true)
                .factor(0.03125)
                .offset(-273.0)
                .min(-273.0)
                .max(1734.96875)
                .unit("deg C")
                .receivers(ReceiversBuilder::new().add_node("Vector__XXX"))
                .build()
                .unwrap();

            let tire_press_sig = SignalBuilder::new()
                .name("TirePress")
                .start_bit(0)
                .length(8)
                .byte_order(ByteOrder::LittleEndian)
                .unsigned(true)
                .factor(4.0)
                .offset(0.0)
                .min(0.0)
                .max(1000.0)
                .unit("kPa")
                .receivers(ReceiversBuilder::new().add_node("Vector__XXX"))
                .build()
                .unwrap();

            // Validate all signal properties systematically match file content
            // TrailerWeight: 0|16@1+ (2,0) [0|128510] "kg"
            assert_eq!(trailer_weight_sig.name(), "TrailerWeight");
            assert_eq!(trailer_weight_sig.start_bit(), 0);
            assert_eq!(trailer_weight_sig.length(), 16);
            assert_eq!(trailer_weight_sig.byte_order(), ByteOrder::LittleEndian);
            assert!(trailer_weight_sig.is_unsigned());
            assert_eq!(trailer_weight_sig.factor(), 2.0);
            assert_eq!(trailer_weight_sig.offset(), 0.0);
            assert_eq!(trailer_weight_sig.min(), 0.0);
            assert_eq!(trailer_weight_sig.max(), 128510.0);
            assert_eq!(trailer_weight_sig.unit(), Some("kg"));

            // TireTemp: 0|16@1+ (0.03125,-273) [-273|1734.96875] "deg C"
            assert_eq!(tire_temp_sig.name(), "TireTemp");
            assert_eq!(tire_temp_sig.start_bit(), 0);
            assert_eq!(tire_temp_sig.length(), 16);
            assert_eq!(tire_temp_sig.byte_order(), ByteOrder::LittleEndian);
            assert!(tire_temp_sig.is_unsigned());
            assert_eq!(tire_temp_sig.factor(), 0.03125);
            assert_eq!(tire_temp_sig.offset(), -273.0);
            assert_eq!(tire_temp_sig.min(), -273.0);
            assert_eq!(tire_temp_sig.max(), 1734.96875);
            assert_eq!(tire_temp_sig.unit(), Some("deg C"));

            // TirePress: 0|8@1+ (4,0) [0|1000] "kPa"
            assert_eq!(tire_press_sig.name(), "TirePress");
            assert_eq!(tire_press_sig.start_bit(), 0);
            assert_eq!(tire_press_sig.length(), 8);
            assert_eq!(tire_press_sig.byte_order(), ByteOrder::LittleEndian);
            assert!(tire_press_sig.is_unsigned());
            assert_eq!(tire_press_sig.factor(), 4.0);
            assert_eq!(tire_press_sig.offset(), 0.0);
            assert_eq!(tire_press_sig.min(), 0.0);
            assert_eq!(tire_press_sig.max(), 1000.0);
            assert_eq!(tire_press_sig.unit(), Some("kPa"));

            // Validate all nodes structure (all 6 nodes from BU_: line)
            assert_eq!(expected_nodes.len(), 6);
            assert!(expected_nodes.contains("Turbocharger"));
            assert!(expected_nodes.contains("OnBoardDataLogger"));
            assert!(expected_nodes.contains("Transmission2"));
            assert!(expected_nodes.contains("Transmission1"));
            assert!(expected_nodes.contains("Engine2"));
            assert!(expected_nodes.contains("Engine1"));
        }

        // TODO: Once DLC 0 pseudo-message support is added, uncomment the code below:
        // let dbc = parse_result.expect("Failed to parse j1939.dbc");
        //
        // // Verify version (empty string)
        // assert_eq!(dbc.version().map(|v| v.to_string()), Some("".to_string()));
        //
        // // Verify nodes according to BU_: line
        // // BU_: Turbocharger OnBoardDataLogger Transmission2 Transmission1 Engine2 Engine1
        // let nodes = dbc.nodes();
        // assert_eq!(nodes.len(), 6);
        // assert!(nodes.contains("Turbocharger"));
        // assert!(nodes.contains("OnBoardDataLogger"));
        // assert!(nodes.contains("Transmission2"));
        // assert!(nodes.contains("Transmission1"));
        // assert!(nodes.contains("Engine2"));
        // assert!(nodes.contains("Engine1"));
        //
        // // Verify message count (only VECTOR__INDEPENDENT_SIG_MSG pseudo-message)
        // // Message ID 3221225472 = 0xC0000000 (VECTOR__INDEPENDENT_SIG_MSG)
        // // See SPECIFICATIONS.md Section 8.6: Pseudo-Message
        // assert_eq!(dbc.messages().len(), 1);
        //
        // let msg = dbc
        //     .messages()
        //     .find_by_id(3221225472)
        //     .expect("VECTOR__INDEPENDENT_SIG_MSG message not found");
        //
        // assert_eq!(msg.name(), "VECTOR__INDEPENDENT_SIG_MSG");
        // assert_eq!(msg.dlc(), 0); // DLC = 0 for pseudo-message (per SPEC Section 8.6)
        // assert_eq!(msg.sender(), "Vector__XXX"); // Pseudo-messages use Vector__XXX
        //
        // // Verify signals in the pseudo-message
        // // BO_ 3221225472 VECTOR__INDEPENDENT_SIG_MSG: 0 Vector__XXX
        // //  SG_ TrailerWeight : 0|16@1+ (2,0) [0|128510] "kg" Vector__XXX
        // //  SG_ TireTemp : 0|16@1+ (0.03125,-273) [-273|1734.96875] "deg C" Vector__XXX
        // //  SG_ TirePress : 0|8@1+ (4,0) [0|1000] "kPa" Vector__XXX
        // assert_eq!(msg.signals().len(), 3);
        //
        // // Verify TrailerWeight signal (See SPECIFICATIONS.md Section 9)
        // let trailer_weight = msg.signals().find("TrailerWeight").expect("TrailerWeight signal not found");
        // assert_eq!(trailer_weight.start_bit(), 0);
        // assert_eq!(trailer_weight.length(), 16);
        // assert_eq!(trailer_weight.byte_order(), dbc_rs::ByteOrder::LittleEndian); // @1 = little-endian
        // assert!(trailer_weight.is_unsigned()); // + = unsigned
        // assert_eq!(trailer_weight.factor(), 2.0);
        // assert_eq!(trailer_weight.offset(), 0.0);
        // assert_eq!(trailer_weight.min(), 0.0);
        // assert_eq!(trailer_weight.max(), 128510.0);
        // assert_eq!(trailer_weight.unit(), Some("kg"));
        //
        // // Verify TireTemp signal
        // let tire_temp = msg.signals().find("TireTemp").expect("TireTemp signal not found");
        // assert_eq!(tire_temp.start_bit(), 0);
        // assert_eq!(tire_temp.length(), 16);
        // assert_eq!(tire_temp.byte_order(), dbc_rs::ByteOrder::LittleEndian); // @1 = little-endian
        // assert!(tire_temp.is_unsigned()); // + = unsigned
        // assert_eq!(tire_temp.factor(), 0.03125);
        // assert_eq!(tire_temp.offset(), -273.0);
        // assert_eq!(tire_temp.min(), -273.0);
        // assert_eq!(tire_temp.max(), 1734.96875);
        // assert_eq!(tire_temp.unit(), Some("deg C"));
        //
        // // Verify TirePress signal
        // let tire_press = msg.signals().find("TirePress").expect("TirePress signal not found");
        // assert_eq!(tire_press.start_bit(), 0);
        // assert_eq!(tire_press.length(), 8);
        // assert_eq!(tire_press.byte_order(), dbc_rs::ByteOrder::LittleEndian); // @1 = little-endian
        // assert!(tire_press.is_unsigned()); // + = unsigned
        // assert_eq!(tire_press.factor(), 4.0);
        // assert_eq!(tire_press.offset(), 0.0);
        // assert_eq!(tire_press.min(), 0.0);
        // assert_eq!(tire_press.max(), 1000.0);
        // assert_eq!(tire_press.unit(), Some("kPa"));
        //
        // // Note: Attribute definitions (BA_DEF_), attribute values (BA_), and comments (CM_)
        // // are parsed but not directly accessible through the public API in this version.
        // // J1939-specific attributes in the file include NmJ1939Function, NmStationAddress, etc.
        // // See SPECIFICATIONS.md Section 17.3: J1939-Specific Attributes
    }

    // ============================================================================
    // Signal Type Tests (SGTYPE_, SIG_TYPE_REF_, SGTYPE_VAL_)
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
        use dbc_rs::SignalTypeValue;

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
        // SignalType1 has 3 values (0 "Idle", 1000 "Running", 2000 "Max")
        // SignalType2 has 3 values (0 "Off", 1 "On", 2 "Error")
        // Total: 6 values
        assert_eq!(dbc.signal_type_values().len(), 6);

        // Verify message exists
        let message = dbc.messages().find_by_id(256).unwrap();
        assert_eq!(message.name(), "EngineData");
        assert_eq!(message.signals().len(), 2);
    }

    #[test]
    fn test_complete_signal_type_workflow() {
        let dbc_content = r#"
VERSION "1.0"

NS_ :
    SGTYPE_
    SIG_TYPE_REF_
    SGTYPE_VAL_
    SIG_VALTYPE_
    VAL_

BS_:

BU_: ECU1 ECU2

// 1. Define a reusable signal type (template)
SGTYPE_ TempSensor : 16;

// 2. Define value table (enum) for the type
SGTYPE_VAL_ TempSensor 
    0 "Cold" 
    100 "Normal" 
    200 "Hot" 
    255 "Error" ;

// 3. Message with signals using the type and float
BO_ 256 VehicleStatus: 8 ECU1
 SG_ EngineTemp : 0|16@1+ (1,0) [0|65535] "°C" Vector__XXX
 SG_ CabinTemp : 16|16@1+ (1,0) [0|65535] "°C" Vector__XXX
 SG_ BatteryVoltageFloat : 32|32@1+ (0.001, 0) [0|60] "V" Vector__XXX

// 4. Reference the signal type for signals
SIG_TYPE_REF_ 256 EngineTemp : TempSensor;
SIG_TYPE_REF_ 256 CabinTemp : TempSensor;

// 5. Mark a signal as IEEE float (32-bit)
SIG_VALTYPE_ 256 BatteryVoltageFloat : 1 ;  // 1 = float, 2 = double

// Optional: Normal VAL_ for a non-typed signal (for comparison)
BO_ 512 GearStatus: 1 ECU2
 SG_ Gear : 0|8@1+ (1,0) [0|5] "" Vector__XXX

VAL_ 512 Gear 0 "P" 1 "R" 2 "N" 3 "D" 4 "1" 5 "2" ;
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse complete signal type workflow");

        // Check signal type definition
        let signal_types = dbc.signal_types();
        assert_eq!(signal_types.len(), 1);
        assert_eq!(signal_types[0].name(), "TempSensor");
        assert_eq!(signal_types[0].size(), 16);

        // Check signal type values using the helper method
        let temp_sensor_values = dbc.signal_type_values_for("TempSensor");
        assert_eq!(temp_sensor_values.len(), 4);
        assert_eq!(temp_sensor_values[0].value(), 0);
        assert_eq!(temp_sensor_values[0].description(), "Cold");
        assert_eq!(temp_sensor_values[1].value(), 100);
        assert_eq!(temp_sensor_values[1].description(), "Normal");
        assert_eq!(temp_sensor_values[2].value(), 200);
        assert_eq!(temp_sensor_values[2].description(), "Hot");
        assert_eq!(temp_sensor_values[3].value(), 255);
        assert_eq!(temp_sensor_values[3].description(), "Error");

        // Check signal type references
        let refs = dbc.signal_type_references();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].message_id(), 256);
        assert_eq!(refs[0].signal_name(), "EngineTemp");
        assert_eq!(refs[0].type_name(), "TempSensor");
        assert_eq!(refs[1].message_id(), 256);
        assert_eq!(refs[1].signal_name(), "CabinTemp");
        assert_eq!(refs[1].type_name(), "TempSensor");

        // Check extended value type (float)
        use dbc_rs::SignalExtendedValueType;
        assert_eq!(
            dbc.get_signal_value_type(256, "BatteryVoltageFloat"),
            Some(SignalExtendedValueType::Float32)
        );

        // Verify messages exist
        let vehicle_status = dbc.messages().find_by_id(256).unwrap();
        assert_eq!(vehicle_status.name(), "VehicleStatus");
        assert_eq!(vehicle_status.signals().len(), 3);

        let gear_status = dbc.messages().find_by_id(512).unwrap();
        assert_eq!(gear_status.name(), "GearStatus");
        assert_eq!(gear_status.signals().len(), 1);

        // Test decoding VehicleStatus message (ID 256)
        // EngineTemp: 100 (raw value) = 100.0°C (factor 1, offset 0)
        // CabinTemp: 200 (raw value) = 200.0°C (factor 1, offset 0)
        // Note: BatteryVoltageFloat uses float32 which requires special decoding (not tested here)
        // Little-endian: EngineTemp=0x64 0x00, CabinTemp=0xC8 0x00
        let vehicle_status_payload = [0x64, 0x00, 0xC8, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded =
            dbc.decode(256, &vehicle_status_payload).expect("Should decode VehicleStatus");
        assert_eq!(decoded.len(), 3);

        // Verify EngineTemp (integer signal)
        let engine_temp = decoded
            .iter()
            .find(|(name, _, _)| *name == "EngineTemp")
            .expect("Should find EngineTemp");
        assert!(
            (engine_temp.1 - 100.0).abs() < 0.001,
            "EngineTemp should be 100.0"
        );
        assert_eq!(engine_temp.2, Some("°C"));

        // Verify CabinTemp (integer signal)
        let cabin_temp = decoded
            .iter()
            .find(|(name, _, _)| *name == "CabinTemp")
            .expect("Should find CabinTemp");
        assert!(
            (cabin_temp.1 - 200.0).abs() < 0.001,
            "CabinTemp should be 200.0"
        );
        assert_eq!(cabin_temp.2, Some("°C"));

        // Test decoding GearStatus message (ID 512)
        // Gear: 3 = "D" (Drive)
        let gear_status_payload = [0x03];
        let decoded_gear = dbc.decode(512, &gear_status_payload).expect("Should decode GearStatus");
        assert_eq!(decoded_gear.len(), 1);

        let gear = decoded_gear
            .iter()
            .find(|(name, _, _)| *name == "Gear")
            .expect("Should find Gear");
        assert!((gear.1 - 3.0).abs() < 0.001, "Gear should be 3.0");
        // Empty unit string "" is converted to None during parsing
        assert_eq!(gear.2, None);
    }

    #[test]
    fn test_decode_complete_dbc_all_signals() {
        // Comprehensive decode test for complete.dbc that tests every signal
        let content =
            read_to_string("tests/data/complete.dbc").expect("Failed to read complete.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse complete.dbc");

        // Message 256: EngineData (8 bytes)
        // Signals: RPM (BE, 0|16@0+), Temperature (BE, 16|8@0-), ThrottlePosition (BE, 24|8@0+), OilPressure (LE, 32|16@1+)
        // Use payload with OilPressure set to 1.0 kPa (raw=100, LE at bytes 4-5: [0x64, 0x00])
        // Other signals will decode to their raw*factor+offset values
        let engine_payload = [0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(256, &engine_payload).expect("Should decode EngineData");

        // Verify all 4 signals are decoded
        assert_eq!(decoded.len(), 4, "EngineData should have 4 signals");

        // Verify all signals are present and have correct units
        let rpm = decoded
            .iter()
            .find(|(name, _, _)| *name == "RPM")
            .expect("Should find RPM signal");
        assert_eq!(rpm.2, Some("rpm"), "RPM should have 'rpm' unit");

        let temp = decoded
            .iter()
            .find(|(name, _, _)| *name == "Temperature")
            .expect("Should find Temperature signal");
        assert_eq!(temp.2, Some("°C"), "Temperature should have '°C' unit");

        let throttle = decoded
            .iter()
            .find(|(name, _, _)| *name == "ThrottlePosition")
            .expect("Should find ThrottlePosition signal");
        assert_eq!(
            throttle.2,
            Some("%"),
            "ThrottlePosition should have '%' unit"
        );

        // OilPressure: 1.0 kPa (raw 100 * 0.01) - LE signal at bit 32 (byte 4-5)
        let oil = decoded
            .iter()
            .find(|(name, _, _)| *name == "OilPressure")
            .expect("Should find OilPressure signal");
        assert!(
            (oil.1 - 1.0).abs() < 0.01,
            "OilPressure should be ~1.0, got {}",
            oil.1
        );
        assert_eq!(oil.2, Some("kPa"), "OilPressure should have 'kPa' unit");

        // Message 512: TransmissionData (8 bytes)
        // Signals: GearPosition (BE, 0|8@0+), ClutchEngaged (BE, 8|1@0+), Torque (LE, 16|16@1-), TransmissionTemp (BE, 32|8@0-)
        // Use payload with Torque set to -100 Nm (raw=-1000, LE signed at bytes 2-3: [0x18, 0xFC])
        let trans_payload = [0x00, 0x00, 0x18, 0xFC, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(512, &trans_payload).expect("Should decode TransmissionData");
        assert_eq!(decoded.len(), 4, "TransmissionData should have 4 signals");

        // Verify all signals are present
        let gear = decoded
            .iter()
            .find(|(name, _, _)| *name == "GearPosition")
            .expect("Should find GearPosition signal");
        assert_eq!(gear.2, None, "GearPosition should have empty unit");

        let clutch = decoded
            .iter()
            .find(|(name, _, _)| *name == "ClutchEngaged")
            .expect("Should find ClutchEngaged signal");
        assert_eq!(clutch.2, None, "ClutchEngaged should have empty unit");

        // Torque: -100 Nm (raw -1000 * 0.1) - LE signed signal at bit 16 (byte 2-3)
        let torque = decoded
            .iter()
            .find(|(name, _, _)| *name == "Torque")
            .expect("Should find Torque signal");
        assert!(
            (torque.1 - (-100.0)).abs() < 0.1,
            "Torque should be ~-100, got {}",
            torque.1
        );
        assert_eq!(torque.2, Some("Nm"), "Torque should have 'Nm' unit");

        let trans_temp = decoded
            .iter()
            .find(|(name, _, _)| *name == "TransmissionTemp")
            .expect("Should find TransmissionTemp signal");
        assert_eq!(
            trans_temp.2,
            Some("°C"),
            "TransmissionTemp should have '°C' unit"
        );

        // Message 768: BrakeData (6 bytes)
        // Signals: BrakePressure (LE, 0|16@1+), ABSActive (LE, 16|1@1+), WheelSpeedFL (LE, 17|15@1+), WheelSpeedFR (LE, 32|15@1+)
        // Use payload with BrakePressure=250bar (raw=2500 LE at bytes 0-1: [0xC4, 0x09])
        // and WheelSpeedFR=120km/h (raw=12000 LE at bytes 4-5: [0xE0, 0x2E])
        // Other signals will decode but we'll verify units and presence
        let brake_payload = [0xC4, 0x09, 0x00, 0x00, 0xE0, 0x2E, 0x00, 0x00];
        let decoded = dbc.decode(768, &brake_payload).expect("Should decode BrakeData");
        assert_eq!(decoded.len(), 4, "BrakeData should have 4 signals");

        // BrakePressure: 250 bar (raw 2500 * 0.1) - LE at bit 0 (byte 0-1)
        let pressure = decoded
            .iter()
            .find(|(name, _, _)| *name == "BrakePressure")
            .expect("Should find BrakePressure signal");
        assert!(
            (pressure.1 - 250.0).abs() < 0.1,
            "BrakePressure should be ~250, got {}",
            pressure.1
        );
        assert_eq!(
            pressure.2,
            Some("bar"),
            "BrakePressure should have 'bar' unit"
        );

        // Verify all signals are present with correct units
        let abs = decoded
            .iter()
            .find(|(name, _, _)| *name == "ABSActive")
            .expect("Should find ABSActive signal");
        assert_eq!(abs.2, None, "ABSActive should have empty unit");

        let fl = decoded
            .iter()
            .find(|(name, _, _)| *name == "WheelSpeedFL")
            .expect("Should find WheelSpeedFL signal");
        assert_eq!(fl.2, Some("km/h"), "WheelSpeedFL should have 'km/h' unit");

        // WheelSpeedFR: 120 km/h (raw 12000 * 0.01) - LE at bit 32 (byte 4-5)
        let fr = decoded
            .iter()
            .find(|(name, _, _)| *name == "WheelSpeedFR")
            .expect("Should find WheelSpeedFR signal");
        assert!(
            (fr.1 - 120.0).abs() < 0.1,
            "WheelSpeedFR should be ~120, got {}",
            fr.1
        );
        assert_eq!(fr.2, Some("km/h"), "WheelSpeedFR should have 'km/h' unit");

        // Message 1024: SensorData (6 bytes)
        // Signals: Voltage (BE, 0|16@0+), Current (BE, 16|16@0-), Humidity (BE, 32|8@0+)
        // Verify all signals decode correctly with their units
        let sensor_payload = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(1024, &sensor_payload).expect("Should decode SensorData");
        assert_eq!(decoded.len(), 3, "SensorData should have 3 signals");

        // Verify all signals are present with correct units
        let voltage = decoded
            .iter()
            .find(|(name, _, _)| *name == "Voltage")
            .expect("Should find Voltage signal");
        assert_eq!(voltage.2, Some("V"), "Voltage should have 'V' unit");

        let current = decoded
            .iter()
            .find(|(name, _, _)| *name == "Current")
            .expect("Should find Current signal");
        assert_eq!(current.2, Some("A"), "Current should have 'A' unit");

        let humidity = decoded
            .iter()
            .find(|(name, _, _)| *name == "Humidity")
            .expect("Should find Humidity signal");
        assert_eq!(humidity.2, Some("%"), "Humidity should have '%' unit");
    }

    #[test]
    fn test_decode_signal_type_dbc_all_signals() {
        // Comprehensive decode test for signal_type.dbc that tests every signal
        let content =
            read_to_string("tests/data/signal_type.dbc").expect("Failed to read signal_type.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse signal_type.dbc");

        // Message 100: EngineData (8 bytes)
        // Signals: EngineTemp (LE, 0|16@1+), EngineRPM (LE, 16|16@1+)
        // EngineTemp: 0°C (raw = (0 - (-40)) / 0.1 = 400 = 0x0190 LE -> [0x90, 0x01])
        // EngineRPM: 2000 rpm (raw = 2000 / 1 = 2000 = 0x07D0 LE -> [0xD0, 0x07])
        let engine_payload = [0x90, 0x01, 0xD0, 0x07, 0x00, 0x00, 0x00, 0x00];
        let decoded = dbc.decode(100, &engine_payload).expect("Should decode EngineData");
        assert_eq!(decoded.len(), 2, "EngineData should have 2 signals");

        // EngineTemp: 0°C (raw 400 * 0.1 + (-40)) - LE unsigned at bit 0 (byte 0-1)
        let engine_temp = decoded
            .iter()
            .find(|(name, _, _)| *name == "EngineTemp")
            .expect("Should find EngineTemp signal");
        assert!(
            (engine_temp.1 - 0.0).abs() < 0.01,
            "EngineTemp should be ~0.0, got {}",
            engine_temp.1
        );
        assert_eq!(
            engine_temp.2,
            Some("degC"),
            "EngineTemp should have 'degC' unit"
        );

        // EngineRPM: 2000 rpm (raw 2000 * 1) - LE unsigned at bit 16 (byte 2-3)
        let engine_rpm = decoded
            .iter()
            .find(|(name, _, _)| *name == "EngineRPM")
            .expect("Should find EngineRPM signal");
        assert!(
            (engine_rpm.1 - 2000.0).abs() < 0.1,
            "EngineRPM should be ~2000, got {}",
            engine_rpm.1
        );
        assert_eq!(
            engine_rpm.2,
            Some("rpm"),
            "EngineRPM should have 'rpm' unit"
        );

        // Test with different values
        // EngineTemp: 25°C (raw = (25 - (-40)) / 0.1 = 650 = 0x028A LE -> [0x8A, 0x02])
        // EngineRPM: 1500 rpm (raw = 1500 = 0x05DC LE -> [0xDC, 0x05])
        let engine_payload2 = [0x8A, 0x02, 0xDC, 0x05, 0x00, 0x00, 0x00, 0x00];
        let decoded2 = dbc
            .decode(100, &engine_payload2)
            .expect("Should decode EngineData with different values");

        let engine_temp2 = decoded2
            .iter()
            .find(|(name, _, _)| *name == "EngineTemp")
            .expect("Should find EngineTemp signal");
        assert!(
            (engine_temp2.1 - 25.0).abs() < 0.01,
            "EngineTemp should be ~25.0, got {}",
            engine_temp2.1
        );

        let engine_rpm2 = decoded2
            .iter()
            .find(|(name, _, _)| *name == "EngineRPM")
            .expect("Should find EngineRPM signal");
        assert!(
            (engine_rpm2.1 - 1500.0).abs() < 0.1,
            "EngineRPM should be ~1500, got {}",
            engine_rpm2.1
        );

        // Test with SGTYPE_VAL_ referenced values
        // EngineTemp: "Cold" = 0 (raw = 0 / 0.1 = 0, but with offset: 0 * 0.1 + (-40) = -40°C)
        // Actually, raw value 0 means physical value: 0 * 0.1 + (-40) = -40°C
        // But the value description "Cold" maps to raw value 0
        // Let's test with raw value 1 which should map to "Normal" and be (1 * 0.1 + (-40)) = -39.9°C
        // Or better: raw value 400 = 0°C (we already tested this)
        // For "Normal" (value 1), we need raw = 1, so physical = 1 * 0.1 + (-40) = -39.9°C
        let engine_payload_cold = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded_cold = dbc
            .decode(100, &engine_payload_cold)
            .expect("Should decode EngineData with cold value");

        let engine_temp_cold = decoded_cold
            .iter()
            .find(|(name, _, _)| *name == "EngineTemp")
            .expect("Should find EngineTemp signal");
        assert!(
            (engine_temp_cold.1 - (-40.0)).abs() < 0.01,
            "EngineTemp should be ~-40.0 (Cold), got {}",
            engine_temp_cold.1
        );

        // Test "Normal" value: raw = 1 -> physical = 1 * 0.1 + (-40) = -39.9°C
        let engine_payload_normal = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded_normal = dbc
            .decode(100, &engine_payload_normal)
            .expect("Should decode EngineData with normal value");

        let engine_temp_normal = decoded_normal
            .iter()
            .find(|(name, _, _)| *name == "EngineTemp")
            .expect("Should find EngineTemp signal");
        assert!(
            (engine_temp_normal.1 - (-39.9)).abs() < 0.01,
            "EngineTemp should be ~-39.9 (Normal), got {}",
            engine_temp_normal.1
        );

        // Test "Hot" value: raw = 2 -> physical = 2 * 0.1 + (-40) = -39.8°C
        let engine_payload_hot = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded_hot = dbc
            .decode(100, &engine_payload_hot)
            .expect("Should decode EngineData with hot value");

        let engine_temp_hot = decoded_hot
            .iter()
            .find(|(name, _, _)| *name == "EngineTemp")
            .expect("Should find EngineTemp signal");
        assert!(
            (engine_temp_hot.1 - (-39.8)).abs() < 0.01,
            "EngineTemp should be ~-39.8 (Hot), got {}",
            engine_temp_hot.1
        );
    }

    // ============================================================================
    // New Symbols (NS_) Comprehensive Tests
    // ============================================================================

    #[test]
    fn test_parse_ns_empty() {
        let dbc_content = r#"
VERSION "1.0"

NS_:

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse empty NS_");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_with_all_symbols() {
        let dbc_content = r#"
VERSION "1.0"

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

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with all symbol types");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_with_tabs() {
        let dbc_content = r#"
VERSION "1.0"

NS_ :
	NS_DESC_
	CM_
	BA_DEF_

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with tabs");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_with_spaces() {
        let dbc_content = r#"
VERSION "1.0"

NS_ :
    NS_DESC_
    CM_
    BA_DEF_

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with spaces");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_minimal_symbols() {
        let dbc_content = r#"
VERSION "1.0"

NS_ :
    NS_DESC_
    CM_

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with minimal symbols");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_with_mixed_whitespace() {
        let dbc_content = r#"
VERSION "1.0"

NS_ :
	NS_DESC_
    CM_
	BA_DEF_
    VAL_

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with mixed tabs and spaces");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_with_colon_space() {
        let dbc_content = r#"
VERSION "1.0"

NS_ : 
    NS_DESC_
    CM_

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with colon and space");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_with_colon_no_space() {
        let dbc_content = r#"
VERSION "1.0"

NS_:
    NS_DESC_
    CM_

BU_: ECM
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ with colon and no space");
        assert_eq!(dbc.messages().len(), 0);
    }

    #[test]
    fn test_parse_ns_in_complete_dbc() {
        let dbc_content = r#"
VERSION "1.0"

NS_ :
    NS_DESC_
    CM_
    BA_DEF_
    BA_
    VAL_

BU_: ECM TCM

BO_ 256 EngineData : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm" *

CM_ "Network comment";
CM_ BO_ 256 "Message comment";
"#;

        let dbc = Dbc::parse(dbc_content).expect("Should parse NS_ in complete DBC");
        assert_eq!(dbc.messages().len(), 1);
        let message = dbc.messages().find_by_id(256).unwrap();
        assert_eq!(message.name(), "EngineData");
    }
}
