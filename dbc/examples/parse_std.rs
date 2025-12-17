//! Example: Parse, modify, build, and save DBC files (std only).
//!
//! This example demonstrates the complete workflow:
//! 1. Parse an existing DBC file
//! 2. Create a builder from the parsed DBC
//! 3. Modify parameters (add nodes, messages, change version)
//! 4. Build a new DBC from the modified builder
//! 5. Save the new DBC to a file

use dbc_rs::{ByteOrder, ReceiversBuilder};
use dbc_rs::{Dbc, DbcBuilder, MessageBuilder, NodesBuilder, SignalBuilder, VersionBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Parse an existing DBC
    println!("Step 1: Parsing original DBC...");
    let original_dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"
"#;

    let original = Dbc::parse(original_dbc_content)?;
    println!("  Parsed {} messages", original.messages().len());
    println!(
        "  Version: {}",
        original.version().map(|v| v.as_str()).unwrap_or("N/A")
    );
    println!("  Nodes: {}", original.nodes().len());

    // Step 2: Create a builder from the parsed DBC
    println!("\nStep 2: Creating builder from existing DBC...");
    let mut builder = DbcBuilder::from_dbc(&original);

    // Step 3: Modify parameters
    println!("\nStep 3: Modifying DBC...");

    // Change version
    builder = builder.version(VersionBuilder::new().version("2.0"));
    println!("  Changed version to 2.0");

    // Add a new node
    let updated_nodes = NodesBuilder::new().add_node("ECM").add_node("TCM").add_node("BCM"); // Add new node
    builder = builder.nodes(updated_nodes);
    println!("  Added new node: BCM");

    // Add a new message with signal
    let new_signal = SignalBuilder::new()
        .name("Pressure")
        .start_bit(0)
        .length(16)
        .byte_order(ByteOrder::LittleEndian)
        .unsigned(true)
        .factor(0.1)
        .offset(0.0)
        .min(0.0)
        .max(1000.0)
        .unit("bar")
        .receivers(ReceiversBuilder::new().add_node("ECM"));

    let new_message = MessageBuilder::new()
        .id(512)
        .name("Brake")
        .dlc(4)
        .sender("TCM")
        .add_signal(new_signal);

    builder = builder.add_message(new_message);
    println!("  Added new message: Brake (ID: 512)");

    // Step 4: Build the new DBC
    println!("\nStep 4: Building new DBC...");
    let modified = builder.build()?;
    println!("  Built DBC with {} messages", modified.messages().len());

    // Step 5: Save the new DBC to a file
    println!("\nStep 5: Saving new DBC to file...");
    let output_content = modified.to_dbc_string();
    std::fs::write("output.dbc", &output_content)?;
    println!("  Saved to output.dbc");

    // Display the result
    println!("\n=== Generated DBC Content ===");
    println!("{}", output_content);

    // Verify by parsing the saved file
    println!("\n=== Verification ===");
    let verified = Dbc::parse(&output_content)?;
    println!(
        "  Version: {}",
        verified.version().map(|v| v.as_str()).unwrap_or("N/A")
    );
    println!("  Nodes: {}", verified.nodes().len());
    println!("  Messages: {}", verified.messages().len());
    for message in verified.messages().iter() {
        println!(
            "    - {} (ID: {}) with {} signals",
            message.name(),
            message.id(),
            message.signals().len()
        );
    }

    Ok(())
}
