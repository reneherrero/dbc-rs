use std::env;

fn main() {
    // Allow override of MAX_SIGNALS_PER_MESSAGE via environment variable
    let max_signals = env::var("DBC_MAX_SIGNALS_PER_MESSAGE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64); // Default to 64

    // Allow override of MAX_MESSAGES via environment variable
    let max_messages = env::var("DBC_MAX_MESSAGES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10_000); // Default to 10000

    // Allow override of MAX_NODES via environment variable
    let max_nodes = env::var("DBC_MAX_NODES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256); // Default to 256

    // Allow override of MAX_VALUE_DESCRIPTIONS via environment variable
    let max_value_descriptions = env::var("DBC_MAX_VALUE_DESCRIPTIONS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64); // Default to 64

    // Allow override of MAX_RECEIVER_NODES via environment variable
    let max_receiver_nodes = env::var("DBC_MAX_RECEIVER_NODES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64); // Default to 64

    // Write the constants to a file in OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("limits.rs");
    std::fs::write(
               &dest_path,
               format!(
                   "#[allow(dead_code)]\npub const MAX_SIGNALS_PER_MESSAGE: usize = {};\n#[allow(dead_code)]\npub const MAX_MESSAGES: usize = {};\n#[allow(dead_code)]\npub const MAX_NODES: usize = {};\n#[allow(dead_code)]\npub const MAX_VALUE_DESCRIPTIONS: usize = {};\n#[allow(dead_code)]\npub const MAX_RECEIVER_NODES: usize = {};",
                   max_signals, max_messages, max_nodes, max_value_descriptions, max_receiver_nodes
               ),
           )
           .unwrap();

    // Rebuild if the environment variables change
    println!("cargo:rerun-if-env-changed=DBC_MAX_SIGNALS_PER_MESSAGE");
    println!("cargo:rerun-if-env-changed=DBC_MAX_MESSAGES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NODES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_VALUE_DESCRIPTIONS");
    println!("cargo:rerun-if-env-changed=DBC_MAX_RECEIVER_NODES");
}
