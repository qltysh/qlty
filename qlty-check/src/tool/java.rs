use super::runnable_archive::RunnableArchive;
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
use sha2::Digest;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Java {
    pub version: String,
}

impl Tool for Java {
    fn name(&self) -> String {
        "java".to_string()
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
        task.set_message(&format!("Installing Java v{}", self.version().unwrap()));
        self.download().install(self)?;
        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        Some("java --version".to_string())
    }

    fn version_regex(&self) -> String {
        r"(\d+\.\d+\.\d+\+\d+)".to_string()
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    // So Java linux and macos releases have different directory structures
    fn extra_env_paths(&self) -> Vec<String> {
        vec![
            join_path_string!(self.directory(), "bin"),
            join_path_string!(self.directory(), "Contents", "Home", "bin"),
        ]
    }
}

impl Java {
    fn download(&self) -> Download {
        let major_version = self.version.split('.').next().unwrap();
        let url_version = self.version.replace('+', "_");

        Download::new(
            &DownloadDef {
                strip_components: 1,
                systems: vec![System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_x64_mac_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_aarch64_mac_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::MacOS,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_x64_linux_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_aarch64_linux_hotspot_{}.tar.gz", major_version, major_version, url_version),
                    cpu: Cpu::Aarch64,
                    os: OperatingSystem::Linux,
                },
                System {
                    url: format!("https://github.com/adoptium/temurin{}-binaries/releases/download/jdk-${{version}}/OpenJDK{}U-jdk_x64_windows_hotspot_{}.zip", major_version, major_version, url_version),
                    cpu: Cpu::X86_64,
                    os: OperatingSystem::Windows,
                }],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
    }
}

impl RuntimeTool for Java {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(JavaPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct JavaPackage {
    pub name: String,
    pub plugin: PluginDef,
    pub runtime: Java,
}

impl Tool for JavaPackage {
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

    fn update_hash(&self, sha: &mut sha2::Sha256) -> Result<()> {
        sha.update(self.name().as_bytes());

        Ok(())
    }

    fn version_command(&self) -> Option<String> {
        self.plugin.version_command.clone()
    }

    fn version_regex(&self) -> String {
        self.plugin.version_regex.clone()
    }

    fn install(&self, _task: &ProgressTask) -> Result<()> {
        self.download().install(self)?;
        Ok(())
    }

    fn extra_env_paths(&self) -> Vec<String> {
        let mut paths = vec![self.directory(), join_path_string!(self.directory(), "bin")];
        paths.extend(self.runtime.extra_env_paths());

        paths
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }
}

impl RunnableArchive for JavaPackage {}
