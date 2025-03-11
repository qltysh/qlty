use super::{
    config_files::PluginConfigFile,
    invocation_directory::InvocationDirectoryPlanner,
    plugin::PluginPlanner,
    target::{Target, TargetFinder},
    PluginWorkspaceEntryFinderBuilder, Settings,
};
use crate::{
    cache::{IssueCache, IssuesCacheHit, IssuesCacheKey},
    executor::staging_area::StagingArea,
    planner::{target_batcher::TargetBatcher, InvocationPlan},
    utils::generate_random_id,
    Tool,
};
use anyhow::{bail, Result};
use qlty_analysis::{workspace_entries::TargetMode, WorkspaceEntry};
use qlty_config::{
    config::{DriverDef, DriverType, PluginDef, TargetType},
    Workspace,
};
use qlty_types::analysis::v1::ExecutionVerb;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info, trace};

#[derive(Debug, Clone)]
pub struct DriverPlanner {
    driver: DriverDef,
    driver_name: String,
    workspace: Workspace,
    verb: ExecutionVerb,
    settings: Settings,
    plugin_name: String,
    plugin: PluginDef,
    plugin_configs: Vec<PluginConfigFile>,
    issue_cache: IssueCache,
    staging_area: StagingArea,
    runtime_version: Option<String>,
    tool: Box<dyn Tool>,
    workspace_entries: Arc<Vec<WorkspaceEntry>>,
    target_mode: TargetMode,
    workspace_entry_finder_builder: Arc<Mutex<PluginWorkspaceEntryFinderBuilder>>,
    invocation_directory_planner: InvocationDirectoryPlanner,
    pub cache_hits: Vec<IssuesCacheHit>,
    pub invocations: Vec<InvocationPlan>,
    pub targets: Vec<Target>,
    pub all_prefixes: Vec<String>,
}

impl DriverPlanner {
    pub fn new(driver: DriverDef, driver_name: String, plugin_planner: &PluginPlanner) -> Self {
        let invocation_directory_planner = InvocationDirectoryPlanner {
            driver: driver.clone(),
            plugin: plugin_planner.plugin.clone(),
            tool: plugin_planner.tool.clone(),
            target_root: Self::compute_target_root(&driver, plugin_planner),
        };

        Self {
            driver,
            driver_name,
            workspace: plugin_planner.workspace.clone(),
            verb: plugin_planner.verb,
            settings: plugin_planner.settings.clone(),
            plugin_configs: plugin_planner.plugin_configs.clone(),
            issue_cache: plugin_planner.issue_cache.clone(),
            staging_area: plugin_planner.staging_area.clone(),
            runtime_version: plugin_planner.runtime_version.clone(),
            tool: plugin_planner.tool.clone(),
            plugin: plugin_planner.plugin.clone(),
            plugin_name: plugin_planner.plugin_name.clone(),
            workspace_entries: plugin_planner.workspace_entries.clone(),
            target_mode: plugin_planner.target_mode.clone(),
            workspace_entry_finder_builder: plugin_planner.workspace_entry_finder_builder.clone(),
            invocation_directory_planner,
            cache_hits: vec![],
            invocations: vec![],
            targets: vec![],
            all_prefixes: plugin_planner.all_prefixes.clone(),
        }
    }

    pub fn compute(&mut self) -> Result<()> {
        self.compute_driver_workspace_entries()?;
        self.compute_targets()?;

        let (cache_hits, uncached_targets) = self.compute_uncached_targets()?;
        self.cache_hits.extend(cache_hits);

        if uncached_targets.is_empty() {
            debug!("No uncached targets, skipping driver: {:?}", self.driver);
        } else {
            let invocations = self.compute_invocations(uncached_targets)?;
            self.invocations.extend(invocations);
        }

        Ok(())
    }

    fn compute_uncached_targets(&self) -> Result<(Vec<IssuesCacheHit>, Vec<WorkspaceEntry>)> {
        match self.verb {
            ExecutionVerb::Check => {
                if self.settings.cache {
                    self.compute_missed_targets()
                } else {
                    self.compute_all_targets()
                }
            }
            ExecutionVerb::Fmt | ExecutionVerb::Validate => self.compute_all_targets(),
            _ => bail!("Unsupported verb: {:?}", self.verb),
        }
    }

