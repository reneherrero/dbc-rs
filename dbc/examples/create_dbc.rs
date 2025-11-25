use dbc_rs::{ByteOrder, Dbc, Message, Nodes, Receivers, Signal, Version};

fn main() -> Result<(), dbc_rs::Error> {
    // Create version "1.0" using builder
    let version = Version::builder().major(1).minor(0).build()?;

    // Create nodes: ECM and TCM using builder
    let nodes = Nodes::builder().add_node("ECM").add_node("TCM").build()?;

    // Create signals for Engine message using the builder pattern
    let rpm_signal = Signal::builder()
        .name("RPM")
        .start_bit(0)
        .length(16)
        .byte_order(ByteOrder::LittleEndian)
        .unsigned(true)
        .factor(0.25)
        .offset(0.0)
        .min(0.0)
        .max(8000.0)
        .unit("rpm")
        .receivers(Receivers::None)
        .build()?;

    let temp_signal = Signal::builder()
        .name("Temp")
        .start_bit(16)
        .length(8)
        .byte_order(ByteOrder::LittleEndian)
        .unsigned(false)
        .factor(1.0)
        .offset(-40.0)
        .min(-40.0)
        .max(215.0)
        .unit("°C")
        .receivers(Receivers::None)
        .build()?;

    // Create signals for Brake message
    let pressure_signal = Signal::builder()
        .name("Pressure")
        .start_bit(0)
        .length(16)
        .byte_order(ByteOrder::BigEndian)
        .unsigned(true)
        .factor(0.1)
        .offset(0.0)
        .min(0.0)
        .max(1000.0)
        .unit("bar")
        .receivers(Receivers::None)
        .build()?;

    // Create Engine message (ID 256, DLC 8, sender ECM) using the builder pattern
    let engine_message = Message::builder()
        .id(256)
        .name("Engine")
        .dlc(8)
        .sender("ECM")
        .add_signal(rpm_signal)
        .add_signal(temp_signal)
        .build()?;

    // Create Brake message (ID 512, DLC 4, sender TCM) using the builder pattern
    let brake_message = Message::builder()
        .id(512)
        .name("Brake")
        .dlc(4)
        .sender("TCM")
        .add_signal(pressure_signal)
        .build()?;

    // Create DBC with all components using the builder pattern
    let dbc = Dbc::builder()
        .version(version)
        .nodes(nodes)
        .add_message(engine_message)
        .add_message(brake_message)
        .build()?;

    // Verify the created DBC
    println!("Created DBC with version: {}", dbc.version().to_string());
    println!("Nodes: {}", dbc.nodes().to_string());
    println!("Messages: {}", dbc.messages().len());

    for msg in dbc.messages() {
        println!(
            "  Message {} (ID: {}, DLC: {}, Sender: {})",
            msg.name(),
            msg.id(),
            msg.dlc(),
            msg.sender()
        );
        for sig in msg.signals() {
            println!(
                "    Signal {}: {}|{}@{} (factor: {}, offset: {}) [{:.1}|{:.1}] \"{}\"",
                sig.name(),
                sig.start_bit(),
                sig.length(),
                if sig.byte_order() == ByteOrder::LittleEndian {
                    "0"
                } else {
                    "1"
                },
                sig.factor(),
                sig.offset(),
                sig.min(),
                sig.max(),
                sig.unit().unwrap_or("")
            );
        }
    }

    // The DBC object is now ready to use
    // You can access messages, signals, and other data through the getter methods

    Ok(())
}
