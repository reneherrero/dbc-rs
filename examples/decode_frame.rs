//! Example: Decoding CAN frames using the embedded-can Frame trait.
//!
//! This example demonstrates how to decode CAN frames using the `embedded-can` crate's
//! `Frame` trait, which is useful when working with CAN drivers that implement this trait.
//!
//! The `decode_frame()` method automatically handles:
//! - Standard (11-bit) vs Extended (29-bit) ID detection
//! - Payload extraction from the frame
//!
//! Run with: `cargo run --example decode_frame --features std,embedded-can`

use dbc_rs::Dbc;
use embedded_can::{ExtendedId, Frame, Id, StandardId};

/// A simple CAN frame implementation for demonstration purposes.
///
/// In practice, you would use the frame type provided by your CAN driver
/// (e.g., `bxcan::Frame`, `socketcan::CANFrame`, etc.).
struct TestFrame {
    id: Id,
    data: [u8; 8],
    dlc: usize,
}

impl TestFrame {
    fn new_standard(id: u16, data: &[u8]) -> Self {
        let mut frame_data = [0u8; 8];
        let dlc = data.len().min(8);
        frame_data[..dlc].copy_from_slice(&data[..dlc]);
        Self {
            id: Id::Standard(StandardId::new(id).expect("Invalid standard ID")),
            data: frame_data,
            dlc,
        }
    }

    fn new_extended(id: u32, data: &[u8]) -> Self {
        let mut frame_data = [0u8; 8];
        let dlc = data.len().min(8);
        frame_data[..dlc].copy_from_slice(&data[..dlc]);
        Self {
            id: Id::Extended(ExtendedId::new(id).expect("Invalid extended ID")),
            data: frame_data,
            dlc,
        }
    }
}

impl Frame for TestFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let mut frame_data = [0u8; 8];
        let dlc = data.len().min(8);
        frame_data[..dlc].copy_from_slice(&data[..dlc]);
        Some(Self {
            id: id.into(),
            data: frame_data,
            dlc,
        })
    }

    fn new_remote(_id: impl Into<Id>, _dlc: usize) -> Option<Self> {
        None // Remote frames not supported in this example
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        false
    }

    fn id(&self) -> Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.dlc
    }

    fn data(&self) -> &[u8] {
        &self.data[..self.dlc]
    }
}

fn main() -> Result<(), dbc_rs::Error> {
    // Parse a DBC file with messages and signals
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"
 SG_ Throttle : 24|8@1+ (0.392157,0) [0|100] "%"

BO_ 2147484672 ExtendedMsg : 8 ECM
 SG_ Speed : 0|16@1+ (0.1,0) [0|6553.5] "km/h"
"#;
    // Note: 2147484672 = 0x80000400 = extended CAN ID 0x400 (bit 31 set indicates extended)

    let dbc = Dbc::parse(dbc_content)?;
    println!("Parsed DBC with {} messages\n", dbc.messages().len());

    // Example 1: Decode a standard CAN frame (11-bit ID)
    println!("Example 1: Decoding standard frame (ID 0x100)");
    let std_frame = TestFrame::new_standard(256, &[0x40, 0x1F, 0x5A, 0x7F, 0x00, 0x00, 0x00, 0x00]);

    match dbc.decode_frame(std_frame) {
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

    // Example 2: Decode an extended CAN frame (29-bit ID)
    println!("Example 2: Decoding extended frame (ID 0x400)");
    let ext_frame =
        TestFrame::new_extended(0x400, &[0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    match dbc.decode_frame(ext_frame) {
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

    // Example 3: Error handling - Message not found
    println!("Example 3: Error handling - Non-existent message");
    let unknown_frame = TestFrame::new_standard(999, &[0x00; 8]);

    match dbc.decode_frame(unknown_frame) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Error (expected): {}", e),
    }

    Ok(())
}
