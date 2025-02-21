mod driver;
mod invocation_result;
mod invocation_script;
pub mod staging_area;

use self::staging_area::{load_config_file_from_qlty_dir, load_config_file_from_repository};
use crate::llm::Fixer;
use crate::planner::check_filters::CheckFilters;
use crate::planner::config_files::config_globset;
use crate::planner::source_extractor::SourceExtractor;
use crate::Tool;
use crate::{
    cache::IssueCache,
    planner::InvocationPlan,
    ui::{Progress, ProgressBar},
};
use crate::{cache::IssuesCacheHit, planner::Plan, Results};
use anyhow::{bail, Context, Result};
use chrono::Utc;
pub use driver::Driver;
use ignore::{DirEntry, WalkBuilder, WalkState};
pub use invocation_result::{InvocationResult, InvocationStatus};
pub use invocation_script::{compute_invocation_script, plan_target_list};
use itertools::Itertools;
use qlty_analysis::utils::fs::path_to_string;
use qlty_config::config::DriverType;
use qlty_config::issue_transformer::IssueTransformer;
use qlty_types::analysis::v1::{Issue, Message, MessageLevel};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::{debug, error, info, warn};

const MAX_ISSUES: usize = 10_000;
const MAX_ISSUES_PER_FILE: usize = 100;

#[derive(Debug, Clone)]
pub struct Executor {
    plan: Plan,
    progress: Progress,
    total_issues: Arc<AtomicUsize>,
}

