mod apt;
pub mod composer;
use super::command_builder::{default_command_builder, CommandBuilder};
use super::installations::initialize_installation;
use super::runnable_archive::RunnableArchive;
use super::Tool;
use super::ToolType;
use crate::tool::{finalize_installation_from_cmd_result, update_package_hash, RuntimeTool};
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::{bail, Context, Result};
use composer::Composer;
use duct::cmd;
use itertools::Itertools;
use once_cell::sync::OnceCell;
use qlty_analysis::utils::fs::path_to_native_string;
use qlty_config::config::PluginDef;
use regex::Regex;
use sha2::Digest;
use std::collections::HashMap;
use std::env::split_paths;
use std::fmt::Debug;
use std::path::PathBuf;
use tracing::debug;

const UNKNOWN_SYSTEM_PHP_VERSION: &str = "unknown";

static SYSTEM_PHP_MINOR_VERSION: OnceCell<String> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct Php {
    pub version: String,
    pub(crate) apt: Option<apt::AptPhp>,
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
        if let Some(apt) = &self.apt {
            apt.update_hash(sha);
        } else {
            // Composer resolves plugin dependencies for the PHP that installs them,
            // so a new minor version of the system PHP reinstalls PHP plugins.
            sha.update(self.system_minor_version().as_bytes());
        }
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        if let Some(apt) = &self.apt {
            task.set_message(&format!(
                "Extracting PHP {} ({})",
                apt.version, apt.debian_version
            ));
            return apt.install(self, task);
        }
        task.set_message("Verifying Php installation");
        self.verify_installation(self.env()?)?;

        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        self.apt.as_ref().map(|_| "php --version".to_owned())
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        let mut paths = Vec::new();
        if self.apt.is_some() {
            paths.push(path_to_native_string(
                PathBuf::from(self.directory()).join("bin"),
            ));
        }
        paths.extend(
            std::env::var("PATH")
                .with_context(|| "PATH environment variable not found for php runtime")
                .map(|path| split_paths(&path).map(path_to_native_string).collect_vec())?,
        );
        Ok(paths)
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        if self.apt.is_some() {
            let directory = PathBuf::from(self.directory());
            Ok(HashMap::from([
                (
                    "PHPRC".to_owned(),
                    path_to_native_string(directory.join("php.ini")),
                ),
                (
                    "PHP_INI_SCAN_DIR".to_owned(),
                    path_to_native_string(directory.join("conf.d")),
                ),
            ]))
        } else {
            Ok(["PHPRC", "PHP_INI_SCAN_DIR"]
                .into_iter()
                .filter_map(|key| std::env::var(key).ok().map(|value| (key.to_owned(), value)))
                .collect())
        }
    }
}

impl Php {
    pub fn new(version: &str) -> Result<Self> {
        let enabled = std::env::var("QLTY_FEATURE_PHP_APT_INSTALL")
            .is_ok_and(|value| ["true", "1", "on"].contains(&value.to_ascii_lowercase().as_str()));
        Self::with_apt_enabled(version, enabled)
    }

    fn with_apt_enabled(version: &str, enabled: bool) -> Result<Self> {
        let apt = if enabled {
            Some(apt::AptPhp::resolve(version)?)
        } else {
            None
        };
        Ok(Self {
            version: apt
                .as_ref()
                .map_or_else(|| version.to_owned(), |runtime| runtime.version.clone()),
            apt,
        })
    }

    fn verify_installation(&self, env: HashMap<String, String>) -> Result<()> {
        let cmd = cmd!("php", "--version")
            .full_env(env)
            .unchecked()
            .stderr_to_stdout()
            .stdout_capture();

        let script = format!("{cmd:?}");
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

    fn system_minor_version(&self) -> &'static str {
        SYSTEM_PHP_MINOR_VERSION.get_or_init(|| self.detect_system_minor_version())
    }

    fn detect_system_minor_version(&self) -> String {
        let output = self.env().and_then(|env| {
            Ok(cmd!(
                "php",
                "-r",
                "echo PHP_MAJOR_VERSION, '.', PHP_MINOR_VERSION;"
            )
            .full_env(env)
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()?)
        });

        let version = match output {
            Ok(output) if output.status.success() => {
                Self::minor_version_from_output(&String::from_utf8_lossy(&output.stdout))
            }
            _ => None,
        };

        version.unwrap_or_else(|| {
            debug!("Could not read the system PHP version");
            UNKNOWN_SYSTEM_PHP_VERSION.to_string()
        })
    }

