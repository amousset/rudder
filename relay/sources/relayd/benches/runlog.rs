extern crate bencher;
extern crate relayd;

use bencher::Bencher;
use bencher::{benchmark_group, benchmark_main};
use relayd::data::reporting::RunLog;
use std::fs::{read_to_string, read};
use flate2::read::GzDecoder;
use std::str::FromStr;
use std::io::Read;

fn bench_parse_runlog(b: &mut Bencher) {
    let runlog = read_to_string("tests/runlogs/normal.log").unwrap();
    b.iter(|| RunLog::from_str(&runlog).unwrap())
}

// Allows comparing gzip implementations
fn bench_uncompress_runlog(b: &mut Bencher) {
    // same as in input.rs
    let data = read("tests/runlogs/normal.log.gz").unwrap();
    b.iter(|| 
        {
            let mut gz = GzDecoder::new(data.as_slice());
            let mut s = String::new();
            gz.read_to_string(&mut s).unwrap();
        }
    )
}

benchmark_group!(benches, bench_parse_runlog, bench_uncompress_runlog);
benchmark_main!(benches);
