use super::Tool;
use super::ToolType;
use crate::tool::download::Download;
use crate::tool::RuntimeTool;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use anyhow::Result;
use duct::cmd;
use qlty_analysis::join_path_string;
use qlty_config::config::OperatingSystem;
use qlty_config::config::PluginDef;
use qlty_config::config::{Cpu, DownloadDef, System};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Go {
    pub version: String,
}

impl Tool for Go {
    fn name(&self) -> String {
        "go".to_string()
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
        task.set_message(&format!("Installing go v{}", self.version));
        self.download().install(self)?;
        Ok(())
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();
        env.insert("GOROOT".to_string(), self.directory());
        env.insert("GO111MODULE".to_string(), "on".to_string());
        env.insert("CGO_ENABLED".to_string(), "0".to_string());
        env.insert(
            "LD_LIBRARY_PATH".to_string(),
            join_path_string!(self.directory(), "lib"),
        );
        Ok(env)
    }

    fn version_command(&self) -> Option<String> {
        None
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Go {
    fn download(&self) -> Download {
        Download::new(
            &DownloadDef {
                systems: vec![
                    System {
                        url: "https://go.dev/dl/go${version}.darwin-arm64.tar.gz".to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url: "https://go.dev/dl/go${version}.darwin-amd64.tar.gz".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url: "https://go.dev/dl/go${version}.linux-arm64.tar.gz".to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url: "https://go.dev/dl/go${version}.linux-amd64.tar.gz".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url: "https://go.dev/dl/go${version}.windows-amd64.zip".to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Windows,
                    },
                    System {
                        url: "https://go.dev/dl/go${version}.windows-arm64.zip".to_string(),
                        cpu: Cpu::Aarch64,
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

impl RuntimeTool for Go {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(GoPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct GoPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Go,
}

impl Tool for GoPackage {
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

    fn package_install(&self, task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        task.set_message(&format!("go install {}@latest", name));

        // hackery: gofmt is not a package, it is bundled with go
        if name != "gofmt" {
            let formatted_name =
                name.replace("${major_version}", version.split('.').next().unwrap());

            self.run_command(cmd!(
                "go",
                "install",
                format!("{}@v{}", formatted_name, version)
            ))?;
        }

        Ok(())
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = self.runtime.extra_env_vars()?;
        env.insert("GOPATH".to_string(), self.directory());

        Ok(env)
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![
            join_path_string!(self.directory(), "bin"),
            join_path_string!(self.runtime.directory(), "bin"),
        ])
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}
