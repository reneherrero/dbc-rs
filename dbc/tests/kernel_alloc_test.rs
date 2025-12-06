//! Test file to evaluate kernel alloc API compatibility
//!
//! This test file uses kernel alloc APIs to identify what needs to be changed
//! to support the kernel feature.
//!
//! Uses `mockalloc` to detect allocation bugs during testing.

#![cfg(feature = "kernel")]

use mockalloc::Mockalloc;
use std::alloc::System;

#[global_allocator]
static ALLOCATOR: Mockalloc<System> = Mockalloc(System);

// In kernel mode, we use kernel::alloc instead of alloc
use dbc_rs::kernel::alloc::{string::String, vec::Vec};

#[test]
fn test_kernel_alloc_basic() {
    // Test basic String allocation
    let s = String::try_from("Hello").unwrap();
    assert_eq!(s.as_str(), "Hello");
}

#[test]
fn test_kernel_alloc_vec() {
    // Test Vec allocation
    let mut v = Vec::new();
    v.try_push(1).unwrap();
    v.try_push(2).unwrap();
    v.try_push(3).unwrap();
    assert_eq!(v.len(), 3);
}

#[test]
fn test_kernel_alloc_string_operations() {
    // Test String operations that might be used in dbc-rs
    let mut s = String::try_from("test").unwrap();
    s.try_push_str("ing").unwrap();
    assert_eq!(s.as_str(), "testing");
}

#[test]
fn test_kernel_alloc_collections() {
    // Test Vec operations that might be used
    let mut vec = Vec::new();
    for i in 0..10 {
        vec.try_push(i).unwrap();
    }
    assert_eq!(vec.len(), 10);

    // Test extend
    let mut vec2 = Vec::new();
    vec2.try_extend_from_slice(&[1, 2, 3]).unwrap();
    assert_eq!(vec2.len(), 3);
}

// Now test actual dbc-rs APIs with kernel feature
#[test]
fn test_dbc_parse_with_kernel() {
    use dbc_rs::Dbc;

    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
"#;

    // This should work in no_std mode
    let dbc = Dbc::parse(dbc_content).unwrap();
    assert_eq!(dbc.version().map(|v| v.as_str()), Some("1.0"));
    assert_eq!(dbc.nodes().len(), 2);
}

// Test what happens when we try to use alloc-dependent features
#[test]
fn test_kernel_alloc_error_handling() {
    // In kernel mode, alloc operations return Result
    // This test shows the pattern we'd need to follow
    use dbc_rs::kernel::alloc::string::String;

    let result = String::try_from("test");
    assert!(result.is_ok());

    let mut s = result.unwrap();
    let push_result = s.try_push_str("ing");
    assert!(push_result.is_ok());
}

// Test builders with kernel feature
#[test]
fn test_builders_with_kernel() {
    use dbc_rs::{
        ByteOrder, DbcBuilder, MessageBuilder, NodesBuilder, Receivers, SignalBuilder,
        VersionBuilder,
    };

    // Create a DBC using builders with kernel feature
    // This tests that builders work correctly with kernel::alloc
    let dbc = DbcBuilder::new(None)
        .version(VersionBuilder::new().version("1.0").build().unwrap())
        .nodes(NodesBuilder::new().add_node("ECM").add_node("TCM").build().unwrap())
        .add_message(
            MessageBuilder::new()
                .id(256)
                .name("EngineData")
                .dlc(8)
                .sender("ECM")
                .add_signal(
                    SignalBuilder::new()
                        .name("RPM")
                        .start_bit(0)
                        .length(16)
                        .byte_order(ByteOrder::BigEndian)
                        .unsigned(true)
                        .factor(0.25)
                        .offset(0.0)
                        .min(0.0)
                        .max(8000.0)
                        .receivers(Receivers::Broadcast)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    // Verify the built DBC
    assert_eq!(dbc.version().map(|v| v.as_str()), Some("1.0"));
    assert_eq!(dbc.nodes().len(), 2);
    assert!(dbc.nodes().contains("ECM"));
    assert!(dbc.nodes().contains("TCM"));

    assert_eq!(dbc.messages().len(), 1);
    let msg = dbc.messages().at(0).unwrap();
    assert_eq!(msg.id(), 256);
    assert_eq!(msg.name(), "EngineData");
    assert_eq!(msg.dlc(), 8);
    assert_eq!(msg.sender(), "ECM");

    assert_eq!(msg.signals().len(), 1);
    let sig = msg.signals().at(0).unwrap();
    assert_eq!(sig.name(), "RPM");
    assert_eq!(sig.length(), 16);
    assert_eq!(sig.factor(), 0.25);
}

// Test individual builder with kernel feature
#[test]
fn test_signal_builder_with_kernel() {
    use dbc_rs::{ByteOrder, Receivers, SignalBuilder};

    let signal = SignalBuilder::new()
        .name("TestSignal")
        .start_bit(8)
        .length(8)
        .byte_order(ByteOrder::LittleEndian)
        .unsigned(false)
        .factor(1.0)
        .offset(0.0)
        .min(-128.0)
        .max(127.0)
        .receivers(Receivers::Broadcast)
        .build()
        .unwrap();

    assert_eq!(signal.name(), "TestSignal");
    assert_eq!(signal.start_bit(), 8);
    assert_eq!(signal.length(), 8);
    assert_eq!(signal.byte_order(), ByteOrder::LittleEndian);
    assert!(!signal.is_unsigned());
}

// Test nodes builder with kernel feature
#[test]
fn test_nodes_builder_with_kernel() {
    use dbc_rs::NodesBuilder;

    let nodes = NodesBuilder::new()
        .add_node("ECM")
        .add_node("TCM")
        .add_node("BCM")
        .build()
        .unwrap();

    assert_eq!(nodes.len(), 3);
    assert!(nodes.contains("ECM"));
    assert!(nodes.contains("TCM"));
    assert!(nodes.contains("BCM"));
}
