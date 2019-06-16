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

use crate::error::Error;
use crate::{configuration::LogComponent, stats::Stats, status::Status, JobConfig};
use futures::Future;
use slog::slog_info;
use slog_scope::info;
use std::collections::HashMap;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use warp::Filter;

use crate::remote_run::{nodes_handle, nodes_handle2, AgentParameters, RemoteRun, RemoteRunTarget};

pub fn api(
    listen: SocketAddr,
    shutdown: impl Future<Item = ()> + Send + 'static,
    job_config: Arc<JobConfig>,
    stats: Arc<RwLock<Stats>>,
) -> impl Future<Item = (), Error = ()> {
    let job_config2 = job_config.clone();
    let job_config3 = job_config.clone();
    let stats_simple = warp::path("stats").map(move || {
        info!("/stats queried"; "component" => LogComponent::Statistics);
        warp::reply::json(&(*stats.clone().read().unwrap()))
    });

    let status = warp::path("status").map(move || {
        info!("/status queried"; "component" => LogComponent::Statistics);
        warp::reply::json(&Status::poll(job_config.clone()))
    });

    let nodes = warp::path("nodes").and(warp::path::end().and(warp::body::form()).and_then(
        move |simple_map: HashMap<String, String>| match nodes_handle(
            &simple_map,
            "nodes".to_string(),
        ) {
            Ok(handle) => nodes_handle2(&handle, job_config2.clone()),
            Err(e) => Err(warp::reject::custom(Error::InvalidCondition(e.to_string()))),
        },
    ));

    let node_id = warp::path("nodes").and(warp::path::param::<String>().map(|node| {
        info!("remote run triggered on node {}", node; "component" => LogComponent::Statistics);
        warp::reply()
    }));

    let all = warp::path("all").and(warp::body::form()).and_then(
        move |simple_map: HashMap<String, String>| match nodes_handle(
            &simple_map,
            "nodes".to_string(),
        ) {
            Ok(handle) => nodes_handle2(&handle, job_config3.clone()),
            Err(e) => Err(warp::reject::custom(Error::InvalidCondition(e.to_string()))),
        },
    );

    let rudder = warp::path("rudder");
    let relay_api = warp::path("relay-api");
    let remote_run = warp::path("remote-run");

    let routes = warp::get2().and(status.or(stats_simple)).or(warp::post2()
        .and(rudder)
        .and(relay_api)
        .and(remote_run)
        .and(nodes.or(all).or(node_id)));

    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(listen, shutdown);
    info!("Started stats API on {}", addr; "component" => LogComponent::Statistics);
    server
}
