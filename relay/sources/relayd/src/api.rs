// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

//mod remote_run;
mod shared_files;
mod shared_folder;
mod system;

use crate::{
    api::{
        //remote_run::{RemoteRun, RemoteRunTarget},
        shared_files::{SharedFilesHeadParams, SharedFilesPutParams},
        shared_folder::SharedFolderParams,
    },
    error::Error,
    stats::Stats,
    JobConfig,
};
use bytes::Bytes;
use futures::Future;
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::Display,
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tracing::{error, info, span, Level};
use warp::{
    body,
    filters::{method::*, path::Peek},
    fs,
    http::StatusCode,
    path, query, reject,
    reject::Reject,
    reply, Filter, Rejection, Reply,
};

impl Reject for Error {}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ApiResult {
    Success,
    Error,
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ApiResponse<T: Serialize> {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    result: ApiResult,
    action: &'static str,
    #[serde(rename = "errorDetails")]
    #[serde(skip_serializing_if = "Option::is_none")]
    error_details: Option<String>,
    #[serde(skip)]
    status_code: StatusCode,
}

impl<T: Serialize> ApiResponse<T> {
    fn new<E: Display>(
        action: &'static str,
        data: Result<Option<T>, E>,
        status_code: Option<StatusCode>,
    ) -> Self {
        match data {
            Ok(Some(d)) => ApiResponse {
                data: Some(d),
                result: ApiResult::Success,
                action,
                error_details: None,
                status_code: status_code.unwrap_or(StatusCode::OK),
            },
            Ok(None) => ApiResponse {
                data: None,
                result: ApiResult::Success,
                action,
                error_details: None,
                status_code: status_code.unwrap_or(StatusCode::OK),
            },
            Err(e) => ApiResponse {
                data: None,
                result: ApiResult::Error,
                action,
                error_details: Some(e.to_string()),
                status_code: status_code.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            },
        }
    }

