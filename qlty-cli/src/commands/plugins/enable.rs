use crate::{
    initializer::{PluginEnabler, Settings},
    Arguments, CommandError, CommandSuccess, Initializer, QltyRelease,
};
use anyhow::{Context, Result};
use clap::Args;
use console::style;
use qlty_config::{config::IssueMode, Workspace};
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
        let config = self.workspace.config()?;

        let plugin_def = config
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

        if let Some(plugin_tables) = self.document["plugin"].as_array_of_tables_mut() {
            for plugin_table in plugin_tables.iter_mut() {
                if plugin_table["name"].as_str() == Some(name) {
                    match plugin_table.get("mode") {
                        Some(value) if value.as_str() == Some(IssueMode::Disabled.to_str()) => {
                            plugin_table.remove("mode");
                            return Ok(());
                        }
                        Some(_) | None => {
                            eprintln!("{} Plugin {} is already enabled", style("⚠").yellow(), name);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Use the unified enabler to check for package files and filters
        let enabler = PluginEnabler::new(self.workspace.clone());
        let enable_result = enabler.enable_plugin(name, &plugin_def)?;

        let mut plugin_table = table();
        plugin_table["name"] = value(name);

        // Use the provided version if it's not "latest", otherwise check if enabler found a specific version
        let final_version = if version != "latest" {
            Some(version.to_string())
        } else if !enable_result.version.is_empty() && enable_result.version != "latest" {
            // Only use the enabler version if it's different from latest (e.g., found in lockfile)
            Some(enable_result.version)
        } else {
            None
        };

        if let Some(version_str) = final_version {
            plugin_table["version"] = value(&version_str);
        }

        // Add package file information if found
        if let Some(package_file) = &enable_result.package_file {
            plugin_table["package_file"] = value(package_file);
        }

        if !enable_result.package_filters.is_empty() {
            let mut filters_array = array();
            let filters_as_array = filters_array.as_array_mut().unwrap();
            for filter in &enable_result.package_filters {
                filters_as_array.push(filter);
            }
            plugin_table["package_filters"] = filters_array;
        }

        if let Some(prefix) = &enable_result.prefix {
            plugin_table["prefix"] = value(prefix);
        }

        self.document["plugin"]
            .as_array_of_tables_mut()
            .unwrap()
            .push(plugin_table.as_table().unwrap().clone());

        // Copy configs using the enabler
        enabler.copy_configs(name, &plugin_def)?;

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::write(self.workspace.config_path()?, self.document.to_string())?;
        Ok(())
    }
}

impl Enable {
    pub fn execute(&self, args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if !args.no_upgrade_check {
            QltyRelease::upgrade_check().ok();
        }

        Workspace::assert_git_directory_root()?;

        let workspace = Workspace::new()?;

        if workspace.config_exists()? {
            workspace.fetch_sources()?;
        } else {
            let library = workspace.library()?;
            library.create()?;

            let mut initializer = Initializer::new(Settings {
                workspace: workspace.clone(),
                skip_plugins: true,
                ..Default::default()
            })?;

            initializer.prepare()?;
            initializer.compute()?;
            initializer.write()?;

            eprintln!("{} Created .qlty/qlty.toml", style("✔").green());
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_analysis::utils::fs::path_to_native_string;
    use qlty_test_utilities::git::sample_repo;

    #[test]
    fn test_enable_plugin() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[plugins.definitions.to_enable]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.to_enable.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config.enable_plugin("to_enable", "latest").unwrap();

        let expected = r#"
config_version = "0"

[plugins.definitions.to_enable]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.to_enable.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "to_enable"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_enable_plugin_wrong_plugin_name() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[plugins.definitions.to_enable]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.to_enable.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config.enable_plugin("to_enable", "1.2.1").unwrap();

        let expected = r#"
config_version = "0"

[plugins.definitions.to_enable]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.to_enable.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "to_enable"
version = "1.2.1"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_enable_plugin_when_already_enabled() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[plugins.definitions.already_enabled]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.already_enabled.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "already_enabled"
version = "0.9.0"
mode = "monitor"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config.enable_plugin("already_enabled", "1.2.1").unwrap();

        let expected = r#"
config_version = "0"

[plugins.definitions.already_enabled]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.already_enabled.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "already_enabled"
version = "0.9.0"
mode = "monitor"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_enable_plugin_when_plugin_disabled() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[plugins.definitions.marked_disabled]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.marked_disabled.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "marked_disabled"
version = "0.9.0"
mode = "disabled"
            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config.enable_plugin("marked_disabled", "1.2.1").unwrap();

        let expected = r#"
config_version = "0"

[plugins.definitions.marked_disabled]
file_types = ["ALL"]
latest_version = "1.1.0"

[plugins.definitions.marked_disabled.drivers.lint]
script = "ls -l ${target}"
success_codes = [0]
output = "pass_fail"

[[plugin]]
name = "marked_disabled"
version = "0.9.0"
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }

    #[test]
    fn test_copying_configs() {
        let (temp_dir, _) = sample_repo();
        let temp_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&temp_path.join(path_to_native_string(".qlty"))).ok();
        fs::write(
            &temp_path.join(path_to_native_string(".qlty/qlty.toml")),
            r#"
config_version = "0"

[[source]]
name = "default"
default = true

            "#,
        )
        .ok();

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        config.enable_plugin("shellcheck", "latest").unwrap();

        let expected = r#"
config_version = "0"

[[source]]
name = "default"
default = true

[[plugin]]
name = "shellcheck"
        "#;

        let expected_config_path = temp_dir
            .path()
            .join(path_to_native_string(".qlty/configs"))
            .join(".shellcheckrc");

        assert!(expected_config_path.exists(), "Config file was not copied");
        assert_eq!(config.document.to_string().trim(), expected.trim());
    }
}