impl Executor {
    pub fn new(plan: &Plan) -> Self {
        let progress = Progress::new(plan.settings.progress, plan.progress_increments());
        Self {
            plan: plan.clone(),
            progress,
            total_issues: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn install_and_invoke(&self) -> Result<Results> {
        self.install()?;
        self.run_prepare_scripts()?;
        self.invoke()
    }

    pub fn install(&self) -> Result<()> {
        Self::install_tools(self.plan.tools(), self.plan.jobs, self.progress.clone())
    }

    pub fn install_tools(
        tools: Vec<(String, Box<dyn Tool>)>,
        jobs: usize,
        progress: Progress,
    ) -> Result<()> {
        let timer = Instant::now();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap();
        let tasks_count = tools.len();

        let mut install_results = vec![];

        pool.install(|| {
            install_results = tools
                .into_par_iter()
                .map(|(name, tool)| Self::install_tool(name, tool, progress.clone()))
                .collect::<Vec<_>>();
        });

        for result in install_results {
            result?;
        }

        info!(
            "All {} install tasks complete in {:.2}s",
            tasks_count,
            timer.elapsed().as_secs_f32()
        );
        Ok(())
    }

    pub fn run_prepare_scripts(&self) -> Result<()> {
        let mut prepare_scripts: HashMap<String, &InvocationPlan> = HashMap::new();

        self.plan
            .invocations
            .iter()
            .for_each(|invocation: &InvocationPlan| {
                if invocation.driver.prepare_script.is_some() {
                    // Prevent multiple prepare scripts for the same driver and plugin and
                    // store invocation plan to run the prepare script later
                    prepare_scripts.insert(invocation.invocation_label(), invocation);
                }
            });

        for (key, invocation) in prepare_scripts {
            let task = self.progress.task(&key, "Running prepare script...");
            invocation.driver.run_prepare_script(invocation, &task)?;
            task.clear();
        }

        Ok(())
    }

    pub fn invoke(&self) -> Result<Results> {
        let timer = Instant::now();
        let mut invocations = vec![];
        self.plan.workspace.library()?.create()?;

        let mut transformers: Vec<Box<dyn IssueTransformer>> = vec![Box::new(CheckFilters {
            filters: self.plan.settings.filters.clone(),
        })];

        transformers.push(Box::new(SourceExtractor {
            staging_area: self.plan.staging_area.clone(),
        }));

        if self.plan.settings.ai {
            transformers.push(Box::new(Fixer::new(&self.plan, self.progress.clone())));
        }

        if !self.plan.invocations.is_empty() {
            let loaded_config_files = self.stage_workspace_entries()?;
            invocations = self.run_invocations(&transformers)?;
            self.cleanup_config_files(&loaded_config_files)?;
        } else {
            info!("No invocations to run, skipping all runtimes.");
        }

        self.progress.clear();
        let mut issues = Self::build_issue_results(
            &self.plan.hits,
            &invocations,
            self.plan.settings.skip_errored_plugins,
        );
        let formatted = Self::build_formatted(&invocations);

        let mut messages = invocations
            .iter()
            .flat_map(|invocation| invocation.messages.clone())
            .collect::<Vec<_>>();

        info!(
            "Executed {} invocations in {:.2}s",
            invocations.len(),
            timer.elapsed().as_secs_f32()
        );

        if issues.len() >= MAX_ISSUES {
            issues.truncate(MAX_ISSUES);
            issues.shrink_to_fit();

            messages.push(Message {
                timestamp: Some(Utc::now().into()),
                module: "qlty_check::executor".to_string(),
                ty: "executor.limit.total_issue_count".to_string(),
                level: MessageLevel::Error.into(),
                message: format!(
                    "Maximum issue count of {} reached, skipping any further issues.",
                    MAX_ISSUES
                ),
                ..Default::default()
            });
        }

        Ok(Results::new(messages, invocations, issues, formatted))
    }

    fn install_tool(name: String, tool: Box<dyn Tool>, progress: Progress) -> Result<()> {
        let task = progress.task(&name, "Installing...");
        tool.pre_setup(&task)?;
        tool.setup(&task)?;
        progress.increment(1);
        Ok(())
    }

    fn stage_workspace_entries(&self) -> Result<Vec<String>> {
        let timer = Instant::now();
        let sub_timer = Instant::now();

        let results = self
            .plan
            .workspace_entry_paths()
            .par_iter()
            .map(|path| self.plan.staging_area.stage(path))
            .collect::<Vec<_>>();

        for result in results {
            result?;
        }

        debug!(
            "Staged {} workspace entries in {:.2}s",
            self.plan.workspace_entry_paths().len(),
            sub_timer.elapsed().as_secs_f32()
        );

        let sub_timer = Instant::now();
        let config_file_names = self
            .plan
            .invocations
            .iter()
            .flat_map(|invocation| &invocation.plugin.config_files)
            .map(path_to_string)
            .collect::<std::collections::HashSet<_>>() // Unique
            .into_iter()
            .collect::<Vec<_>>();

        let mut repository_config_files = vec![];
        let walk_builder = self.plan.workspace.walk_builder();

        let mut config_paths = self
            .plan
            .invocations
            .iter()
            .flat_map(|invocation| invocation.plugin.config_files.clone())
            .collect::<Vec<PathBuf>>();

        for invocation in self.plan.invocations.iter() {
            for affects_cache in &invocation.plugin.affects_cache {
                config_paths.push(PathBuf::from(affects_cache))
            }
        }

        let config_globset = config_globset(&config_paths)?;

        for entry in walk_collect_entries_parallel(&walk_builder) {
            let file_name = entry.file_name().to_str().unwrap();
            let path = entry.path().to_str().unwrap();

            if config_globset.is_match(file_name)
                && !path.contains(
                    self.plan
                        .workspace
                        .library()?
                        .configs_dir()
                        .to_str()
                        .unwrap(),
                )
            {
                repository_config_files.push(entry.path().to_owned());
            }
        }

        debug!(
            "Walker found {} config and affects cache files in {:.2}s",
            repository_config_files.len(),
            sub_timer.elapsed().as_secs_f32()
        );

        let sub_timer = Instant::now();

        let mut loaded_config_files = vec![];

        self.check_and_copy_configs_into_tool_install(&mut loaded_config_files)?;
        self.plan_plugins_fetch(&mut loaded_config_files)?;

        for config_file in &repository_config_files {
            if let Err(err) = load_config_file_from_repository(
                config_file,
                &self.plan.workspace,
                &self.plan.staging_area.destination_directory,
            ) {
                error!("Failed to load config file from repository: {:?}", err);
            }
        }

        for config_file in &config_file_names {
            if self.plan.workspace.root != self.plan.staging_area.destination_directory {
                // for formatters
                let loaded_config_file = load_config_file_from_qlty_dir(
                    &PathBuf::from(config_file),
                    &self.plan.workspace,
                    &self.plan.staging_area.destination_directory,
                )?;

                if !loaded_config_file.is_empty() {
                    loaded_config_files.push(loaded_config_file);
                }
            }

            // for linters
            let loaded_config_file = load_config_file_from_qlty_dir(
                &PathBuf::from(config_file),
                &self.plan.workspace,
                &self.plan.workspace.root,
            )?;

            if !loaded_config_file.is_empty() {
                loaded_config_files.push(loaded_config_file);
            }
        }

        debug!(
            "Staged {} config files in {:.2}s",
            repository_config_files.len(),
            sub_timer.elapsed().as_secs_f32()
        );

        info!(
            "Staged {} workspace entries and {} config files in {:.2}s",
            self.plan.workspace_entry_paths().len(),
            repository_config_files.len(),
            timer.elapsed().as_secs_f32()
        );

        Ok(loaded_config_files)
    }

    fn plan_plugins_fetch(&self, loaded_config_files: &mut Vec<String>) -> Result<()> {
        let mut plugin_fetches = HashMap::new();

        for invocation in self.plan.invocations.iter() {
            if !invocation.plugin.fetch.is_empty() {
                plugin_fetches.insert(&invocation.plugin_name, &invocation.plugin.fetch);
            }
        }

        let directories = [
            self.plan.workspace.root.clone(),
            self.plan.staging_area.destination_directory.clone(),
        ];

        for (plugin_name, fetches) in plugin_fetches {
            for fetch in fetches {
                fetch
                    .download_file_to(&directories)
                    .with_context(|| format!("Failed to fetch file for plugin: {}", plugin_name))?;

                loaded_config_files.push(fetch.path.clone());
            }
        }

        Ok(())
    }

    fn check_and_copy_configs_into_tool_install(
        &self,
        loaded_config_files: &mut Vec<String>,
    ) -> Result<()> {
        for invocation in self.plan.invocations.iter() {
            if invocation.driver.copy_configs_into_tool_install {
                for config_file in &invocation.plugin_configs {
                    if let Some(config_file) = Self::copy_configs_into_tool_install(
                        &config_file.path,
                        &PathBuf::from(invocation.tool.directory()),
                    )? {
                        loaded_config_files.push(path_to_string(config_file));
                    }
                }
            }
        }

        Ok(())
    }

    fn copy_configs_into_tool_install(
        file_path: &PathBuf,
        tool_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        // in case tool directory does not exist
        // happens for tools without downloads
        if !tool_dir.exists() {
            std::fs::create_dir_all(tool_dir).with_context(|| {
                format!("Failed to create tool directory {}", tool_dir.display())
            })?;
        }

        // This file_path comes from Walkbuilder, so it should be valid/safe to unwrap
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let target = tool_dir.join(file_name);
        let result = std::fs::copy(file_path, &target);

        debug!("Copying {} to {}", file_path.display(), target.display());
        result.with_context(|| {
            format!(
                "Failed to copy config file {} to {}",
                file_path.display(),
                target.display()
            )
        })?;

        Ok(Some(target))
    }

    fn run_invocation_pools(
        &self,
        invocations: Vec<&InvocationPlan>,
        transformers: &[Box<dyn IssueTransformer>],
    ) -> Vec<PlanResult> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.plan.jobs)
            .build()
            .unwrap();

        pool.install(|| {
            invocations
                .into_par_iter()
                .filter_map(|plan| {
                    if self.total_issues.load(Ordering::SeqCst) > MAX_ISSUES {
                        warn!(
                            "Stopping invocations: Maximum total issue count of {} was reached",
                            MAX_ISSUES
                        );

                        return None;
                    }

                    let plan_result = run_invocation_with_error_capture(
                        plan.clone(),
                        self.plan.issue_cache.clone(),
                        self.progress.clone(),
                        transformers,
                    );

                    if let Ok(invocation_result) = &plan_result.result {
                        self.total_issues.fetch_add(
                            invocation_result.invocation.issues_count as usize,
                            Ordering::SeqCst,
                        );
                    }

                    Some(plan_result)
                })
                .collect::<Vec<_>>()
        })
    }

