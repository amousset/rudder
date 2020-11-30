// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use crate::{
    configuration::main::RemoteRun as RemoteRunCfg, data::node::Host, error::Error, JobConfig,
};
use bytes::Bytes;
use futures::{
    stream::{select, select_all},
    Stream, StreamExt, TryStreamExt,
};
use hyper::Body;
use regex::Regex;
use std::{collections::HashMap, pin::Pin, process::Stdio, str::FromStr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use tracing::{debug, error, span, trace, Level};

#[derive(Debug)]
pub struct RemoteRun {
    target: RemoteRunTarget,
    run_parameters: RunParameters,
}

impl RemoteRun {
    pub fn new(target: RemoteRunTarget, options: &HashMap<String, String>) -> Result<Self, Error> {
        Ok(RemoteRun {
            target,
            run_parameters: RunParameters::new(
                options.get("asynchronous"),
                options.get("keep_output"),
                if options.contains_key("conditions") {
                    options.get("conditions")
                } else {
                    options.get("classes")
                },
            )?,
        })
    }

    async fn consume(
        mut stream: Pin<&mut dyn Stream<Item = Result<Bytes, Error>>>,
    ) -> Result<(), ()> {
        while let Some(l) = stream.next().await {
            match l {
                Ok(l) => trace!("Read {:#?}", l),
                Err(e) => error!("Stream error: {}", e),
            }
        }
        Ok(())
    }

    pub async fn run(
        &self,
        job_config: Arc<JobConfig>,
    ) -> Result<impl warp::reply::Reply, warp::reject::Rejection> {
        debug!(
            "Starting remote run (asynchronous: {}, keep_output: {})",
            self.run_parameters.asynchronous, self.run_parameters.keep_output
        );
        match (
            self.run_parameters.asynchronous,
            self.run_parameters.keep_output,
        ) {
            // Async and output -> spawn in background and stream output
            (true, true) => Ok(warp::reply::html(Body::wrap_stream(select(
                self.run_parameters.remote_run(
                    &job_config.cfg.remote_run,
                    self.target.neighbors(job_config.clone()),
                    self.run_parameters.asynchronous,
                ),
                select_all(self.target.next_hops(job_config.clone()).iter().map(
                    |(relay, target)| {
                        self.forward_call(job_config.clone(), relay.clone(), target.clone())
                    },
                )),
            )))),
            // Async and no output -> spawn in background and return early
            (true, false) => {
                for (relay, target) in self.target.next_hops(job_config.clone()) {
                    tokio::spawn(RemoteRun::consume(self.forward_call(
                        job_config.clone(),
                        relay,
                        target,
                    )));
                }
                tokio::spawn(RemoteRun::consume(self.run_parameters.remote_run(
                    &job_config.cfg.remote_run,
                    self.target.neighbors(job_config.clone()),
                    self.run_parameters.asynchronous,
                )));

                Ok(warp::reply::html(Body::empty()))
            }
            // Sync and no output -> wait until the send and return empty output
            (false, false) => Ok(warp::reply::html(Body::wrap_stream(
                self.run_parameters
                    .remote_run(
                        &job_config.cfg.remote_run,
                        self.target.neighbors(job_config.clone()),
                        self.run_parameters.asynchronous,
                    )
                    .map(|_| Chunk::from(""))
                    .select(select_all(
                        self.target
                            .next_hops(job_config.clone())
                            .iter()
                            .map(|(relay, target)| {
                                self.forward_call(job_config.clone(), relay.clone(), target.clone())
                            }),
                    )),
            ))),
            // Sync and output -> wait until the end and return output
            (false, true) => Ok(warp::reply::html(Body::wrap_stream(
                self.run_parameters
                    .remote_run(
                        &job_config.cfg.remote_run,
                        self.target.neighbors(job_config.clone()),
                        self.run_parameters.asynchronous,
                    )
                    .select(select_all(
                        self.target
                            .next_hops(job_config.clone())
                            .iter()
                            .map(|(relay, target)| {
                                self.forward_call(job_config.clone(), relay.clone(), target.clone())
                            }),
                    )),
            ))),
        }
    }

    async fn forward_call(
        &self,
        job_config: Arc<JobConfig>,
        node: Host,
        // Target for the sub relay
        target: RemoteRunTarget,
    ) -> Box<dyn Stream<Item = Result<Bytes, Error>>> {
        let report_span = span!(Level::TRACE, "upstream");
        let _report_enter = report_span.enter();

        debug!("Forwarding remote-run to {} for {:#?}", node, self.target);

        // We cannot simply serialize it using `.form()` as we
        // need specific formatting
        let mut params = HashMap::new();
        params.insert("keep_output", self.run_parameters.keep_output.to_string());
        params.insert("asynchronous", self.run_parameters.asynchronous.to_string());
        params.insert(
            "classes",
            self.run_parameters
                .conditions
                .iter()
                .map(|c| c.data.as_ref())
                .collect::<Vec<&str>>()
                .join(","),
        );
        if let RemoteRunTarget::Nodes(nodes) = &target {
            params.insert("nodes", nodes.join(","));
        }

        Box::new(
            job_config
                .client
                .clone()
                .post(&format!(
                    "https://{}/rudder/relay-api/remote-run/{}",
                    node,
                    match target {
                        RemoteRunTarget::All => "all",
                        RemoteRunTarget::Nodes(_) => "nodes",
                    },
                ))
                .form(&params)
                .send()
                .await
                // Fail if HTTP error
                .and_then(|response| response.error_for_status())
                // FIXME
                // Result<Stream<Result<Bytes, E>>, E>
                .and_then(|response| response.bytes_stream())
                .or_else(|e: Error| {
                    error!("{}", e);
                    futures::stream::once(futures::future::ready(Err(e.into())))
                })
                .then(|r| r.unwrap()), //.map(|c| c.into()),
        )
        // Don't fail if a relay is not available,
        // just log it
        // FIXME check error behavior with tests
        //.or_else(|_: Error| futures::future::empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteRunTarget {
    All,
    Nodes(Vec<String>),
}

impl RemoteRunTarget {
    pub fn neighbors(&self, job_config: Arc<JobConfig>) -> Vec<Host> {
        let nodes = job_config.nodes.read().expect("Cannot read nodes list");
        let neighbors = match self {
            RemoteRunTarget::All => nodes.my_neighbors(),
            RemoteRunTarget::Nodes(nodeslist) => nodes.my_neighbors_from(nodeslist),
        };
        debug!("Neighbors: {:#?}", neighbors);
        neighbors
    }

    pub fn next_hops(&self, job_config: Arc<JobConfig>) -> Vec<(Host, RemoteRunTarget)> {
        let nodes = job_config.nodes.read().expect("Cannot read nodes list");
        let next_hops = match self {
            RemoteRunTarget::All => nodes
                .my_sub_relays()
                .into_iter()
                .map(|r| (r, RemoteRunTarget::All))
                .collect(),
            RemoteRunTarget::Nodes(nodeslist) => nodes
                .my_sub_relays_from(nodeslist)
                .into_iter()
                .map(|(relay, nodes)| (relay, RemoteRunTarget::Nodes(nodes)))
                .collect(),
        };
        debug!("Next-hops: {:#?}", next_hops);
        next_hops
    }
}

#[derive(Debug, PartialEq)]
struct Condition {
    data: String,
}

impl FromStr for Condition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let condition_regex = r"^[a-zA-Z0-9][a-zA-Z0-9_]*$";
        let re = Regex::new(condition_regex).unwrap();
        let max_length = 1024;
        if s.len() > max_length {
            return Err(Error::MaxLengthCondition {
                condition: s.to_string(),
                max_length,
            });
        }
        if !re.is_match(s) {
            Err(Error::InvalidCondition {
                condition: s.to_string(),
                condition_regex,
            })
        } else {
            Ok(Condition {
                data: s.to_string(),
            })
        }
    }
}

