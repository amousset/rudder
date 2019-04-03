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

#[macro_use]
extern crate diesel;

pub mod api;
pub mod configuration;
pub mod data;
pub mod error;
pub mod fake;
pub mod input;
pub mod output;
pub mod stats;

use crate::{
    api::api,
    configuration::{
        CliConfiguration, Configuration, InventoryOutputSelect, LogConfig, ReportingOutputSelect,
    },
    data::nodes::parse_nodeslist,
    error::Error,
    input::{serve_inventories, serve_reports},
    output::database::{pg_pool, PgPool},
    stats::Stats,
};
use clap::crate_version;
use data::nodes::NodesList;
use futures::{
    future::{lazy, Future},
    stream::Stream,
    sync::mpsc,
};
use slog::{o, slog_debug, slog_error, slog_info, slog_trace, Drain, Level, Logger};
use slog_async::Async;
use slog_atomic::{AtomicSwitch, AtomicSwitchCtrl};
use slog_kvfilter::KVFilter;
use slog_scope::{debug, error, info, trace};
use slog_term::{CompactFormat, TermDecorator};
use stats::{stats_job, Event};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    sync::{Arc, RwLock},
};
use tokio_signal::unix::{Signal, SIGHUP, SIGINT, SIGTERM};

use std::collections::HashSet;
use std::iter::FromIterator;

pub struct JobConfig {
    pub cfg: Configuration,
    pub nodes: NodesList,
    pub pool: Option<PgPool>,
}

pub fn stats(rx: mpsc::Receiver<Event>) -> impl Future<Item = (), Error = ()> {
    let mut stats = Stats::default();
    rx.for_each(move |event| {
        stats.event(event);
        Ok(())
    })
}

pub fn load_configuration(file: &Path) -> Result<Configuration, Error> {
    Ok(Configuration::read_configuration(&read_to_string(file)?)?)
}

pub fn load_nodeslist(file: &Path) -> Result<NodesList, Error> {
    info!("Parsing nodes list from {:#?}", file);
    let nodes = parse_nodeslist(&read_to_string(file)?)?;
    trace!("Parsed nodes list:\n{:#?}", nodes);
    Ok(nodes)
}

fn logger_drain() -> slog::Fuse<slog_async::Async> {
    let decorator = TermDecorator::new().stdout().build();
    let drain = CompactFormat::new(decorator).build().fuse();
    Async::new(drain)
        .thread_name("relayd-logger".to_string())
        .chan_size(2048)
        .build()
        .fuse()
}

fn load_loggers(ctrl: &AtomicSwitchCtrl, cfg: &LogConfig) {
    if cfg.general.level == Level::Trace {
        // No filter at all if general level is trace.
        // This needs to be handled separately as KVFilter cannot skip
        // its filters completely.
        ctrl.set(logger_drain());
    } else {
        let mut node_filter = HashMap::new();
        node_filter.insert("node".to_string(), cfg.filter.nodes.clone());
        node_filter.insert("component".to_string(), HashSet::from_iter(cfg.filter.components.clone().iter().map(|s|s.to_string())));
        let drain = KVFilter::new(
            slog::LevelFilter::new(logger_drain(), cfg.filter.level),
            // decrement because the user provides the log level they want to see
            // while this displays logs unconditionally above the given level included.
            match cfg.general.level {
                Level::Critical => Level::Error,
                Level::Error => Level::Warning,
                Level::Warning => Level::Info,
                Level::Info => Level::Debug,
                Level::Debug => Level::Trace,
                Level::Trace => unreachable!("Global trace log level is handled separately"),
            },
        )
        .only_pass_any_on_all_keys(Some(node_filter.clone()));
        ctrl.set(drain.map(slog::Fuse));
        debug!("Log filters are {:#?}", node_filter);

    }
}

pub fn start(cli_cfg: CliConfiguration) -> Result<(), Error> {
    // ---- Load configuration ----

    let cfg = load_configuration(&cli_cfg.configuration_file)?;

    // ---- Setup loggers ----

    let drain = AtomicSwitch::new(logger_drain());
    let ctrl = drain.ctrl();
    let log = Logger::root(drain.fuse(), o!());
    // Make sure to save the guard
    let _guard = slog_scope::set_global_logger(log);
    // Integrate libs using standard log crate
    slog_stdlog::init().expect("Could not initialize standard logging");
    // Load configuration
    load_loggers(&ctrl, &cfg.logging);

    // ---- Start execution ----

    info!("Starting rudder-relayd {}", crate_version!());
    debug!("Parsed cli configuration:\n{:#?}", &cli_cfg);
    info!("Read configuration from {:#?}", &cli_cfg.configuration_file);
    debug!("Parsed configuration:\n{:#?}", &cfg);

    let nodes = load_nodeslist(&cfg.general.nodes_list_file)?;

    // ---- Setup signal handlers ----

    debug!("Setup signal handlers");

    // SIGINT or SIGTERM: graceful shutdown
    let shutdown = Signal::new(SIGINT)
        .flatten_stream()
        .select(Signal::new(SIGTERM).flatten_stream())
        .into_future()
        .map(|_sig| {
            info!("Signal received: shutdown requested");
            ::std::process::exit(1);
        })
        .map_err(|e| error!("signal error {}", e.0));

    // SIGHUP: reload logging configuration + nodes list
    let reload = Signal::new(SIGHUP)
        .flatten_stream()
        .for_each(move |_signal| {
            info!("Signal received: reload requested");
            let cfg = load_configuration(&cli_cfg.configuration_file.clone())
                .expect("Could not reload config");
            debug!("Parsed configuration:\n{:#?}", &cfg);
            load_loggers(&ctrl, &cfg.logging);
            // TODO reload nodeslist
            Ok(())
        })
        .map_err(|e| error!("signal error {}", e));

    // ---- Setup data structures ----

    let stats = Arc::new(RwLock::new(Stats::default()));
    let http_api = api(cfg.general.listen, shutdown, stats.clone());

    let pool = if cfg.processing.reporting.output == ReportingOutputSelect::Database {
        Some(pg_pool(&cfg.output.database)?)
    } else {
        None
    };

    let job_config = Arc::new(JobConfig { cfg, nodes, pool });

    // ---- Start server ----

    tokio::run(lazy(move || {
        let (tx_stats, rx_stats) = mpsc::channel(1_024);

        tokio::spawn(stats_job(stats.clone(), rx_stats));
        tokio::spawn(http_api);

        //tokio::spawn(shutdown);
        tokio::spawn(reload);

        if job_config.cfg.processing.reporting.output != ReportingOutputSelect::Disabled {
            serve_reports(job_config.clone(), tx_stats.clone());
        }
        if job_config.cfg.processing.inventory.output != InventoryOutputSelect::Disabled {
            serve_inventories(job_config, tx_stats);
        }
        Ok(())
    }));

    unreachable!("Server halted unexpectedly");
}