    fn run_invocations(
        &self,
        transformers: &[Box<dyn IssueTransformer>],
    ) -> Result<Vec<InvocationResult>> {
        self.progress.set_prefix("Checking");

        if self.plan.invocations.is_empty() {
            return Ok(vec![]);
        }

        let (mut linters, mut formatters): (Vec<_>, Vec<_>) = self
            .plan
            .invocations
            .iter()
            .partition(|invocation| invocation.driver.driver_type == DriverType::Linter);

        linters.shuffle(&mut thread_rng());
        formatters.shuffle(&mut thread_rng());

        let timer = Instant::now();
        info!("Running {} invocations...", linters.len());

        let mut plan_results = self.run_invocation_pools(linters, transformers);
        plan_results.extend(self.run_invocation_pools(formatters, transformers));

        debug!(
            "All {} invocation tasks complete in {:.2}s",
            self.plan.invocations.len(),
            timer.elapsed().as_secs_f32()
        );

        let mut err_count = 0;

        for plan_result in &plan_results {
            if let Err(ref err) = plan_result.result {
                error!(
                    "Invocation failed for {}: {:?}",
                    plan_result.plan.invocation_label(),
                    err
                );

                err_count += 1;
            }
        }

        if err_count > 0 {
            bail!("FATAL error occurred running {} invocations ", err_count);
        }

        let invocation_results = plan_results
            .into_iter()
            .map(|plan_result| plan_result.result)
            .collect::<Vec<_>>();

        self.process_invocation_results(invocation_results)
    }

