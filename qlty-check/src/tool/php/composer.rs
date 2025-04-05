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
use tracing::{debug, error, info};

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

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        std::env::var("PATH")
            .with_context(|| "PATH environment variable not found for composer")
            .map(|path| split_paths(&path).map(path_to_native_string).collect_vec())
    }
}

impl Composer {
    pub fn install_package_file(&self, php_package: &PhpPackage) -> Result<()> {
        info!("Installing composer package file");
        Self::update_composer_json(php_package)?;
        let composer_phar = PathBuf::from(self.directory()).join("composer.phar");
        let composer_path = composer_phar.to_str().with_context(|| {
            format!(
                "Failed to convert composer path to string: {:?}",
                composer_phar
            )
        })?;

        let cmd = self
            .cmd
            .build(
                "php",
                vec![
                    &path_to_native_string(composer_path),
                    "update",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                ],
            )
            .dir(php_package.directory())
            .full_env(self.env()?)
            .stderr_capture()
            .stdout_capture()
            .unchecked(); // Capture output for debugging

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(php_package)?;
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
        let package_file = php_package
            .plugin
            .package_file
            .as_ref()
            .with_context(|| "Missing package_file in plugin definition")?;
        let composer_file_contents = std::fs::read_to_string(package_file)?;
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

    fn update_composer_json(php_package: &PhpPackage) -> Result<()> {
        let install_dir = PathBuf::from(php_package.directory());
        let package_file_contents = Self::filter_composer(php_package)?;
        let user_json = serde_json::from_str::<Value>(&package_file_contents)?;
        let staged_file = install_dir.join("composer.json");
        let mut data_json = Value::Object(serde_json::Map::new());

        if staged_file.exists() {
            // use the original composer.json contents, merging package_file contents on top.
            // this will retain any existing dependencies provided by the initial tool installation
            let contents = std::fs::read_to_string(&staged_file)?;
            data_json = match serde_json::from_str::<Value>(&contents) {
                Ok(json) => json,
                Err(err) => {
                    error!("Failed to parse existing composer.json: {}", err);
                    Value::Object(serde_json::Map::new())
                }
            };
        }

        PackageJson::merge_json(&mut data_json, user_json);

        let final_composer_file = serde_json::to_string_pretty(&data_json)?;
        debug!(
            "Writing {} composer.json to {:?}: {}",
            php_package.name, staged_file, final_composer_file
        );

        std::fs::write(staged_file, final_composer_file)?;

        if php_package.plugin.package_filters.is_empty() {
            let package_file = php_package
                .plugin
                .package_file
                .as_ref()
                .with_context(|| "Missing package_file in plugin definition")?;

            let package_file_path = PathBuf::from(package_file);
            if let Some(parent_dir) = package_file_path.parent() {
                let lock_file = parent_dir.join("composer.lock");

                if lock_file.exists() {
                    let staging_lock_file = install_dir.join("composer.lock");
                    debug!(
                        "Copying lock file from {:?} to {:?}",
                        lock_file, staging_lock_file
                    );
                    std::fs::copy(lock_file, staging_lock_file)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::tool::{
        command_builder::test::{reroute_tools_root, stub_cmd},
        php::{test::with_php_package, Php},
    };
    use qlty_analysis::utils::fs::path_to_string;
    use qlty_config::config::PluginDef;
    use std::path::PathBuf;
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

    #[test]
    fn test_update_existing_composer_json() {
        with_php_package(|pkg, tempdir, _| {
            let existing_composer_file = PathBuf::from(pkg.directory()).join("composer.json");
            std::fs::write(
                &existing_composer_file,
                r#"
                {
                    "require": {
                        "tool": "1.0.0"
                    }
                }"#,
            )
            .unwrap();

            let package_file = tempdir.path().join("user-composer.json");
            std::fs::write(
                &package_file,
                r#"
                {
                    "require": {
                        "other-tool": "1.0.0"
                    },
                    "require-dev": {
                        "other-tool-dev": "1.0.0"
                    },
                    "autoload": {
                        "random": "value"
                    },
                    "autoload-dev": {
                        "random": "value"
                    }
                }"#,
            )
            .unwrap();

            pkg.plugin.package_file = Some(path_to_string(package_file));
            reroute_tools_root(tempdir, pkg);

            Composer::update_composer_json(pkg).unwrap();

            let composer_file_contents = std::fs::read_to_string(&existing_composer_file)?;
            let composer_json = serde_json::from_str::<Value>(&composer_file_contents)?;

            assert_eq!(
                composer_json,
                serde_json::json!({
                    "require": {
                        "tool": "1.0.0",
                        "other-tool": "1.0.0",
                        "other-tool-dev": "1.0.0"
                    }
                })
            );

            Ok(())
        });
    }

    #[test]
    fn test_lock_file_copying_with_empty_filters() {
        with_php_package(|pkg, tempdir, _| {
            // Create parent directory with composer.json and composer.lock
            let parent_dir = tempdir.path().join("project");
            std::fs::create_dir_all(&parent_dir)?;

            // Create composer.json file
            let user_composer_file = parent_dir.join("composer.json");
            let composer_contents = r#"{
                "require": {
                    "phpstan/phpstan": "^1.10.0"
                }
            }"#;
            std::fs::write(&user_composer_file, composer_contents)?;

            // Create composer.lock file
            let lock_file = parent_dir.join("composer.lock");
            let lock_contents = r#"{
                "packages": [{
                    "name": "phpstan/phpstan",
                    "version": "1.10.35"
                }]
            }"#;
            std::fs::write(&lock_file, lock_contents)?;

            // Configure package and execute
            pkg.plugin.package_file = Some(path_to_string(&user_composer_file));
            pkg.plugin.package_filters = vec![]; // Empty filters should trigger lock file copying
            reroute_tools_root(tempdir, pkg);

            let install_dir = PathBuf::from(pkg.directory());
            std::fs::create_dir_all(&install_dir)?;

            Composer::update_composer_json(pkg)?;

            // Verify lock file was copied
            let staged_lock_file = install_dir.join("composer.lock");
            assert!(staged_lock_file.exists(), "Lock file was not copied");

            let lock_content = std::fs::read_to_string(&staged_lock_file)?;
            assert!(
                lock_content.contains("phpstan/phpstan"),
                "Lock file contents are incorrect"
            );

            Ok(())
        });
    }

    #[test]
    fn test_lock_file_not_copied_with_filters() {
        with_php_package(|pkg, tempdir, _| {
            // Create parent directory with composer.json and composer.lock
            let parent_dir = tempdir.path().join("project");
            std::fs::create_dir_all(&parent_dir)?;

            // Create composer.json file
            let user_composer_file = parent_dir.join("composer.json");
            let composer_contents = r#"{
                "require": {
                    "phpstan/phpstan": "^1.10.0"
                }
            }"#;
            std::fs::write(&user_composer_file, composer_contents)?;

            // Create composer.lock file
            let lock_file = parent_dir.join("composer.lock");
            let lock_contents = r#"{
                "packages": [{
                    "name": "phpstan/phpstan",
                    "version": "1.10.35"
                }]
            }"#;
            std::fs::write(&lock_file, lock_contents)?;

            // Configure package with filters
            pkg.plugin.package_file = Some(path_to_string(&user_composer_file));
            pkg.plugin.package_filters = vec!["phpstan".to_string()]; // With filters, lock file should not be copied
            reroute_tools_root(tempdir, pkg);

            let install_dir = PathBuf::from(pkg.directory());
            std::fs::create_dir_all(&install_dir)?;

            Composer::update_composer_json(pkg)?;

            // Verify lock file was not copied
            let staged_lock_file = install_dir.join("composer.lock");
            assert!(
                !staged_lock_file.exists(),
                "Lock file was copied but should not have been"
            );

            Ok(())
        });
    }
}
