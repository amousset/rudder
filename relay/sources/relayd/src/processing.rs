// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use crate::{error::Error, stats::Event};
use std::path::PathBuf;
use tokio::{
    fs::{remove_file, rename},
    sync::mpsc,
};
use tracing::{debug, error};

pub mod inventory;
pub mod reporting;

pub type ReceivedFile = PathBuf;
pub type RootDirectory = PathBuf;

#[derive(Debug, Copy, Clone)]
enum OutputError {
    Transient,
    Permanent,
}

impl From<Error> for OutputError {
    fn from(err: Error) -> Self {
        match err {
            Error::Database(_) | Error::DatabaseConnection(_) | Error::HttpClient(_) => {
                OutputError::Transient
            }
            _ => OutputError::Permanent,
        }
    }
}

async fn success(
    file: ReceivedFile,
    event: Event,
    stats: &mut mpsc::Sender<Event>,
) -> Result<(), ()> {
    stats
        .send(event)
        .await
        .map_err(|e| error!("send error: {}", e))?;

    remove_file(file.clone())
        .await
        .map(move |_| debug!("deleted: {:#?}", file))
        .map_err(|e| error!("error: {}", e));
    Ok(())
}

async fn failure(
    file: ReceivedFile,
    directory: RootDirectory,
    event: Event,
    stats: &mut mpsc::Sender<Event>,
) -> Result<(), ()> {
    stats
        .send(event)
        .await
        .map_err(|e| error!("send error: {}", e));

    rename(
        file.clone(),
        directory
            .join("failed")
            .join(file.file_name().expect("not a file")),
    )
    .await
    .map_err(|e| error!("error: {}", e))?;

    debug!(
        "moved: {:#?} to {:#?}",
        file,
        directory
            .join("failed")
            .join(file.file_name().expect("not a file"))
    );
    Ok(())
}
