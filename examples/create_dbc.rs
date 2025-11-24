use dbc::{ByteOrder, Dbc, Message, Nodes, Receivers, Signal, Version};

fn main() -> Result<(), dbc::Error> {
    // Create version "1.0"
    let version = Version::new(1, Some(0), None)?;

    // Create nodes: ECM and TCM
    let nodes = Nodes::new(&["ECM", "TCM"]);

    // Create signals for Engine message
    let rpm_signal = Signal::new(
        "RPM",                   // name
        0,                       // start_bit
        16,                      // length
        ByteOrder::LittleEndian, // byte_order (@0)
        true,                    // unsigned (+)
        0.25,                    // factor
        0.0,                     // offset
        0.0,                     // min
        8000.0,                  // max
        Some("rpm"),             // unit
        Receivers::None,         // receivers
    )?;

    let temp_signal = Signal::new(
        "Temp",                  // name
        16,                      // start_bit
        8,                       // length
        ByteOrder::LittleEndian, // byte_order (@0)
        false,                   // signed (-)
        1.0,                     // factor
        -40.0,                   // offset
        -40.0,                   // min
        215.0,                   // max
        Some("°C"),              // unit
        Receivers::None,         // receivers
    )?;

    // Create signals for Brake message
    let pressure_signal = Signal::new(
        "Pressure",           // name
        0,                    // start_bit
        16,                   // length
        ByteOrder::BigEndian, // byte_order (@1)
        true,                 // unsigned (+)
        0.1,                  // factor
        0.0,                  // offset
        0.0,                  // min
        1000.0,               // max
        Some("bar"),          // unit
        Receivers::None,      // receivers
    )?;

    // Create Engine message (ID 256, DLC 8, sender ECM)
    let engine_message = Message::new(
        256,                           // id
        "Engine",                      // name
        8,                             // dlc
        "ECM",                         // sender
        vec![rpm_signal, temp_signal], // signals
    )?;

    // Create Brake message (ID 512, DLC 4, sender TCM)
    let brake_message = Message::new(
        512,                   // id
        "Brake",               // name
        4,                     // dlc
        "TCM",                 // sender
        vec![pressure_signal], // signals
    )?;

    // Create DBC with all components
    let dbc = Dbc::new(version, nodes, vec![engine_message, brake_message])?;

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
