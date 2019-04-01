#![feature(test)]

extern crate test;
extern crate relayd;

use std::fs::read_to_string;
use relayd::data::reporting::RunLog;
use std::str::FromStr;

#[bench]
fn bench_parse_report(b: &mut test::Bencher) {
    let runlog = read_to_string("tests/runlogs/normal.log").unwrap();
    b.iter(||
        RunLog::from_str(&runlog).unwrap()    
    )
}