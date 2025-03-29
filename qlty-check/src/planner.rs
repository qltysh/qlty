use self::config::enabled_plugins; //, enabled_runtime_template};
use self::level_filter::LevelFilter;
use self::plugin::PluginPlanner;
use self::plugin_mode_transformer::PluginModeTransformer;
use crate::cache::{IssueCache, IssuesCacheHit};
use crate::executor::staging_area::{Mode, StagingArea};
use crate::issue_muter::IssueMuter;
use crate::patch_builder::PatchBuilder;
use crate::planner::config_files::plugin_configs;
use crate::planner::config_files::PluginConfigFile;
use crate::Settings;
use anyhow::{bail, Error, Result};
use check_filters::CheckFilters;
use document_url_generator::DocumentUrlGenerator;
use itertools::Itertools;
use qlty_analysis::cache::{Cache, FilesystemCache, NullCache};
use qlty_analysis::git::{compute_upstream, DiffLineFilter};
use qlty_analysis::workspace_entries::TargetMode;
use qlty_config::config::issue_transformer::IssueTransformer;
use qlty_config::config::{DriverType, PluginDef};
use qlty_config::{QltyConfig, Workspace};
use qlty_types::analysis::v1::ExecutionVerb;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, info};

pub mod check_filters;
mod config;
pub mod config_files;
mod document_url_generator;
mod driver;
mod invocation_directory;
mod invocation_plan;
mod level_filter;
mod plan;
mod plugin;
mod plugin_mode_transformer;
mod plugin_workspace_entry_finder_builder;
pub mod source_extractor;
pub mod target;
mod target_batcher;

pub use config::plugin_supported_on_platform;
pub use invocation_plan::InvocationPlan;
pub use plan::Plan;
pub use plugin_workspace_entry_finder_builder::PluginWorkspaceEntryFinderBuilder;

#[derive(Debug, Clone)]
pub struct ActivePlugin {
    pub name: String,
    pub plugin: PluginDef,
}

#[derive(Debug, Clone)]
pub struct Planner {
    verb: ExecutionVerb,
    settings: Settings,
    workspace: Workspace,
    config: QltyConfig,
    staging_area: StagingArea,
    issue_cache: IssueCache,
    target_mode: Option<TargetMode>,
    workspace_entry_finder_builder: Option<Arc<Mutex<PluginWorkspaceEntryFinderBuilder>>>,
    cache_hits: Vec<IssuesCacheHit>,
    active_plugins: Vec<ActivePlugin>,
    plugin_configs: HashMap<String, Vec<PluginConfigFile>>,
    invocations: Vec<InvocationPlan>,
    transformers: Vec<Box<dyn IssueTransformer>>,
}

impl Planner {
    pub fn new(verb: ExecutionVerb, settings: &Settings) -> Result<Self> {
        let workspace = Workspace::for_root(&settings.root)?;
        let cache = Self::build_cache(&workspace, settings)?;
        let issue_cache = IssueCache::new(cache.clone());

        Ok(Self {
            verb,
            settings: settings.clone(),
            workspace: workspace.clone(),
            config: workspace.config()?,
            staging_area: StagingArea::generate(Mode::Source, workspace.root.clone(), None),
            issue_cache,
            target_mode: None,
            workspace_entry_finder_builder: None,
            cache_hits: vec![],
            active_plugins: vec![],
            plugin_configs: HashMap::new(),
            invocations: vec![],
            transformers: vec![],
        })
    }

    pub fn compute(&mut self) -> Result<Plan, Error> {
        let timer = Instant::now();
        self.compute_workspace_entries_strategy()?;
        self.compute_enabled_plugins()?;
        self.compute_staging_area()?;
        self.compute_invocations()?;
        self.compute_transformers();
        let plan = self.build_plan();
        info!(
            "Planned {} invocations ({} cache hits) in {:.2}s",
            self.invocations.len(),
            self.cache_hits.len(),
            timer.elapsed().as_secs_f32()
        );
        plan
    }

