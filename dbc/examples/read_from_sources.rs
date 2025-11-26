//! Example: Reading DBC files from various sources.
//!
//! This example demonstrates different ways to parse DBC files:
//! - From string slices (no_std compatible)
//! - From bytes (no_std compatible)
//! - From owned strings (no_std compatible)
//! - From std::io::Read (requires std feature)

use dbc_rs::Dbc;
use std::io::Cursor;

fn main() -> Result<(), dbc_rs::Error> {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    // Method 1: Parse from string slice (works in no_std)
    println!("1. Parsing from &str:");
    let dbc1 = Dbc::parse(dbc_content)?;
    println!("   Parsed {} messages", dbc1.messages().len());

    // Method 2: Parse from bytes (works in no_std)
    println!("2. Parsing from &[u8]:");
    let bytes = dbc_content.as_bytes();
    let dbc2 = Dbc::parse_bytes(bytes)?;
    println!("   Parsed {} messages", dbc2.messages().len());

    // Method 3: Parse from String (works in no_std)
    println!("3. Parsing from String:");
    let string = String::from(dbc_content);
    let dbc3 = Dbc::parse_from(string)?;
    println!("   Parsed {} messages", dbc3.messages().len());

    // Method 4: Parse from std::io::Read (requires std feature)
    #[cfg(feature = "std")]
    {
        println!("4. Parsing from std::io::Read (Cursor):");
        let cursor = Cursor::new(dbc_content.as_bytes());
        let dbc4 = Dbc::from_reader(cursor)?;
        println!("   Parsed {} messages", dbc4.messages().len());

        // Method 5: Parse from File (requires std feature)
        println!("5. Parsing from File:");
        if let Ok(file) = std::fs::File::open("tests/data/simple.dbc") {
            let dbc5 = Dbc::from_reader(file)?;
            println!("   Parsed {} messages from file", dbc5.messages().len());
        } else {
            println!("   File not found (this is OK for the example)");
        }
    }

    Ok(())
}
