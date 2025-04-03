use super::ActivePlugin;
use super::{config_files::PluginConfigFile, Planner, PluginWorkspaceEntryFinderBuilder, Settings};
use crate::planner::driver::DriverPlanner;
use crate::tool::tool_builder::ToolBuilder;
use crate::{cache::IssueCache, executor::staging_area::StagingArea, tool::Tool};
use anyhow::{Context, Result};
use qlty_analysis::{workspace_entries::TargetMode, WorkspaceEntry};
use qlty_config::{
    config::{DriverDef, DriverType, PluginDef},
    Workspace,
};
use qlty_types::analysis::v1::ExecutionVerb;
use std::sync::Arc;
use tracing::{debug, info, trace};

#[derive(Debug, Clone)]
pub struct PluginPlanner {
    formatters: bool,
    pub target_mode: TargetMode,
    pub workspace_entry_finder_builder: PluginWorkspaceEntryFinderBuilder,
    pub plugin_name: String,
    pub plugin: PluginDef,
    pub verb: ExecutionVerb,
    pub settings: Settings,
    pub workspace: Workspace,
    pub plugin_configs: Vec<PluginConfigFile>,
    pub issue_cache: IssueCache,
    pub workspace_entries: Arc<Vec<WorkspaceEntry>>,
    pub staging_area: StagingArea,
    pub runtime_version: Option<String>,
    pub tool: Box<dyn Tool>,
    pub driver_planners: Vec<DriverPlanner>,
    pub all_prefixes: Vec<String>,
}

impl PluginPlanner {
    pub fn new(
        planner: &Planner,
        active_plugin: ActivePlugin,
        all_prefixes: Vec<String>,
    ) -> Result<Self> {
        let plugin = active_plugin.plugin;
        let plugin_name = &active_plugin.name;

        let runtime_version = match plugin.runtime {
            Some(ref runtime) => planner.config.runtimes.enabled.get(runtime).cloned(),
            None => None,
        };

        let tool = ToolBuilder::new(&planner.config, plugin_name, &plugin)
            .build_tool()
            .context("Failed to build tool")?;

        let workspace_entry_finder_builder = planner
            .workspace_entry_finder_builder
            .clone()
            .context("Workspace entry finder builder is missing")?;

        let target_mode = planner
            .target_mode
            .clone()
            .context("Target mode is missing")?;

        Ok(Self {
            plugin_name: plugin_name.to_owned(),
            plugin,
            verb: planner.verb,
            settings: planner.settings.clone(),
            tool,
            runtime_version,
            workspace: planner.workspace.clone(),
            target_mode,
            workspace_entry_finder_builder,
            formatters: planner.settings.formatters,
            plugin_configs: planner
                .plugin_configs
                .get(plugin_name)
                .unwrap_or(&vec![])
                .to_vec(),
            issue_cache: planner.issue_cache.clone(),
            staging_area: planner.staging_area.clone(),
            workspace_entries: Arc::new(vec![]),
            driver_planners: vec![],
            all_prefixes,
        })
    }

    pub fn compute(&mut self) -> Result<()> {
        self.compute_workspace_entries()?;

        for (driver_name, driver) in self.plugin.drivers.clone() {
            if self.include_driver(driver.driver_type) {
                self.compute_driver_planners(driver, driver_name)?;
            } else {
                debug!(
                    "Skipping driver {:?} of type {:?}",
                    driver, driver.driver_type
                );
            }
        }

        Ok(())
    }

    fn compute_driver_planners(&mut self, driver: DriverDef, driver_name: String) -> Result<()> {
        let mut driver_planner = DriverPlanner::new(driver, driver_name, self);
        driver_planner.compute()?;
        self.driver_planners.push(driver_planner);

        Ok(())
    }

    fn include_driver(&self, driver_type: DriverType) -> bool {
        match self.verb {
            ExecutionVerb::Check => match driver_type {
                DriverType::Formatter => self.formatters,
                DriverType::Linter => true,
                DriverType::Validator => false,
            },
            ExecutionVerb::Fmt => driver_type == DriverType::Formatter,
            ExecutionVerb::Validate => driver_type == DriverType::Validator,
            _ => false,
        }
    }

    fn compute_workspace_entries(&mut self) -> Result<()> {
        let mut workspace_entry_finder = self
            .workspace_entry_finder_builder
            .build(&self.plugin.file_types, self.plugin.prefix.clone())?;

        self.workspace_entries = match self.target_mode {
            TargetMode::Sample(sample) => Arc::new(workspace_entry_finder.sample(sample)?),
            _ => Arc::new(workspace_entry_finder.workspace_entries()?),
        };

        if self.workspace_entries.is_empty() {
            debug!("Found 0 workspace_entries for plugin {}", self.plugin_name);
        } else {
            info!(
                "Found {} workspace_entries for plugin {}",
                self.workspace_entries.len(),
                self.plugin_name
            );
        }

        trace!(
            "WorkspaceEntries for {}: {:?}",
            self.plugin_name,
            self.workspace_entries
        );
        Ok(())
    }
}