    // default staging area is set to Source for linters
    // unless we encounter a formatter
    fn compute_staging_area(&mut self) -> Result<()> {
        for active_plugin in &self.active_plugins {
            let plugin = &active_plugin.plugin;
            for (_, driver) in &plugin.drivers {
                if driver.driver_type == DriverType::Formatter {
                    self.staging_area = StagingArea::generate(
                        Mode::ReadWrite,
                        self.workspace.root.clone(),
                        Some(self.workspace.library()?.tmp_dir()),
                    );

                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn compute_workspace_entries_strategy(&mut self) -> Result<()> {
        self.target_mode = Some(self.compute_target_mode());

        self.workspace_entry_finder_builder =
            Some(Arc::new(Mutex::new(PluginWorkspaceEntryFinderBuilder {
                mode: self.target_mode.as_ref().unwrap().clone(),
                root: self.settings.root.clone(),
                paths: self.settings.paths.clone(),
                file_types: self.config.file_types.clone(),
                ignores: self.config.ignore.clone(),
                ..Default::default()
            })));

        Ok(())
    }

    fn compute_enabled_plugins(&mut self) -> Result<()> {
        self.active_plugins = enabled_plugins(self)?;
        self.plugin_configs = plugin_configs(self)?;
        Ok(())
    }

    fn compute_invocations(&mut self) -> Result<()> {
        let timer = Instant::now();

        let mut plugin_planners = vec![];
        let mut plugin_prefixes = HashMap::new();

        for active_plugin in &self.active_plugins {
            if let Some(prefix) = &active_plugin.plugin.prefix {
                plugin_prefixes
                    .entry(&active_plugin.name)
                    .or_insert(vec![])
                    .push(prefix.clone());
            }
        }

        for active_plugin in &self.active_plugins {
            match PluginPlanner::new(
                self,
                active_plugin.clone(),
                plugin_prefixes
                    .get(&active_plugin.name)
                    .cloned()
                    .unwrap_or_default(),
            ) {
                Ok(planner) => plugin_planners.push(planner),
                Err(err) => bail!(
                    "Failed to create plugin planner for {}: {}",
                    active_plugin.name,
                    err
                ),
            }
        }

        let results = plugin_planners
            .par_iter_mut()
            .map(|planner| planner.compute())
            .collect::<Vec<_>>();

        for result in results {
            if let Err(err) = result {
                bail!("Failed to compute invocations: {}", err);
            }
        }

        let driver_planners = plugin_planners
            .iter()
            .flat_map(|planner| planner.driver_planners.clone())
            .collect_vec();

        self.cache_hits = driver_planners
            .iter()
            .flat_map(|driver_planner| driver_planner.cache_hits.clone())
            .collect();

        self.invocations = driver_planners
            .iter()
            .flat_map(|driver_planner| driver_planner.invocations.clone())
            .collect();

        debug!(
            "Planned {} enabled plugins in {:.2}s",
            self.active_plugins.len(),
            timer.elapsed().as_secs_f32()
        );
        Ok(())
    }

    fn compute_transformers(&mut self) {
        let workspace_entry_finder_builder = match self.workspace_entry_finder_builder.as_mut() {
            Some(builder) => builder,
            None => {
                debug!("No workspace entry finder builder available for transformers");
                return;
            }
        };

        let result = workspace_entry_finder_builder
            .clone()
            .lock()
            .map_err(|_| {
                debug!("Failed to lock workspace entry finder builder");
            })
            .and_then(|mut builder| {
                builder.diff_line_filter().map_err(|_| {
                    debug!("Failed to get diff line filter");
                })
            });

        if let Ok(diff_line_filter) = result {
            self.transformers.push(diff_line_filter);

            if !self.settings.emit_existing_issues {
                match &self.target_mode.as_ref() {
                    Some(TargetMode::UpstreamDiff(_)) | Some(TargetMode::HeadDiff) => {
                        self.transformers.push(Box::new(DiffLineFilter));
                    }
                    _ => {}
                }
            }
        }

        for ignore in &self.config.ignore {
            self.transformers.push(Box::new(ignore.clone()));
        }

        self.transformers.push(Box::new(CheckFilters {
            filters: self.settings.filters.clone(),
        }));

        self.transformers.push(Box::new(LevelFilter {
            level: self.settings.level,
        }));

        self.transformers.push(Box::new(DocumentUrlGenerator {
            enabled_plugins: self.active_plugins.clone(),
        }));

        self.transformers.push(Box::new(PluginModeTransformer {
            plugins: self.config.plugin.clone(),
        }));

        self.transformers
            .push(Box::new(PatchBuilder::new(self.staging_area.clone())));

        self.transformers
            .push(Box::new(IssueMuter::new(self.staging_area.clone())));

        // keep overrides last
        for issue_override in &self.config.overrides {
            self.transformers.push(Box::new(issue_override.clone()));
        }
    }

    fn build_plan(&mut self) -> Result<Plan> {
        let target_mode = self
            .target_mode
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Target mode not computed"))?
            .clone();

        Ok(Plan {
            verb: self.verb,
            target_mode,
            settings: self.settings.clone(),
            workspace: self.workspace.clone(),
            config: self.config.clone(),
            issue_cache: self.issue_cache.clone(),
            hits: self.cache_hits.clone(),
            invocations: self.invocations.clone(),
            jobs: self.jobs(),
            transformers: self.transformers.clone(),
            staging_area: self.staging_area.clone(),
            fail_level: self.settings.fail_level,
        })
    }

    fn compute_target_mode(&self) -> TargetMode {
        if self.settings.all {
            TargetMode::All
        } else if self.settings.index {
            TargetMode::Index
        } else if self.settings.index_file.is_some() {
            TargetMode::IndexFile(self.settings.index_file.clone().unwrap())
        } else if self.settings.sample.is_some() {
            TargetMode::Sample(self.settings.sample.unwrap())
        } else if !self.settings.paths.is_empty() {
            TargetMode::Paths(self.settings.paths.len())
        } else if let Some(upstream) = compute_upstream(&self.workspace, &self.settings.upstream) {
            TargetMode::UpstreamDiff(upstream)
        } else {
            TargetMode::HeadDiff
        }
    }

    fn build_cache(workspace: &Workspace, settings: &Settings) -> Result<Box<dyn Cache>> {
        if settings.cache {
            let library = workspace.library()?;
            library.create()?;

            Ok(Box::new(FilesystemCache::new(
                library.results_dir().join("issues"),
                "protos",
            )))
        } else {
            Ok(Box::new(NullCache::new()))
        }
    }

    fn jobs(&self) -> usize {
        Self::jobs_count(self.settings.jobs)
    }

    pub fn jobs_count(jobs: Option<u32>) -> usize {
        match jobs {
            Some(jobs) => {
                info!("Overriding max threads to {}", jobs);
                jobs as usize
            }
            None => {
                let system_cpus = num_cpus::get();

                info!(
                    "Found {} CPUs, setting max threads to {}",
                    system_cpus, system_cpus
                );

                system_cpus
            }
        }
    }
}
