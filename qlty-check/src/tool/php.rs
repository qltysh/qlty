pub mod composer;
use super::command_builder::{default_command_builder, CommandBuilder};
use super::download::Download;
use super::runnable_archive::RunnableArchive;
use super::Tool;
use super::ToolType;
use crate::tool::{update_package_hash, RuntimeTool};
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::{Context, Result};
use composer::Composer;
use duct::cmd;
use qlty_analysis::utils::fs::path_to_native_string;
use qlty_config::config::{Cpu, DownloadDef, OperatingSystem, PluginDef, System};
use sha2::Digest;
use std::collections::HashMap;
use std::env::split_paths;
use std::fmt::Debug;
use std::path::PathBuf;
use tracing::{debug, warn};

const PHP_INI: &str = "short_open_tag=Off\n";

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
        if self.is_system() {
            sha.update(self.name().as_bytes());
        } else {
            self.download().update_hash(sha, &self.name())?;
            sha.update(PHP_INI.as_bytes());
        }
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        if self.is_system() {
            task.set_message("Verifying system PHP installation");
            return self
                .run_command(cmd!("php", "--version"))
                .context("Ensure `php` is installed and in PATH to use system PHP");
        }

        task.set_message(&format!("Installing PHP v{}", self.version));
        self.download()
            .install(self)
            .with_context(|| format!("Failed to install managed PHP {}", self.version))?;
        std::fs::write(self.ini_path(), PHP_INI)?;
        std::fs::create_dir_all(self.ini_scan_directory())?;
        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        Some("php --version".to_string())
    }

    fn expected_version(&self) -> Result<Option<String>> {
        Ok((!self.is_system()).then(|| self.version.clone()))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        let host_path = std::env::var("PATH")
            .with_context(|| "PATH environment variable not found for php runtime")?;
        let mut paths = Vec::new();
        if !self.is_system() {
            paths.push(self.directory());
        }
        paths.extend(split_paths(&host_path).map(path_to_native_string));
        Ok(paths)
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        if self.is_system() {
            Ok(["PHPRC", "PHP_INI_SCAN_DIR"]
                .into_iter()
                .filter_map(|key| {
                    std::env::var(key)
                        .ok()
                        .map(|value| (key.to_string(), value))
                })
                .collect())
        } else {
            Ok(HashMap::from([
                ("PHPRC".to_string(), path_to_native_string(self.ini_path())),
                (
                    "PHP_INI_SCAN_DIR".to_string(),
                    path_to_native_string(self.ini_scan_directory()),
                ),
            ]))
        }
    }
}

impl Php {
    pub fn new(version: &str) -> Self {
        if version == "system" || semver::Version::parse(version).is_ok() {
            Self {
                version: version.to_string(),
            }
        } else {
            let message = format!(
                "PHP version {version:?} is not an exact version (for example, 8.3.30); falling back to system PHP from PATH. Set [runtimes.enabled] php = \"system\" to use system PHP explicitly."
            );
            warn!("{message}");
            eprintln!("Warning: {message}");
            Self {
                version: "system".to_string(),
            }
        }
    }

    fn is_system(&self) -> bool {
        self.version == "system"
    }

    fn ini_path(&self) -> PathBuf {
        PathBuf::from(self.directory()).join("php.ini")
    }

    fn ini_scan_directory(&self) -> PathBuf {
        PathBuf::from(self.directory()).join("conf.d")
    }

