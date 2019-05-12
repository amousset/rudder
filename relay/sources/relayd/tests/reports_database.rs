use diesel::{self, prelude::*, PgConnection};
use filetime::{set_file_times, FileTime};
use relayd::{
    configuration::CliConfiguration, data::report::QueryableReport, init,
    output::database::schema::ruddersysevents::dsl::*,
};
use std::{
    fs::{copy, create_dir_all, remove_dir_all},
    path::Path,
    thread, time,
};

fn db_connection() -> PgConnection {
    PgConnection::establish("postgres://rudderreports:PASSWORD@127.0.0.1/rudder").unwrap()
}

// Checks number of start execution reports
// (so number of runlogs if everything goes well)
fn start_number(db: &PgConnection, expected: usize) -> Result<(), ()> {
    let mut retry = 10;
    while retry > 0 {
        thread::sleep(time::Duration::from_millis(200));
        retry -= 1;
        let results = ruddersysevents
            .filter(component.eq("start"))
            .limit(10)
            .load::<QueryableReport>(db)
            .unwrap();
        if results.len() == expected {
            return Ok(());
        }
    }
    Err(())
}

#[test]
fn it_reads_and_inserts_a_runlog() {
    let db = db_connection();
    diesel::delete(ruddersysevents).execute(&db).unwrap();

    assert!(start_number(&db, 0).is_ok());

    let _ = remove_dir_all("target/tmp/test_simple");
    create_dir_all("target/tmp/test_simple/incoming").unwrap();
    let cli_cfg = CliConfiguration::new("tests/test_simple/relayd.conf", false);

    let file_old = "target/tmp/test_simple/incoming/2017-01-24T15:55:01+00:00@root.log";
    let file_new = "target/tmp/test_simple/incoming/2018-01-24T15:55:01+00:00@root.log";
    let file_broken = "target/tmp/test_simple/incoming/2018-02-24T15:55:01+00:00@root.log";
    let file_failed = "target/tmp/test_simple/failed/2018-02-24T15:55:01+00:00@root.log";

    copy("tests/runlogs/normal_old.log", file_old).unwrap();
    // We need to file to be old
    set_file_times(file_old, FileTime::zero(), FileTime::zero()).unwrap();

    thread::spawn(move || {
        init(cli_cfg).unwrap();
    });

    assert!(start_number(&db, 1).is_ok());

    copy("tests/runlogs/normal.log", file_new).unwrap();
    copy("tests/files/relayd.toml", file_broken).unwrap();

    assert!(start_number(&db, 2).is_ok());

    // Test files have been removed
    assert!(!Path::new(file_old).exists());
    assert!(!Path::new(file_new).exists());
    // Test broken file has been moved
    assert!(!Path::new(file_broken).exists());
    assert!(Path::new(file_failed).exists());

    // TODO check stats api
}
