//! Integration tests for DBC file parsing and manipulation.
#![allow(clippy::float_cmp)]

#[cfg(feature = "std")]
mod std {
    use dbc_rs::Dbc;

    #[test]
    fn test_parse_simple_dbc() {
        let content =
            std::fs::read_to_string("tests/data/simple.dbc").expect("Failed to read simple.dbc");
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

        let speed = engine_msg.find_signal("EngineSpeed").unwrap();
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
        let content = std::fs::read_to_string("tests/data/multiplexed.dbc")
            .expect("Failed to read multiplexed.dbc");
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

        let temp = sensor_msg.find_signal("Temperature").unwrap();
        assert_eq!(temp.start_bit(), 8);
        assert_eq!(temp.offset(), -50.0);
        assert_eq!(temp.unit(), Some("°C"));

        // Verify ActuatorControl message
        let actuator_msg = dbc.messages().iter().find(|m| m.id() == 400).unwrap();
        assert_eq!(actuator_msg.name(), "ActuatorControl");
        assert_eq!(actuator_msg.dlc(), 6);
        assert_eq!(actuator_msg.signals().len(), 3);

        let force = actuator_msg.find_signal("Force").unwrap();
        assert!(!force.is_unsigned()); // Should be signed
        assert_eq!(force.unit(), Some("N"));
    }

    #[test]
    fn test_parse_minimal_dbc() {
        let content =
            std::fs::read_to_string("tests/data/minimal.dbc").expect("Failed to read minimal.dbc");
        let dbc = Dbc::parse(&content).expect("Failed to parse minimal.dbc");

        // Verify version (just major, no minor/patch)
        assert_eq!(dbc.version().map(|v| v.to_string()), Some("1".to_string()));

        // Verify single node
        assert!(dbc.nodes().contains("NODE1"));
        assert_eq!(dbc.nodes().nodes().unwrap().len(), 1);

        // Verify single message
        assert_eq!(dbc.messages().len(), 1);

        let msg = &dbc.messages()[0];
        assert_eq!(msg.id(), 256);
        assert_eq!(msg.name(), "TestMessage");
        assert_eq!(msg.dlc(), 1); // Minimal DLC
        assert_eq!(msg.signals().len(), 1);

        let sig = &msg.signals()[0];
        assert_eq!(sig.name(), "TestSignal");
        assert_eq!(sig.length(), 8);
    }

    #[test]
    fn test_parse_extended_ids_dbc() {
        let content = std::fs::read_to_string("tests/data/extended_ids.dbc")
            .expect("Failed to read extended_ids.dbc");
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
        let gear = trans_msg.find_signal("Gear").unwrap();
        assert_eq!(gear.length(), 4);
        assert_eq!(gear.start_bit(), 0);

        let clutch = trans_msg.find_signal("Clutch").unwrap();
        assert_eq!(clutch.length(), 1);
        assert_eq!(clutch.start_bit(), 4);
    }

    #[test]
    fn test_parse_broadcast_signals_dbc() {
        let content = std::fs::read_to_string("tests/data/broadcast_signals.dbc")
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
        let status = msg.find_signal("Status").unwrap();
        assert_eq!(status.receivers(), &dbc_rs::Receivers::Broadcast);

        // Verify signals with specific receivers
        let data1 = msg.find_signal("Data1").unwrap();
        match data1.receivers() {
            dbc_rs::Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert!(nodes.iter().any(|n| n == "RECEIVER1"));
                assert!(nodes.iter().any(|n| n == "RECEIVER2"));
            }
            _ => panic!("Data1 should have specific receivers"),
        }

        let data2 = msg.find_signal("Data2").unwrap();
        match data2.receivers() {
            dbc_rs::Receivers::Nodes(nodes) => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(nodes[0], "RECEIVER1");
            }
            _ => panic!("Data2 should have specific receivers"),
        }
    }

    #[test]
    fn test_parse_complete_dbc_file() {
        // Parse the complete.dbc file
        let content = std::fs::read_to_string("tests/data/complete.dbc")
            .expect("Failed to read complete.dbc");
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
        let rpm = engine_msg.find_signal("RPM").expect("RPM signal not found");
        assert_eq!(rpm.start_bit(), 0);
        assert_eq!(rpm.length(), 16);
        assert_eq!(rpm.factor(), 0.25);
        assert_eq!(rpm.unit(), Some("rpm"));

        let temp = engine_msg.find_signal("Temperature").expect("Temperature signal not found");
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
        let brake_pressure =
            brake_msg.find_signal("BrakePressure").expect("BrakePressure signal not found");
        assert_eq!(brake_pressure.start_bit(), 0);
        assert_eq!(brake_pressure.length(), 16);
        assert_eq!(brake_pressure.unit(), Some("bar"));

        let abs_active = brake_msg.find_signal("ABSActive").expect("ABSActive signal not found");
        assert_eq!(abs_active.start_bit(), 16);
        assert_eq!(abs_active.length(), 1);

        let wheel_speed_front_left =
            brake_msg.find_signal("WheelSpeedFL").expect("WheelSpeedFL signal not found");
        assert_eq!(wheel_speed_front_left.start_bit(), 17);
        assert_eq!(wheel_speed_front_left.length(), 15);
        assert_eq!(wheel_speed_front_left.unit(), Some("km/h"));

        let wheel_speed_front_right =
            brake_msg.find_signal("WheelSpeedFR").expect("WheelSpeedFR signal not found");
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
}
