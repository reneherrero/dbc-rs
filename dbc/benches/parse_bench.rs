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

#[cfg(not(feature = "std"))]
criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large
);

#[cfg(feature = "std")]
criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_to_dbc_string
);

criterion_main!(benches);
