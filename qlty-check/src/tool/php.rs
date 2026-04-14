pub mod composer;
use super::command_builder::{default_command_builder, CommandBuilder};
use super::installations::initialize_installation;
use super::runnable_archive::RunnableArchive;
use super::Tool;
use super::ToolType;
use crate::tool::{finalize_installation_from_cmd_result, RuntimeTool};
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::{bail, Context, Result};
use composer::Composer;
use duct::cmd;
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_native_string;
use qlty_config::config::PluginDef;
use sha2::Digest;
use std::collections::HashMap;
use std::env::split_paths;
use std::fmt::Debug;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Php {
    pub version: String,
}

impl Tool for Php {
    fn name(&self) -> String {
        "php".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn version(&self) -> Option<String> {
        Some(self.version.clone())
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        sha.update(self.name().as_bytes());
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message("Verifying Php installation");
        self.verify_installation(self.env()?)?;

        task.set_message("Installing composer");
        let composer = Composer {
            cmd: default_command_builder(),
        };
        composer.setup(task)?;

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
            .with_context(|| "PATH environment variable not found for php runtime")
            .map(|path| split_paths(&path).map(path_to_native_string).collect_vec())
    }
}

impl Php {
    fn verify_installation(&self, env: HashMap<String, String>) -> Result<()> {
        let cmd = cmd!("php", "--version")
            .full_env(env)
            .unchecked()
            .stderr_to_stdout()
            .stdout_capture();

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(self)?;
        let result = cmd.run();
        finalize_installation_from_cmd_result(self, &result, &mut installation, script).ok();

        let output = result?;
        if !output.status.success() {
            bail!("Ensure `php` is installed and in $PATH");
        }

        Ok(())
    }
}

impl RuntimeTool for Php {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(PhpPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
            cmd: default_command_builder(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PhpPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Php,
    cmd: Box<dyn CommandBuilder>,
}

impl Tool for PhpPackage {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn directory(&self) -> String {
        self.system_install_directory()
            .unwrap_or_else(|| self.cache_directory())
    }

    fn tool_type(&self) -> ToolType {
        ToolType::RuntimePackage
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        Some(Box::new(self.runtime.clone()))
    }

    fn version(&self) -> Option<String> {
        self.plugin.version.clone()
    }

    fn version_command(&self) -> Option<String> {
        if self.plugin.package_file.is_none() {
            self.plugin.version_command.clone()
        } else {
            None
        }
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn package_install(&self, task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        task.set_dim_message(&format!("Installing {name}"));
        let composer = Composer {
            cmd: default_command_builder(),
        };
        let phar = composer.phar_path()?;

        if self.plugin.system {
            return self.run_command(self.cmd.build(
                "php",
                vec![
                    &phar,
                    "require",
                    "--dev",
                    "--with-all-dependencies",
                    "--ignore-platform-reqs",
                    "--no-interaction",
                    &format!("{name}:{version}"),
                ],
            ));
        }

        self.run_command(self.cmd.build(
            "php",
            vec![
                &phar,
                "require",
                "--no-interaction",
                &format!("{name}:{version}"),
            ],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        let composer = Composer {
            cmd: self.cmd.clone(),
        };
        composer.setup(task)?;

        if self.plugin.system {
            task.set_dim_message("composer install");
            let phar = composer.phar_path()?;
            return self.run_command(self.cmd.build(
                "php",
                vec![
                    &phar,
                    "install",
                    "--no-interaction",
                    "--ignore-platform-reqs",
                ],
            ));
        }

        debug!("installing package file");
        composer.install_package_file(self)?;

        Ok(())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        if self.plugin.system {
            Ok(vec![PathBuf::from(self.directory())
                .join("vendor")
                .join("bin")
                .to_string_lossy()
                .to_string()])
        } else {
            Ok(vec![self.directory()])
        }
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        if self.plugin.system {
            return Ok(HashMap::new());
        }

        let mut env = HashMap::new();
        env.insert(
            "COMPOSER_VENDOR_DIR".to_string(),
            path_to_native_string(PathBuf::from(format!("{}/vendor", self.directory()))),
        );
        env.insert(
            "COMPOSER_HOME".to_string(),
            path_to_native_string(PathBuf::from(format!("{}/.composer", self.directory()))),
        );
        env.insert(
            "COMPOSER_CACHE_DIR".to_string(),
            path_to_native_string(PathBuf::from(format!(
                "{}/.composer-cache",
                self.directory()
            ))),
        );
        Ok(env)
    }
}

impl RunnableArchive for PhpPackage {}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        tool::command_builder::test::{reroute_tools_root, stub_cmd, ENV_LOCK},
        ui::ProgressTask,
        Progress, Tool,
    };
    use qlty_config::config::PluginDef;
    use std::sync::{Arc, Mutex};
    use tempfile::{tempdir, TempDir};

    pub fn with_php_package(
        callback: impl Fn(
            &mut PhpPackage,
            &TempDir,
            &Arc<Mutex<Vec<Vec<String>>>>,
        ) -> anyhow::Result<()>,
    ) {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| {
            ENV_LOCK.clear_poison();
            err.into_inner()
        });
        let list = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
        let temp_path = tempdir().unwrap();
        let mut pkg = PhpPackage {
            cmd: stub_cmd(list.clone()),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            runtime: super::Php {
                version: "1.0.0".to_string(),
            },
        };
        reroute_tools_root(&temp_path, &pkg);
        callback(&mut pkg, &temp_path, &list).unwrap();
        drop(temp_path);
    }

    fn new_task() -> ProgressTask {
        Progress::new(true, 1).task("PREFIX", "message")
    }

    #[test]
    fn php_package_file_install() {
        with_php_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("composer.json");
            std::fs::write(&pkg_file, r#"{}"#)?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            reroute_tools_root(&temp_path, pkg);

            let composer = Composer {
                cmd: stub_cmd(list.clone()),
            };

            pkg.package_file_install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    vec![
                        "php",
                        "-r",
                        "copy('https://getcomposer.org/installer', 'composer-setup.php');"
                    ],
                    vec!["php", "composer-setup.php"],
                    vec![
                        "php",
                        &path_to_native_string(format!(
                            "{}/.qlty/cache/tools/composer/{}/composer.phar",
                            temp_path.path().display(),
                            composer.directory_name()
                        )),
                        "update",
                        "--no-interaction",
                        "--ignore-platform-reqs"
                    ]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn php_package_system_mode_installs_in_workspace() {
        with_php_package(|pkg, temp_path, list| {
            let pkg_root = temp_path.path().join("backend");
            let pkg_file = pkg_root.join("composer.json");
            std::fs::create_dir_all(&pkg_root)?;
            std::fs::write(&pkg_file, r#"{}"#)?;

            pkg.plugin.system = true;
            pkg.plugin.package = Some("vendor/test".to_string());
            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());

            let composer = Composer {
                cmd: stub_cmd(list.clone()),
            };

            assert_eq!(
                pkg.directory(),
                pkg_root.to_str().unwrap().replace('\\', "/")
            );
            assert_eq!(
                pkg.extra_env_paths()?,
                vec![pkg_root
                    .join("vendor")
                    .join("bin")
                    .to_string_lossy()
                    .to_string()]
            );
            assert!(pkg.extra_env_vars()?.is_empty());

            let composer_phar = path_to_native_string(format!(
                "{}/.qlty/cache/tools/composer/{}/composer.phar",
                temp_path.path().display(),
                composer.directory_name()
            ));

            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    vec![
                        "php".to_string(),
                        composer_phar.clone(),
                        "require".to_string(),
                        "--dev".to_string(),
                        "--with-all-dependencies".to_string(),
                        "--ignore-platform-reqs".to_string(),
                        "--no-interaction".to_string(),
                        "vendor/test:1.0.0".to_string()
                    ],
                    vec![
                        "php".to_string(),
                        "-r".to_string(),
                        "copy('https://getcomposer.org/installer', 'composer-setup.php');"
                            .to_string(),
                    ],
                    vec!["php".to_string(), "composer-setup.php".to_string()],
                    vec![
                        "php".to_string(),
                        composer_phar,
                        "install".to_string(),
                        "--no-interaction".to_string(),
                        "--ignore-platform-reqs".to_string(),
                    ]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn php_package_system_mode_fingerprint_changes_with_package_file() {
        with_php_package(|pkg, temp_path, _| {
            let pkg_root = temp_path.path().join("backend");
            let pkg_file = pkg_root.join("composer.json");
            std::fs::create_dir_all(&pkg_root)?;
            std::fs::write(&pkg_file, r#"{}"#)?;

            pkg.plugin.system = true;
            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            reroute_tools_root(&temp_path, pkg);

            let fingerprint_before = pkg.fingerprint();
            let donefile_before = pkg.donefile_path();

            std::fs::write(&pkg_file, r#"{"require":{"phpstan/phpstan":"^2.0"}}"#)?;

            let fingerprint_after = pkg.fingerprint();
            let donefile_after = pkg.donefile_path();

            assert_ne!(fingerprint_before, fingerprint_after);
            assert_ne!(donefile_before, donefile_after);

            Ok(())
        });
    }
}
