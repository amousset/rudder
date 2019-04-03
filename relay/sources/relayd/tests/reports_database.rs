use diesel::{self, prelude::*, PgConnection};
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
    let _ = remove_dir_all("tests/tmp/test_simple");
    create_dir_all("tests/tmp/test_simple/incoming").unwrap();
    let cli_cfg = CliConfiguration::new("tests/test_simple/relayd.conf");

    thread::spawn(move || {
        start(cli_cfg).unwrap();
    });

    copy(
        "tests/runlogs/normal.log",
        "tests/tmp/test_simple/incoming/2019-01-24T15:55:01+00:00@root.log",
    )
    .unwrap();
    thread::sleep(time::Duration::from_secs(1));

    let results = ruddersysevents
        .filter(component.eq("start"))
        .limit(1)
        .load::<QueryableReport>(&db)
        .expect("Error loading posts");

    assert_eq!(
        results.first().unwrap().clone().event_type.unwrap(),
        "control"
    );
}
