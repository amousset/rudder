// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Normation SAS

use chrono::{Duration, Utc};
use pretty_assertions::assert_eq;
use rudder_module_system_updates::campaign::{check_update, FullSchedule, SchedulerParameters};
use rudder_module_system_updates::db::PackageDatabase;
use rudder_module_system_updates::output::{ResultOutput, ScheduleReport, Status};
use rudder_module_system_updates::package_manager::{
    LinuxPackageManager, PackageList, PackageManager, PackageSpec,
};
use rudder_module_system_updates::state::UpdateStatus;
use rudder_module_system_updates::system::System;
use rudder_module_system_updates::{
    CampaignType, PackageParameters, RebootType, Schedule, ScheduleParameters,
};
use rudder_module_type::Outcome;
use std::collections::HashMap;
use std::fs::read_to_string;
use tempfile::tempdir;
use uuid::Uuid;

struct MockSystem {}

impl System for MockSystem {
    fn reboot(&self) -> ResultOutput<()> {
        ResultOutput::new(Ok(()))
    }

    fn restart_services(&self, _services: &[String]) -> ResultOutput<()> {
        ResultOutput::new(Ok(()))
    }
}

#[derive(Default, Clone, Copy)]
struct MockPackageManager {}

impl LinuxPackageManager for MockPackageManager {
    fn list_installed(&mut self) -> ResultOutput<PackageList> {
        ResultOutput::new(Ok(PackageList::new(HashMap::new())))
    }

    fn full_upgrade(&mut self) -> ResultOutput<()> {
        ResultOutput::new(Ok(()))
    }

    fn security_upgrade(&mut self) -> ResultOutput<()> {
        ResultOutput::new(Ok(()))
    }

    fn upgrade(&mut self, _packages: &[PackageSpec]) -> ResultOutput<()> {
        ResultOutput::new(Ok(()))
    }

    fn reboot_pending(&self) -> ResultOutput<bool> {
        ResultOutput::new(Ok(true))
    }

    fn services_to_restart(&self) -> ResultOutput<Vec<String>> {
        ResultOutput::new(Ok(vec![]))
    }
}

fn default_scheduler() -> SchedulerParameters {
    SchedulerParameters {
        campaign_type: CampaignType::SystemUpdate,
        event_id: "".to_string(),
        campaign_name: "".to_string(),
        schedule: FullSchedule::Immediate,
        reboot_type: RebootType::Disabled,
        package_list: vec![],
        report_file: None,
        schedule_file: None,
    }
}

fn mock_package_manager() -> MockPackageManager {
    MockPackageManager {}
}

fn in_memory_package_db() -> PackageDatabase {
    PackageDatabase::new(None).unwrap()
}

#[test]
pub fn event_scheduled_in_the_future_reports_its_schedule_and_does_nothing() {
    let pm = mock_package_manager();

    let start = Utc::now() + Duration::days(10);
    let end = start + Duration::days(1);
    let schedule = Schedule::Scheduled(ScheduleParameters { start, end });
    let full_schedule = FullSchedule::new(&schedule, "id".to_string(), Duration::minutes(5));

    let state_dir = tempdir().unwrap();
    let report_file = state_dir.path().join("report.json");
    let schedule_file = state_dir.path().join("schedule.json");

    let package_parameters = SchedulerParameters {
        schedule: full_schedule,
        report_file: Some(report_file.clone()),
        schedule_file: Some(schedule_file.clone()),
        ..default_scheduler()
    };

    let mut db = in_memory_package_db();
    let mut pm: Box<dyn LinuxPackageManager> = Box::new(pm);

    let outcome =
        check_update(package_parameters.clone(), &mut pm, &mut db, &MockSystem {}).unwrap();
    assert_eq!(
        db.get_status(&package_parameters.event_id).unwrap(),
        UpdateStatus::ScheduledUpdate
    );
    assert_eq!(outcome, Outcome::Repaired("Send schedule".to_string()));
    assert!(!report_file.exists());
    assert!(schedule_file.exists());
    let schedule = read_to_string(schedule_file).unwrap();
    let parsed: ScheduleReport = serde_json::from_str(&schedule).unwrap();
    assert_eq!(parsed.status, Status::Success);
    assert!(start <= parsed.date);
    assert!(parsed.date <= end);
}

#[test]
pub fn event_scheduled_in_the_future_called_a_second_time_does_nothing() {
    let pm = mock_package_manager();

    let start = Utc::now() + Duration::days(10);
    let end = start + Duration::days(1);
    let schedule = Schedule::Scheduled(ScheduleParameters { start, end });
    let full_schedule = FullSchedule::new(&schedule, "id".to_string(), Duration::minutes(5));

    let state_dir = tempdir().unwrap();

    let package_parameters = SchedulerParameters {
        schedule: full_schedule,
        ..default_scheduler()
    };

    let mut db = in_memory_package_db();
    let mut pm: Box<dyn LinuxPackageManager> = Box::new(pm);

    let report_file_second = state_dir.path().join("report.second.json");
    let schedule_file_second = state_dir.path().join("schedule.second.json");
    let package_parameter_second = SchedulerParameters {
        report_file: Some(report_file_second.clone()),
        schedule_file: Some(schedule_file_second.clone()),
        ..package_parameters.clone()
    };

    check_update(package_parameters.clone(), &mut pm, &mut db, &MockSystem {}).unwrap();
    let outcome = check_update(package_parameter_second, &mut pm, &mut db, &MockSystem {}).unwrap();

    assert_eq!(
        db.get_status(&package_parameters.event_id).unwrap(),
        UpdateStatus::ScheduledUpdate
    );
    assert_eq!(outcome, Outcome::Success(None));
    assert!(!report_file_second.exists());
    assert!(!schedule_file_second.exists());
}

#[test]
pub fn event_runs_and_stops_before_reboot() {
    let pm = mock_package_manager();

    let package_parameters = SchedulerParameters {
        reboot_type: RebootType::Always,
        ..default_scheduler()
    };

    let mut db = in_memory_package_db();
    let mut pm: Box<dyn LinuxPackageManager> = Box::new(pm);

    let outcome =
        check_update(package_parameters.clone(), &mut pm, &mut db, &MockSystem {}).unwrap();

    assert_eq!(
        db.get_status(&package_parameters.event_id).unwrap(),
        UpdateStatus::PendingPostActions
    );
    assert_eq!(outcome, Outcome::Success(None));
}

#[test]
pub fn event_runs_until_completion() {
    let pm = mock_package_manager();

    let package_parameters = SchedulerParameters {
        ..default_scheduler()
    };

    let mut db = in_memory_package_db();
    let mut pm: Box<dyn LinuxPackageManager> = Box::new(pm);

    let outcome =
        check_update(package_parameters.clone(), &mut pm, &mut db, &MockSystem {}).unwrap();

    assert_eq!(
        db.get_status(&package_parameters.event_id).unwrap(),
        UpdateStatus::Completed
    );
    assert_eq!(outcome, Outcome::Repaired("Update has run".to_string()));
}
