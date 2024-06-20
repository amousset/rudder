// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2024 Normation SAS

use crate::PackageSpec;
use anyhow::{bail, Result};
use std::io::BufRead;
use std::process::Command;

pub const NEED_RESTART_PATH: &str = "/usr/bin/needs-restarting";

pub struct Yum {}

impl Yum {
    pub fn system_update(&self) -> Result<()> {
        Command::new("yum").arg("-y").arg("update").output()?;
        Ok(())
    }

    pub fn packages_update(&self) -> Result<()> {
        Ok(())
    }

    pub fn package_spec_as_argument(p: PackageSpec) -> String {
        let mut res = p.name;
        if let Some(v) = p.version {
            res.push_str("-");
            res.push_str(&v);
        }
        if let Some(a) = p.architecture {
            res.push_str(".");
            res.push_str(&a);
        }
        res
    }

    pub fn services_to_restart(&self) -> Result<Vec<String>> {
        let o = Command::new(NEED_RESTART_PATH).arg("--services").output()?;
        if !o.status.success() {
            bail!("TODO");
        }
        // One service name per line
        o.stdout
            .lines()
            .map(|s| {
                s.map(|service| service.trim().to_string())
                    .map_err(|e| e.into())
            })
            .collect()
    }

    pub fn reboot_required(&self) -> Result<bool> {
        // only report whether a reboot is required (exit code 1) or not (exit code 0)
        Ok(!Command::new(NEED_RESTART_PATH)
            .arg("--reboothint")
            .status()?
            .success())
    }
}
