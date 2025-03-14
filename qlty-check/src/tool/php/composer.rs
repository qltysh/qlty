use crate::tool::command_builder::CommandBuilder;
use crate::tool::finalize_installation_from_cmd_result;
use crate::tool::installations::initialize_installation;
use crate::tool::node::package_json::PackageJson;
use crate::ui::ProgressBar;
use crate::{tool::ToolType, ui::ProgressTask, Tool};
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_native_string;
use serde_json::Value;
use sha2::Digest;
use std::env::split_paths;
use std::path::PathBuf;
use tracing::{debug, info};

use super::PhpPackage;

#[derive(Debug, Clone)]
pub struct Composer {
    pub cmd: Box<dyn CommandBuilder>,
}

impl Tool for Composer {
    fn name(&self) -> String {
        "composer".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn version(&self) -> Option<String> {
        None
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        sha.update(self.name().as_bytes());

        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message("Installing composer");
        info!("Installing composer");

        self.run_command(self.cmd.build(
            "php",
            vec![
                "-r",
                "copy('https://getcomposer.org/installer', 'composer-setup.php');",
            ],
        ))?;
        self.run_command(self.cmd.build("php", vec!["composer-setup.php"]))?;

        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        None // None so that version is not validated for now
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn extra_env_paths(&self) -> Vec<String> {
        split_paths(
            &std::env::var("PATH")
                .with_context(|| "PATH not found for composer")
                .unwrap(),
        )
        .map(path_to_native_string)
        .collect_vec()
    }
}

impl Composer {
    pub fn install_package_file(&self, php_package: &PhpPackage) -> Result<()> {
        info!("Installing composer package file");
        let install_dir = PathBuf::from(php_package.directory());
        let final_composer_file = Self::filter_composer(php_package)?;
        debug!(
            "Writing {} composer.json: {}",
            php_package.name(),
            final_composer_file
        );

        std::fs::write(install_dir.join("composer.json"), final_composer_file)?;
        let composer_phar = PathBuf::from(self.directory()).join("composer.phar");

        let cmd = self
            .cmd
            .build(
                "php",
                vec![
                    &path_to_native_string(composer_phar.to_str().unwrap()),
                    "install",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                    "--no-plugins",
                ],
            )
            .dir(install_dir)
            .full_env(self.env())
            .stderr_capture()
            .stdout_capture()
            .unchecked(); // Capture output for debugging

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(php_package);
        let result = cmd.run();
        let _ =
            finalize_installation_from_cmd_result(php_package, &result, &mut installation, script);

        if result?.status.code() != Some(0) {
            bail!("Failed to install composer package file");
        }

        Ok(())
    }

    // Filter out any dependencies that don't seem related to the plugin
    fn remove_unrelated_dependencies(
        dependencies: &mut Value,
        tool_name: &str,
        package_filters: &[String],
    ) {
        if dependencies.is_null() {
            return;
        }

        let filters = package_filters;
        if !filters.is_empty() {
            if let Some(deps) = dependencies.as_object_mut() {
                deps.retain(|dep_name, _| {
                    dep_name == tool_name || filters.iter().any(|filter| dep_name.contains(filter))
                });
            }
        }
    }

    fn filter_composer(php_package: &PhpPackage) -> Result<String> {
        let composer_file_contents =
            std::fs::read_to_string(php_package.plugin.package_file.as_ref().unwrap())?;
        let mut composer_json = serde_json::from_str::<Value>(&composer_file_contents)?;
        if let Some(root_object) = composer_json.as_object_mut() {
            // Remove autoloads that might be relative to project root
            root_object.remove("autoload");
            root_object.remove("autoload-dev");

            // collapse require-dev into require
            if let Some(dev_dependencies) = root_object.clone().get("require-dev") {
                if let Some(dependencies) = root_object.get_mut("require") {
                    PackageJson::merge_json(dependencies, dev_dependencies.clone());
                } else {
                    root_object.insert("require".to_string(), dev_dependencies.clone());
                }
                root_object.remove("require-dev");
            }

            // clear out unrelated deps
            if let Some(dependencies) = root_object.get_mut("require") {
                Self::remove_unrelated_dependencies(
                    dependencies,
                    &php_package.name(),
                    &php_package.plugin.package_filters,
                );
            }
        }

        Ok(serde_json::to_string_pretty(&composer_json)?)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::tool::{command_builder::test::stub_cmd, php::Php};
    use qlty_config::config::PluginDef;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[test]
    fn test_filter_composer() {
        let temp_path = tempdir().unwrap();
        let list = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));

        let package = PhpPackage {
            cmd: stub_cmd(list.clone()),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                package_file: Some(format!(
                    "{}/composer.json",
                    temp_path.path().to_str().unwrap()
                )),
                package_filters: vec!["foo".to_string()],
                ..Default::default()
            },
            runtime: Php {
                version: "1.0.0".to_string(),
            },
        };

        let composer_file = temp_path.path().join("composer.json");
        std::fs::write(
            composer_file,
            r#"
            {
                "autoload": {
                    "random": "value"
                },
                "autoload-dev": {
                    "random": "value"
                },
                "require": {
                    "foo": "1.0.0",
                    "bar": "1.0.0"
                },
                "require-dev": {
                    "foo-dev": "1.0.0",
                    "bar-dev": "1.0.0"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            Composer::filter_composer(&package).unwrap(),
            r#"{
  "require": {
    "foo": "1.0.0",
    "foo-dev": "1.0.0"
  }
}"#
        );
    }

    #[test]
    fn test_filter_composer_no_filter() {
        let temp_path = tempdir().unwrap();
        let list = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));

        let package = PhpPackage {
            cmd: stub_cmd(list.clone()),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                package_file: Some(format!(
                    "{}/composer.json",
                    temp_path.path().to_str().unwrap()
                )),
                ..Default::default()
            },
            runtime: Php {
                version: "1.0.0".to_string(),
            },
        };

        let composer_file = temp_path.path().join("composer.json");
        std::fs::write(
            composer_file,
            r#"
            {
                "autoload": {
                    "random": "value"
                },
                "autoload-dev": {
                    "random": "value"
                },
                "require": {
                    "foo": "1.0.0",
                    "bar": "1.0.0"
                },
                "require-dev": {
                    "foo-dev": "1.0.0",
                    "bar-dev": "1.0.0"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(
            Composer::filter_composer(&package).unwrap(),
            r#"{
  "require": {
    "foo": "1.0.0",
    "bar": "1.0.0",
    "foo-dev": "1.0.0",
    "bar-dev": "1.0.0"
  }
}"#
        );
    }
}
