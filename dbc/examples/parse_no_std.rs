//! Example: Reading and decoding DBC files in no_std environments.
//!
//! This example demonstrates parsing DBC files and decoding CAN message data
//! in `no_std` environments using the `alloc` feature. The `Dbc::parse()` method
//! works without the standard library, and `Signal::decode()` can decode CAN
//! message bytes into physical values.
//!
//! Note: To parse messages (required for decoding), the `alloc` feature must be enabled.
//! In pure no_std without alloc, only VERSION and BU_ sections are parsed.

// Note: This example demonstrates no_std usage, but the example itself
// runs with std for output. The library code uses no_std when compiled
// with --no-default-features.

use dbc_rs::Dbc;

fn main() {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    // Parse from string slice (works in no_std with alloc feature)
    if let Ok(dbc) = Dbc::parse(dbc_content) {
        // Access version (available in both std and no_std)
        let _version_str = dbc.version().map(|v| v.as_str()).unwrap_or("");

        // Access nodes using iterator (available in both std and no_std)
        let _node_count = dbc.nodes().len();

        // With alloc feature, messages are parsed and available for decoding
        let _messages_count = dbc.messages().len();

        // Decode CAN message data into physical values
        // Example: Decode RPM signal from Engine message (ID 256)
        // Signal: RPM : 0|16@1+ (0.25,0) - 16 bits, little-endian, unsigned
        // CAN data: [0x40, 0x1F, ...] = 8000 raw value (little-endian: 0x1F40)
        // Physical: 8000 * 0.25 = 2000.0 rpm
        if let Some(engine_msg) = dbc.messages().find_by_id(256) {
            if let Some(rpm_signal) = engine_msg.signals().find("RPM") {
                // Simulated CAN message data (8 bytes)
                // Raw value: 0x1F40 = 8000 (little-endian: bytes 0x40, 0x1F)
                let can_data = [0x40, 0x1F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                if let Ok(rpm_value) = rpm_signal.decode(&can_data) {
                    // Decoded physical value: 8000 * 0.25 = 2000.0 rpm
                    // Verify decode result matches expected value
                    assert!((rpm_value - 2000.0).abs() < 0.1, "RPM decode failed");
                    // In embedded: use rpm_value for control logic
                }
            }

            // Decode temperature signal from Engine message
            // Signal: Temp : 16|8@1- (1,-40) - 8 bits at bit 16 (byte 2), signed
            // CAN data: [..., 0x64, ...] = 100 raw value (signed 8-bit at byte 2)
            // Physical: 100 * 1 + (-40) = 60.0°C
            if let Some(temp_signal) = engine_msg.signals().find("Temp") {
                let can_data = [0x40, 0x1F, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00];
                if let Ok(temp_value) = temp_signal.decode(&can_data) {
                    // Decoded physical value: 100 + (-40) = 60.0°C
                    // Verify decode result matches expected value
                    assert!((temp_value - 60.0).abs() < 0.1, "Temperature decode failed");
                    // In embedded: use temp_value for control logic
                }
            }
        }

        // Decode pressure signal from Brake message (ID 512)
        // Signal: Pressure : 0|16@1+ (0.1,0) - 16 bits, little-endian, unsigned
        // CAN data: [0x64, 0x00, ...] = 100 raw value
        // Physical: 100 * 0.1 = 10.0 bar
        if let Some(brake_msg) = dbc.messages().find_by_id(512) {
            if let Some(pressure_signal) = brake_msg.signals().find("Pressure") {
                // Simulated CAN message data (4 bytes for DLC 4)
                // Raw value: 0x0064 = 100 (little-endian: bytes 0x64, 0x00)
                let can_data = [0x64, 0x00, 0x00, 0x00];
                if let Ok(pressure_value) = pressure_signal.decode(&can_data) {
                    // Decoded physical value: 100 * 0.1 = 10.0 bar
                    // Verify decode result matches expected value
                    assert!(
                        (pressure_value - 10.0).abs() < 0.1,
                        "Pressure decode failed"
                    );
                    // In embedded: use pressure_value for control logic
                }
            }
        }

        // In embedded: use decoded values for control logic, status updates, etc.
    } else {
        // In embedded: set error flag, trigger error handler, etc.
    }
}
