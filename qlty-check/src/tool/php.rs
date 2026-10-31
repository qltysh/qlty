pub mod composer;
use super::command_builder::{default_command_builder, CommandBuilder};
use super::installations::initialize_installation;
use super::runnable_archive::RunnableArchive;
use super::Tool;
use super::ToolType;
use crate::tool::{finalize_installation_from_cmd_result, update_package_hash, RuntimeTool};
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::{bail, ensure, Context, Result};
use composer::{Composer, PHP_PACKAGE_CACHE_BUSTER};
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
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

const UNKNOWN_SYSTEM_PHP_VERSION: &str = "unknown";
const RUNNER_PHP_FALLBACK_VERSION: &str = "8.3";

static SYSTEM_PHP_MINOR_VERSION: OnceCell<String> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct Php {
    pub version: String,
    pub(crate) runner_root: Option<PathBuf>,
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
        if let Some(root) = &self.runner_root {
            sha.update(b"runner-php-v1\0");
            sha.update(path_to_native_string(root).as_bytes());
            sha.update(b"\0");
            sha.update(self.runner_minor_version().as_bytes());
        } else {
            // Composer resolves plugin dependencies for the PHP that installs them,
            // so a new minor version of the system PHP reinstalls PHP plugins
            sha.update(self.system_minor_version().as_bytes());
        }
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message("Verifying Php installation");
        self.verify_runner_version()?;
        self.verify_installation(self.env()?)?;

        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        self.runner_root.as_ref().map(|_| "php --version".into())
    }

    fn expected_version(&self) -> Result<Option<String>> {
        match self.runner_directory() {
            Some(directory) => Ok(Some(
                std::fs::read_to_string(directory.join("version"))
                    .context("Failed to read the runner's PHP version")?
                    .trim()
                    .to_owned(),
            )),
            None => Ok(None),
        }
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        let mut paths = Vec::new();
        if let Some(directory) = self.runner_directory() {
            paths.push(path_to_native_string(directory.join("bin")));
        }
        paths.extend(
            std::env::var("PATH")
                .with_context(|| "PATH environment variable not found for php runtime")
                .map(|path| split_paths(&path).map(path_to_native_string).collect_vec())?,
        );
        Ok(paths)
    }
}

