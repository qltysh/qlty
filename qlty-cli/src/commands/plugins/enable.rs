use crate::{
    initializer::{DetectedPlugin, Settings},
    Arguments, CommandError, CommandSuccess, Initializer, QltyRelease,
};
use anyhow::{Context, Result};
use clap::Args;
use console::style;
use qlty_config::{
    config::{IssueMode, PluginDef},
    Workspace,
};
use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
};
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

    pub fn enable_plugin(&mut self, plugin: DetectedPlugin) -> Result<()> {
        let config = self.workspace.config()?;

        let plugin_def = config
            .plugins
            .definitions
            .get(&plugin.name)
            .cloned()
            .with_context(|| {
                format!(
                    "Unknown plugin: The {} plugin was not found in any source.",
                    &plugin.name
                )
            })?;

        if self.document.get("plugin").is_none() {
            self.document["plugin"] = array();
        }

        if let Some(plugin_tables) = self.document["plugin"].as_array_of_tables_mut() {
            for plugin_table in plugin_tables.iter_mut() {
                if plugin_table["name"].as_str() == Some(&plugin.name) {
                    match plugin_table.get("mode") {
                        Some(value) if value.as_str() == Some(IssueMode::Disabled.to_str()) => {
                            plugin_table.remove("mode");
                            return Ok(());
                        }
                        Some(_) | None => {
                            eprintln!(
                                "{} Plugin {} is already enabled",
                                style("⚠").yellow(),
                                &plugin.name
                            );
                            return Ok(());
                        }
                    }
                }
            }
        }

        let mut plugin_table = table();
        plugin_table["name"] = value(&plugin.name);

        if &plugin.version != "latest" {
            plugin_table["version"] = value(&plugin.version);
        }

        if let Some(package_file) = &plugin.package_file {
            plugin_table["package_file"] = value(package_file);
        }

        if !plugin.package_filters.is_empty() {
            let mut filters_array = toml_edit::Array::new();
            for filter in &plugin.package_filters {
                filters_array.push(filter);
            }
            plugin_table["package_filters"] = filters_array.into();
        }

        if let Some(prefix) = &plugin.prefix {
            plugin_table["prefix"] = value(prefix);
        }

        if plugin.mode != IssueMode::Block {
            plugin_table["mode"] = value(plugin.mode.to_str());
        }

        self.document["plugin"]
            .as_array_of_tables_mut()
            .unwrap()
            .push(plugin_table.as_table().unwrap().clone());

        self.copy_configs(&plugin.name, plugin_def)?;

        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        fs::write(self.workspace.config_path()?, self.document.to_string())?;
        Ok(())
    }

    fn copy_configs(&self, plugin_name: &str, plugin_def: PluginDef) -> Result<()> {
        let mut config_files = plugin_def.config_files.clone();

        plugin_def.drivers.iter().for_each(|(_, driver)| {
            config_files.extend(driver.config_files.clone());
        });

        for config_file in &config_files {
            if self.workspace.root.join(config_file).exists() {
                return Ok(()); // If any config file for the plugin already exists, skip copying
            }
        }

        for config_file in &config_files {
            for source in self.workspace.sources_list()?.sources.iter() {
                if let Some(source_file) = source.get_config_file(plugin_name, config_file)? {
                    let file_name = source_file.path.file_name().unwrap();
                    let library_configs_dir = self.workspace.library()?.configs_dir();

                    create_dir_all(&library_configs_dir)?; // Ensure the directory exists
                    let destination = library_configs_dir.join(file_name);
                    source_file.write_to(&destination)?;
                }
            }
        }

        Ok(())
    }

    pub fn enable_plugins(
        &mut self,
        workspace: &Workspace,
        plugins_to_enable: HashMap<String, String>,
    ) -> Result<()> {
        let settings = Settings {
            workspace: workspace.clone(),
            skip_plugins: false,
            skip_default_source: false,
            plugins_to_enable: Some(plugins_to_enable),
            ..Default::default()
        };

        let mut initializer = Initializer::new(settings)?;
        initializer.prepare()?;
        initializer.compute()?;

        for plugin in initializer.plugins {
            self.enable_plugin(plugin)?;
        }

        self.write()?;

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

        let mut plugins_to_enable = HashMap::new();

        for plugin in &self.plugins {
            let parts: Vec<&str> = plugin.split('=').collect();

            match parts.len() {
                1 => {
                    plugins_to_enable.insert(parts[0].to_string(), "latest".to_string());
                }
                2 => {
                    plugins_to_enable.insert(parts[0].to_string(), parts[1].to_string());
                }
                _ => {
                    return CommandError::err("Invalid plugin format");
                }
            }
        }

        let mut config = ConfigDocument::new(&workspace)?;
        config.enable_plugins(&workspace, plugins_to_enable)?;

        CommandSuccess::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_analysis::utils::fs::path_to_native_string;
    use qlty_test_utilities::git::{create_temp_dir, init as init_repo, sample_repo};

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
        let plugin = DetectedPlugin {
            name: "to_enable".to_string(),
            version: "latest".to_string(),
            ..Default::default()
        };
        config.enable_plugin(plugin).unwrap();

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
suggested_mode = "comment"

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
        let plugin = DetectedPlugin {
            name: "to_enable".to_string(),
            version: "1.2.1".to_string(),
            ..Default::default()
        };
        config.enable_plugin(plugin).unwrap();

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
        let plugin = DetectedPlugin {
            name: "already_enabled".to_string(),
            version: "1.2.1".to_string(),
            ..Default::default()
        };
        config.enable_plugin(plugin).unwrap();

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
        let plugin = DetectedPlugin {
            name: "marked_disabled".to_string(),
            version: "1.2.1".to_string(),
            ..Default::default()
        };
        config.enable_plugin(plugin).unwrap();

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
        let plugin = DetectedPlugin {
            name: "shellcheck".to_string(),
            version: "latest".to_string(),
            ..Default::default()
        };
        config.enable_plugin(plugin).unwrap();

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

    #[test]
    fn test_package_files() {
        let temp_dir = create_temp_dir();
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

        let package_file = temp_path.join("package.json");
        let package_file_contents = r#"{
                "devDependencies": {
                    "eslint": "8.0.0",
                    "eslint-remix": "14.0.0",
                    "eslint-prettier": "4.0.0",
                    "other": "1.0.0"
                },
                "scripts": {
                    "test": "echo hello && exit 1"
                }
            }"#;
        std::fs::write(&package_file, package_file_contents).ok();

        init_repo(&temp_path);

        let workspace = Workspace {
            root: temp_path.clone(),
        };

        let mut config = ConfigDocument::new(&workspace).unwrap();
        let plugin = HashMap::from([("eslint".to_string(), "latest".to_string())]);
        config.enable_plugins(&workspace, plugin).unwrap();

        let expected = r#"
config_version = "0"

[[source]]
name = "default"
default = true

[[plugin]]
name = "eslint"
package_file = "package.json"
package_filters = ["eslint", "prettier"]
        "#;

        assert_eq!(config.document.to_string().trim(), expected.trim());
    }
}
