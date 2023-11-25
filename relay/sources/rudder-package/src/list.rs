// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2023 Normation SAS

use std::collections::HashSet;

use crate::{
    cli::Format, database::Database, plugin::PluginType, repo_index::RepoIndex, webapp::Webapp,
};
use anyhow::Result;
use serde::Serialize;

pub struct ListOutput {
    inner: Vec<ListEntry>,
}

/// Content we want to display about each plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ListEntry {
    /// Plugin name ("short")
    name: String,
    /// Full plugin version
    version: String,
    /// None means no higher version is available
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_version: Option<String>,
    enabled: bool,
    #[serde(rename = "type")]
    plugin_type: PluginType,
}

impl ListOutput {
    fn human_table(self) -> Result<()> {
        use cli_table::{format::Justify, print_stdout, Cell, Style, Table};

        let table = self
            .inner
            .into_iter()
            .map(|e| {
                vec![
                    e.name.cell(),
                    e.version.cell().justify(Justify::Right),
                    e.latest_version.unwrap_or("".to_string()).cell(),
                    e.plugin_type.cell(),
                    match e.plugin_type {
                        PluginType::Web => {
                            if e.enabled {
                                "installed (enabled)"
                            } else {
                                "installed (disabled)"
                            }
                        }
                        PluginType::Standalone => "installed",
                    }
                    .cell(),
                ]
            })
            .table()
            .title(vec![
                "Name".cell().bold(true),
                "Installed".cell().bold(true),
                "Latest".cell().bold(true),
                "Type".cell().bold(true),
                "Status".cell().bold(true),
            ]);
        print_stdout(table)?;
        Ok(())
    }

    fn json(&self) -> Result<()> {
        let out = serde_json::to_string(&self.inner)?;
        println!("{}", out);
        Ok(())
    }

    pub fn new(
        show_all: bool,
        show_only_enabled: bool,
        db: &Database,
        index: &RepoIndex,
        webapp: &Webapp,
    ) -> Result<Self> {
        let mut plugins: Vec<ListEntry> = vec![];
        // Principles:
        //
        // * By default, display installed plugins.
        // * If an index is available and "all", also add available plugins.
        // * If "enabled", display installed package minus disabled ones.
        let jars = webapp.jars()?;
        let enabled_plugins: HashSet<String> = jars
            .into_iter()
            .flat_map(|j| db.plugin_provides_jar(j))
            .map(|p| p.metadata.name.clone())
            .collect();

        for p in db.plugins.values() {
            let name = p
                .metadata
                .name
                .strip_prefix("rudder-plugin-")
                .unwrap()
                .to_string();
            let enabled = p
                .metadata
                .jar_files
                .as_ref()
                .map(|_| enabled_plugins.contains(&p.metadata.name))
                .unwrap_or(true);
            let e = ListEntry {
                name,
                version: p.metadata.version.to_string(),
                latest_version: None,
                plugin_type: p.metadata.plugin_type(),
                enabled,
            };
            if !show_only_enabled || enabled {
                // Standalone plugins are always considered enabled
                // (even if some might not be: disabled systemd service for notify, etc.)
                plugins.push(e);
            }
        }

        if show_all {
            for p in index.inner().iter() {
                //dbg!(&p.metadata.name);
                //dbg!(&p.metadata.version);
            }
        }

        // Sort by name alphabetical order
        plugins.sort_by_key(|e| e.name.clone());

        Ok(Self { inner: plugins })
    }

    pub fn display(self, format: Format) -> Result<()> {
        match format {
            Format::Json => self.json()?,
            Format::Human => self.human_table()?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{cli::Format, database::Database, repo_index::RepoIndex, webapp::Webapp};

    #[test]
    fn it_lists_plugins() {
        let w = Webapp::new(PathBuf::from("tests/webapp_xml/example.xml"));
        let r = RepoIndex::from_path("./tests/repo_index.json").unwrap();
        let d = Database::read("./tests/database/plugin_database_update_sample.json").unwrap();

        let out = super::ListOutput::new(true, false, &d, &r, &w).unwrap();
        out.display(Format::Human).unwrap();
    }
}
