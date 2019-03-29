// Copyright 2019 Normation SAS
//
// This file is part of Rudder.
//
// Rudder is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// In accordance with the terms of section 7 (7. Additional Terms.) of
// the GNU General Public License version 3, the copyright holders add
// the following Additional permissions:
// Notwithstanding to the terms of section 5 (5. Conveying Modified Source
// Versions) and 6 (6. Conveying Non-Source Forms.) of the GNU General
// Public License version 3, when you create a Related Module, this
// Related Module is not considered as a part of the work and may be
// distributed under the license agreement of your choice.
// A "Related Module" means a set of sources files including their
// documentation that, without modification of the Source Code, enables
// supplementary functions or services in addition to those offered by
// the Software.
//
// Rudder is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Rudder.  If not, see <http://www.gnu.org/licenses/>.

use crate::{data::nodes::NodeId, error::Error, output::database::schema::ruddersysevents};
use chrono::prelude::*;
use lazy_static::lazy_static;
use nom::types::CompleteStr;
use nom::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use slog::{slog_debug, slog_trace, slog_warn};
use slog_scope::{debug, trace, warn};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/*
"^@@(?P<policy>.*)@@(?P<event_type>.*)@@(?P<rule_id>.*)@@(?P<directive_id>.*)@@(?P<serial>.*)@@
(?P<component>.*)@@(?P<key_value>.*)@@(?P<start_datetime>.*)##(?P<node_id>.*)@#(?s)(?P<msg>.*)$"
*/

/*
impl FromStr for RunInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {

        }
    }
}
*/

// A detail log entry
struct LogEntry {
    Level: AgentLogLevel,
    Message: String,
}

type AgentLogLevel = &'static str;

named!(
    agent_log_level<Option<AgentLogLevel>>,
    opt!(alt!(
        // CFEngine logs
        tag_s!("CRITICAL:")   => { |_| "log_warn" }  |
        tag_s!("   error:")   => { |_| "log_warn" }  |
        tag_s!(" warning:")   => { |_| "log_warn" }  |
        tag_s!("  notice:")   => { |_| "log_info" }  |
        tag_s!("    info:")   => { |_| "log_info" }  |
        tag_s!(" verbose:")   => { |_| "log_debug" } |
        tag_s!("   debug:")   => { |_| "log_debug" } |
        // ncf logs
        tag_s!("R: [FATAL]")  => { |_| "log_warn" }  |
        tag_s!("R: [ERROR]")  => { |_| "log_warn" }  |
        tag_s!("R: [INFO]")   => { |_| "log_info" }  |
        tag_s!("R: [DEBUG]")  => { |_| "log_debug" } |
        // ncf non-standard log
        tag_s!("R: WARNING")  => { |_| "log_warn" }  |
        // CFEngine stdlib log
        tag_s!("R: DEBUG")    => { |_| "log_debug" } |
        // Untagged non-Rudder reports report, assume info
        tag_s!("R: ")         => { |_| "log_info" }
        // TODO R: not @@
    ))
);

fn is_space(chr: char) -> bool {
    chr == ' ' || chr == '\t'
}
named!(space<CompleteStr,CompleteStr>, take_while_s!(is_space));

named!(log_entry<CompleteStr, LogEntry>,
    do_parse!(
               tag_s!("") >>
        level: agent_log_level >>
               opt!(space) >>
        message: take_till_s!(is_line_ending_or_comment) >>
    )
);

named!(runlog<CompleteStr, Vec<Report>>,
    many1!(
        alt!(
            report |
            log_entry
        )
    )
);

named!(report_entry<CompleteStr, CompleteStr>,
  is_not_s!("\n\r")
);

named!(
    check<u32>,
    verify!(nom::be_u32, |val: u32| val >= 0 && val < 3)
);

named!(report<CompleteStr, Report>, do_parse!(
    // TODO NOT CORRECT
    // pas de line break dans une ligne
    detail: take_until_and_consume_s!("R: @@") >>
    policy: take_until_and_consume_s!("@@") >>
    event_type: take_until_and_consume_s!("@@") >>
    rule_id: take_until_and_consume_s!("@@") >>
    directive_id: take_until_and_consume_s!("@@") >>
    serial: take_until_and_consume_s!("@@") >>
    component: take_until_and_consume_s!("@@") >>
    key_value: take_until_and_consume_s!("@@") >>
    start_datetime: take_until_and_consume_s!("##") >>
    node_id: take_until_and_consume_s!("@#") >>
    msg: map!(take_until_s!("\n")|String::from) >>
        (Report {
           // FIXME execution date should be generated at execution
            execution_datetime: DateTime::parse_from_str(start_datetime, "%Y-%m-%d %H:%M:%S%z").unwrap(),
            node_id: node_id.map(String::from),
            rule_id: rule_id.map(String::from),
            directive_id: directive_id.map(String::from),
            serial: serial.parse::<i32>().unwrap(),
            component: component.map(String::from),
            key_value: key_value.map(String::from),
            start_datetime: DateTime::parse_from_str(start_datetime, "%Y-%m-%d %H:%M:%S%z").unwrap(),
            event_type: event_type.map(String::from),
            msg: msg.map(String::from),
            policy: policy.map(String::from),
        })
));

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawReport {
    result_report: Report,
    log_entries: Vec<LogEntry>,
}

