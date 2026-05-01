use super::composer::Composer;
use super::PhpPackage;
use crate::tool::ToolType;
use crate::ui::{ProgressBar, ProgressTask};
use crate::Tool;
use anyhow::Result;
use qlty_analysis::join_path_string;
use qlty_config::config::PluginDef;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PhpProjectInstaller {
    package: PhpPackage,
}

impl PhpProjectInstaller {
    pub fn new(package: PhpPackage) -> Self {
        Self { package }
    }
}

impl Tool for PhpProjectInstaller {
    fn name(&self) -> String {
        self.package.name()
    }

    fn directory(&self) -> String {
        self.project_install_directory()
            .unwrap_or_else(|| self.package.cache_directory())
    }

    fn tool_type(&self) -> ToolType {
        ToolType::RuntimePackage
    }

    fn runtime(&self) -> Option<Box<dyn Tool>> {
        self.package.runtime()
    }

    fn version(&self) -> Option<String> {
        self.package.version()
    }

    fn version_command(&self) -> Option<String> {
        self.package.version_command()
    }

    fn version_regex(&self) -> String {
        self.package.version_regex()
    }

    fn install(&self, task: &ProgressTask) -> Result<()> {
        self.package_file_install(task)
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        let composer = Composer {
            cmd: self.package.cmd().clone_box(),
        };
        composer.setup(task)?;

        task.set_dim_message("composer install");
        let phar = composer.phar_path()?;
        self.run_command(self.package.cmd().build(
            "php",
            vec![
                &phar,
                "install",
                "--no-interaction",
                "--ignore-platform-reqs",
            ],
        ))
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        Ok(vec![join_path_string!(self.directory(), "vendor", "bin")])
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }

    fn plugin(&self) -> Option<PluginDef> {
        self.package.plugin()
    }
}
