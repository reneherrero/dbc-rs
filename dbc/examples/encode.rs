//! Example: Encoding CAN messages using DBC files.
//!
//! This example demonstrates how to encode signal values into CAN message payloads:
//! - Parse a DBC file
//! - Encode signal values into CAN message payloads
//! - Round-trip verification (encode then decode)
//!
//! Run with: `cargo run --example encode`

use dbc_rs::Dbc;

fn main() -> Result<(), dbc_rs::Error> {
    // Parse a DBC file with messages and signals
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
    // Note: 2147484672 = 0x80000400 = extended CAN ID 0x400

    let dbc = Dbc::parse(dbc_content)?;
    println!("Parsed DBC with {} messages\n", dbc.messages().len());

    // Example 1: Encode an Engine message (standard CAN ID 256)
    println!("Example 1: Encoding Engine message (ID 0x{:03X})", 256);
    let payload = dbc.encode(
        256,
        &[("RPM", 2000.0), ("Temp", 50.0), ("Throttle", 50.0)],
        false,
    )?;
    println!("  Encoded payload: {:02X?}", payload.as_slice());
    println!("  Signals: RPM=2000 rpm, Temp=50°C, Throttle=50%");

    // Verify by decoding
    let decoded = dbc.decode(256, &payload, false)?;
    println!("  Verified by decoding:");
    for signal in decoded.iter() {
        let unit = signal.unit.unwrap_or("");
        println!("    {}: {:.2} {}", signal.name, signal.value, unit);
    }

    println!();

    // Example 2: Encode a Transmission message with negative torque
    println!(
        "Example 2: Encoding Transmission message (ID 0x{:03X})",
        512
    );
    let payload = dbc.encode(
        512,
        &[
            ("GearPosition", 3.0),
            ("Torque", -100.0),
            ("TransmissionTemp", 85.0),
        ],
        false,
    )?;
    println!("  Encoded payload: {:02X?}", payload.as_slice());
    println!("  Signals: Gear=3, Torque=-100 Nm, Temp=85°C");

    // Verify
    let decoded = dbc.decode(512, &payload, false)?;
    println!("  Verified by decoding:");
    for signal in decoded.iter() {
        let unit = signal.unit.unwrap_or("");
        println!("    {}: {:.2} {}", signal.name, signal.value, unit);
    }

    println!();

    // Example 3: Encode a Brake message
    println!("Example 3: Encoding Brake message (ID 0x{:03X})", 768);
    let payload = dbc.encode(768, &[("BrakePressure", 250.0), ("ABSActive", 1.0)], false)?;
    println!("  Encoded payload: {:02X?}", payload.as_slice());
    println!("  Signals: Pressure=250 bar, ABS=active");

    // Verify
    let decoded = dbc.decode(768, &payload, false)?;
    println!("  Verified by decoding:");
    for signal in decoded.iter() {
        let unit = signal.unit.unwrap_or("");
        println!("    {}: {:.2} {}", signal.name, signal.value, unit);
    }

    println!();

    // Example 4: Encode an extended CAN message (29-bit ID)
    println!(
        "Example 4: Encoding extended CAN message (ID 0x{:08X})",
        0x400
    );
    let payload = dbc.encode(0x400, &[("Speed", 100.0)], true)?;
    println!("  Encoded payload: {:02X?}", payload.as_slice());
    println!("  Signals: Speed=100 km/h");

    // Verify
    let decoded = dbc.decode(0x400, &payload, true)?;
    println!("  Verified by decoding:");
    for signal in decoded.iter() {
        let unit = signal.unit.unwrap_or("");
        println!("    {}: {:.2} {}", signal.name, signal.value, unit);
    }

    println!();

    // Example 5: Error handling - Value out of range
    println!("Example 5: Error handling - Value out of range");
    match dbc.encode(256, &[("RPM", 9000.0)], false) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Error (expected): {}", e),
    }

    println!();

    // Example 6: Error handling - Signal not found
    println!("Example 6: Error handling - Signal not found");
    match dbc.encode(256, &[("NonExistent", 100.0)], false) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Error (expected): {}", e),
    }

    println!();

    // Example 7: Partial encoding (only some signals)
    println!("Example 7: Partial encoding - Only RPM signal");
    let payload = dbc.encode(256, &[("RPM", 3500.0)], false)?;
    println!("  Encoded payload: {:02X?}", payload.as_slice());
    println!("  Note: Unspecified signals default to 0");

    Ok(())
}
