// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use crate::{
    configuration::main::InventoryOutputSelect,
    input::watch::*,
    output::upstream::send_inventory,
    processing::{failure, success, OutputError, ReceivedFile},
    stats::Event,
    JobConfig,
};
use md5::{Digest, Md5};
use std::{os::unix::ffi::OsStrExt, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, error, info, span, Level};

static INVENTORY_EXTENSIONS: &[&str] = &["gz", "xml", "sign"];

#[derive(Debug, Copy, Clone)]
pub enum InventoryType {
    New,
    Update,
}

pub fn start(job_config: &Arc<JobConfig>, stats: mpsc::Sender<Event>) {
    let span = span!(Level::TRACE, "inventory");
    let _enter = span.enter();

    let (sender, receiver) = mpsc::channel(1_024);

    let incoming_path = job_config
        .cfg
        .processing
        .inventory
        .directory
        .join("incoming");
    tokio::spawn(serve(
        job_config.clone(),
        receiver,
        InventoryType::New,
        stats.clone(),
    ));
    tokio::spawn(cleanup(
        incoming_path.clone(),
        job_config.cfg.processing.inventory.cleanup,
    ));
    watch(&incoming_path, &job_config, sender);

    let updates_path = job_config
        .cfg
        .processing
        .inventory
        .directory
        .join("accepted-nodes-updates");
    let (sender, receiver) = mpsc::channel(1_024);
    tokio::spawn(serve(
        job_config.clone(),
        receiver,
        InventoryType::Update,
        stats.clone(),
    ));
    tokio::spawn(cleanup(
        updates_path.clone(),
        job_config.cfg.processing.inventory.cleanup,
    ));
    watch(&updates_path, &job_config, sender);
}

async fn serve(
    job_config: Arc<JobConfig>,
    mut rx: mpsc::Receiver<ReceivedFile>,
    inventory_type: InventoryType,
    stats: mpsc::Sender<Event>,
) -> Result<(), ()> {
    while let Some(file) = rx.recv().await {
        // allows skipping temporary .dav files
        if !file
            .extension()
            .map(|f| INVENTORY_EXTENSIONS.contains(&f.to_string_lossy().as_ref()))
            .unwrap_or(false)
        {
            debug!(
                "skipping {:#?} as it does not have a known inventory extension",
                file
            );
            return Ok(());
        }

        let queue_id = format!(
            "{:X}",
            Md5::digest(
                file.file_name()
                    .unwrap_or_else(|| file.as_os_str())
                    .as_bytes()
            )
        );
        let span = span!(
            Level::INFO,
            "inventory",
            queue_id = %queue_id,
        );
        let _enter = span.enter();

        stats
            .clone()
            .send(Event::InventoryReceived)
            .await
            .map_err(|e| error!("receive error: {}", e))
            .map(|_| ())?;

        debug!("received: {:?}", file);

        match job_config.cfg.processing.inventory.output {
            InventoryOutputSelect::Upstream => {
                output_inventory_upstream(file, inventory_type, job_config.clone(), stats.clone())
                    .await?
            }
            // The job should not be started in this case
            InventoryOutputSelect::Disabled => unreachable!("Inventory server should be disabled"),
        };
    }
    Ok(())
}

async fn output_inventory_upstream(
    path: ReceivedFile,
    inventory_type: InventoryType,
    job_config: Arc<JobConfig>,
    stats: mpsc::Sender<Event>,
) -> Result<(), ()> {
    let job_config_clone = job_config.clone();
    let path_clone2 = path.clone();
    let stats_clone = stats.clone();

    let result = send_inventory(job_config, path.clone(), inventory_type).await;

    match result {
        Ok(_) => success(path.clone(), Event::InventorySent, stats_clone).await,
        Err(e) => {
            error!("output error: {}", e);
            match OutputError::from(e) {
                OutputError::Permanent => {
                    failure(
                        path_clone2.clone(),
                        job_config_clone.cfg.processing.inventory.directory.clone(),
                        Event::InventoryRefused,
                        stats,
                    )
                    .await
                }
                OutputError::Transient => {
                    info!("transient error, skipping");
                    Ok(())
                }
            }
        }
    }
}