impl Php {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_owned(),
            runner_root: std::env::var_os("QLTY_PHP_RUNTIMES_DIR").map(PathBuf::from),
        }
    }

    fn runner_directory(&self) -> Option<PathBuf> {
        self.runner_root.as_ref().map(|root| {
            self.requested_runner_directory(root)
                .unwrap_or_else(|| root.join(RUNNER_PHP_FALLBACK_VERSION))
        })
    }

    fn requested_runner_directory(&self, root: &Path) -> Option<PathBuf> {
        if !Regex::new(r"^\d+\.\d+(?:\.(?:\d+|[xX]))?$")
            .unwrap()
            .is_match(&self.version)
        {
            return None;
        }
        let directory = root.join(self.runner_minor_version());
        directory.join("bin/php").is_file().then_some(directory)
    }

    fn runner_minor_version(&self) -> String {
        self.version.split('.').take(2).join(".")
    }

    fn verify_runner_version(&self) -> Result<()> {
        let Some(root) = &self.runner_root else {
            return Ok(());
        };
        let directory = self.runner_directory().unwrap();
        ensure!(
            directory.join("bin/php").is_file(),
            "Runner PHP installation is missing: {}",
            directory.display()
        );
        let minor = directory.file_name().unwrap().to_string_lossy();
        let installed = self.expected_version()?.unwrap();
        ensure!(
            installed.starts_with(&format!("{minor}.")),
            "Runner PHP {} does not match selected minor {}",
            installed,
            minor
        );
        if self.requested_runner_directory(root).is_none() {
            warn!(
                "Requested PHP {:?} is not available in this runner image; using PHP {}",
                self.version, installed
            );
        }
        Ok(())
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
        sha.update(PHP_PACKAGE_CACHE_BUSTER.as_bytes());

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

impl PhpPackage {
    fn composer(&self) -> Composer {
        Composer {
            cmd: self.cmd.clone(),
            runtime: self
                .runtime
                .runner_root
                .as_ref()
                .map(|_| self.runtime.clone()),
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
                runner_root: None,
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

    fn baked_php_fixture(root: &std::path::Path, requested: &str) -> Php {
        let directory = root.join("8.3");
        std::fs::create_dir_all(directory.join("bin")).unwrap();
        std::fs::write(directory.join("bin/php"), "fixture").unwrap();
        std::fs::write(directory.join("version"), "8.3.36").unwrap();
        Php {
            version: requested.into(),
            runner_root: Some(root.to_path_buf()),
        }
    }

    fn assert_baked_php_selection(requested: &str) {
        with_php_package(|pkg, temp, _| {
            let root = temp.path().join("runtimes");
            pkg.runtime = baked_php_fixture(&root, requested);
            pkg.runtime.verify_runner_version()?;
            let bin = root.join("8.3/bin");
            assert_eq!(
                split_paths(&pkg.composer().env()?["PATH"]).next().unwrap(),
                bin
            );
            assert!(split_paths(&pkg.env()?["PATH"]).any(|path| path == bin));
            assert_eq!(pkg.runtime.expected_version()?, Some("8.3.36".into()));
            Ok(())
        });
    }

    #[test]
    fn baked_php_selects_a_minor_line() {
        assert_baked_php_selection("8.3");
    }

    #[test]
    fn baked_php_resolves_an_old_patch_to_its_minor_line() {
        assert_baked_php_selection("8.3.7");
    }

    #[test]
    fn baked_php_selects_a_patch_wildcard() {
        assert_baked_php_selection("8.3.x");
    }

    #[test]
    fn baked_php_selects_an_uppercase_patch_wildcard() {
        assert_baked_php_selection("8.3.X");
    }

    #[test]
    fn baked_php_package_cache_matches_for_the_same_minor() {
        with_php_package(|pkg, temp, _| {
            pkg.runtime = baked_php_fixture(temp.path(), "8.3");
            let minor_fingerprint = pkg.fingerprint();
            pkg.runtime.version = "8.3.7".into();
            assert_eq!(pkg.fingerprint(), minor_fingerprint);
            Ok(())
        });
    }

    #[test]
    fn baked_php_package_cache_differs_between_minors() {
        with_php_package(|pkg, temp, _| {
            pkg.runtime = baked_php_fixture(temp.path(), "8.3");
            let fingerprint = pkg.fingerprint();
            pkg.runtime.version = "8.4".into();
            assert_ne!(pkg.fingerprint(), fingerprint);
            Ok(())
        });
    }

    #[test]
    fn baked_php_hashes_without_an_installed_interpreter() {
        let root = tempdir().unwrap();
        let runtime = Php {
            version: "8.3".into(),
            runner_root: Some(root.path().join("missing")),
        };
        assert!(!runtime.fingerprint().is_empty());
    }

    #[test]
    #[tracing_test::traced_test]
    fn baked_php_warns_when_falling_back_to_8_3() {
        let root = tempdir().unwrap();
        let runtime = baked_php_fixture(root.path(), "8.0");
        runtime.verify_runner_version().unwrap();
        assert!(logs_contain("Requested PHP \"8.0\""));
        assert!(logs_contain("using PHP 8.3.36"));
    }

    #[test]
    fn baked_php_falls_back_for_7_4() {
        assert_baked_php_selection("7.4");
    }

    #[test]
    fn baked_php_falls_back_for_8_0() {
        assert_baked_php_selection("8.0");
    }

    #[test]
    fn baked_php_falls_back_for_major_only_requests() {
        assert_baked_php_selection("8");
    }

    #[test]
    fn baked_php_falls_back_for_version_ranges() {
        assert_baked_php_selection("^8.3");
    }

    #[test]
    fn baked_php_falls_back_for_latest() {
        assert_baked_php_selection("latest");
    }

    #[test]
    fn baked_php_falls_back_for_parent_directory_requests() {
        assert_baked_php_selection("../8.3");
    }

    #[test]
    fn baked_php_falls_back_for_system_version_requests() {
        assert_baked_php_selection("system");
    }

    #[test]
    fn baked_php_errors_when_the_fallback_is_missing() {
        let root = tempdir().unwrap();
        let runtime = Php {
            version: "latest".into(),
            runner_root: Some(root.path().to_path_buf()),
        };
        let error = runtime.install(&new_task()).unwrap_err().to_string();
        assert!(error.contains("Runner PHP installation is missing"));
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
                runner_root: None,
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

    #[test]
    fn php_package_cache_key_matches_v0_665_0() {
        let pkg = PhpPackage {
            cmd: default_command_builder(),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            runtime: Php {
                version: "1.0.0".to_string(),
                runner_root: None,
            },
        };
        let mut expected = sha2::Sha256::new();
        update_package_hash(&pkg, &mut expected).unwrap();
        expected.update(b"composer");
        expected.update(b"2.11.0");
        let mut actual = sha2::Sha256::new();
        pkg.update_hash(&mut actual).unwrap();

        assert_eq!(actual.finalize(), expected.finalize());
    }
}