// Impl from between the 2

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Insertable)]
#[table_name = "ruddersysevents"]
pub struct Report {
    #[column_name = "executiondate"]
    pub start_datetime: DateTime<FixedOffset>,
    #[column_name = "ruleid"]
    pub rule_id: String,
    #[column_name = "directiveid"]
    pub directive_id: String,
    pub component: String,
    #[column_name = "keyvalue"]
    pub key_value: String,
    // Not parsed as we do not use it and do not want to prevent future changes
    #[column_name = "eventtype"]
    pub event_type: String,
    #[column_name = "msg"]
    pub msg: String,
    #[column_name = "policy"]
    pub policy: String,
    #[column_name = "nodeid"]
    pub node_id: NodeId,
    #[column_name = "executiontimestamp"]
    pub execution_datetime: DateTime<FixedOffset>,
    pub serial: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunInfo {
    pub node_id: NodeId,
    pub timestamp: DateTime<FixedOffset>,
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "@@{:}@@{:}@@{:}@@{:}@@{:}@@{:}@@{:}@@{:}##{:}@#{:}",
            self.policy,
            self.event_type,
            self.rule_id,
            self.directive_id,
            self.serial,
            self.component,
            self.key_value,
            self.start_datetime,
            self.node_id,
            self.msg,
        )
    }
}

impl Display for RunLog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for report in &self.reports {
            writeln!(f, "R: {:}", report)?
        }
        Ok(())
    }
}

impl FromStr for RunInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RUNINFO: Regex = Regex::new(r"^(?P<timestamp>.+)@(?P<node_id>.+)\.log$")
                .expect("Invalid report parsing regex");
        }
        trace!("Parsing report: '{}'", s; "component" => "parser");

        let cap = match RUNINFO.captures(s) {
            None => {
                warn!("Could not parse run info: '{}'", s; "component" => "parser");
                return Err(Error::InvalidReport);
            }
            Some(capture) => capture,
        };

        let report = RunInfo {
            node_id: cap["node_id"].into(),
            timestamp: DateTime::parse_from_str(&cap["timestamp"], "%+")?,
        };
        debug!("Parsed run info {:#?}", report; "component" => "parser");
        Ok(report)
    }
}

impl FromStr for Report {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref REPORT: Regex = Regex::new(
                r"^@@(?P<policy>.*)@@(?P<report_type>.*)@@(?P<rule_id>.*)@@(?P<directive_id>.*)@@(?P<serial>.*)@@(?P<component>.*)@@(?P<key_value>.*)@@(?P<timestamp>.*)##(?P<node_id>.*)@#(?s)(?P<msg>.*)$"
            ).expect("Invalid report parsing regex");
        }
        debug!("Parsing report: '{}'", s; "component" => "parser");

        let cap = match REPORT.captures(s) {
            None => {
                warn!("Could not parse report: '{}'", s; "component" => "parser");
                return Err(Error::InvalidReport);
            }
            Some(capture) => capture,
        };

        let report = Report {
            node_id: cap["node_id"].into(),
            start_datetime: DateTime::parse_from_str(&cap["timestamp"], "%Y-%m-%d %H:%M:%S%z")?,
            execution_datetime: DateTime::parse_from_str(&cap["timestamp"], "%Y-%m-%d %H:%M:%S%z")?,
            rule_id: cap["rule_id"].into(),
            directive_id: cap["directive_id"].into(),
            component: cap["component"].into(),
            key_value: cap["key_value"].into(),
            event_type: cap["report_type"].into(),
            msg: cap["msg"].into(),
            detail: "".into(),
            policy: cap["policy"].into(),
            serial: cap["serial"].parse::<i32>()?,
        };
        debug!("Parsed report {:#?}", report; "component" => "parser");
        Ok(report)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunLog {
    pub info: RunInfo,
    pub reports: Vec<Report>,
}

impl FromStr for RunLog {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut info = None;
        let mut reports = vec![];
        lazy_static! {
            static ref REPORT: Regex =
                Regex::new(r"\n?R: ").expect("Invalid report begin parsing regex");
        }
        for cap in REPORT.split(s).skip(1) {
            let report = match Report::from_str(cap) {
                Ok(report) => report,
                Err(Error::InvalidReport) => continue,
                Err(e) => {
                    return Err(e);
                }
            };

            match info {
                None => {
                    info = Some(RunInfo {
                        node_id: report.node_id.clone(),
                        timestamp: report.start_datetime,
                    });
                }
                Some(ref info) => {
                    if info.node_id != report.node_id {
                        warn!("Wrong node id in report {:#?}, skipping", report; "component" => "parser");
                    }
                    if info.timestamp != report.start_datetime {
                        warn!(
                            "Wrong execution timestamp in report {:#?}, skipping",
                            report; "component" => "parser"
                        );
                    }
                }
            }
            reports.push(report)
        }

