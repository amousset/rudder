// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

mod common;

use relayd::{configuration::cli::CliConfiguration, init_logger, start};
use std::{
    fs::{read_to_string, remove_file},
    thread, time,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_runs_local_remote_run() {
        let cli_cfg = CliConfiguration::new("tests/files/config/", false);

        thread::spawn(move || {
            start(cli_cfg, init_logger().unwrap()).unwrap();
        });

        assert!(common::start_api().is_ok());
        let client = reqwest::blocking::Client::new();

        // Async & keep

        let _ = remove_file("target/tmp/api_test.txt");
        let params_async = [
            ("asynchronous", "true"),
            ("keep_output", "true"),
            ("classes", "class2,class3"),
            ("nodes", "root"),
        ];
        let mut response = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes")
            .form(&params_async)
            .send()
            .unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(response.text().unwrap(), "OK\nEND\n".to_string());
        assert_eq!(
            "remote run -D class2,class3 server.rudder.local".to_string(),
            read_to_string("target/tmp/api_test.txt").unwrap()
        );

        let _ = remove_file("target/tmp/api_test.txt");
        let params_async = [
            ("asynchronous", "true"),
            ("keep_output", "true"),
            ("classes", "class2,class45"),
            ("nodes", "e745a140-40bc-4b86-b6dc-084488fc906b"),
        ];
        let mut response = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes")
            .form(&params_async)
            .send()
            .unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(response.text().unwrap(), "OK\nEND\n".to_string());
        assert_eq!(
            "remote run -D class2,class45 node1.rudder.local".to_string(),
            read_to_string("target/tmp/api_test.txt").unwrap()
        );

        let _ = remove_file("target/tmp/api_test.txt");
        let params_async = [
            ("asynchronous", "true"),
            ("keep_output", "true"),
            ("classes", "class2,class46"),
        ];
        let mut response = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes/e745a140-40bc-4b86-b6dc-084488fc906b")
            .form(&params_async)
            .send()
            .unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(response.text().unwrap(), "OK\nEND\n".to_string());
        assert_eq!(
            "remote run -D class2,class46 node1.rudder.local".to_string(),
            read_to_string("target/tmp/api_test.txt").unwrap()
        );

        // Async & no keep

        let _ = remove_file("target/tmp/api_test.txt");
        let params_sync = [
            ("asynchronous", "true"),
            ("keep_output", "false"),
            ("classes", "class2,class4"),
            ("nodes", "root"),
        ];
        let mut response = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes")
            .form(&params_sync)
            .send()
            .unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(response.text().unwrap(), "".to_string());
        // async, let's wait a bit
        thread::sleep(time::Duration::from_millis(700));
        assert_eq!(
            "remote run -D class2,class4 server.rudder.local".to_string(),
            read_to_string("target/tmp/api_test.txt").unwrap()
        );

        // Sync & keep

        let _ = remove_file("target/tmp/api_test.txt");
        let params_sync = [
            ("asynchronous", "false"),
            ("keep_output", "true"),
            ("classes", "class2,class5"),
            ("nodes", "root"),
        ];
        let mut response = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes")
            .form(&params_sync)
            .send()
            .unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(response.text().unwrap(), "OK\nEND\n".to_string());
        assert_eq!(
            "remote run -D class2,class5 server.rudder.local".to_string(),
            read_to_string("target/tmp/api_test.txt").unwrap()
        );

        // Sync & no keep

        let _ = remove_file("target/tmp/api_test.txt");
        let params_sync = [
            ("asynchronous", "false"),
            ("keep_output", "false"),
            ("classes", "class2,class6"),
            ("nodes", "root"),
        ];
        let mut response = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes")
            .form(&params_sync)
            .send()
            .unwrap();
        assert_eq!(response.status(), hyper::StatusCode::OK);
        assert_eq!(response.text().unwrap(), "".to_string());
        assert_eq!(
            "remote run -D class2,class6 server.rudder.local".to_string(),
            read_to_string("target/tmp/api_test.txt").unwrap()
        );

        // Failure

        let params = [
            ("asynchronous", "false"),
            ("keep_output", "true"),
            ("classes", "clas~1,class2,class3"),
            ("nodes", "node2.rudder.local,server.rudder.local"),
        ];
        let res = client
            .post("http://localhost:3030/rudder/relay-api/1/remote-run/nodes")
            .form(&params)
            .send();

        assert_eq!(res.unwrap().text().unwrap(), "Unhandled rejection: invalid condition: clas~1, should match ^[a-zA-Z0-9][a-zA-Z0-9_]*$".to_string());
    }
}
