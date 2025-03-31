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
use std::sync::{Arc, Mutex};
use tracing::{debug, info, trace};

#[derive(Debug, Clone)]
pub struct PluginPlanner {
    formatters: bool,
    pub target_mode: TargetMode,
    pub workspace_entry_finder_builder: Arc<Mutex<PluginWorkspaceEntryFinderBuilder>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::staging_area::Mode;
    use crate::tool::null_tool::NullTool;
    use qlty_analysis::cache::NullCache;
    use std::path::PathBuf;

    #[test]
    fn test_compute_workspace_entries_lock_error() {
        // Create a workspace_entry_finder_builder that will fail when locked
        let broken_mutex = Arc::new(Mutex::new(PluginWorkspaceEntryFinderBuilder::default()));
        // Poison the mutex by forcing a panic in a thread that holds the lock
        let broken_mutex_clone = broken_mutex.clone();
        let handle = std::thread::spawn(move || {
            let _guard = broken_mutex_clone.lock().unwrap();
            panic!("This panic is intentional for testing");
        });
        let _ = handle.join(); // This should be an Err because the thread panicked

        // Now create a PluginPlanner with the poisoned mutex
        let mut planner = PluginPlanner {
            formatters: false,
            target_mode: TargetMode::All,
            workspace_entry_finder_builder: broken_mutex,
            plugin_name: "test".to_string(),
            plugin: PluginDef::default(),
            verb: ExecutionVerb::Check,
            settings: Settings::default(),
            workspace: Workspace {
                root: PathBuf::from("/"),
            },
            plugin_configs: vec![],
            issue_cache: IssueCache::new(Box::new(NullCache::new())),
            workspace_entries: Arc::new(vec![]),
            staging_area: StagingArea::generate(Mode::Source, PathBuf::from("/"), None),
            runtime_version: None,
            tool: Box::new(NullTool {
                plugin_name: "test".to_string(),
                plugin: PluginDef::default(),
            }),
            driver_planners: vec![],
            all_prefixes: vec![],
        };

        // Verify that compute_workspace_entries returns an error instead of panicking
        let result = planner.compute_workspace_entries();
        assert!(result.is_err());
    }
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
            .context("Workspace entry finder builder is missing")?
            .clone();

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
        let mut workspace_entry_finder_builder =
            self.workspace_entry_finder_builder.lock().map_err(|_| {
                anyhow::anyhow!("Failed to acquire lock on workspace_entry_finder_builder")
            })?;
        let prefix = workspace_entry_finder_builder.prefix.clone();
        if let Some(prefix) = &self.plugin.prefix {
            workspace_entry_finder_builder.prefix = Some(prefix.clone());
        };
        let mut workspace_entry_finder =
            workspace_entry_finder_builder.build(&self.plugin.file_types)?;

        self.workspace_entries = match self.target_mode {
            TargetMode::Sample(sample) => Arc::new(workspace_entry_finder.sample(sample)?),
            _ => Arc::new(workspace_entry_finder.workspace_entries()?),
        };

        workspace_entry_finder_builder.prefix = prefix;

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
