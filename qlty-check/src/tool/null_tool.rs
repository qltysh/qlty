use super::ToolType;
use crate::{ui::ProgressTask, Tool};
use anyhow::Result;
use qlty_config::config::PluginDef;

#[derive(Debug, Clone)]
pub struct NullTool {
    pub plugin_name: String,
    pub plugin: PluginDef,
}

impl Tool for NullTool {
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