    fn process_invocation_results(
        &self,
        invocation_results: Vec<Result<InvocationResult>>,
    ) -> Result<Vec<InvocationResult>> {
        let mut invocations = vec![];

        for result in invocation_results {
            match result {
                Ok(invocation) => {
                    invocations.push(invocation);
                }
                Err(err) => {
                    bail!("Error running task: {:?}", err);
                }
            }
        }

        Ok(invocations)
    }

    pub fn build_formatted(invocations: &[InvocationResult]) -> Vec<PathBuf> {
        let mut results = vec![];

        for invocation in invocations {
            if let Some(formatted) = &invocation.formatted {
                results.extend(formatted.clone());
            }
        }

        results
    }

    pub fn build_issue_results(
        cache_hits: &[IssuesCacheHit],
        invocations: &[InvocationResult],
        skip_errored_plugins: bool,
    ) -> Vec<Issue> {
        let mut issues = vec![];

        for cache_hit in cache_hits {
            for issue in &cache_hit.issues {
                issues.push(issue.to_owned());

                if issues.len() >= MAX_ISSUES {
                    warn!(
                        "Maximum issue count of {} reached in cache, skipping further issues.",
                        MAX_ISSUES
                    );
                    return issues;
                }
            }
        }

        let mut errored_plugins = HashSet::new();

        'invocation_loop: for invocation in invocations {
            if skip_errored_plugins && invocation.status() != InvocationStatus::Success {
                errored_plugins.insert(invocation.invocation.plugin_name.clone());
            }

            let mut issues_count = 0;
            let invocation_label = invocation.plan.invocation_label();

            for file_result in invocation.file_results.as_ref().unwrap_or(&vec![]) {
                for issue in &file_result.issues {
                    issues.push(issue.to_owned());
                    issues_count += 1;

                    if issues.len() >= MAX_ISSUES {
                        warn!(
                            "{}: Maximum issue count of {} reached in {}, skipping further issues.",
                            invocation.invocation.id, MAX_ISSUES, invocation_label,
                        );
                        break 'invocation_loop;
                    }
                }
            }

            debug!(
                "{}: {} issues found by {}",
                invocation.invocation.id, issues_count, invocation_label,
            );
        }