    fn minor_version_from_output(stdout: &str) -> Option<String> {
        let version = stdout.trim().lines().last()?.trim();
        Regex::new(r"^\d+\.\d+$")
            .unwrap()
            .is_match(version)
            .then(|| version.to_string())
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

    fn tool_type(&self) -> ToolType {
        ToolType::RuntimePackage
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        Some(Box::new(self.runtime.clone()))
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        update_package_hash(self, sha)?;
        self.composer().update_hash(sha)?;

        if self.plugin.package_file.is_some() {
            if let Some(lock_file) = Composer::lock_file_to_stage(&self.plugin)? {
                sha.update(b"composer.lock");
                std::io::copy(&mut std::fs::File::open(lock_file)?, sha)?;
            }
        }

        Ok(())
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

    // The Php runtime may already be installed from an earlier CLI release, so
    // set up Composer here, where a change to the pinned version takes effect
    fn pre_install(&self, task: &ProgressTask) -> Result<()> {
        self.composer().setup(task)
    }

    fn package_install(&self, task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        task.set_dim_message(&format!("Installing {name}"));
        let composer = self.composer();

        let composer_phar = PathBuf::from(composer.directory()).join("composer.phar");
        let composer_path = composer_phar.to_str().with_context(|| {
            format!("Failed to convert composer path to string: {composer_phar:?}")
        })?;

        self.run_command(self.cmd.build(
            "php",
            vec![
                &path_to_native_string(composer_path),
                "require",
                "--no-interaction",
                format!("{name}:{version}").as_str(),
            ],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.package_file.is_some() {
            debug!("installing package file");
            let composer = self.composer();
            composer.setup(task)?;
            composer.install_package_file(self)?;
        }

        Ok(())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![self.directory()])
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = self.runtime.extra_env_vars()?;
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

    fn extra_env_vars_with_plugin_env(&self) -> Result<HashMap<String, String>> {
        let mut env = self.extra_env_vars()?;
        env.extend(self.load_environment_vars(&self.plugin.environment));
        if self.runtime.apt.is_some() {
            // Plugin definitions historically forwarded the host scan directory.
            // Never mix those modules with an extracted interpreter's ABI.
            env.extend(self.runtime.extra_env_vars()?);
        }
        Ok(env)
    }
}

impl PhpPackage {
    fn composer(&self) -> Composer {
        Composer {
            cmd: self.cmd.clone(),
            runtime: self.runtime.apt.as_ref().map(|_| self.runtime.clone()),
        }
    }
}

impl RunnableArchive for PhpPackage {}

#[cfg(test)]
pub mod test {
    use super::composer::COMPOSER_VERSION;
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
                apt: None,
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
    fn php_minor_version_from_output() {
        assert_eq!(
            Php::minor_version_from_output("8.4\n"),
            Some("8.4".to_string())
        );
    }

    #[test]
    fn php_minor_version_from_output_after_startup_warning() {
        assert_eq!(
            Php::minor_version_from_output(
                "Warning: Module \"xdebug\" is already loaded in Unknown on line 0\n8.3"
            ),
            Some("8.3".to_string())
        );
    }

    #[test]
    fn php_minor_version_from_output_rejects_unexpected_output() {
        assert_eq!(Php::minor_version_from_output("PHP 8.4.8 (cli)"), None);
    }

    #[test]
    fn php_apt_flag_controls_package_resolution() {
        assert!(Php::with_apt_enabled("8.4", false).unwrap().apt.is_none());
        assert!(Php::with_apt_enabled("invalid old setting", false)
            .unwrap()
            .apt
            .is_none());
        assert!(Php::with_apt_enabled("8.4", false)
            .unwrap()
            .version_command()
            .is_none());
        assert!(Php::with_apt_enabled("system", true)
            .unwrap_err()
            .to_string()
            .contains("major.minor"));
    }

    #[test]
    fn php_apt_revision_separates_runtime_composer_and_package_caches() {
        with_php_package(|pkg, _, _| {
            let system_package = pkg.fingerprint();
            let system_composer = pkg.composer().fingerprint();
            pkg.runtime = Php {
                version: "8.4.27".into(),
                apt: Some(apt::test_runtime("8.4.27", "1")),
            };
            let runtime_a = pkg.runtime.fingerprint();
            let composer_a = pkg.composer().fingerprint();
            let package_a = pkg.fingerprint();
            assert_ne!(system_package, package_a);
            assert_ne!(system_composer, composer_a);
            pkg.runtime.apt = Some(apt::test_runtime("8.4.27", "2"));
            assert_ne!(runtime_a, pkg.runtime.fingerprint());
            assert_ne!(composer_a, pkg.composer().fingerprint());
            assert_ne!(package_a, pkg.fingerprint());
            Ok(())
        });
    }

    #[test]
    fn php_apt_configuration_reaches_composer_and_overrides_host_plugin_settings() {
        with_php_package(|pkg, _, _| {
            pkg.runtime = Php {
                version: "8.4.27".into(),
                apt: Some(apt::test_runtime("8.4.27", "1")),
            };
            pkg.plugin
                .environment
                .push(qlty_config::config::PluginEnvironment {
                    name: "PHP_INI_SCAN_DIR".into(),
                    value: "/host/php/conf.d".into(),
                    ..Default::default()
                });
            let runtime_env = pkg.runtime.env()?;
            let composer_env = pkg.composer().env()?;
            let package_env = pkg.env()?;
            for name in ["PHPRC", "PHP_INI_SCAN_DIR"] {
                assert_eq!(runtime_env[name], composer_env[name]);
                assert_eq!(runtime_env[name], package_env[name]);
            }
            let bin = PathBuf::from(pkg.runtime.directory()).join("bin");
            assert_eq!(split_paths(&composer_env["PATH"]).next().unwrap(), bin);
            let package_paths = split_paths(&package_env["PATH"]).collect_vec();
            assert!(package_paths.contains(&bin));
            assert!(package_env.contains_key("COMPOSER_VENDOR_DIR"));
            for path in split_paths(&std::env::var("PATH")?) {
                assert!(package_paths.contains(&path));
            }
            assert_eq!(pkg.runtime.expected_version()?, Some("8.4.27".into()));
            Ok(())
        });
    }

    fn failing_install_cmd(list: Arc<Mutex<Vec<Vec<String>>>>) -> Box<dyn CommandBuilder> {
        #[derive(Debug)]
        struct FailingInstallCommandBuilder {
            list: Arc<Mutex<Vec<Vec<String>>>>,
        }
        impl CommandBuilder for FailingInstallCommandBuilder {
            fn build(&self, bin: &str, args: Vec<&str>) -> duct::Expression {
                let failing = args.contains(&"install");
                let mut pushed_list = vec![bin];
                pushed_list.extend(args);
                self.list
                    .lock()
                    .unwrap()
                    .push(pushed_list.iter().map(|s| s.to_string()).collect());

                if failing {
                    crate::tool::command_builder::Command::new(None, "exit 1").cmd
                } else {
                    crate::tool::command_builder::Command::new(None, "exit 0").cmd
                }
            }

            fn clone_box(&self) -> Box<dyn CommandBuilder> {
                Box::new(FailingInstallCommandBuilder {
                    list: Arc::clone(&self.list),
                })
            }
        }
        Box::new(FailingInstallCommandBuilder { list })
    }

    #[test]
    fn php_package_pre_install_installs_pinned_composer() {
        with_php_package(|pkg, _, list| {
            pkg.pre_install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    vec![
                        "php",
                        "-r",
                        "copy('https://getcomposer.org/installer', 'composer-setup.php');"
                    ],
                    vec![
                        "php",
                        "composer-setup.php",
                        &format!("--version={COMPOSER_VERSION}")
                    ]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn php_package_install_no_package_file() {
        with_php_package(|pkg, _, list| {
            pkg.package_file_install(&new_task())?;
            assert!(list.lock().unwrap().is_empty());

            Ok(())
        });
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
                runtime: None,
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
                    vec![
                        "php",
                        "composer-setup.php",
                        &format!("--version={COMPOSER_VERSION}")
                    ],
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
    fn php_package_file_install_with_lock_file() {
        with_php_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("composer.json");
            std::fs::write(&pkg_file, r#"{"require":{"test":"1.0.0"}}"#)?;

            let lock_file = temp_path.path().join("composer.lock");
            std::fs::write(
                &lock_file,
                r#"{"packages":[{"name":"test","version":"1.0.0"}]}"#,
            )?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            reroute_tools_root(&temp_path, pkg);

            let composer = Composer {
                cmd: stub_cmd(list.clone()),
                runtime: None,
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
                    vec![
                        "php",
                        "composer-setup.php",
                        &format!("--version={COMPOSER_VERSION}")
                    ],
                    vec![
                        "php",
                        &path_to_native_string(format!(
                            "{}/.qlty/cache/tools/composer/{}/composer.phar",
                            temp_path.path().display(),
                            composer.directory_name()
                        )),
                        "install",
                        "--no-interaction",
                        "--ignore-platform-reqs"
                    ]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn php_package_file_install_falls_back_to_update_when_install_fails() {
        with_php_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("composer.json");
            std::fs::write(&pkg_file, r#"{"require":{"test":"1.0.0"}}"#)?;

            let lock_file = temp_path.path().join("composer.lock");
            std::fs::write(
                &lock_file,
                r#"{"packages":[{"name":"other","version":"1.0.0"}]}"#,
            )?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            pkg.cmd = failing_install_cmd(list.clone());
            reroute_tools_root(&temp_path, pkg);

            let composer = Composer {
                cmd: failing_install_cmd(list.clone()),
                runtime: None,
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
                    vec![
                        "php",
                        "composer-setup.php",
                        &format!("--version={COMPOSER_VERSION}")
                    ],
                    vec![
                        "php",
                        &path_to_native_string(format!(
                            "{}/.qlty/cache/tools/composer/{}/composer.phar",
                            temp_path.path().display(),
                            composer.directory_name()
                        )),
                        "install",
                        "--no-interaction",
                        "--ignore-platform-reqs"
                    ],
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
    fn php_package_file_install_with_lock_file_and_package_filters() {
        with_php_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("composer.json");
            std::fs::write(&pkg_file, r#"{"require":{"test":"1.0.0"}}"#)?;

            let lock_file = temp_path.path().join("composer.lock");
            std::fs::write(
                &lock_file,
                r#"{"packages":[{"name":"test","version":"1.0.0"}]}"#,
            )?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            pkg.plugin.package_filters = vec!["test".to_string()];
            reroute_tools_root(&temp_path, pkg);

            let composer = Composer {
                cmd: stub_cmd(list.clone()),
                runtime: None,
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
                    vec![
                        "php",
                        "composer-setup.php",
                        &format!("--version={COMPOSER_VERSION}")
                    ],
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

    fn php_package_with_package_file(package_file: &std::path::Path) -> PhpPackage {
        PhpPackage {
            cmd: default_command_builder(),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                package_file: Some(package_file.to_str().unwrap().to_string()),
                ..Default::default()
            },
            runtime: super::Php {
                version: "1.0.0".to_string(),
                apt: None,
            },
        }
    }

    #[test]
    fn php_package_directory_name_differs_by_package_file_contents() {
        let temp_path = tempdir().unwrap();
        let file_a = temp_path.path().join("a.json");
        let file_b = temp_path.path().join("b.json");
        std::fs::write(&file_a, r#"{"require": {"a/a": "1.0"}}"#).unwrap();
        std::fs::write(&file_b, r#"{"require": {"b/b": "1.0"}}"#).unwrap();

        assert_ne!(
            php_package_with_package_file(&file_a).directory_name(),
            php_package_with_package_file(&file_b).directory_name()
        );
    }

    #[test]
    fn php_package_directory_name_differs_by_lock_file_contents() {
        let dir_a = tempdir().unwrap();
        let dir_b = tempdir().unwrap();
        let file_a = dir_a.path().join("composer.json");
        let file_b = dir_b.path().join("composer.json");
        std::fs::write(&file_a, "{}").unwrap();
        std::fs::write(&file_b, "{}").unwrap();
        std::fs::write(dir_a.path().join("composer.lock"), r#"{"packages": []}"#).unwrap();
        std::fs::write(dir_b.path().join("composer.lock"), r#"{"packages": [{}]}"#).unwrap();

        assert_ne!(
            php_package_with_package_file(&file_a).directory_name(),
            php_package_with_package_file(&file_b).directory_name()
        );
    }

    #[test]
    fn php_package_directory_name_differs_by_lock_file_presence() {
        let dir_a = tempdir().unwrap();
        let dir_b = tempdir().unwrap();
        let file_a = dir_a.path().join("composer.json");
        let file_b = dir_b.path().join("composer.json");
        std::fs::write(&file_a, "{}").unwrap();
        std::fs::write(&file_b, "{}").unwrap();
        std::fs::write(dir_a.path().join("composer.lock"), "").unwrap();

        assert_ne!(
            php_package_with_package_file(&file_a).directory_name(),
            php_package_with_package_file(&file_b).directory_name()
        );
    }

    #[test]
    fn php_package_directory_name_ignores_lock_file_with_package_filters() {
        let dir_a = tempdir().unwrap();
        let dir_b = tempdir().unwrap();
        let file_a = dir_a.path().join("composer.json");
        let file_b = dir_b.path().join("composer.json");
        std::fs::write(&file_a, "{}").unwrap();
        std::fs::write(&file_b, "{}").unwrap();
        std::fs::write(dir_a.path().join("composer.lock"), r#"{"packages": []}"#).unwrap();
        let mut pkg_a = php_package_with_package_file(&file_a);
        let mut pkg_b = php_package_with_package_file(&file_b);
        pkg_a.plugin.package_filters = vec!["tool".to_string()];
        pkg_b.plugin.package_filters = vec!["tool".to_string()];

        assert_eq!(pkg_a.directory_name(), pkg_b.directory_name());
    }
}