    fn reply(&self) -> impl Reply {
        reply::with_status(reply::json(self), self.status_code)
    }
}

pub async fn run(job_config: Arc<JobConfig>, stats: Arc<RwLock<Stats>>) -> Result<(), ()> {
    let span = span!(Level::TRACE, "api");
    let _enter = span.enter();

    // Old compatible endpoints
    /*
        let job_config2 = job_config.clone();
        let node_id =
            post()
                .and(path("nodes"))
                .and(path::param::<String>().and(body::form()).and_then(
                    move |node_id, simple_map: HashMap<String, String>| match RemoteRun::new(
                        RemoteRunTarget::Nodes(vec![node_id]),
                        &simple_map,
                    ) {
                        Ok(handle) => handle.run(job_config2.clone()).await,
                        Err(e) => Err(reject::custom(e.to_string())),
                    },
                ));

        let job_config3 = job_config.clone();
        let nodes =
            post()
                .and(path("nodes"))
                .and(path::end().and(body::form()).and_then(
                    move |simple_map: HashMap<String, String>| {
                        match simple_map.get("nodes") {
                            Some(nodes) => match RemoteRun::new(
                                RemoteRunTarget::Nodes(
                                    nodes
                                        .split(',')
                                        .map(|s| s.to_string())
                                        .collect::<Vec<String>>(),
                                ),
                                &simple_map,
                            ) {
                                Ok(handle) => handle.run(job_config3.clone()),
                                Err(e) => Err(reject::custom(e.to_string())),
                            },
                            None => Err(reject::custom(Error::MissingTargetNodes)),
                        }
                    },
                ));

        let job_config4 = job_config.clone();
        let all = post().and(path("all")).and(body::form()).and_then(
            move |simple_map: HashMap<String, String>| match RemoteRun::new(
                RemoteRunTarget::All,
                &simple_map,
            ) {
                Ok(handle) => handle.run(job_config4.clone()),
                Err(e) => Err(reject::custom(e.to_string())),
            },
        );

    */

    let listen = &job_config.cfg.general.listen;

    info!("Starting API on {}", listen);
    // TODO graceful shutdown
    let mut addresses = listen.to_socket_addrs().map_err(|e| {
        // Log resolution error
        error!("{}", e);
    })?;

    // Use first resolved address for now
    let socket = addresses.next().unwrap();

    let routes = routes_1(job_config, stats)
        .recover(customize_error)
        .with(warp::log("relayd::api"));

    warp::serve(routes).bind(socket).await;
    Ok(())
}

pub fn routes_1(
    job_config: Arc<JobConfig>,
    stats: Arc<RwLock<Stats>>,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let info = get()
        .and(path!("rudder" / "relay-api" / "1" / "system" / "info"))
        .and_then(system::handlers::info);

    let job_config_reload = job_config.clone();
    let reload = post()
        .and(path!("rudder" / "relay-api" / "1" / "system" / "reload"))
        .map(move || job_config_reload.clone())
        .and_then(|j| system::handlers::reload(j));

    let job_config_status = job_config.clone();
    let status = get()
        .and(path!("rudder" / "relay-api" / "1" / "system" / "status"))
        .map(move || job_config_status.clone())
        .and_then(|j| system::handlers::status(j));

    // WARNING: Not stable, will be replaced soon
    // Kept for testing mainly
    let stats = get()
        .and(path!("rudder" / "relay-api" / "1" / "system" / "stats"))
        .map(move || stats.clone())
        .and_then(|s| system::handlers::stats(s));

    let job_config_shared_folder_head = job_config.clone();
    let shared_folder_head = head()
        .and(path!("rudder" / "relay-api" / "1" / "shared-folder"))
        .map(move || job_config_shared_folder_head.clone())
        .and(path::peek())
        .and(query::<SharedFolderParams>())
        .and_then(|j, p, q| shared_folder::handlers::head(p, q, j));

    let job_config_shared_folder_get = job_config.clone();
    let shared_folder_get = head()
        .and(path!("rudder" / "relay-api" / "1" / "shared-folder"))
        .and(fs::dir(
            job_config_shared_folder_get.cfg.shared_folder.path.clone(),
        ));

    let job_config_shared_files_head = job_config.clone();
    let shared_files_head = head()
        .and(path!("rudder" / "relay-api" / "1" / "shared-files"))
        .map(move || job_config_shared_files_head.clone())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(query::<SharedFilesHeadParams>())
        .and_then(move |j, target_id, source_id, file_id, params| {
            shared_files::handlers::head(target_id, source_id, file_id, params, j)
        });

    let job_config_shared_files_put = job_config.clone();
    let shared_files_put = put()
        .and(path!("rudder" / "relay-api" / "1" / "shared-files"))
        .map(move || job_config_shared_files_put.clone())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(path::param::<String>())
        .and(query::<SharedFilesPutParams>())
        .and(body::bytes())
        .and_then(move |j, target_id, source_id, file_id, params, buf| {
            shared_files::handlers::put(target_id, source_id, file_id, params, buf, j)
        });

    //let remote_run = path("remote-run").and(nodes.or(all).or(node_id));

    info.or(reload)
        .or(status)
        .or(stats)
        .or(shared_folder_head)
        .or(shared_folder_get)
        .or(shared_files_head)
        .or(shared_files_put)
}

async fn customize_error(reject: Rejection) -> Result<impl Reply, Rejection> {
    // See https://github.com/seanmonstar/warp/issues/77
    // We generally prefer 404 to 405 when they are conflicting.
    // Maybe be improved in the future
    if reject.is_not_found() || reject.find::<reject::MethodNotAllowed>().is_some() {
        Ok(reply::with_status("", StatusCode::NOT_FOUND))
    } else {
        Err(reject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_api_response() {
        assert_eq!(
            serde_json::to_string(&ApiResponse::<()>::new::<Error>(
                "actionName1",
                Ok(None),
                None
            ))
            .unwrap(),
            "{\"result\":\"success\",\"action\":\"actionName1\"}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&ApiResponse::new::<Error>(
                "actionName2",
                Ok(Some("thing".to_string())),
                None
            ))
            .unwrap(),
            "{\"data\":\"thing\",\"result\":\"success\",\"action\":\"actionName2\"}".to_string()
        );
        assert_eq!(
            serde_json::to_string(&ApiResponse::<()>::new::<Error>(
                "actionName3",
                Err(Error::InconsistentRunlog),
                None
            ))
            .unwrap(),
            "{\"result\":\"error\",\"action\":\"actionName3\",\"errorDetails\":\"inconsistent run log\"}".to_string()
        );
    }
}