    fn download(&self) -> Download {
        Download::new(
            &DownloadDef {
                strip_components: 0,
                systems: vec![
                    System {
                        url: "https://dl.static-php.dev/static-php-cli/common/php-${version}-cli-linux-x86_64.tar.gz".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url: "https://dl.static-php.dev/static-php-cli/common/php-${version}-cli-linux-aarch64.tar.gz".to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url: "https://dl.static-php.dev/static-php-cli/common/php-${version}-cli-macos-x86_64.tar.gz".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url: "https://dl.static-php.dev/static-php-cli/common/php-${version}-cli-macos-aarch64.tar.gz".to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url: "https://dl.static-php.dev/static-php-cli/windows/spc-max/php-${version}-cli-win.zip".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Windows,
                    },
                ],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
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
}

impl PhpPackage {
    fn composer(&self) -> Composer {
        Composer {
            cmd: self.cmd.clone(),
            runtime: self.runtime.clone(),
        }
    }
}

impl RunnableArchive for PhpPackage {}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::{
        tool::{
            command_builder::test::{reroute_tools_root, stub_cmd, ENV_LOCK},
            test::expanded_base_shell_path,
        },
        ui::ProgressTask,
        Progress, Tool,
    };
    use qlty_config::config::PluginDef;
    use std::env::join_paths;
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
    fn php_download_uses_configured_version() {
        let php = Php {
            version: "8.4.18".to_string(),
        };

        assert!(php.download().url().unwrap().contains("php-8.4.18-cli-"));
    }

    #[test]
    fn php_fingerprint_differs_by_version() {
        let php83 = Php {
            version: "8.3.30".to_string(),
        };
        let php84 = Php {
            version: "8.4.18".to_string(),
        };

        assert_ne!(php83.fingerprint(), php84.fingerprint());
    }

    #[test]
    fn php_version_command() {
        let php = Php {
            version: "8.4.18".to_string(),
        };

        assert_eq!(php.version_command(), Some("php --version".to_string()));
    }

    #[test]
    fn php_expected_version() {
        let php = Php {
            version: "8.4.18".to_string(),
        };

        assert_eq!(php.expected_version().unwrap(), Some("8.4.18".to_string()));
    }

    #[test]
    fn php_managed_runtime_preserves_host_path_after_its_own_directory() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| {
            ENV_LOCK.clear_poison();
            err.into_inner()
        });
        let php = Php::new("8.4.18");
        let env = php.env().unwrap();
        let mut paths = vec![PathBuf::from(php.directory())];
        paths.extend(split_paths(&std::env::var_os("PATH").unwrap()));
        paths.extend(expanded_base_shell_path().into_iter().map(PathBuf::from));

        assert_eq!(split_paths(&env["PATH"]).collect::<Vec<_>>(), paths);
        assert_eq!(PathBuf::from(&env["PHPRC"]), php.ini_path());
        assert_eq!(
            PathBuf::from(&env["PHP_INI_SCAN_DIR"]),
            php.ini_scan_directory()
        );
    }

