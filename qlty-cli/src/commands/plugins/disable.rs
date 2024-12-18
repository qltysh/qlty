use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{bail, Result};
use clap::Args;
use qlty_config::{config::IssueMode, Workspace};
use std::fs;
use toml_edit::{DocumentMut, value};

#[derive(Args, Debug)]
pub struct Disable {
    /// Plugins to disable by name
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

    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        let mut updated = false;

        if let Some(plugin_tables) = self.document["plugin"].as_array_of_tables_mut() {
            for plugin_table in plugin_tables.iter_mut() {
                if plugin_table["name"].as_str() == Some(name) {
                    updated = true;
                    plugin_table["mode"] = value(IssueMode::Disabled.to_str());
                }
            }
        }

        if !updated {
            bail!("Plugin '{}' not found in qlty.toml", name);
        }

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::write(self.workspace.config_path()?, self.document.to_string())?;
        Ok(())
    }
}

impl Disable {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let mut config = ConfigDocument::new(&workspace)?;

        for plugin in &self.plugins {
            config.disable_plugin(plugin)?;
        }

        config.write()?;
        CommandSuccess::ok()
    }
}

#[cfg(test)]
mod tests {
    use qlty_analysis::utils::fs::path_to_native_string;
    use qlty_test_utilities::git::sample_repo;

    use super::*;

    #[test]
    fn test_disable_plugin() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        let workspace = Workspace {
            root: temp_path.clone(),
        };
        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();

        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[[plugin]]
name = "stays"
version = "1.0.0"

[[plugin]]
name = "to_disable"
version = "1.0.0"

[[plugin]]
name = "also_stays"
version = "1.0.0"
            "#,
        )
        .ok();
        let mut config = ConfigDocument::new(&workspace).unwrap();

        config.disable_plugin("to_disable").unwrap();

        let expected = r#"
config_version = "0"

[[plugin]]
name = "stays"
version = "1.0.0"

[[plugin]]
name = "to_disable"
version = "1.0.0"
mode = "disabled"

[[plugin]]
name = "also_stays"
version = "1.0.0"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_disable_plugin_wrong_name() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        let workspace = Workspace {
            root: temp_path.clone(),
        };
        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();

        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[[plugin]]
name = "disableme"
version = "1.0.0"
            "#,
        )
        .ok();
        let mut config = ConfigDocument::new(&workspace).unwrap();

        assert!(config.disable_plugin("typo-command").is_err());
    }
}
