use criterion::{black_box, criterion_group, criterion_main, Criterion};
use diesel::{self, prelude::*};
use flate2::read::GzDecoder;
use openssl::{stack::Stack, x509::X509};
use relayd::{
    configuration::DatabaseConfig,
    data::{report::QueryableReport, RunLog},
    input::signature,
    output::database::{schema::ruddersysevents::dsl::*, *},
};
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

fn bench_signature_runlog(c: &mut Criterion) {
    let data = read("tests/test_smime/normal.signed").unwrap();

    let x509 = X509::from_pem(
        read_to_string("tests/keys/e745a140-40bc-4b86-b6dc-084488fc906b.cert")
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    let mut certs = Stack::new().unwrap();
    certs.push(x509).unwrap();

    c.bench_function("verify runlog signature", move |b| {
        b.iter(|| {
            black_box(signature(&data, &certs).unwrap());
        })
    });
}

// Allows comparing gzip implementations
fn bench_uncompress_runlog(c: &mut Criterion) {
    // same as in input.rs
    let data = read("tests/test_gz/normal.log.gz").unwrap();
    c.bench_function("uncompress runlog", move |b| {
        b.iter(|| {
            let mut gz = GzDecoder::new(data.as_slice());
            let mut s = String::new();
            gz.read_to_string(&mut s).unwrap();
            black_box(s);
        })
    });
}

pub fn db() -> PgPool {
    let db_config = DatabaseConfig {
        url: "postgres://rudderreports:PASSWORD@127.0.0.1/rudder".to_string(),
        max_pool_size: 10,
    };
    pg_pool(&db_config).unwrap()
}

fn bench_insert_runlog(c: &mut Criterion) {
    let pool = db();
    let db = &*pool.get().unwrap();

    diesel::delete(ruddersysevents).execute(db).unwrap();
    let results = ruddersysevents
        .limit(1)
        .load::<QueryableReport>(db)
        .unwrap();
    assert_eq!(results.len(), 0);

    let runlog = RunLog::from_str(&read_to_string("tests/runlogs/normal.log").unwrap()).unwrap();

    // Test inserting the runlog

    c.bench_function("insert runlog", move |b| {
        b.iter(|| {
            assert_eq!(
                insert_runlog(&pool, &runlog, InsertionBehavior::AllowDuplicate).unwrap(),
                RunlogInsertion::Inserted
            );
        })
    });

    let results = ruddersysevents
        .limit(1)
        .load::<QueryableReport>(db)
        .unwrap();
    assert_eq!(results.len(), 1);
}

criterion_group!(
    benches,
    bench_uncompress_runlog,
    bench_signature_runlog,
    bench_parse_runlog,
    bench_insert_runlog
);
criterion_main!(benches);
