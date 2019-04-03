use diesel::{self, prelude::*, PgConnection};
use filetime::{set_file_times, FileTime};
use relayd::{
    configuration::CliConfiguration, data::reporting::QueryableReport,
    output::database::schema::ruddersysevents::dsl::*, start,
};
use std::{
    fs::{copy, create_dir_all, remove_dir_all},
    thread, time,
};

pub fn db_connection() -> PgConnection {
    PgConnection::establish("postgres://rudderreports:PASSWORD@127.0.0.1/rudder").unwrap()
}

#[test]
fn it_reads_and_inserts_a_runlog() {
    let db = db_connection();
    diesel::delete(ruddersysevents).execute(&db).unwrap();
    let _ = remove_dir_all("target/tmp/test_simple");
    create_dir_all("target/tmp/test_simple/incoming").unwrap();
    let cli_cfg = CliConfiguration::new("tests/test_simple/relayd.conf", false);

    copy(
        "tests/runlogs/normal_old.log",
        "target/tmp/test_simple/incoming/2017-01-24T15:55:01+00:00@root.log",
    )
    .unwrap();
    // We need to file to be old
    set_file_times(
        "target/tmp/test_simple/incoming/2017-01-24T15:55:01+00:00@root.log",
        FileTime::zero(),
        FileTime::zero(),
    )
    .unwrap();

    thread::spawn(move || {
        start(cli_cfg).unwrap();
    });
    thread::sleep(time::Duration::from_millis(100));

    copy(
        "tests/runlogs/normal.log",
        "target/tmp/test_simple/incoming/2018-01-24T15:55:01+00:00@root.log",
    )
    .unwrap();
    thread::sleep(time::Duration::from_millis(100));

    let results = ruddersysevents
        .filter(component.eq("start"))
        .limit(3)
        .load::<QueryableReport>(&db)
        .expect("Error loading posts");

    assert_eq!(results.len(), 2);
}
