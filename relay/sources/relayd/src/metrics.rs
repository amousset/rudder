// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use crate::JobConfig;
use lazy_static::lazy_static;
use prometheus::{IntCounter, IntGauge, Registry};
use std::{sync::Arc, time::Duration};
use tokio::time::interval;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    // reports
    pub static ref REPORTS_RECEIVED: IntCounter =
        IntCounter::new("reports_received", "Reports Received").expect("metric can be created");
    pub static ref REPORTS_FORWARDED: IntCounter =
        IntCounter::new("reports_forwarded", "Reports Sent").expect("metric can be created");
    pub static ref REPORTS_INSERTED: IntCounter =
        IntCounter::new("reports_inserted", "Reports Inserted").expect("metric can be created");
    pub static ref REPORTS_REFUSED: IntCounter =
        IntCounter::new("reports_refused", "Reports Refused").expect("metric can be created");
    // inventories
    pub static ref INVENTORIES_RECEIVED: IntCounter =
        IntCounter::new("inventories_received", "Inventories Received")
            .expect("metric can be created");
    pub static ref INVENTORIES_FORWARDED: IntCounter =
        IntCounter::new("inventories_sent", "Inventories Sent").expect("metric can be created");
    pub static ref INVENTORIES_REFUSED: IntCounter =
        IntCounter::new("inventories_refused", "Inventories Refused")
            .expect("metric can be created");
    // nodes
    pub static ref MANAGED_NODES: IntGauge =
        IntGauge::new("managed_nodes", "Managed Nodes").expect("metric can be created");
    pub static ref SUB_NODES: IntGauge =
        IntGauge::new("sub_nodes", "Nodes behind this policy server").expect("metric can be created");
}

/// Registers custom metrics
pub fn register() {
    REGISTRY
        .register(Box::new(REPORTS_RECEIVED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(REPORTS_FORWARDED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(REPORTS_INSERTED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(REPORTS_REFUSED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(INVENTORIES_RECEIVED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(INVENTORIES_FORWARDED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(INVENTORIES_REFUSED.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(MANAGED_NODES.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(SUB_NODES.clone()))
        .expect("collector can be registered");
}

pub async fn data_collector(job_config: Arc<JobConfig>) {
    let mut collect_interval = interval(Duration::from_secs(1));
    loop {
        collect_interval.tick().await;

        let node_counts = job_config.nodes.read().unwrap().counts();
        MANAGED_NODES.set(node_counts.managed_nodes as i64);
        SUB_NODES.set(node_counts.sub_nodes as i64);
    }
}
