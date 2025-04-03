use std::collections::HashMap;

use super::{
    command_builder::default_command_builder, download::Download, ruby::RubygemsPackage,
    RuntimeTool, Tool, ToolType,
};
use crate::ui::{ProgressBar, ProgressTask};
use anyhow::Result;
use duct::cmd;
use qlty_analysis::join_path_string;
use qlty_config::config::{Cpu, DownloadDef, OperatingSystem, PluginDef, System};

// This version of the Ruby tool performs a from-source installation using ruby-build.
// It is activated when QLTY_FEATURE_RUBY_BINARY_INSTALL is falsey (`/false/i`, `/off/i` or `0`).
#[derive(Debug, Clone)]
pub struct RubySource {
    pub version: String,
}

impl Tool for RubySource {
    fn name(&self) -> String {
        "ruby".to_string()
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
        task.set_message("Installing ruby-build");
        self.download().install(self)?;
        Ok(())
    }

    fn post_install(&self, task: &ProgressTask) -> Result<()> {
        task.set_message(&format!(
            "Installing Ruby v{} with ruby-build",
            self.version().unwrap()
        ));
        self.run_command(cmd!(
            "ruby-build",
            "--verbose",
            self.version.clone(),
            self.directory(),
        ))?;

        Ok(())
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![
            join_path_string!(self.directory(), "bin"),
            "/opt/homebrew/bin".to_string(),
        ])
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = HashMap::new();
        env.insert(
            "LD_LIBRARY_PATH".to_string(),
            join_path_string!(self.directory(), "lib"),
        );
        Ok(env)
    }

    fn version_command(&self) -> Option<String> {
        Some("ruby --version".to_string())
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl RubySource {
    fn download(&self) -> Download {
        Download::new(
            &DownloadDef {
                systems: vec![
                    System {
                        url:
                            "https://github.com/rbenv/ruby-build/archive/refs/tags/v20250215.tar.gz"
                                .to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url:
                            "https://github.com/rbenv/ruby-build/archive/refs/tags/v20250215.tar.gz"
                                .to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::MacOS,
                    },
                    System {
                        url:
                            "https://github.com/rbenv/ruby-build/archive/refs/tags/v20250215.tar.gz"
                                .to_string(),
                        cpu: Cpu::Aarch64,
                        os: OperatingSystem::Linux,
                    },
                    System {
                        url:
                            "https://github.com/rbenv/ruby-build/archive/refs/tags/v20250215.tar.gz"
                                .to_string(),
                        cpu: Cpu::X86_64,
                        os: OperatingSystem::Linux,
                    },
                ],
                ..Default::default()
            },
            &self.name(),
            &self.version,
        )
    }
}

impl RuntimeTool for RubySource {
    fn package_tool(&self, name: &str, plugin: &PluginDef) -> Box<dyn Tool> {
        Box::new(RubygemsPackage {
            name: name.to_owned(),
            plugin: plugin.clone(),
            runtime: self.clone_box(),
            cmd: default_command_builder(),
        })
    }
}