#[derive(Debug, PartialEq)]
struct RunParameters {
    asynchronous: bool,
    keep_output: bool,
    conditions: Vec<Condition>,
}

impl RunParameters {
    pub fn new(
        raw_asynchronous: Option<&String>,
        raw_keep_output: Option<&String>,
        raw_conditions: Option<&String>,
    ) -> Result<Self, Error> {
        let conditions: Vec<_> = match raw_conditions {
            Some(conditions) if !conditions.is_empty() => {
                let split_conditions: Result<Vec<_>, _> = conditions
                    .split(',')
                    .map(|s| Condition::from_str(s))
                    .collect();
                split_conditions?
            }
            _ => vec![],
        };
        let asynchronous = match raw_asynchronous {
            Some(asynchronous) => asynchronous.parse::<bool>()?,
            None => false,
        };
        let keep_output = match raw_keep_output {
            Some(keep_output) => keep_output.parse::<bool>()?,
            None => false,
        };

        Ok(RunParameters {
            asynchronous,
            keep_output,
            conditions,
        })
    }

    pub fn command(&self, cfg: &RemoteRunCfg, nodes: Vec<String>) -> Command {
        assert!(!nodes.is_empty());

        let mut cmd = if cfg.use_sudo {
            let mut tmp = Command::new("sudo");
            tmp.arg(&cfg.command);
            tmp
        } else {
            Command::new(&cfg.command)
        };
        cmd.arg("remote".to_string());
        cmd.arg("run".to_string());
        if !&self.conditions.is_empty() {
            cmd.arg("-D".to_string());
            cmd.arg(
                self.conditions
                    .iter()
                    .map(|c| c.data.as_str())
                    .collect::<Vec<&str>>()
                    .join(","),
            );
        }
        cmd.arg(nodes.join(","));
        debug!("Remote run command: '{:#?}'", cmd);
        cmd
    }

