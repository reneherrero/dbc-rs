//! Example: Decoding CAN messages using DBC files.
//!
//! This example demonstrates how to decode CAN message payloads using a parsed DBC file:
//! - Parse a DBC file
//! - Decode CAN message payloads using message IDs (standard and extended)
//! - Display decoded signal values with units
//!
//! Run with: `cargo run --example decode`
//!
//! For decoding using the embedded-can Frame trait, see the `decode_frame` example.

use dbc_rs::Dbc;

fn main() -> Result<(), dbc_rs::Error> {
    // Parse a DBC file with messages and signals
    // Includes both standard (11-bit) and extended (29-bit) CAN IDs
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM ABS

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"
 SG_ Throttle : 24|8@1+ (0.392157,0) [0|100] "%"

BO_ 512 Transmission : 8 TCM
 SG_ GearPosition : 0|8@1+ (1,0) [0|5] ""
 SG_ Torque : 16|16@1- (0.1,0) [-3276.8|3276.7] "Nm"
 SG_ TransmissionTemp : 32|8@1- (1,-40) [-40|215] "°C"

BO_ 768 Brake : 6 ABS
 SG_ BrakePressure : 0|16@1+ (0.1,0) [0|1000] "bar"
 SG_ ABSActive : 16|1@1+ (1,0) [0|1] ""

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h"
"#;
    // Note: 2147484672 = 0x80000400 = extended CAN ID 0x400 (bit 31 set indicates extended)

    let dbc = Dbc::parse(dbc_content)?;
    println!("Parsed DBC with {} messages\n", dbc.messages().len());

    // Example 1: Decode an Engine message (standard CAN ID 256)
    // Payload represents: RPM = 2000 (raw: 8000 = 0x1F40), Temp = 50°C (raw: 90), Throttle = 50% (raw: 127)
    // Little-endian format: bytes at positions 0-1 = RPM, byte at position 2 = Temp, byte at position 3 = Throttle
    println!(
        "Example 1: Decoding Engine message (standard ID 0x{:03X})",
        256
    );
    let engine_payload = [0x40, 0x1F, 0x5A, 0x7F, 0x00, 0x00, 0x00, 0x00];
    match dbc.decode(256, &engine_payload, false) {
        Ok(decoded) => {
            println!("  Decoded {} signals:", decoded.len());
            for signal in decoded.iter() {
                let unit_str = signal.unit.map(|u| format!(" {}", u)).unwrap_or_default();
                println!("    {}: {:.2}{}", signal.name, signal.value, unit_str);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // Example 2: Decode a Transmission message (standard CAN ID 512)
    // Payload represents: Gear = 3, Torque = -100 Nm (raw: -1000), Temp = 85°C (raw: 125)
    println!(
        "Example 2: Decoding Transmission message (standard ID 0x{:03X})",
        512
    );
    let transmission_payload = [0x03, 0x00, 0x18, 0xFC, 0x7D, 0x00, 0x00, 0x00];
    match dbc.decode(512, &transmission_payload, false) {
        Ok(decoded) => {
            println!("  Decoded {} signals:", decoded.len());
            for signal in decoded.iter() {
                let unit_str = signal.unit.map(|u| format!(" {}", u)).unwrap_or_default();
                println!("    {}: {:.2}{}", signal.name, signal.value, unit_str);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // Example 3: Decode a Brake message (standard CAN ID 768)
    // Payload represents: Pressure = 250 bar (raw: 2500), ABS Active = 1
    println!(
        "Example 3: Decoding Brake message (standard ID 0x{:03X})",
        768
    );
    let brake_payload = [0xC4, 0x09, 0x01, 0x00, 0x00, 0x00];
    match dbc.decode(768, &brake_payload, false) {
        Ok(decoded) => {
            println!("  Decoded {} signals:", decoded.len());
            for signal in decoded.iter() {
                let unit_str = signal.unit.map(|u| format!(" {}", u)).unwrap_or_default();
                println!("    {}: {:.2}{}", signal.name, signal.value, unit_str);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // Example 4: Decode an extended CAN message (29-bit ID 0x400)
    // When decoding, pass the raw CAN ID (0x400) with is_extended=true
    println!(
        "Example 4: Decoding extended CAN message (extended ID 0x{:08X})",
        0x400
    );
    let extended_payload = [0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]; // Speed = 100 km/h
    match dbc.decode(0x400, &extended_payload, true) {
        Ok(decoded) => {
            println!("  Decoded {} signals:", decoded.len());
            for signal in decoded.iter() {
                let unit_str = signal.unit.map(|u| format!(" {}", u)).unwrap_or_default();
                println!("    {}: {:.2}{}", signal.name, signal.value, unit_str);
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();

    // Example 5: Error handling - Message not found
    println!("Example 5: Error handling - Non-existent message");
    let invalid_payload = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    match dbc.decode(999, &invalid_payload, false) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Error (expected): {}", e),
    }

    println!();

    // Example 6: Error handling - Payload length mismatch
    println!("Example 6: Error handling - Payload too short");
    let short_payload = [0x40, 0x1F, 0x5A]; // Only 3 bytes, but message requires 8
    match dbc.decode(256, &short_payload, false) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Error (expected): {}", e),
    }

    Ok(())
}
