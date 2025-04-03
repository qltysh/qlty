use std::path::PathBuf;

use super::{global_tools_root, ToolType};
use crate::{ui::ProgressTask, Tool};
use anyhow::Result;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::PluginDef;

#[derive(Debug, Clone)]
pub struct NullTool {
    pub parent_directory: PathBuf,
    pub plugin_name: String,
    pub plugin: PluginDef,
}

impl Default for NullTool {
    fn default() -> Self {
        Self {
            parent_directory: PathBuf::from(global_tools_root()),
            plugin_name: "NullTool".to_string(),
            plugin: Default::default(),
        }
    }
}

impl Tool for NullTool {
    fn parent_directory(&self) -> String {
        path_to_string(self.parent_directory.join(self.name()))
    }

    fn name(&self) -> String {
        self.plugin_name.clone()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::NullTool
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

    fn package_install(&self, _: &ProgressTask, _: &str, _: &str) -> Result<()> {
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn install_and_validate(&self, _: &ProgressTask) -> Result<()> {
        Ok(())
    }

    fn is_installed(&self) -> bool {
        true
    }
}
