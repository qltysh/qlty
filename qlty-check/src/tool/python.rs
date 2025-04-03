use super::command_builder::default_command_builder;
use super::command_builder::CommandBuilder;
use super::download::Download;
use super::Tool;
use super::ToolType;
use crate::tool::RuntimeTool;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use anyhow::Result;
use qlty_analysis::join_path_string;
use qlty_config::config::OperatingSystem;
use qlty_config::config::{Cpu, DownloadDef, PluginDef, System};
use std::collections::HashMap;
use std::fmt::Debug;

#[cfg(unix)]
const PYTHON_COMMAND: &str = "python3";
#[cfg(unix)]
const BIN_DIRECTORY: &str = "bin";
#[cfg(windows)]
const PYTHON_COMMAND: &str = "python";
#[cfg(windows)]
const BIN_DIRECTORY: &str = "Scripts";

#[derive(Debug, Clone)]
pub struct Python {
    pub version: String,
}

impl Tool for Python {
    fn name(&self) -> String {
        "python".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn version(&self) -> Option<String> {
        Some(self.version.clone())
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        self.download().update_hash(sha, &self.name())?;
        Ok(())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message(&format!("Installing Python v{}", self.version().unwrap()));
        self.download().install(self)?;
        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        Some(format!("{} --version", PYTHON_COMMAND))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Python {
    fn download(&self) -> Download {
        Download::new(
            &DownloadDef {
                systems: vec![
                    System {
                        url: "https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${version}+20240107-x86_64-apple-darwin-install_only.tar.gz"
                            .to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url: "https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${version}+20240107-aarch64-apple-darwin-install_only.tar.gz"
                            .to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url: "https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${version}+20240107-x86_64-unknown-linux-gnu-install_only.tar.gz"
                            .to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url: "https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${version}+20240107-aarch64-unknown-linux-gnu-install_only.tar.gz"
                            .to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::Linux,
                    },
                    System{
                        url: "https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${version}+20240107-x86_64-pc-windows-msvc-shared-install_only.tar.gz".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Windows,
                    },
                    System{
                        url: "https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${version}+20240107-aarch64-pc-windows-msvc-shared-install_only.tar.gz".to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::Windows,
                    }
                ],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
    }
}

impl RuntimeTool for Python {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(PipVenvPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
            cmd: default_command_builder(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PipVenvPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Python,
    cmd: Box<dyn CommandBuilder>,
}

impl Tool for PipVenvPackage {
    fn name(&self) -> String {
        self.name.clone()
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
        self.plugin.version_command.clone()
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn pre_install(&self, task: &ProgressTask) -> Result<()> {
        self.initialize_venv(task)
    }

    fn package_install(&self, task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        task.set_dim_message(&format!("pip install {}@{}", name, version));

        self.run_command(self.cmd.build(
            PYTHON_COMMAND,
            vec![
                "-m",
                "pip",
                "install",
                "--prefix",
                &self.directory(),
                &format!("{}=={}", name, version),
            ],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        task.set_dim_message(&format!(
            "pip install -r {}",
            self.plugin.package_file.as_deref().unwrap_or_default()
        ));
        self.run_command(self.cmd.build(
            PYTHON_COMMAND,
            vec![
                "-m",
                "pip",
                "install",
                "--prefix",
                &self.directory(),
                "-r",
                self.plugin.package_file.as_deref().unwrap_or_default(),
            ],
        ))
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![join_path_string!(self.directory(), BIN_DIRECTORY)])
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = self.runtime.extra_env_vars()?;
        env.insert("VIRTUAL_ENV".to_string(), self.directory());

        Ok(env)
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}

impl PipVenvPackage {
    fn initialize_venv(&self, task: &ProgressTask) -> Result<()> {
        task.set_dim_message(&format!("python3 -m venv {}", &self.directory()));

        // on Windows we need to force the runtime python to avoid using the existing venv directory python
        // which will fail when trying to reset the venv directory (since the python in question will be in use)
        let python_root = if cfg!(windows) {
            self.runtime.directory()
        } else {
            join_path_string!(self.runtime.directory(), BIN_DIRECTORY)
        };
        self.run_command(self.cmd.build(
            &join_path_string!(python_root, PYTHON_COMMAND),
            vec!["-m", "venv", &self.directory()],
        ))
    }
}

#[cfg(test)]
mod test {
    use super::PipVenvPackage;
    use crate::{
        tool::{
            command_builder::test::{reroute_tools_root, stub_cmd, ENV_LOCK},
            python::{BIN_DIRECTORY, PYTHON_COMMAND},
            test::expanded_base_shell_path,
        },
        ui::ProgressTask,
        Progress, Tool,
    };
    use qlty_analysis::{join_path_string, utils::fs::path_to_native_string};
    use qlty_config::config::PluginDef;
    use std::{
        env::join_paths,
        sync::{Arc, Mutex},
    };
    use tempfile::{tempdir, TempDir};

    fn with_pip_venv_package(
        callback: impl Fn(
            &mut PipVenvPackage,
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
        let mut pkg = PipVenvPackage {
            cmd: stub_cmd(list.clone()),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            runtime: super::Python {
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

    fn python_venv_command(pkg: &PipVenvPackage) -> String {
        let python_path = if cfg!(windows) {
            pkg.runtime.directory()
        } else {
            join_path_string!(pkg.runtime.directory(), BIN_DIRECTORY)
        };
        join_path_string!(python_path, PYTHON_COMMAND)
    }

    #[test]
    fn test_pip_venv_package_install_and_validate() {
        with_pip_venv_package(|pkg, _, list| {
            pkg.install_and_validate(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                vec![
                    vec![&python_venv_command(pkg), "-m", "venv", &pkg.directory()],
                    vec![
                        PYTHON_COMMAND,
                        "-m",
                        "pip",
                        "install",
                        "--prefix",
                        &pkg.directory(),
                        "test==1.0.0"
                    ]
                ]
            );
            Ok(())
        });
    }

    #[test]
    fn test_pip_venv_package_install_and_validate_with_package_file() {
        with_pip_venv_package(|pkg, temp_path, list| {
            let req_file = &temp_path.path().join("requirements.txt");
            std::fs::write(req_file, "other==1.0.0").unwrap();

            pkg.plugin.package_file = Some(req_file.to_str().unwrap().into());
            reroute_tools_root(temp_path, pkg);

            pkg.install_and_validate(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                vec![
                    vec![&python_venv_command(pkg), "-m", "venv", &pkg.directory()],
                    vec![
                        PYTHON_COMMAND,
                        "-m",
                        "pip",
                        "install",
                        "--prefix",
                        &pkg.directory(),
                        "test==1.0.0"
                    ],
                    vec![
                        PYTHON_COMMAND,
                        "-m",
                        "pip",
                        "install",
                        "--prefix",
                        &pkg.directory(),
                        "-r",
                        req_file.to_str().unwrap(),
                    ]
                ]
            );
            Ok(())
        });
    }

    #[test]
    fn test_pip_venv_package_env() {
        with_pip_venv_package(|pkg, _, _| {
            let env = pkg.env().unwrap();
            let mut paths = vec![
                join_path_string!(pkg.directory(), BIN_DIRECTORY),
                join_path_string!(pkg.runtime.directory(), "bin"),
                pkg.runtime.directory(),
            ];
            paths.extend(expanded_base_shell_path());

            assert_eq!(
                env.get("PATH"),
                Some(&path_to_native_string(
                    join_paths(paths).unwrap_or_default()
                ))
            );
            assert_eq!(env.get("VIRTUAL_ENV"), Some(&pkg.directory()));
            Ok(())
        });
    }
}
