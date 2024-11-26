use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::{bail, Context, Result};
use clap::Args;
use qlty_config::Workspace;
use std::fs;
use toml_edit::{value, DocumentMut};

#[derive(Args, Debug)]
pub struct Upgrade {
    /// Plugin to upgrade
    pub plugin: String,

    /// Optional - Specific version to upgrade to
    #[clap(long)]
    pub version: Option<String>,
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

    pub fn upgrade_plugin(&mut self, name: &str, version: &Option<String>) -> Result<()> {
        let version = if let Some(version) = version {
            version
        } else {
            let config = self.workspace.config()?;

            let plugin = config
                .plugins
                .definitions
                .get(name)
                .context("Plugin not found")?;

            &plugin.latest_version.clone().unwrap_or_else(|| {
                plugin
                    .known_good_version
                    .clone()
                    .unwrap_or("latest".to_string())
            })
        };

        if self.document.get("plugin").is_none() {
            bail!("No plugins found in qlty.toml");
        }

        let mut updated = false;

        if let Some(plugin_tables) = self.document["plugin"].as_array_of_tables_mut() {
            for plugin_table in plugin_tables.iter_mut() {
                if plugin_table["name"].as_str() == Some(name) {
                    updated = true;
                    if version != "latest" {
                        plugin_table["version"] = value(version);
                    }
                }
            }
        }

        if !updated {
            bail!("Plugin not found in qlty.toml");
        }

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::write(self.workspace.config_path()?, self.document.to_string())?;
        Ok(())
    }
}

impl Upgrade {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let mut config = ConfigDocument::new(&workspace)?;

        config.upgrade_plugin(&self.plugin, &self.version)?;

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
    fn test_upgrade_plugin() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[plugins.definitions.upgradeable]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.upgradeable.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "upgradeable"
version = "1.0.0"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config.upgrade_plugin("upgradeable", &None).unwrap();

        let expected = r#"
config_version = "0"

[plugins.definitions.upgradeable]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.upgradeable.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "upgradeable"
version = "1.1.0"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_upgrade_plugin_wrong_plugin_name() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[[plugin]]
name = "actual_plugin"
version = "1.0.0"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();

        assert!(config.upgrade_plugin("actual_typo", &None).is_err());
    }

    #[test]
    fn test_upgrade_plugin_with_given_version() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[[plugin]]
name = "plugin_to_upgrade"
version = "1.0.0"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config
            .upgrade_plugin("plugin_to_upgrade", &Some("2.1.0".to_string()))
            .unwrap();

        let expected = r#"
config_version = "0"

[[plugin]]
name = "plugin_to_upgrade"
version = "2.1.0"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }
}
