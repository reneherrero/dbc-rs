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
    let dbc_content = r#"VERSION "1.0"

BU_: ECM

BO_ 256 Engine : 8 ECM
 SG_ RPM : 0|16@0+ (1.0,0) [0|65535] "rpm"
 SG_ Pressure : 16|16@0+ (0.1,0) [0|6553.5] "bar"
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
    bench_decode_little_endian
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
    bench_decode_little_endian
);

criterion_main!(benches);
