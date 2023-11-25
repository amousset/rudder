// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Normation SAS

use clap::{Parser, Subcommand, ValueEnum};

use crate::CONFIG_PATH;

#[derive(ValueEnum, Copy, Clone, Debug, Default)]
pub enum Format {
    Json,
    #[default]
    Human,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Configuration file path
    #[arg(short, long, default_value_t = CONFIG_PATH.into())]
    pub config: String,

    /// Enable verbose logs
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Install {
        #[clap(long, short, help = "Force installation of given plugin")]
        force: bool,

        #[clap()]
        package: Vec<String>,
    },
    /// Show the plugin list
    List {
        #[clap(long, short, help = "Show all available plugins")]
        all: bool,

        #[clap(long, short, help = "Show enabled plugins")]
        enabled: bool,

        #[clap(long, short, help = "Output format")]
        format: Format,
    },
    /// Show detailed information about a plugin
    Show {
        #[clap()]
        package: String,
    },
    Uninstall {
        #[clap()]
        package: Vec<String>,
    },
    /// Update package index and licenses
    Update {},
    Enable {
        #[clap()]
        package: Option<Vec<String>>,

        #[clap(long, short, help = "Enable all installed plugins")]
        all: bool,

        #[clap(long, short, help = "Snapshot the list of enabled plugins")]
        save: bool,

        #[clap(
            long,
            short,
            help = "Restore the list of enabled plugins from latest snapshot"
        )]
        restore: bool,
    },
    Disable {
        #[clap()]
        package: Option<Vec<String>>,

        #[clap(long, short, help = "Enable all installed plugins")]
        all: bool,
    },
    CheckConnection {},
}