    #[test]
    #[ignore = "downloads a managed PHP runtime"]
    fn php_managed_runtime_install_and_execute() {
        // Downloads normally run after the CLI initializes the TLS provider.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let _lock = ENV_LOCK.lock().unwrap_or_else(|err| {
            ENV_LOCK.clear_poison();
            err.into_inner()
        });
        let temp_path = tempdir().unwrap();
        let php = Php::new("8.4.18");
        reroute_tools_root(&temp_path, &php);

        php.install_and_validate(&new_task()).unwrap();

        assert_eq!(php.installed_version().unwrap(), "8.4.18");
        let binary = cmd!("php", "-r", "echo PHP_BINARY;")
            .full_env(php.env().unwrap())
            .read()
            .unwrap();
        let expected_binary =
            PathBuf::from(php.directory()).join(format!("php{}", std::env::consts::EXE_SUFFIX));
        assert_eq!(
            std::fs::canonicalize(binary).unwrap(),
            std::fs::canonicalize(expected_binary).unwrap()
        );
    }

    #[test]
    fn php_package_env() {
        with_php_package(|pkg, _, _| {
            let env = pkg.env()?;
            let mut paths = vec![pkg.directory(), pkg.runtime.directory()];
            paths.extend(split_paths(&std::env::var("PATH").unwrap()).map(path_to_native_string));
            paths.extend(expanded_base_shell_path());

            assert_eq!(
                env.get("PATH"),
                Some(&path_to_native_string(join_paths(paths).unwrap()))
            );
            assert_eq!(
                env.get("PHPRC"),
                Some(&path_to_native_string(
                    PathBuf::from(pkg.runtime.directory()).join("php.ini")
                ))
            );
            assert_eq!(
                env.get("PHP_INI_SCAN_DIR"),
                Some(&path_to_native_string(
                    PathBuf::from(pkg.runtime.directory()).join("conf.d")
                ))
            );
            Ok(())
        });
    }

    #[test]
    fn php_system_mode_does_not_require_a_specific_version() {
        let php = Php {
            version: "system".to_string(),
        };

        assert_eq!(php.expected_version().unwrap(), None);
    }

    #[test]
    #[tracing_test::traced_test]
    fn php_invalid_versions_warn_and_use_system() {
        for version in [
            "", "8", "7.4", "8.2", "8.3", "8.4", "^8.3", "8.3.x", "latest",
        ] {
            let php = Php::new(version);
            assert!(php.is_system());
            assert_eq!(php.expected_version().unwrap(), None);
            assert_eq!(php.fingerprint(), Php::new("system").fingerprint());
            assert!(logs_contain(&format!(
                "PHP version {version:?} is not an exact version"
            )));
        }
        assert!(logs_contain("falling back to system PHP from PATH"));
        assert!(logs_contain("php = \"system\""));
    }

    #[test]
    fn php_exact_versions_select_managed_php() {
        for version in ["8.3.7", "8.3.30", "8.4.18", "8.3.999"] {
            let php = Php::new(version);
            assert!(!php.is_system());
            assert_eq!(php.expected_version().unwrap().as_deref(), Some(version));
            assert!(php
                .download()
                .url()
                .unwrap()
                .contains(&format!("php-{version}-cli-")));
        }
    }

    #[test]
    fn php_invalid_version_uses_system_environment_and_package_caches() {
        with_php_package(|pkg, _, _| {
            pkg.runtime = Php::new("system");
            let system_env = pkg.env()?;
            let system_directory = pkg.directory();
            let system_composer = pkg.composer().directory();
            pkg.runtime = Php::new("8.3");
            assert_eq!(pkg.env()?, system_env);
            assert_eq!(pkg.directory(), system_directory);
            assert_eq!(pkg.composer().directory(), system_composer);
            Ok(())
        });
    }

    #[test]
    fn php_system_environment_is_preserved_for_runtime_composer_and_packages() {
        with_php_package(|pkg, _, _| {
            pkg.runtime = Php::new("system");
            let keys = ["PHPRC", "PHP_INI_SCAN_DIR"];
            let original = keys.map(std::env::var_os);

            for value in [None, Some(""), Some("/custom/php/config")] {
                for key in keys {
                    if let Some(value) = value {
                        std::env::set_var(key, value);
                    } else {
                        std::env::remove_var(key);
                    }
                }
                let environments = [pkg.runtime.env(), pkg.composer().env(), pkg.env()];
                for (key, previous) in keys.iter().zip(&original) {
                    if let Some(previous) = previous {
                        std::env::set_var(key, previous);
                    } else {
                        std::env::remove_var(key);
                    }
                }

                let host_paths: Vec<_> = split_paths(&std::env::var("PATH")?).collect();
                for env in environments {
                    let env = env?;
                    for key in keys {
                        assert_eq!(env.get(key).map(String::as_str), value);
                    }
                    let paths: Vec<_> = split_paths(&env["PATH"]).collect();
                    assert!(paths
                        .windows(host_paths.len())
                        .any(|paths| paths == host_paths));
                    assert!(!paths.contains(&PathBuf::from(pkg.runtime.directory())));
                }
            }
            Ok(())
        });
    }

    #[test]
    fn php_system_mode_preserves_legacy_cache_keys() {
        with_php_package(|pkg, _, _| {
            pkg.runtime = Php::new("system");
            // Main hashed only "php" for the runtime, "composer" for Composer,
            // and the runtime name plus package name/version for packages.
            for (actual, contents) in [
                (pkg.runtime.fingerprint(), "php"),
                (pkg.composer().fingerprint(), "composer"),
                (pkg.fingerprint(), "phptest1.0.0"),
            ] {
                let expected = format!("{:x}", sha2::Sha256::digest(contents.as_bytes()));
                assert_eq!(actual, expected[..12]);
            }
            let system_package = pkg.fingerprint();
            let system_composer = pkg.composer().fingerprint();
            pkg.runtime = Php::new("8.3.30");
            assert_ne!(pkg.fingerprint(), system_package);
            assert_ne!(pkg.composer().fingerprint(), system_composer);
            Ok(())
        });
    }

    #[test]
    fn php_package_install_and_validate() {
        with_php_package(|pkg, _, list| {
            pkg.install_and_validate(&new_task())?;

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
                        &path_to_native_string(
                            PathBuf::from(pkg.composer().directory()).join("composer.phar")
                        ),
                        "require",
                        "--no-interaction",
                        "test:1.0.0"
                    ]
                ]
            );
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
                runtime: pkg.runtime.clone(),
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
                runtime: pkg.runtime.clone(),
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
                runtime: pkg.runtime.clone(),
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
                runtime: pkg.runtime.clone(),
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
