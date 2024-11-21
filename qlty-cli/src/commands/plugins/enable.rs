use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{Context, Result};
use clap::Args;
use qlty_config::Workspace;
use std::fs;
use toml_edit::{array, table, value, DocumentMut};

#[derive(Args, Debug)]
pub struct Enable {
    /// Plugins to enable specified as name=version
    pub plugins: Vec<String>,
}

struct ConfigDocument {
    workspace: Workspace,
    document: DocumentMut,
}

impl ConfigDocument {
    pub fn new(workspace: &Workspace) -> Result<Self> {
        let contents = fs::read_to_string(workspace.config_path()?)?;
        let document = contents.parse::<DocumentMut>().expect("Invalid config doc");

        Ok(Self {
            workspace: workspace.clone(),
            document,
        })
    }

    pub fn enable_plugin(&mut self, name: &str, version: &str) -> Result<()> {
        let workspace = Workspace::new()?;
        workspace.fetch_sources()?;

        let config = workspace.config()?;

        config
            .plugins
            .definitions
            .get(name)
            .cloned()
            .with_context(|| {
                format!(
                    "Unknown plugin: The {} plugin was not found in any source.",
                    name
                )
            })?;

        if self.document.get("plugin").is_none() {
            self.document["plugin"] = array();
        }

        let mut plugin_table = table();
        plugin_table["name"] = value(name);

        if version != "latest" {
            plugin_table["version"] = value(version);
        }

        self.document["plugin"]
            .as_array_of_tables_mut()
            .unwrap()
            .push(plugin_table.as_table().unwrap().clone());

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::write(self.workspace.config_path()?, self.document.to_string())?;
        Ok(())
    }
}

impl Enable {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let mut config = ConfigDocument::new(&workspace)?;

        for plugin in &self.plugins {
            let parts: Vec<&str> = plugin.split('=').collect();

            match parts.len() {
                1 => {
                    config.enable_plugin(parts[0], "latest")?;
                }
                2 => {
                    config.enable_plugin(parts[0], parts[1])?;
                }
                _ => {
                    return CommandError::err("Invalid plugin format");
                }
            }
        }

        config.write()?;
        CommandSuccess::ok()
    }
}
