use super::{NodePackage, NPM_COMMAND};
use crate::tool::ToolType;
use crate::ui::{ProgressBar, ProgressTask};
use crate::Tool;
use anyhow::Result;
use qlty_analysis::join_path_string;
use qlty_config::config::PluginDef;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NodeProjectInstaller {
    package: NodePackage,
}

impl NodeProjectInstaller {
    pub fn new(package: NodePackage) -> Self {
        Self { package }
    }
}

impl Tool for NodeProjectInstaller {
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
        let plugin = self.package.plugin.clone();
        if plugin.package_file.is_some() {
            self.package_file_install(task)?;
        } else {
            if let (Some(package), Some(version)) = (&plugin.package, &plugin.version) {
                self.package_install(task, package, version)?;
            }
            for pkg in &plugin.extra_packages {
                self.package_install(task, &pkg.name, &pkg.version)?;
            }
        }
        Ok(())
    }

    fn package_install(&self, _task: &ProgressTask, name: &str, version: &str) -> Result<()> {
        let package = format!("{name}@{version}");
        self.run_command(self.package.cmd().build(
            NPM_COMMAND,
            vec![
                "install",
                "--no-save",
                "--package-lock=false",
                "--force",
                package.as_str(),
            ],
        ))
    }

    fn package_file_install(&self, task: &ProgressTask) -> Result<()> {
        task.set_dim_message(&format!("{NPM_COMMAND} install"));
        self.run_command(
            self.package
                .cmd()
                .build(NPM_COMMAND, vec!["install", "--force", "--no-package-lock"]),
        )
    }

    fn extra_env_paths(&self) -> Result<Vec<String>> {
        let mut paths = self.package.runtime.extra_env_paths()?;
        paths.insert(
            0,
            join_path_string!(self.directory(), "node_modules", ".bin"),
        );
        Ok(paths)
    }

    fn extra_env_vars(&self) -> Result<HashMap<String, String>> {
        let mut env = self.package.runtime.extra_env_vars()?;
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
        self.package.plugin()
    }
}