    fn compute_driver_workspace_entries(&mut self) -> Result<()> {
        if let Some(file_types) = &self.driver.file_types {
            let mut workspace_entry_finder = self
                .workspace_entry_finder_builder
                .lock()
                .unwrap()
                .build(file_types)?;

            self.workspace_entries = match self.target_mode {
                TargetMode::Sample(sample) => Arc::new(workspace_entry_finder.sample(sample)?),
                _ => Arc::new(workspace_entry_finder.workspace_entries()?),
            };

            if self.workspace_entries.is_empty() {
                debug!(
                    "Found 0 workspace_entries for plugins driver {}.{}",
                    self.plugin_name, self.driver_name
                );
            } else {
                info!(
                    "Found {} workspace_entries for plugins driver {}.{}",
                    self.workspace_entries.len(),
                    self.plugin_name,
                    self.driver_name
                );
            }

            trace!(
                "WorkspaceEntries for {}, {}: {:?}",
                self.plugin_name,
                self.driver_name,
                self.workspace_entries
            );
        }

        Ok(())
    }

    fn compute_missed_targets(&self) -> Result<(Vec<IssuesCacheHit>, Vec<WorkspaceEntry>)> {
        if self.driver.target.target_type != TargetType::File {
            return Ok((vec![], self.targets.clone()));
        }

        let mut cache_hits = vec![];
        let mut uncached_targets = vec![];

        let cache_key = IssuesCacheKey::new(
            self.tool.clone(),
            Arc::new(self.plugin.clone()),
            self.driver_name.clone(),
            Arc::new(self.plugin_configs.clone()),
            self.plugin.affects_cache.clone(),
        );

        let items = self
            .targets
            .par_iter()
            .map(|target| {
                let mut cache_key = cache_key.clone();
                cache_key.finalize(target);
                (target, cache_key)
            })
            .collect::<Vec<(&WorkspaceEntry, IssuesCacheKey)>>();
        for (target, cache_key) in items.into_iter() {
            match self.issue_cache.read(&cache_key) {
                Ok(Some(cache_hit)) => {
                    cache_hits.push(cache_hit);
                }
                Ok(None) => {
                    uncached_targets.push(target.to_owned());
                }
                Err(err) => {
                    error!("Error reading cache: {:?}", err);
                    uncached_targets.push(target.to_owned());
                }
            }
        }

        Ok((cache_hits, uncached_targets))
    }

    fn compute_all_targets(&self) -> Result<(Vec<IssuesCacheHit>, Vec<WorkspaceEntry>)> {
        Ok((vec![], self.targets.clone()))
    }

    fn compute_invocations(&mut self, targets: Vec<Target>) -> Result<Vec<InvocationPlan>> {
        debug!(
            "Missing {} cached targets, planning invocation: {:?}",
            targets.len(),
            self.driver
        );

        let mut invocations = vec![];

        let target_batcher = TargetBatcher {
            driver: self.driver.clone(),
            driver_name: self.driver_name.clone(),
            invocation_directory_planner: self.invocation_directory_planner.clone(),
            plugin_configs: self.plugin_configs.clone(),
            current_prefix: self.plugin.prefix.clone(),
            all_prefixes: self.all_prefixes.clone(),
            workspace_root: self.workspace.root.clone(),
        };

        for driver_target_batch in target_batcher.compute(&targets)? {
            let plugin_configs = if driver_target_batch.config_file.is_some() {
                vec![driver_target_batch.config_file.unwrap().clone()]
            } else {
                self.plugin_configs.clone()
            };

            let invocation_directory =
                driver_target_batch.invocation_directory.unwrap_or_else(|| {
                    self.invocation_directory_planner
                        .compute(&driver_target_batch.targets[0].clone())
                        .unwrap()
                });

            invocations.push(InvocationPlan {
                invocation_id: generate_random_id(6),
                verb: self.verb,
                settings: self.settings.clone(),
                tool: self.tool.clone(),
                runtime: self.plugin.runtime,
                runtime_version: self.runtime_version.clone(),
                plugin_name: self.plugin_name.to_owned(),
                plugin: self.plugin.to_owned(),
                driver_name: self.driver_name.to_owned(),
                driver: self.driver.clone().into(),
                workspace: self.workspace.clone(),
                workspace_entries: self.workspace_entries.clone(),
                plugin_configs,
                target_root: self.invocation_directory_planner.target_root.clone(),
                invocation_directory,
                targets: driver_target_batch.targets,
                invocation_directory_def: self.driver.invocation_directory_def.clone(),
            });
        }

        Ok(invocations)
    }

    fn compute_targets(&mut self) -> Result<()> {
        let target_finder =
            TargetFinder::new(self.staging_area.clone(), self.driver.target.clone());
        self.targets = target_finder.find(&self.workspace_entries)?;

        Ok(())
    }

    fn compute_target_root(driver: &DriverDef, plugin_planner: &PluginPlanner) -> PathBuf {
        if driver.driver_type == DriverType::Formatter {
            &plugin_planner.staging_area.destination_directory
        } else {
            &plugin_planner.workspace.root
        }
        .join(
            plugin_planner
                .plugin
                .prefix
                .clone()
                .unwrap_or("".to_string()),
        )
    }
}
