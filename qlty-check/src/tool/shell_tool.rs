use std::path::PathBuf;

use super::{global_tools_root, ToolType};
use crate::{ui::ProgressTask, Tool};
use anyhow::Result;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::PluginDef;

#[derive(Debug, Clone)]
pub struct ShellTool {
    pub parent_directory: PathBuf,
    pub plugin_name: String,
    pub plugin: PluginDef,
}

impl Default for ShellTool {
    fn default() -> Self {
        Self {
            parent_directory: PathBuf::from(global_tools_root()),
            plugin_name: "ShellTool".to_string(),
            plugin: Default::default(),
        }
    }
}

impl Tool for ShellTool {
    fn parent_directory(&self) -> String {
        path_to_string(self.parent_directory.join(self.name()))
    }

    fn plugin(&self) -> Option<PluginDef> {
        Some(self.plugin.clone())
    }

    fn name(&self) -> String {
        self.plugin_name.clone()
    }

    fn tool_type(&self) -> ToolType {
        ToolType::ShellTool
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

    fn install_and_validate(&self, task: &ProgressTask) -> Result<()> {
        if self.plugin.install_script.is_some() {
            self.internal_pre_install(task)?;
            self.install(task)?;
        }
        Ok(())
    }

    fn install(&self, _task: &ProgressTask) -> Result<()> {
        if let Some(ref script_path) = self.plugin.install_script {
            let script_path =
                std::fs::canonicalize(script_path).unwrap_or_else(|_| PathBuf::from(script_path));
            self.run_command(duct::cmd!("sh", script_path))?;
        }
        Ok(())
    }

    fn is_installed(&self) -> bool {
        if self.plugin.install_script.is_some() {
            self.donefile_path().exists() && self.exists()
        } else {
            true
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Progress;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_setup_runs_install_script() {
        let temp_dir = tempdir().unwrap();
        let script_path = temp_dir.path().join("install.sh");
        std::fs::write(&script_path, "touch marker.txt").unwrap();

        let tool = ShellTool {
            parent_directory: temp_dir.path().join("tools"),
            plugin_name: "test_plugin".to_string(),
            plugin: PluginDef {
                version: Some("1.0.0".to_string()),
                install_script: Some(script_path.to_string_lossy().to_string()),
                ..Default::default()
            },
        };

        let task = Progress::new(false, 1).task("TEST", "installing");
        tool.setup(&task).unwrap();

        assert!(tool.is_installed());
        assert!(PathBuf::from(tool.directory()).join("marker.txt").exists());
    }

    #[test]
    fn test_setup_without_install_script_is_noop() {
        let tool = ShellTool {
            plugin_name: "noop_plugin".to_string(),
            plugin: PluginDef {
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(tool.is_installed());
    }

    #[test]
    fn test_setup_failing_install_script() {
        let temp_dir = tempdir().unwrap();
        let script_path = temp_dir.path().join("bad_install.sh");
        std::fs::write(&script_path, "exit 1").unwrap();

        let tool = ShellTool {
            parent_directory: temp_dir.path().join("tools"),
            plugin_name: "failing_plugin".to_string(),
            plugin: PluginDef {
                version: Some("1.0.0".to_string()),
                install_script: Some(script_path.to_string_lossy().to_string()),
                ..Default::default()
            },
        };

        let task = Progress::new(false, 1).task("TEST", "installing");
        let result = tool.setup(&task);

        assert!(result.is_err());
        assert!(!tool.is_installed());
    }

    #[test]
    fn test_setup_is_idempotent() {
        let temp_dir = tempdir().unwrap();
        let script_path = temp_dir.path().join("install.sh");
        std::fs::write(&script_path, "touch marker.txt").unwrap();

        let tool = ShellTool {
            parent_directory: temp_dir.path().join("tools"),
            plugin_name: "test_plugin".to_string(),
            plugin: PluginDef {
                version: Some("1.0.0".to_string()),
                install_script: Some(script_path.to_string_lossy().to_string()),
                ..Default::default()
            },
        };

        let task = Progress::new(false, 1).task("TEST", "installing");
        tool.setup(&task).unwrap();
        tool.setup(&task).unwrap();

        assert!(tool.is_installed());
    }
}
