extern crate bencher;
extern crate relayd;

use bencher::Bencher;
use bencher::{benchmark_group, benchmark_main};
use relayd::data::reporting::RunLog;
use std::fs::read_to_string;
use std::str::FromStr;

fn bench_parse_runlog(b: &mut Bencher) {
    let runlog = read_to_string("tests/runlogs/normal.log").unwrap();
    b.iter(|| RunLog::from_str(&runlog).unwrap())
}

benchmark_group!(benches, bench_parse_runlog);
benchmark_main!(benches);
