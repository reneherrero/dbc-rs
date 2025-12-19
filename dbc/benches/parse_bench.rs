use criterion::{Criterion, criterion_group, criterion_main};
use dbc_rs::Dbc;
use std::hint::black_box;

fn bench_parse_small(c: &mut Criterion) {
    let small_dbc = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1+ (1,-40) [-40|215] "°C"
"#;

    c.bench_function("parse_small", |b| {
        b.iter(|| Dbc::parse(black_box(small_dbc)))
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let mut medium_dbc = String::from(
        r#"VERSION "1.0"

BU_: ECM TCM BCM

"#,
    );

    // Add 50 messages
    for i in 0..50 {
        medium_dbc.push_str(&format!("BO_ {} Message{} : 8 ECM\n", 256 + i, i));
        for j in 0..4 {
            medium_dbc.push_str(&format!(
                " SG_ Signal{} : {}|8@1+ (1,0) [0|255] \"\"\n",
                j,
                j * 8
            ));
        }
    }

    c.bench_function("parse_medium", |b| {
        b.iter(|| Dbc::parse(black_box(&medium_dbc)))
    });
}

fn bench_parse_large(c: &mut Criterion) {
    let mut large_dbc = String::from(
        r#"VERSION "1.0"

BU_: ECM TCM BCM GATEWAY SENSOR ACTUATOR

"#,
    );

    // Add 200 messages
    for i in 0..200 {
        large_dbc.push_str(&format!("BO_ {} Message{} : 8 ECM\n", 256 + i, i));
        for j in 0..8 {
            large_dbc.push_str(&format!(
                " SG_ Signal{} : {}|8@1+ (1,0) [0|255] \"\"\n",
                j,
                j * 8
            ));
        }
    }

    c.bench_function("parse_large", |b| {
        b.iter(|| Dbc::parse(black_box(&large_dbc)))
    });
}

#[cfg(feature = "std")]
fn bench_to_dbc_string(c: &mut Criterion) {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM TCM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1+ (1,-40) [-40|215] "°C"

BO_ 512 Brake : 4 TCM
 SG_ Pressure : 0|16@1+ (0.1,0) [0|1000] "bar"
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();

    c.bench_function("to_dbc_string", |b| {
        b.iter(|| black_box(&dbc).to_dbc_string())
    });
}

fn bench_decode_simple(c: &mut Criterion) {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // Payload: RPM = 2000 (raw: 8000 = 0x1F40), Temp = 50°C (raw: 90)
    let payload = [0x40, 0x1F, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00];

    c.bench_function("decode_simple", |b| {
        b.iter(|| dbc.decode(black_box(256), black_box(&payload), false))
    });
}

fn bench_decode_multiple_signals(c: &mut Criterion) {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 EngineData : 8 ECM
 SG_ Signal0 : 0|8@1+ (1,0) [0|255] ""
 SG_ Signal1 : 8|8@1+ (1,0) [0|255] ""
 SG_ Signal2 : 16|8@1+ (1,0) [0|255] ""
 SG_ Signal3 : 24|8@1+ (1,0) [0|255] ""
 SG_ Signal4 : 32|8@1+ (1,0) [0|255] ""
 SG_ Signal5 : 40|8@1+ (1,0) [0|255] ""
 SG_ Signal6 : 48|8@1+ (1,0) [0|255] ""
 SG_ Signal7 : 56|8@1+ (1,0) [0|255] ""
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    let payload = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

    c.bench_function("decode_multiple_signals", |b| {
        b.iter(|| dbc.decode(black_box(256), black_box(&payload), false))
    });
}

fn bench_decode_message_lookup_first(c: &mut Criterion) {
    let mut dbc_content = String::from(
        r#"VERSION "1.0"

BU_: ECM

"#,
    );

    // Create 100 messages to test lookup performance
    for i in 0..100 {
        dbc_content.push_str(&format!("BO_ {} Message{} : 8 ECM\n", 256 + i, i));
        dbc_content.push_str(" SG_ Signal : 0|8@1+ (1,0) [0|255] \"\"\n");
    }

    let dbc = Dbc::parse(&dbc_content).unwrap();
    let payload = [0x42; 8]; // All bytes set to 0x42

    c.bench_function("decode_message_lookup_first", |b| {
        b.iter(|| dbc.decode(black_box(256), black_box(&payload), false))
    });
}

fn bench_decode_message_lookup_middle(c: &mut Criterion) {
    let mut dbc_content = String::from(
        r#"VERSION "1.0"

BU_: ECM

"#,
    );

    // Create 100 messages to test lookup performance
    for i in 0..100 {
        dbc_content.push_str(&format!("BO_ {} Message{} : 8 ECM\n", 256 + i, i));
        dbc_content.push_str(" SG_ Signal : 0|8@1+ (1,0) [0|255] \"\"\n");
    }

    let dbc = Dbc::parse(&dbc_content).unwrap();
    let payload = [0x42; 8]; // All bytes set to 0x42
    let middle_id = 256 + 50; // Middle message

    c.bench_function("decode_message_lookup_middle", |b| {
        b.iter(|| dbc.decode(black_box(middle_id), black_box(&payload), false))
    });
}

fn bench_decode_message_lookup_last(c: &mut Criterion) {
    let mut dbc_content = String::from(
        r#"VERSION "1.0"

BU_: ECM

"#,
    );

    // Create 100 messages to test lookup performance
    for i in 0..100 {
        dbc_content.push_str(&format!("BO_ {} Message{} : 8 ECM\n", 256 + i, i));
        dbc_content.push_str(" SG_ Signal : 0|8@1+ (1,0) [0|255] \"\"\n");
    }

    let dbc = Dbc::parse(&dbc_content).unwrap();
    let payload = [0x42; 8]; // All bytes set to 0x42
    let last_id = 256 + 99; // Last message

    c.bench_function("decode_message_lookup_last", |b| {
        b.iter(|| dbc.decode(black_box(last_id), black_box(&payload), false))
    });
}

fn bench_decode_high_throughput(c: &mut Criterion) {
    let mut dbc_content = String::from(
        r#"VERSION "1.0"

BU_: ECM

"#,
    );

    // Create 50 messages for throughput test
    for i in 0..50 {
        dbc_content.push_str(&format!("BO_ {} Message{} : 8 ECM\n", 256 + i, i));
        dbc_content.push_str(" SG_ Signal0 : 0|8@1+ (1,0) [0|255] \"\"\n");
        dbc_content.push_str(" SG_ Signal1 : 8|8@1+ (1,0) [0|255] \"\"\n");
    }

    let dbc = Dbc::parse(&dbc_content).unwrap();
    let payload = [0x42; 8];

    c.bench_function("decode_high_throughput", |b| {
        b.iter(|| {
            // Decode all 50 messages in sequence
            for i in 0..50 {
                black_box(dbc.decode(256 + i, &payload, false).unwrap());
            }
        })
    });
}

fn bench_decode_big_endian(c: &mut Criterion) {
    // For Motorola (big-endian) byte order, start_bit is the MSB position
    // RPM: start_bit=7 means MSB at bit 7 of byte 0, 16-bit spans bytes 0-1
    // Pressure: start_bit=23 means MSB at bit 7 of byte 2, 16-bit spans bytes 2-3
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 7|16@0+ (1.0,0) [0|65535] "rpm"
 SG_ Pressure : 23|16@0+ (0.1,0) [0|6553.5] "bar"
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // Big-endian: RPM = 256 (0x0100), Pressure = 1000 (0x03E8)
    let payload = [0x01, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x00, 0x00];

    c.bench_function("decode_big_endian", |b| {
        b.iter(|| dbc.decode(black_box(256), black_box(&payload), false))
    });
}

fn bench_decode_little_endian(c: &mut Criterion) {
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@1+ (0.25,0) [0|8000] "rpm"
 SG_ Temp : 16|8@1- (1,-40) [-40|215] "°C"
 SG_ Throttle : 24|8@1+ (0.392157,0) [0|100] "%"
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // Little-endian: RPM = 2000 (0x1F40), Temp = 50 (0x5A), Throttle = 50% (0x32)
    let payload = [0x40, 0x1F, 0x5A, 0x32, 0x00, 0x00, 0x00, 0x00];

    c.bench_function("decode_little_endian", |b| {
        b.iter(|| dbc.decode(black_box(256), black_box(&payload), false))
    });
}

fn bench_decode_with_value_descriptions(c: &mut Criterion) {
    // DBC with value descriptions - tests the value description lookup overhead
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 200 GearboxData : 8 ECM
 SG_ GearActual : 0|8@1+ (1,0) [0|7] ""
 SG_ GearRequest : 8|8@1+ (1,0) [0|7] ""
 SG_ ShiftInProgress : 16|1@1+ (1,0) [0|1] ""
 SG_ TransTemp : 24|8@1+ (1,-40) [-40|215] "°C"

VAL_ 200 GearActual 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" 5 "Manual" 6 "Low" 7 "Invalid" ;
VAL_ 200 GearRequest 0 "Park" 1 "Reverse" 2 "Neutral" 3 "Drive" 4 "Sport" 5 "Manual" 6 "Low" 7 "Invalid" ;
VAL_ 200 ShiftInProgress 0 "No" 1 "Yes" ;
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // GearActual=3 (Drive), GearRequest=3 (Drive), ShiftInProgress=0 (No), TransTemp=80°C
    let payload = [0x03, 0x03, 0x00, 0x78, 0x00, 0x00, 0x00, 0x00];

    c.bench_function("decode_with_value_descriptions", |b| {
        b.iter(|| dbc.decode(black_box(200), black_box(&payload), false))
    });
}

fn bench_decode_without_value_descriptions(c: &mut Criterion) {
    // Same structure as above but WITHOUT value descriptions - for comparison
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 200 GearboxData : 8 ECM
 SG_ GearActual : 0|8@1+ (1,0) [0|7] ""
 SG_ GearRequest : 8|8@1+ (1,0) [0|7] ""
 SG_ ShiftInProgress : 16|1@1+ (1,0) [0|1] ""
 SG_ TransTemp : 24|8@1+ (1,-40) [-40|215] "°C"
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    let payload = [0x03, 0x03, 0x00, 0x78, 0x00, 0x00, 0x00, 0x00];

    c.bench_function("decode_without_value_descriptions", |b| {
        b.iter(|| dbc.decode(black_box(200), black_box(&payload), false))
    });
}

fn bench_decode_multiplexed(c: &mut Criterion) {
    // DBC with basic multiplexing (m0, m1, etc.)
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 300 MultiplexedSensors : 8 ECM
 SG_ SensorID M : 0|8@1+ (1,0) [0|3] ""
 SG_ Temperature m0 : 8|16@1- (0.1,-40) [-40|125] "°C"
 SG_ Pressure m1 : 8|16@1+ (0.01,0) [0|655.35] "kPa"
 SG_ Humidity m2 : 8|16@1+ (0.01,0) [0|100] "%"
 SG_ Voltage m3 : 8|16@1+ (0.001,0) [0|65.535] "V"
 SG_ CommonStatus : 56|8@1+ (1,0) [0|255] ""
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // SensorID=0 (Temperature), Temperature=500 (10.0°C), CommonStatus=0xFF
    let payload = [0x00, 0xF4, 0x01, 0x00, 0x00, 0x00, 0x00, 0xFF];

    c.bench_function("decode_multiplexed", |b| {
        b.iter(|| dbc.decode(black_box(300), black_box(&payload), false))
    });
}

fn bench_decode_multiplexed_throughput(c: &mut Criterion) {
    // Test throughput when cycling through different multiplexer values
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 300 MultiplexedSensors : 8 ECM
 SG_ SensorID M : 0|8@1+ (1,0) [0|3] ""
 SG_ Temperature m0 : 8|16@1- (0.1,-40) [-40|125] "°C"
 SG_ Pressure m1 : 8|16@1+ (0.01,0) [0|655.35] "kPa"
 SG_ Humidity m2 : 8|16@1+ (0.01,0) [0|100] "%"
 SG_ Voltage m3 : 8|16@1+ (0.001,0) [0|65.535] "V"
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    let payloads = [
        [0x00, 0xF4, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00], // SensorID=0
        [0x01, 0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00], // SensorID=1
        [0x02, 0x88, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00], // SensorID=2
        [0x03, 0xE8, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00], // SensorID=3
    ];

    c.bench_function("decode_multiplexed_throughput", |b| {
        b.iter(|| {
            for payload in &payloads {
                black_box(dbc.decode(300, payload, false).unwrap());
            }
        })
    });
}

fn bench_decode_extended_multiplexing(c: &mut Criterion) {
    // DBC with extended multiplexing (SG_MUL_VAL_)
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 400 ExtMuxMessage : 8 ECM
 SG_ Mode M : 0|8@1+ (1,0) [0|255] ""
 SG_ SubMode M : 8|8@1+ (1,0) [0|255] ""
 SG_ DataA m0 : 16|16@1+ (1,0) [0|65535] ""
 SG_ DataB m0 : 32|16@1+ (1,0) [0|65535] ""

SG_MUL_VAL_ 400 DataA Mode 0-10 ;
SG_MUL_VAL_ 400 DataA SubMode 0-5 ;
SG_MUL_VAL_ 400 DataB Mode 0-10 ;
SG_MUL_VAL_ 400 DataB SubMode 6-10 ;
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // Mode=5, SubMode=3 -> DataA should decode, DataB should not
    let payload = [0x05, 0x03, 0x00, 0x10, 0x00, 0x20, 0x00, 0x00];

    c.bench_function("decode_extended_multiplexing", |b| {
        b.iter(|| dbc.decode(black_box(400), black_box(&payload), false))
    });
}

fn bench_decode_signed_signals(c: &mut Criterion) {
    // DBC with signed signals - tests sign extension code path
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 SignedData : 8 ECM
 SG_ SignedTemp : 0|8@1- (1,-40) [-40|87] "°C"
 SG_ SignedAccel : 8|16@1- (0.01,0) [-327.68|327.67] "m/s²"
 SG_ SignedAngle : 24|16@1- (0.1,0) [-3276.8|3276.7] "°"
 SG_ SignedSmall : 40|4@1- (1,0) [-8|7] ""
"#;

    let dbc = Dbc::parse(dbc_content).unwrap();
    // SignedTemp=-5 (0xFB), SignedAccel=-100 (0xFF9C), SignedAngle=450.0 (0x1194), SignedSmall=-3 (0xD)
    let payload = [0xFB, 0x9C, 0xFF, 0x94, 0x11, 0x0D, 0x00, 0x00];

    c.bench_function("decode_signed_signals", |b| {
        b.iter(|| dbc.decode(black_box(256), black_box(&payload), false))
    });
}

#[cfg(not(feature = "std"))]
criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_decode_simple,
    bench_decode_multiple_signals,
    bench_decode_message_lookup_first,
    bench_decode_message_lookup_middle,
    bench_decode_message_lookup_last,
    bench_decode_high_throughput,
    bench_decode_big_endian,
    bench_decode_little_endian,
    bench_decode_with_value_descriptions,
    bench_decode_without_value_descriptions,
    bench_decode_multiplexed,
    bench_decode_multiplexed_throughput,
    bench_decode_extended_multiplexing,
    bench_decode_signed_signals
);

#[cfg(feature = "std")]
criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_to_dbc_string,
    bench_decode_simple,
    bench_decode_multiple_signals,
    bench_decode_message_lookup_first,
    bench_decode_message_lookup_middle,
    bench_decode_message_lookup_last,
    bench_decode_high_throughput,
    bench_decode_big_endian,
    bench_decode_little_endian,
    bench_decode_with_value_descriptions,
    bench_decode_without_value_descriptions,
    bench_decode_multiplexed,
    bench_decode_multiplexed_throughput,
    bench_decode_extended_multiplexing,
    bench_decode_signed_signals
);

criterion_main!(benches);