    async fn remote_run(
        &self,
        cfg: &RemoteRunCfg,
        nodes: Vec<String>,
        asynchronous: bool,
    ) -> Box<dyn Stream<Item = Result<Bytes, Error>>> {
        trace!("Starting local remote run on {:#?} with {:#?}", nodes, cfg);

        if nodes.is_empty() {
            debug!("No nodes to trigger locally, skipping");
            return Box::new(futures::stream::empty());
        }

        let mut cmd = self.command(cfg, nodes);
        cmd.stdout(Stdio::piped());

        match (asynchronous, cmd.spawn()) {
            (false, Ok(c)) =>
            // send output at once
            {
                Box::new(futures::stream::once(futures::future::ready(
                    c.wait_with_output()
                        .await
                        .map(|o| o.stdout)
                        .map(Bytes::from)
                        .map_err(|e| e.into()),
                )))
            }

            (true, Ok(mut c)) => {
                // stream lines
                let lines = RunParameters::lines_stream(&mut c);
                // FIXME check if it actually runs
                //tokio::spawn(c);
                Box::new(lines.await)
            }
            (_, Err(e)) => {
                error!("Remote run error while running '{:#?}': {}", cmd, e);
                Box::new(futures::stream::once(futures::future::ready(Err(e.into()))))
            }
        }
    }

    /// Stream command output as a stream of lines
    async fn lines_stream(child: &mut Child) -> Box<impl Stream<Item = Result<Bytes, Error>>> {
        let stdout = child
            .stdout
            .take()
            .expect("child did not have a handle to stdout");

        Box::new(
            BufReader::new(stdout)
                .lines()
                .map_err(Error::from)
                .inspect(|line| trace!("output: {:?}", line))
                .map(|r| {
                    r.map(|mut l| {
                        l.push('\n');
                        Bytes::from(l)
                    })
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_handles_command_injection() {
        assert!(Condition::from_str("cl$$y").is_err());
        assert!(Condition::from_str("cl~#~").is_err());
    }

    #[test]
    fn it_defines_parameters() {
        assert_eq!(
            RunParameters::new(None, None, Some(&"".to_string())).unwrap(),
            RunParameters {
                asynchronous: false,
                keep_output: false,
                conditions: vec![],
            }
        );
        assert_eq!(
            RunParameters::new(
                Some(&"true".to_string()),
                Some(&"true".to_string()),
                Some(&"test".to_string())
            )
            .unwrap(),
            RunParameters {
                asynchronous: true,
                keep_output: true,
                conditions: vec![Condition::from_str("test").unwrap()],
            }
        );
    }

    #[test]
    fn it_handles_too_long_conditions() {
        assert!(Condition::from_str("Qr6U6s161z8umvzZTMSPtsZpe3s2sAjwUeCD5pbzvwtT9jg8AsqaW1hbgJhDvOQ34J6GdUS0bEJLKz4zfWHO70rYdq70jrKip5gYwdbVyB7APyK3RRAGHGS7EZ8bUNEXUlHp1QsYOQeqPyPKCCJUYhAzWsD8b1lC4gOkmzATyabEBhaoAb5TLELtBra5dS1YzG1TxgHEthd8z7Qf7PHeltK1X628rfwPqVY2FHkgBGvNMAFTYUdnyabV0j7PHal4f31nNRCqZPdUv6iIlHHQo0oUQlwZ7ATUNYt2cznLYu5v8RhBL0uqOxMD9xHAnRxYRo57BDQxkunNyb7oTjruainGIqbXoDPjcKCQRrf3IrVvAQ6mwAgIdEzJkxBaZUkAGeNQFZEh5b3zJSryfgML2kc87ohLMmsIh5OvNnrPUipSnkpGruJV2uCRX1EYNH6skC9QY1oji6D3SYNeH0lZFIe8goO0Sa1geORlB5UpDwrGeWKgo6k7xBORpPdiVFjR1fAsO7po2CPrR2OwBv6IP0VcU4pPY3eIXgSWSecRE4UXDR2dyaSqSyo4E2l4KAIwy7LieKechiA3yROPrkk0MBC6JfUeOXrCvFBDpQ29Q0TE1J8LK0Xt8DexBZdTUI2ni3Gs1Clli4cvXwfyvTGWFpnTsgS7S7zOyYaIGVqI8UmmszQM8Y4IZBt5nmUsMcrsNBvp4ZqseHoaR0WHTp93c6l83dw3EuuQyFvbqmwQAeDNOrSW2YYAL6Ab5ru5XoRfxCB0LitHWeocyUCo6ukE7YnS8ZmqBIWjLizUD7OnaCSWajdalXINhHDmUQgBehAbPOOiFSlLEyUQeBfZEmWvV5CJ4NN2gBgpDGJywm9mKxr8KcN1TPtp4rGpVYWgDK4N3RjUcQiH7rkSN2zd3vb1MkvtvQsMSX45CpmVng6UQf2LPeRIBNBEaiiNeQAvhfTm86EWNkOwnhHr8QHd7yzLQ6kd4D7Q05oNkRrDDNn5zhS6rvJCujTVFqp5eMa2jbiUa").is_err());
    }
}
