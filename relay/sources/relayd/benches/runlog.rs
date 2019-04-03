use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flate2::read::GzDecoder;
use relayd::data::reporting::RunLog;
use std::{
    fs::{read, read_to_string},
    io::Read,
    str::FromStr,
};

fn bench_parse_runlog(c: &mut Criterion) {
    let runlog = read_to_string("tests/runlogs/normal.log").unwrap();
    c.bench_function("parse runlog", move |b| {
        b.iter(|| black_box(RunLog::from_str(&runlog).unwrap()))
    });
}

// Allows comparing gzip implementations
fn bench_uncompress_runlog(c: &mut Criterion) {
    // same as in input.rs
    let data = read("tests/runlogs/normal.log.gz").unwrap();
    c.bench_function("uncompress runlog", move |b| {
        b.iter(|| {
            let mut gz = GzDecoder::new(data.as_slice());
            let mut s = String::new();
            gz.read_to_string(&mut s).unwrap();
            black_box(s);
        })
    });
}

criterion_group!(benches, bench_parse_runlog, bench_uncompress_runlog);
criterion_main!(benches);
