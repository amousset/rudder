// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Normation SAS

use crate::output::Status;
use crate::package_manager::PackageManager;
use crate::{
    campaign,
    db::PackageDatabase,
    hooks::Hooks,
    output::{Report, ScheduleReport},
    package_manager::{LinuxPackageManager, PackageSpec},
    scheduler,
    system::System,
    CampaignType, PackageParameters, RebootType, Schedule, ScheduleParameters,
};
use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use rudder_module_type::Outcome;
use std::fs;
use std::path::PathBuf;

/// How long to keep events in the database
pub(crate) const RETENTION_DAYS: u32 = 60;

#[derive(Clone)]
pub struct SchedulerParameters {
    pub campaign_type: CampaignType,
    pub event_id: String,
    pub campaign_name: String,
    pub schedule: FullSchedule,
    pub reboot_type: RebootType,
    pub package_list: Vec<PackageSpec>,
    pub report_file: Option<PathBuf>,
    pub schedule_file: Option<PathBuf>,
}

impl SchedulerParameters {
    pub fn new(
        package_parameters: PackageParameters,
        node_id: String,
        agent_frequency: Duration,
    ) -> Self {
        Self {
            campaign_type: package_parameters.campaign_type,
            event_id: package_parameters.event_id,
            campaign_name: package_parameters.campaign_name,
            schedule: FullSchedule::new(&package_parameters.schedule, node_id, agent_frequency),
            reboot_type: package_parameters.reboot_type,
            package_list: package_parameters.package_list,
            report_file: package_parameters.report_file,
            schedule_file: package_parameters.schedule_file,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullScheduleParameters {
    pub(crate) start: DateTime<Utc>,
    pub(crate) end: DateTime<Utc>,
    pub(crate) node_id: String,
    pub(crate) agent_frequency: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FullSchedule {
    Scheduled(FullScheduleParameters),
    Immediate,
}

impl FullSchedule {
    pub fn new(schedule: &Schedule, node_id: String, agent_frequency: Duration) -> Self {
        match schedule {
            Schedule::Scheduled(ref s) => {
                FullSchedule::Scheduled(campaign::FullScheduleParameters {
                    start: s.start,
                    end: s.end,
                    node_id,
                    agent_frequency,
                })
            }
            Schedule::Immediate => FullSchedule::Immediate,
        }
    }
}

/// Called at each module run
///
/// The returned outcome is not linked to the success of the update, but to the success of the
/// process. The update itself can fail, but the process can be successful.
pub fn check_update<T>(
    parameters: SchedulerParameters,
    package_manager: &mut Box<dyn LinuxPackageManager>,
    db: &mut PackageDatabase,
    system: &T,
) -> Result<Outcome>
where
    T: System,
{
    let now = Utc::now();
    let schedule_datetime = match parameters.schedule {
        FullSchedule::Immediate => now,
        FullSchedule::Scheduled(ref s) => {
            scheduler::splayed_start(s.start, s.end, s.agent_frequency, s.node_id.as_str())?
        }
    };
    let already_scheduled = db.schedule_event(
        &parameters.event_id,
        &parameters.campaign_name,
        schedule_datetime,
    )?;

    // Update should have started already
    if now >= schedule_datetime {
        let should_do_update = db.start_event(&parameters.event_id, now)?;

        if should_do_update {
            let reboot = do_update(&parameters, db, package_manager, system)?;
            // Reboot after storing the report
            if reboot {
                // Async reboot
                let result = system.reboot();
                return match result.inner {
                    Ok(_) => Ok(Outcome::Success(None)),
                    Err(e) => bail!("Reboot failed: {:?}", e),
                };
            }
        }

        // Update takes time
        let should_do_post_update = db.post_event(&parameters.event_id)?;
        if should_do_post_update {
            do_post_update(&parameters, db)
        } else {
            Ok(Outcome::Success(None))
        }
    } else {
        // Not the time yet, send the schedule if pending.
        if !already_scheduled {
            let report = ScheduleReport::new(schedule_datetime);
            if let Some(ref f) = parameters.schedule_file {
                // Write the report into the destination tmp file
                fs::write(f, serde_json::to_string(&report)?.as_bytes())?;
            }
            Ok(Outcome::Repaired("Send schedule".to_string()))
        } else {
            Ok(Outcome::Success(None))
        }
    }
}

fn do_update<T>(
    p: &SchedulerParameters,
    db: &mut PackageDatabase,
    package_manager: &mut Box<dyn LinuxPackageManager>,
    system: &T,
) -> Result<bool>
where
    T: System,
{
    let (report, reboot) = update(
        package_manager,
        p.reboot_type,
        p.campaign_type,
        &p.package_list,
        system,
    )?;
    let pending_post_action = db.schedule_post_event(&p.event_id, &report)?;
    if !pending_post_action {
        bail!("Several agents seem to have run the update at the same time, aborting");
    }
    Ok(reboot)
}

fn do_post_update(p: &SchedulerParameters, db: &mut PackageDatabase) -> Result<Outcome> {
    let init_report = db.get_report(&p.event_id)?;
    let report = post_update(init_report)?;

    if let Some(ref f) = p.report_file {
        // Write the report into the destination tmp file
        fs::write(f, serde_json::to_string(&report)?.as_bytes())?;
    }

    let now_finished = Utc::now();
    db.completed(&p.event_id, now_finished, &report)?;

    // The repaired status is the trigger to read and send the report.
    Ok(Outcome::Repaired("Update has run".to_string()))
}

/// Shortcut method to send an error report directly
pub fn fail_campaign(reason: &str, report_file: Option<PathBuf>) -> Result<Outcome> {
    let mut report = Report::new();
    report.stderr(reason);
    report.status = Status::Error;
    if let Some(ref f) = report_file {
        // Write the report into the destination tmp file
        fs::write(f, serde_json::to_string(&report)?.as_bytes())?;
    }
    Ok(Outcome::Repaired("Send error".to_string()))
}

/// Actually start the upgrade process immediately
fn update<T>(
    pm: &mut Box<dyn LinuxPackageManager>,
    reboot_type: RebootType,
    campaign_type: CampaignType,
    packages: &[PackageSpec],
    system: &T,
) -> Result<(Report, bool)>
where
    T: System,
{
    let mut report = Report::new();

    let pre_result = Hooks::PreUpgrade.run();
    report.step(pre_result);
    // Pre-run hooks are a blocker
    if report.is_err() {
        report.stderr("Pre-run hooks failed, aborting upgrade");
        return Ok((report, false));
    }

    // We consider failure to probe system state a blocking error
    let before = pm.list_installed();
    let before_list = match before.inner {
        Ok(ref l) => Some(l.clone()),
        _ => None,
    };
    report.step(before);
    if report.is_err() {
        report.stderr("Failed to list installed packages, aborting upgrade");
        return Ok((report, false));
    }
    let before_list = before_list.unwrap();

    // Update package cache
    //
    // Don't fail on cache update failure
    let cache_result = pm.update_cache();
    report.step(cache_result);

    let update_result = match campaign_type {
        CampaignType::SystemUpdate => pm.full_upgrade(),
        CampaignType::SoftwareUpdate => pm.upgrade(packages),
        CampaignType::SecurityUpdate => pm.security_upgrade(),
    };
    report.step(update_result);

    let after = pm.list_installed();
    let after_list = match after.inner {
        Ok(ref l) => Some(l.clone()),
        _ => None,
    };
    report.step(after);
    if report.is_err() {
        report.stderr("Failed to list installed packages, aborting upgrade");
        return Ok((report, false));
    }
    let after_list = after_list.unwrap();

    // Compute package list diff
    report.diff(before_list.diff(after_list));

    // Now take system actions
    let pre_reboot_result = Hooks::PreReboot.run();
    report.step(pre_reboot_result);

    let pending = pm.reboot_pending();
    let is_pending = match pending.inner {
        Ok(p) => p,
        Err(_) => {
            report.stderr("Failed to check if a reboot is pending");
            true
        }
    };
    report.step(pending);

    if reboot_type == RebootType::Always || (reboot_type == RebootType::AsNeeded && is_pending) {
        // Stop there
        return Ok((report, true));
    }

    let services = pm.services_to_restart();
    let services_list = match services.inner {
        Ok(ref p) => p.clone(),
        Err(ref e) => {
            eprintln!("{}", e);
            vec![]
        }
    };
    report.step(services);

    if (reboot_type == RebootType::ServicesOnly || reboot_type == RebootType::AsNeeded)
        && !services_list.is_empty()
    {
        let restart_result = system.restart_services(&services_list);
        // Don't fail on service restart failure
        report.step(restart_result);
    }

    Ok((report, false))
}

/// Can run just after upgrade, or at next run in case of reboot.
fn post_update(mut report: Report) -> Result<Report> {
    let post_result = Hooks::PostUpgrade.run();
    report.step(post_result);
    Ok(report)
}
