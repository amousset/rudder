// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

use crate::error::RudderError;
use anyhow::Error;
use serde::Deserialize;
use std::{fmt, fs::read_to_string, path::Path, str::FromStr};
use tracing::debug;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct LogConfig {
    pub general: LoggerConfig,
}

impl FromStr for LogConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(toml::from_str(s)?)
    }
}

impl LogConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let res = read_to_string(path.as_ref().join("logging.conf"))?.parse::<Self>();
        if let Ok(ref cfg) = res {
            debug!("Parsed logging configuration:\n{:#?}", &cfg);
        }
        res
    }
}

impl fmt::Display for LogConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.general)
    }
}

#[derive(Copy, Debug, Eq, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Off => "off",
                LogLevel::Error => "error",
                LogLevel::Warn => "warn",
                LogLevel::Info => "info",
                LogLevel::Debug => "debug",
                LogLevel::Trace => "trace",
            }
        )
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct LoggerConfig {
    #[serde(with = "LogLevel")]
    pub level: LogLevel,
    pub filter: String,
}

impl fmt::Display for LoggerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.filter.is_empty() {
            write!(f, "{}", self.level)
        } else {
            write!(f, "{},{}", self.level, self.filter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_generates_log_configuration() {
        let log_reference = LogConfig {
            general: LoggerConfig {
                level: LogLevel::Info,
                filter: "".to_string(),
            },
        };
        assert_eq!(&log_reference.to_string(), "info");

        let log_reference = LogConfig {
            general: LoggerConfig {
                level: LogLevel::Info,
                filter: "[database{node=root}]=trace".to_string(),
            },
        };
        assert_eq!(
            &log_reference.to_string(),
            "info,[database{node=root}]=trace"
        );
    }

    #[test]
    fn it_parses_logging_configuration() {
        let log_config = LogConfig::new("tests/files/config/");
        let log_reference = LogConfig {
            general: LoggerConfig {
                level: LogLevel::Off,
                filter: "".to_string(),
            },
        };
        assert_eq!(log_config.unwrap(), log_reference);
    }
}
