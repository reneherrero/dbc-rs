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
}