        if !errored_plugins.is_empty() {
            issues.retain(|issue| !errored_plugins.contains(&issue.tool));
        }

        issues
    }

    fn cleanup_config_files(&self, loaded_config_files: &[String]) -> Result<()> {
        for config_file in loaded_config_files {
            std::fs::remove_file(Path::new(config_file)).ok();
        }

        Ok(())
    }
}

struct PlanResult {
    plan: InvocationPlan,
    result: Result<InvocationResult>,
}

fn run_invocation_with_error_capture(
    plan: InvocationPlan,
    cache: IssueCache,
    progress: Progress,
    transformers: &[Box<dyn IssueTransformer>],
) -> PlanResult {
    let result = run_invocation(plan.clone(), cache, progress, transformers);
    PlanResult { plan, result }
}

fn run_invocation(
    plan: InvocationPlan,
    cache: IssueCache,
    progress: Progress,
    transformers: &[Box<dyn IssueTransformer>],
) -> Result<InvocationResult> {
    let task = progress.task(&plan.plugin_name, &plan.description());
    let mut result = plan.driver.run(&plan, &task)?;
    let issue_limit_reached = Arc::new(Mutex::new(HashSet::<PathBuf>::new()));

    if let Some(file_results) = result.file_results.as_mut() {
        file_results.par_iter_mut().for_each(|file_result| {
            if file_result.issues.len() >= MAX_ISSUES_PER_FILE {
                warn!(
                    "{} on {:?} produced too many results ({} > {}), dropping all issues from file.",
                    plan.plugin_name,
                    file_result.path,
                    file_result.issues.len(),
                    MAX_ISSUES_PER_FILE
                );
                issue_limit_reached.lock().unwrap().insert(PathBuf::from(&file_result.path));
                file_result.issues.truncate(MAX_ISSUES_PER_FILE);
                file_result.issues.shrink_to_fit();
                return;
            }

            file_result.issues = file_result
                .issues
                .par_iter()
                .cloned()
                .filter_map(|mut issue| {
                    for transformer in transformers {
                        if let Some(transformed_issue) = transformer.transform(issue) {
                            issue = transformed_issue;
                        } else {
                            return None;
                        }
                    }
                    Some(issue)
                })
                .collect();
        });
    }

    if plan.driver.cache_results {
        result.cache_issues(&cache)?;
    }

    progress.increment(plan.workspace_entries.len() as u64);
    task.clear();

    let issue_limit_reached = issue_limit_reached.lock().unwrap();
    if !issue_limit_reached.is_empty() {
        result.push_message(
            MessageLevel::Error,
            "invocation.limit.issue_count".to_string(),
            format!(
                "Maximum issue count of {} reached, skipping any further issues in files.",
                MAX_ISSUES_PER_FILE
            ),
            format!(
                "The following files have been skipped due to the issue limit: {}",
                issue_limit_reached.iter().map(path_to_string).join(", ")
            ),
        );
    }

    Ok(result)
}

fn walk_collect_entries_parallel(builder: &WalkBuilder) -> Vec<DirEntry> {
    let dents = Arc::new(Mutex::new(vec![]));

    builder.build_parallel().run(|| {
        let dents = dents.clone();

        Box::new(move |result| {
            if let Ok(dent) = result {
                dents.lock().unwrap().push(dent);
            }
            WalkState::Continue
        })
    });

    let dents = dents.lock().unwrap();
    dents.to_vec()
}
