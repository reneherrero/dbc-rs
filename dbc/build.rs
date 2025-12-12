use std::env;

fn main() {
    // Enforce that either alloc or heapless feature must be enabled
    // Check Cargo feature flags via CARGO_FEATURE_* environment variables
    // (Note: hyphens become underscores, all uppercase)
    let has_alloc = env::var("CARGO_FEATURE_ALLOC").is_ok();
    let has_heapless = env::var("CARGO_FEATURE_HEAPLESS").is_ok();
    let has_std = env::var("CARGO_FEATURE_STD").is_ok();

    // std includes alloc, so we only need to check if neither alloc nor heapless is enabled
    // Note: This check provides a better error message, but mayheap will also enforce this
    if !has_alloc && !has_heapless && !has_std {
        eprintln!("error: Either the `alloc` or `heapless` feature must be enabled");
        eprintln!("\ndbc-rs requires one of the following features:");
        eprintln!("  - `alloc`: Heap-allocated collections via alloc crate");
        eprintln!("  - `heapless`: Stack-allocated, bounded collections");
        eprintln!("  - `std`: Includes alloc + standard library features (default)");
        eprintln!("\nAdd to Cargo.toml:");
        eprintln!("  [dependencies]");
        eprintln!(
            "  dbc-rs = {{ version = \"...\", default-features = false, features = [\"alloc\"] }}"
        );
        eprintln!("  # OR");
        eprintln!(
            "  dbc-rs = {{ version = \"...\", default-features = false, features = [\"heapless\"] }}"
        );
        std::process::exit(1);
    }

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

    // Allow override of MAX_NAME_SIZE via environment variable
    let max_name_size = env::var("DBC_MAX_NAME_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64); // Default to 64

    // Write the constants to a file in OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("limits.rs");
    std::fs::write(
               &dest_path,
               format!(
                   "#[allow(dead_code)]\npub const MAX_SIGNALS_PER_MESSAGE: usize = {};\n#[allow(dead_code)]\npub const MAX_MESSAGES: usize = {};\n#[allow(dead_code)]\npub const MAX_NODES: usize = {};\n#[allow(dead_code)]\npub const MAX_VALUE_DESCRIPTIONS: usize = {};\n#[allow(dead_code)]\npub const MAX_RECEIVER_NODES: usize = {};\n#[allow(dead_code)]\npub const MAX_NAME_SIZE: usize = {};",
                   max_signals, max_messages, max_nodes, max_value_descriptions, max_receiver_nodes, max_name_size
               ),
           )
           .unwrap();

    // Rebuild if the environment variables change
    println!("cargo:rerun-if-env-changed=DBC_MAX_SIGNALS_PER_MESSAGE");
    println!("cargo:rerun-if-env-changed=DBC_MAX_MESSAGES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NODES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_VALUE_DESCRIPTIONS");
    println!("cargo:rerun-if-env-changed=DBC_MAX_RECEIVER_NODES");
    println!("cargo:rerun-if-env-changed=DBC_MAX_NAME_SIZE");
}
