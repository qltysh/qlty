use super::Tool;
use super::ToolType;
use crate::tool::download::Download;
use crate::tool::RuntimeTool;
use crate::ui::ProgressBar;
use crate::ui::ProgressTask;
use anyhow::Result;
use qlty_config::config::OperatingSystem;
use qlty_config::config::PluginDef;
use qlty_config::config::{Cpu, DownloadDef, System};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Rust {
    pub version: String,
}

impl Tool for Rust {
    fn name(&self) -> String {
        "rust".to_string()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Runtime
    }

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        self.download().update_hash(sha, &self.name());
        Ok(())
    }

    fn version(&self) -> Option<String> {
        Some(self.version.clone())
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        let version = self.version().unwrap();
        let version = if version.chars().next().unwrap().is_numeric() {
            format!("v{}", version)
        } else {
            version // e.g. "nightly"
        };
        task.set_message(&format!("Installing Rust {}", version));
        self.download().install(self.directory(), self.name())?;
        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        Some("rustc --version".to_string())
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Rust {
    fn download(&self) -> Download {
        // https://forge.rust-lang.org/infra/other-installation-methods.html#standalone-installers
        Download::new(
            &DownloadDef {
                strip_components: 2,
                systems: vec![System {
                    url: "https://static.rust-lang.org/dist/rust-${version}-x86_64-apple-darwin.tar.gz"
                        .to_string(),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: "https://static.rust-lang.org/dist/rust-${version}-aarch64-apple-darwin.tar.gz"
                        .to_string(),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: "https://static.rust-lang.org/dist/rust-${version}-x86_64-unknown-linux-gnu.tar.gz"
                        .to_string(),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: "https://static.rust-lang.org/dist/rust-${version}-aarch64-unknown-linux-gnu.tar.gz"
                        .to_string(),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: "https://static.rust-lang.org/dist/rust-${version}-x86_64-pc-windows-msvc.tar.gz"
                        .to_string(),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Windows,
                },
                System {
                    url: "https://static.rust-lang.org/dist/rust-${version}-aarch64-pc-windows-msvc.tar.gz"
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

impl RuntimeTool for Rust {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(RustPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RustPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Rust,
}

impl Tool for RustPackage {
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
        if self.plugin.version_command.is_some() {
            self.plugin.version_command.clone()
        } else {
            // use rustc version if version command is missing
            // prevents breakage of rustfmt and clippy tests
            self.runtime.version_command()
        }
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn install(&self, _task: &ProgressTask) -> Result<()> {
        // Nothing actually to install, packages are installed by the runtime
        Ok(())
    }

    fn extra_env_paths(&self) -> Vec<String> {
        self.runtime.extra_env_paths()
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}
