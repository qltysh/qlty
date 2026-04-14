pub mod package_json;
use super::command_builder::default_command_builder;
use super::command_builder::CommandBuilder;
use super::Tool;
use super::ToolType;
use crate::tool::download::Download;
use crate::tool::RuntimeTool;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use anyhow::Result;
use qlty_analysis::join_path_string;
use qlty_config::config::OperatingSystem;
use qlty_config::config::PluginDef;
use qlty_config::config::{Cpu, DownloadDef, System};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

#[cfg(unix)]
const NPM_COMMAND: &str = "npm";
#[cfg(windows)]
const NPM_COMMAND: &str = "npm.cmd";

#[derive(Debug, Clone)]
pub struct NodeJS {
    pub version: String,
}

impl Tool for NodeJS {
    fn name(&self) -> String {
        "node".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        self.download().update_hash(sha, &self.name())?;
        Ok(())
    }

    fn version(&self) -> Option<String> {
        Some(self.version.clone())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message(&format!("Installing NodeJS v{}", self.version().unwrap()));
        self.download().install(self)?;
        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        Some("node --version".to_string())
    }

    fn install_max_retries(&self) -> u32 {
        // We have observed that NodeJS downloads can sometimes fail intermittently
        3
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl NodeJS {
    fn download(&self) -> Download {
        Download::new(
            &DownloadDef {
                systems: vec![System {
                    url: "https://nodejs.org/dist/v${version}/node-v${version}-darwin-x64.tar.gz"
                        .to_string(),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: "https://nodejs.org/dist/v${version}/node-v${version}-darwin-arm64.tar.gz"
                        .to_string(),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: "https://nodejs.org/dist/v${version}/node-v${version}-linux-x64.tar.gz"
                        .to_string(),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: "https://nodejs.org/dist/v${version}/node-v${version}-linux-arm64.tar.gz"
                        .to_string(),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::Linux,
                }
                ,
                System {
                    url: "https://nodejs.org/dist/v${version}/node-v${version}-win-x64.zip"
                        .to_string(),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Windows,
                },
                System {
                    url: "https://nodejs.org/dist/v${version}/node-v${version}-win-arm64.zip"
                        .to_string(),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::Windows,
                }],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
    }
}

impl RuntimeTool for NodeJS {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(NodePackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
            cmd: default_command_builder(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct NodePackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: NodeJS,
    cmd: Box<dyn CommandBuilder>,
}

impl Tool for NodePackage {
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

    fn package_install(&self, _task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        // Create `node_modules` directory as a bandaid for:
        // https://github.com/qltysh/cloud/issues/1588
        let node_modules_path = std::path::PathBuf::from(&self.directory()).join("node_modules");
        std::fs::create_dir_all(node_modules_path)?;

        if self.plugin.system {
            let package = format!("{name}@{version}");
            return self.run_command(self.cmd.build(
                NPM_COMMAND,
                vec![
                    "install",
                    "--no-save",
                    "--package-lock=false",
                    "--force",
                    package.as_str(),
                ],
            ));
        }

        self.run_command(self.cmd.build(
            NPM_COMMAND,
            vec!["install", "--force", &format!("{name}@{version}")],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.system {
            task.set_dim_message(&format!("{NPM_COMMAND} install"));
            return self.run_command(
                self.cmd
                    .build(NPM_COMMAND, vec!["install", "--force", "--no-package-lock"]),
            );
        }

        self.update_package_json(&self.name, &self.plugin.package_file)?;
        task.set_dim_message(
            format!(
                "{} install {}",
                NPM_COMMAND,
                Path::new(&self.plugin.package_file.as_deref().unwrap_or_default())
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            )
            .as_str(),
        );

        self.run_command(
            self.cmd
                .build(NPM_COMMAND, vec!["install", "--force", "--no-package-lock"]),
        )
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        let mut paths = self.runtime.extra_env_paths()?;
        paths.insert(
            0,
            join_path_string!(self.directory(), "node_modules", ".bin"),
        );
        Ok(paths)
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = self.runtime.extra_env_vars()?;
        env.insert(
            "NODE_PATH".to_string(),
            join_path_string!(self.directory(), "node_modules"),
        );

        Ok(env)
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}

#[cfg(test)]
pub mod test {
    use super::NodePackage;
    use crate::{
        tool::{
            command_builder::test::{reroute_tools_root, stub_cmd, ENV_LOCK},
            node::NPM_COMMAND,
        },
        ui::ProgressTask,
        Progress, Tool,
    };
    use assert_json_diff::assert_json_eq;
    use qlty_config::config::{ExtraPackage, PluginDef};
    use serde_json::Value;
    use std::{
        path::Path,
        sync::{Arc, Mutex},
    };
    use tempfile::{tempdir, TempDir};
    use ureq::json;

    pub fn with_node_package(
        callback: impl Fn(
            &mut NodePackage,
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
        let mut pkg = NodePackage {
            cmd: stub_cmd(list.clone()),
            name: "tool".into(),
            plugin: PluginDef {
                package: Some("test".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            runtime: super::NodeJS {
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
    fn node_package_install_no_package_file() {
        with_node_package(|pkg, _, list| {
            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [[NPM_COMMAND, "install", "--force", "test@1.0.0"]]
            );
            Ok(())
        });
    }

    #[test]
    fn node_package_install_with_package_file() {
        with_node_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("package.json");
            std::fs::write(&pkg_file, r#"{"dependencies":{"other":"2.0.0"}}"#)?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            reroute_tools_root(&temp_path, pkg);

            let stage_path = Path::new(&pkg.directory()).join("package.json");
            std::fs::write(
                &stage_path,
                r#"{"dependencies":{"test":"1.0.0", "other":"1.0.0"}}"#,
            )?;

            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    [NPM_COMMAND, "install", "--force", "test@1.0.0"],
                    [NPM_COMMAND, "install", "--force", "--no-package-lock"]
                ]
            );

            let stage_contents = std::fs::read_to_string(stage_path)?;
            let json_contents = serde_json::from_str::<Value>(&stage_contents)?;
            assert_json_eq!(
                json_contents,
                json!({"dependencies":{"other": "1.0.0", "test":"1.0.0"}})
            );
            Ok(())
        });
    }

    #[test]
    fn node_package_install_with_extra_packages() {
        with_node_package(|pkg, temp_path, list| {
            pkg.plugin.extra_packages = vec![
                ExtraPackage {
                    name: "other".to_string(),
                    version: "1.0.0".to_string(),
                },
                ExtraPackage {
                    name: "another".to_string(),
                    version: "1.0.0".to_string(),
                },
            ];
            reroute_tools_root(temp_path, pkg);

            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    [NPM_COMMAND, "install", "--force", "test@1.0.0"],
                    [NPM_COMMAND, "install", "--force", "other@1.0.0"],
                    [NPM_COMMAND, "install", "--force", "another@1.0.0"]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn node_package_install_package_file_overrides_extra_packages() {
        with_node_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("package.json");
            std::fs::write(&pkg_file, r#"{}"#)?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            pkg.plugin.extra_packages = vec![
                ExtraPackage {
                    name: "other".to_string(),
                    version: "1.0.0".to_string(),
                },
                ExtraPackage {
                    name: "another".to_string(),
                    version: "1.0.0".to_string(),
                },
            ];
            reroute_tools_root(&temp_path, pkg);

            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    [NPM_COMMAND, "install", "--force", "test@1.0.0"],
                    [NPM_COMMAND, "install", "--force", "--no-package-lock"]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn node_package_install_with_package_file_with_package_filters() {
        with_node_package(|pkg, temp_path, list| {
            let pkg_file = temp_path.path().join("package.json");
            std::fs::write(
                &pkg_file,
                r#"{"dependencies":{"other":"1.0.0","tool_dep":"1.0.0"}}"#,
            )?;

            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            pkg.plugin.package_filters = vec![pkg.name.clone()];
            reroute_tools_root(&temp_path, pkg);

            let stage_path = Path::new(&pkg.directory()).join("package.json");
            std::fs::write(&stage_path, r#"{"dependencies":{"test":"1.0.0"}}"#)?;

            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                [
                    [NPM_COMMAND, "install", "--force", "test@1.0.0"],
                    [NPM_COMMAND, "install", "--force", "--no-package-lock"]
                ]
            );

            let stage_contents = std::fs::read_to_string(stage_path)?;
            let json_contents = serde_json::from_str::<Value>(&stage_contents)?;
            assert_json_eq!(
                json_contents,
                json!({"dependencies":{"tool_dep":"1.0.0","test":"1.0.0"}})
            );
            Ok(())
        });
    }

    #[test]
    fn node_package_system_mode_installs_in_workspace() {
        with_node_package(|pkg, temp_path, list| {
            let pkg_root = temp_path.path().join("frontend");
            let pkg_file = pkg_root.join("package.json");
            std::fs::create_dir_all(&pkg_root)?;
            std::fs::write(&pkg_file, r#"{"name":"frontend"}"#)?;

            pkg.plugin.system = true;
            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());

            assert_eq!(
                pkg.directory(),
                pkg_root.to_str().unwrap().replace('\\', "/")
            );

            pkg.install(&new_task())?;
            assert_eq!(
                list.lock().unwrap().clone(),
                vec![
                    vec![
                        NPM_COMMAND,
                        "install",
                        "--no-save",
                        "--package-lock=false",
                        "--force",
                        "test@1.0.0"
                    ],
                    vec![NPM_COMMAND, "install", "--force", "--no-package-lock"]
                ]
            );

            Ok(())
        });
    }

    #[test]
    fn node_package_system_mode_fingerprint_changes_with_package_file() {
        with_node_package(|pkg, temp_path, _| {
            let pkg_root = temp_path.path().join("frontend");
            let pkg_file = pkg_root.join("package.json");
            std::fs::create_dir_all(&pkg_root)?;
            std::fs::write(&pkg_file, r#"{"name":"frontend"}"#)?;

            pkg.plugin.system = true;
            pkg.plugin.package_file = Some(pkg_file.to_str().unwrap().to_string());
            reroute_tools_root(&temp_path, pkg);

            let fingerprint_before = pkg.fingerprint();
            let donefile_before = pkg.donefile_path();

            std::fs::write(
                &pkg_file,
                r#"{"name":"frontend","dependencies":{"eslint":"9.0.0"}}"#,
            )?;

            let fingerprint_after = pkg.fingerprint();
            let donefile_after = pkg.donefile_path();

            assert_ne!(fingerprint_before, fingerprint_after);
            assert_ne!(donefile_before, donefile_after);

            Ok(())
        });
    }
}
