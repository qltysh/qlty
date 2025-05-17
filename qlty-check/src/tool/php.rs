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
            .full_env(env.clone())
            .unchecked()
            .stderr_to_stdout()
            .stdout_capture();

        let script = format!("{:?}", cmd);
        debug!(script);

        let mut installation = initialize_installation(self)?;
        let result = cmd.run();
        finalize_installation_from_cmd_result(self, &result, &mut installation, script.clone()).ok();

        if let Err(err) = &result {
            debug!("PHP check failed: {:?}", err);
            // Attempt to auto-install PHP
            if let Ok(()) = self.try_install_php() {
                // Try again after installation
                let result = cmd.run();
                finalize_installation_from_cmd_result(self, &result, &mut installation, script).ok();
                
                let output = result?;
                if !output.status.success() {
                    bail!("PHP was installed but verification failed. Please check your PHP installation manually.");
                }
                return Ok(());
            }
            
            // Provide a helpful error message if auto-installation fails
            let os_hint = self.get_os_install_instructions();
            bail!("PHP is not installed or not in $PATH. {}", os_hint);
        }

        let output = result?;
        if !output.status.success() {
            bail!("PHP installation verification failed. Please ensure PHP is properly installed.");
        }

        Ok(())
    }

    fn try_install_php(&self) -> Result<()> {
        let os_type = std::env::consts::OS;
        debug!("Attempting to auto-install PHP on OS: {}", os_type);

        match os_type {
            "linux" => self.install_php_linux(),
            "macos" => self.install_php_macos(),
            _ => {
                debug!("Auto-installation not supported on this OS: {}", os_type);
                bail!("Auto-installation of PHP is not supported on this operating system")
            }
        }
    }

    fn install_php_linux(&self) -> Result<()> {
        // Detect Linux distribution and package manager
        if self.has_command("apt") {
            debug!("Detected apt-based Linux distribution, installing PHP");
            
            // Check if we're in a Docker container
            let in_docker = std::path::Path::new("/.dockerenv").exists();
            
            if in_docker {
                debug!("Docker environment detected, installing additional dependencies");
                // Install software-properties-common for add-apt-repository
                let deps_cmd = cmd!("apt", "install", "-y", "software-properties-common", "ca-certificates", "apt-transport-https")
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture();
                
                let deps_result = deps_cmd.run();
                if deps_result.is_err() {
                    debug!("Failed to install dependencies: {:?}", deps_result);
                    // Continue anyway, as we might still be able to install PHP
                }
            }
            
            // Update package list first to ensure we get the latest package info
            let update_cmd = cmd!("apt", "update")
                .unchecked()
                .stderr_to_stdout()
                .stdout_capture();
            
            let update_result = update_cmd.run();
            if update_result.is_err() {
                debug!("Failed to update package list: {:?}", update_result);
                // Continue anyway, as we might still be able to install PHP
            }

            // Try to add PPA for PHP (for Ubuntu-based systems)
            if self.has_command("add-apt-repository") {
                let ppa_cmd = cmd!("add-apt-repository", "-y", "ppa:ondrej/php")
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture();
                
                let ppa_result = ppa_cmd.run();
                if ppa_result.is_ok() {
                    // Update again after adding repository
                    let _ = cmd!("apt", "update")
                        .unchecked()
                        .stderr_to_stdout()
                        .stdout_capture()
                        .run();
                } else {
                    debug!("Failed to add PHP repository: {:?}", ppa_result);
                    // Continue anyway with system packages
                }
            }

            // Try to install the specified PHP version first
            let version = self.version.clone();
            let php_package = if !version.is_empty() && version != "latest" {
                format!("php{}-cli php{}-xml php{}-curl php{}-mbstring php{}-zip", 
                       version, version, version, version, version)
            } else {
                "php-cli php-xml php-curl php-mbstring php-zip".to_string()
            };
            
            debug!("Installing PHP packages: {}", php_package);
            
            // Install PHP
            let cmd_args = format!("apt install -y {}", php_package);
            let cmd = cmd!("sh", "-c", cmd_args)
                .unchecked()
                .stderr_to_stdout()
                .stdout_capture();
            
            let result = cmd.run();
            if let Err(err) = result {
                debug!("Failed to install specific PHP version: {:?}", err);
                
                // Fall back to generic PHP package
                let fallback_cmd = cmd!("apt", "install", "-y", "php-cli", "php-xml", "php-curl", "php-mbstring", "php-zip")
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture();
                
                let fallback_result = fallback_cmd.run();
                if let Err(err) = fallback_result {
                    debug!("Failed to install PHP with apt (fallback): {:?}", err);
                    bail!("Failed to automatically install PHP. Please install PHP manually.");
                }
            }
            return Ok(());
        } else if self.has_command("yum") {
            debug!("Detected yum-based Linux distribution, installing PHP");
            
            // Try to install the specified PHP version first
            let version = self.version.clone();
            let php_package = if !version.is_empty() && version != "latest" {
                // RHEL/CentOS require EPEL and Remi repositories for newer PHP versions
                // Try to install them if not present
                let _ = cmd!("yum", "install", "-y", "epel-release")
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture()
                    .run();
                
                let _ = cmd!("yum", "install", "-y", "https://rpms.remirepo.net/enterprise/remi-release-8.rpm")
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture()
                    .run();
                
                // Enable the appropriate PHP repository
                let _ = cmd!("yum", "module", "reset", "php")
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture()
                    .run();
                
                let _ = cmd!("yum", "module", "enable", format!("php:{}", version))
                    .unchecked()
                    .stderr_to_stdout()
                    .stdout_capture()
                    .run();
                
                "php php-xml php-json php-mbstring".to_string()
            } else {
                "php php-xml php-json php-mbstring".to_string()
            };
            
            let cmd = cmd!("yum", "install", "-y", php_package)
                .unchecked()
                .stderr_to_stdout()
                .stdout_capture();
            
            let result = cmd.run();
            if let Err(err) = result {
                debug!("Failed to install PHP with yum: {:?}", err);
                bail!("Failed to automatically install PHP. Please install PHP manually.");
            }
            return Ok(());
        }

        bail!("Unsupported Linux distribution. Please install PHP manually.")
    }

    fn install_php_macos(&self) -> Result<()> {
        if self.has_command("brew") {
            debug!("Detected macOS with Homebrew, installing PHP");
            let cmd = cmd!("brew", "install", "php")
                .unchecked()
                .stderr_to_stdout()
                .stdout_capture();
            
            let result = cmd.run();
            if let Err(err) = result {
                debug!("Failed to install PHP with brew: {:?}", err);
                bail!("Failed to automatically install PHP using Homebrew. Please install PHP manually.");
            }
            return Ok(());
        }

        bail!("Homebrew not found. Please install PHP manually or install Homebrew first.")
    }

    fn has_command(&self, command: &str) -> bool {
        let cmd = cmd!("which", command)
            .unchecked()
            .stderr_to_stdout()
            .stdout_capture();
        
        cmd.run().is_ok()
    }

    fn get_os_install_instructions(&self) -> String {
        let os_type = std::env::consts::OS;
        
        match os_type {
            "linux" => {
                if self.has_command("apt") {
                    "Run 'apt install -y php-cli php-xml php-curl php-mbstring php-zip' to install PHP.".to_string()
                } else if self.has_command("yum") {
                    "Run 'yum install -y php-cli php-xml php-json php-mbstring' to install PHP.".to_string()
                } else {
                    "Please install PHP using your distribution's package manager.".to_string()
                }
            },
            "macos" => {
                if self.has_command("brew") {
                    "Run 'brew install php' to install PHP.".to_string()
                } else {
                    "Install Homebrew (https://brew.sh/) then run 'brew install php' to install PHP.".to_string()
                }
            },
            "windows" => {
                "Download and install PHP from https://windows.php.net/download/ and ensure it's in your PATH.".to_string()
            },
            _ => {
                "Please install PHP for your operating system and ensure it's in your PATH.".to_string()
            }
        }
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
        sha.update(self.name().as_bytes());

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

    fn package_install(&self, task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        task.set_dim_message(&format!("Installing {}", name));
        let composer = Composer {
            cmd: default_command_builder(),
        };

        let composer_phar = PathBuf::from(composer.directory()).join("composer.phar");
        let composer_path = composer_phar.to_str().with_context(|| {
            format!(
                "Failed to convert composer path to string: {:?}",
                composer_phar
            )
        })?;

        self.run_command(self.cmd.build(
            "php",
            vec![
                &path_to_native_string(composer_path),
                "require",
                "--no-interaction",
                format!("{}:{}", name, version).as_str(),
            ],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.package_file.is_some() {
            debug!("installing package file");
            let composer = Composer {
                cmd: self.cmd.clone(),
            };
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
}
