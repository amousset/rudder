// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

mod remote_run;
mod shared_files;
mod shared_folder;
mod system;

use crate::{error::Error, stats::Stats, JobConfig};
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

    let listen = &job_config.cfg.general.listen;

    info!("Starting API on {}", listen);

    let routes_1 = system::routes_1(job_config.clone(), stats.clone())
        .or(shared_folder::routes_1(job_config.clone()))
        .or(shared_files::routes_1(job_config.clone()))
        .or(remote_run::routes_1(job_config.clone()));

    let routes = routes_1
        .recover(customize_error)
        .with(warp::log("relayd::api"));

    let mut addresses = listen.to_socket_addrs().map_err(|e| {
        // Log resolution error
        error!("{}", e);
    })?;
    // Use first resolved address for now
    let socket = addresses.next().unwrap();
    warp::serve(routes).bind(socket).await;
    Ok(())
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