        let runlog = match info {
            None => Err(Error::EmptyRunlog)?,
            Some(info) => Ok(RunLog { info, reports }),
        };
        debug!("Parsed runlog {:#?}", runlog; "component" => "parser");
        runlog
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn test_parse_report() {
        let report = "@@Common@@result_repaired@@hasPolicyServer-root@@common-root@@0@@CRON Daemon@@None@@2018-08-24 15:55:01+00:00##root@#Cron daemon status was repaired";
        let report = Report::from_str(report).unwrap();
        assert_eq!(
            report,
            Report {
                start_datetime: DateTime::parse_from_str(
                    "2018-08-24 15:55:01+00:00",
                    "%Y-%m-%d %H:%M:%S%z"
                )
                .unwrap(),
                rule_id: "hasPolicyServer-root".into(),
                directive_id: "common-root".into(),
                component: "CRON Daemon".into(),
                key_value: "None".into(),
                event_type: "result_repaired".into(),
                msg: "Cron daemon status was repaired".into(),
                policy: "Common".into(),
                node_id: "root".into(),
                serial: 0,
                detail: "".into(),

                execution_datetime: DateTime::parse_from_str(
                    "2018-08-24 15:55:01+00:00",
                    "%Y-%m-%d %H:%M:%S%z"
                )
                .unwrap(),
            }
        );

        let report2 = "@@Common@@result_repaired@@hasPolicyServer-root@@common-root@@0@@CRON Daemon@@None@@2018-08-24 15:55:01+00:00##root@#Cron daemon R: status##was @@ repaired\nnext line";
        let report2 = Report::from_str(report2).unwrap();
        assert_eq!(
            report2,
            Report {
                start_datetime: DateTime::parse_from_str(
                    "2018-08-24 15:55:01+00:00",
                    "%Y-%m-%d %H:%M:%S%z"
                )
                .unwrap(),
                rule_id: "hasPolicyServer-root".into(),
                directive_id: "common-root".into(),
                component: "CRON Daemon".into(),
                key_value: "None".into(),
                event_type: "result_repaired".into(),
                msg: "Cron daemon R: status##was @@ repaired\nnext line".into(),
                policy: "Common".into(),
                node_id: "root".into(),
                serial: 0,
                detail: "".into(),

                execution_datetime: DateTime::parse_from_str(
                    "2018-08-24 15:55:01+00:00",
                    "%Y-%m-%d %H:%M:%S%z"
                )
                .unwrap(),
            }
        );
    }

    #[test]
    fn test_display_report() {
        let report = "@@Common@@result_repaired@@hasPolicyServer-root@@common-root@@0@@CRON Daemon@@None@@2018-08-24 15:55:01 +00:00##root@#Cron daemon status was repaired";
        //let report = Report::from_str(report).unwrap();
        assert_eq!(
            report,
            format!(
                "{:}",
                Report {
                    start_datetime: DateTime::parse_from_str(
                        "2018-08-24 15:55:01+00:00",
                        "%Y-%m-%d %H:%M:%S%z"
                    )
                    .unwrap(),
                    rule_id: "hasPolicyServer-root".into(),
                    directive_id: "common-root".into(),
                    component: "CRON Daemon".into(),
                    key_value: "None".into(),
                    event_type: "result_repaired".into(),
                    msg: "Cron daemon status was repaired".into(),
                    policy: "Common".into(),
                    node_id: "root".into(),
                    serial: 0,
                    detail: "".into(),
                    execution_datetime: DateTime::parse_from_str(
                        "2018-08-24 15:55:01+00:00",
                        "%Y-%m-%d %H:%M:%S%z"
                    )
                    .unwrap(),
                }
            )
        );
    }

    #[test]
    fn test_parse_invalid_report() {
        let report = "Not a report";
        assert!(match Report::from_str(report) {
            Err(Error::InvalidReport) => true,
            _ => false,
        });
    }

    #[test]
    fn test_parse_runinfo() {
        let runlog_file = "2018-08-24T15:55:01+00:00@root.log";
        let runinfo = RunInfo::from_str(runlog_file).unwrap();
        assert_eq!(
            runinfo,
            RunInfo {
                timestamp: DateTime::parse_from_str("2018-08-24T15:55:01+00:00", "%+").unwrap(),
                node_id: "root".into(),
            }
        );
    }

    #[test]
    fn test_parse_runlog() {
        let run_log = &read_to_string("tests/runlogs/2018-08-24T15:55:01+00:00@root.log").unwrap();
        let run = RunLog::from_str(run_log).unwrap();
        assert_eq!(run.info.node_id, "root".to_owned());
        assert_eq!(run.reports[0].msg, "Start execution".to_owned());
        assert_eq!(
            run.reports[1].msg,
            "Configuration library initialization\nwas correct".to_owned()
        );
        assert_eq!(
            run.reports[10].msg,
            "Remove file /var/rudder/tmp/rudder_monitoring.csv was correct".to_owned()
        );
    }
}
