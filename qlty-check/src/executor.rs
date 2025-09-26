pub mod driver;
mod invocation_result;
mod invocation_script;
pub mod staging_area;

use self::staging_area::{
    load_config_file_from_qlty_dir, load_config_file_from_repository, load_config_file_from_source,
};
use crate::llm::Fixer;
use crate::planner::check_filters::CheckFilters;
use crate::planner::config_files::{ConfigOperationType, ConfigStagingOperation};
use crate::planner::source_extractor::SourceExtractor;
use crate::Tool;
use crate::{
    cache::IssueCache,
    planner::InvocationPlan,
    ui::{Progress, ProgressBar},
};
use crate::{cache::IssuesCacheHit, planner::Plan, results::FormattedFile, Results};
use anyhow::{bail, Context, Result};
use chrono::Utc;
pub use driver::Driver;
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

const MAX_ISSUES: usize = 50_000;
const MAX_ISSUES_PER_FILE: usize = 500;

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
        let install_messages = self.install()?;
        self.run_prepare_scripts()?;
        let mut result = self.invoke()?;

        result.messages.extend(install_messages);

        Ok(result)
    }

    pub fn install(&self) -> Result<Vec<Message>> {
        let mut install_messages = vec![];
        let installation_results =
            Self::install_tools(self.plan.tools(), self.plan.jobs, self.progress.clone());

        for installation_result in installation_results {
            let (name, result) = installation_result;
            if self.plan.settings.skip_errored_plugins {
                if let Err(err) = result {
                    error!("Error installing tool {}: {:?}", name, err);

                    install_messages.push(Message {
                        timestamp: Some(Utc::now().into()),
                        module: "qlty_check::executor".to_string(),
                        ty: "executor.install.error".to_string(),
                        level: MessageLevel::Error.into(),
                        message: format!("Error installing tool {}", name),
                        details: err.to_string(),
                        ..Default::default()
                    });
                }
            } else {
                result?;
            }
        }

        Ok(install_messages)
    }

    pub fn install_tools(
        tools: Vec<(String, Box<dyn Tool>)>,
        jobs: usize,
        progress: Progress,
    ) -> Vec<(String, Result<()>)> {
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
                .map(|(name, tool)| {
                    (
                        name.clone(),
                        Self::install_tool(name, tool, progress.clone()),
                    )
                })
                .collect::<Vec<_>>();
        });

        info!(
            "All {} install tasks ran in {:.2}s",
            tasks_count,
            timer.elapsed().as_secs_f32()
        );

        install_results
    }

    pub fn run_prepare_scripts(&self) -> Result<()> {
        let mut prepare_scripts: HashMap<String, &InvocationPlan> = HashMap::new();

        self.plan
            .invocations
            .iter()
            .filter(|invocation| {
                if self.plan.settings.skip_errored_plugins {
                    invocation.tool.is_installed()
                } else {
                    true
                }
            })
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
        )?;
        let formatted = Self::build_formatted(&invocations);

        let messages = invocations
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
            bail!("{}", Self::format_max_issues_error(&issues));
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

        // Stage workspace entries first
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

        // Execute config staging operations from the plan
        let sub_timer = Instant::now();
        let loaded_config_files = self.execute_config_staging_operations()?;

        debug!(
            "Staged {} config files in {:.2}s",
            loaded_config_files.len(),
            sub_timer.elapsed().as_secs_f32()
        );

        info!(
            "Staged {} workspace entries and {} config files in {:.2}s",
            self.plan.workspace_entry_paths().len(),
            loaded_config_files.len(),
            timer.elapsed().as_secs_f32()
        );

        Ok(loaded_config_files)
    }

    fn execute_config_staging_operations(&self) -> Result<Vec<String>> {
        let mut loaded_config_files = Vec::new();

        for operation in &self.plan.config_staging_operations {
            let maybe_loaded = self
                .execute_single_config_operation(operation)
                .with_context(|| {
                    format!("Failed to execute config staging operation: {operation:?}")
                })?;

            if let Some(loaded_file) = maybe_loaded {
                loaded_config_files.push(loaded_file);
            }
        }

        Ok(loaded_config_files)
    }

    fn execute_single_config_operation(
        &self,
        operation: &ConfigStagingOperation,
    ) -> Result<Option<String>> {
        match operation.operation_type {
            ConfigOperationType::CopyToStagingArea => {
                let loaded_file = load_config_file_from_repository(
                    &operation.source_path,
                    &self.plan.workspace,
                    operation.destination_path.parent().unwrap(),
                )?;

                Ok(if loaded_file.is_empty() {
                    None
                } else {
                    Some(loaded_file)
                })
            }
            ConfigOperationType::CopyToWorkspaceRoot => {
                let loaded_file = load_config_file_from_source(
                    &operation.source_path,
                    operation.destination_path.parent().unwrap(),
                )?;
                Ok(if loaded_file.is_empty() {
                    None
                } else {
                    Some(loaded_file)
                })
            }
            ConfigOperationType::LoadFromQltyDir => {
                let config_file_name = operation.source_path.to_string_lossy().to_string();
                let loaded_file = load_config_file_from_qlty_dir(
                    &config_file_name,
                    &self.plan.workspace,
                    operation.destination_path.parent().unwrap(),
                )?;
                Ok(if loaded_file.is_empty() {
                    None
                } else {
                    Some(loaded_file)
                })
            }
            ConfigOperationType::CopyToToolInstall => {
                let tool_dir = &operation.destination_path.parent().unwrap();
                if !tool_dir.exists() {
                    std::fs::create_dir_all(tool_dir).with_context(|| {
                        format!("Failed to create tool directory {}", tool_dir.display())
                    })?;
                }

                debug!(
                    "Copying {} to {}",
                    operation.source_path.display(),
                    operation.destination_path.display()
                );
                std::fs::copy(&operation.source_path, &operation.destination_path).with_context(
                    || {
                        format!(
                            "Failed to copy config file {} to {}",
                            operation.source_path.display(),
                            operation.destination_path.display()
                        )
                    },
                )?;

                Ok(Some(path_to_string(&operation.destination_path)))
            }
            ConfigOperationType::FetchFile => {
                // Find the fetch configuration for this file
                for invocation in &self.plan.invocations {
                    for fetch in &invocation.plugin.fetch {
                        if fetch.path == operation.source_path.to_string_lossy() {
                            let fetch_relative = Path::new(&fetch.path);
                            let dest_parent = operation
                                .destination_path
                                .parent()
                                .context("Fetch destination missing parent directory")?;

                            let directories = if let Some(relative_parent) = fetch_relative.parent()
                            {
                                let depth = relative_parent.components().count();
                                let mut root_dir = dest_parent.to_path_buf();

                                for _ in 0..depth {
                                    root_dir = root_dir
                                        .parent()
                                        .with_context(|| {
                                            format!(
                                                "Destination path {} lacks ancestors for fetch path {}",
                                                dest_parent.display(),
                                                relative_parent.display()
                                            )
                                        })?
                                        .to_path_buf();
                                }

                                vec![root_dir]
                            } else {
                                vec![dest_parent.to_path_buf()]
                            };

                            fetch.download_file_to(&directories).with_context(|| {
                                format!(
                                    "Failed to fetch file for plugin: {}",
                                    invocation.plugin_name
                                )
                            })?;
                            return Ok(Some(path_to_string(&operation.destination_path)));
                        }
                    }
                }
                bail!(
                    "Fetch configuration not found for path: {:?}",
                    operation.source_path
                );
            }
        }
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
                        return None;
                    }

                    if self.plan.settings.skip_errored_plugins && !plan.tool.is_installed() {
                        warn!(
                            "Skipping invocation for {} because --skip-errored-plugins is set and the tool is not installed",
                            plan.invocation_label()
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

    pub fn build_formatted(invocations: &[InvocationResult]) -> Vec<FormattedFile> {
        let mut results = vec![];

        for invocation in invocations {
            if let Some(formatted) = &invocation.formatted {
                results.extend(formatted.clone());
            }
        }
        results.sort_by(|a, b| a.path.cmp(&b.path));
        results
    }

    pub fn build_issue_results(
        cache_hits: &[IssuesCacheHit],
        invocations: &[InvocationResult],
        skip_errored_plugins: bool,
    ) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        for cache_hit in cache_hits {
            for issue in &cache_hit.issues {
                issues.push(issue.to_owned());

                if issues.len() >= MAX_ISSUES {
                    bail!("{}", Self::format_max_issues_error(&issues));
                }
            }
        }

        let mut errored_plugins = HashSet::new();

        for invocation in invocations {
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
                        bail!("{}", Self::format_max_issues_error(&issues));
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

        Ok(issues)
    }

    fn cleanup_config_files(&self, loaded_config_files: &[String]) -> Result<()> {
        for config_file in loaded_config_files {
            std::fs::remove_file(Path::new(config_file)).ok();
        }

        Ok(())
    }

    fn format_number(n: usize) -> String {
        n.to_string()
            .chars()
            .rev()
            .collect::<Vec<_>>()
            .chunks(3)
            .map(|chunk| chunk.iter().rev().collect::<String>())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(",")
    }

    fn format_max_issues_error(issues: &[Issue]) -> String {
        let mut tool_counts: HashMap<String, usize> = HashMap::new();

        for issue in issues {
            *tool_counts.entry(issue.tool.clone()).or_insert(0) += 1;
        }

        let mut tool_summary: Vec<_> = tool_counts.into_iter().collect();
        tool_summary.sort_by(|a, b| b.1.cmp(&a.1));

        let tool_summary_text = tool_summary
            .iter()
            .map(|(tool, count)| format!("  {tool} ({} issues)", Self::format_number(*count)))
            .join("\n");

        format!(
            "Maximum issue count of {} reached. Execution halted.\n\nIssue count by tool:\n{}\n\nPlease adjust your configuration to reduce the number of issues generated.\nFor more information: https://qlty.sh/d/too-many-issues",
            Self::format_number(MAX_ISSUES),
            tool_summary_text
        )
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
    let mut issue_limit_reached = HashSet::<PathBuf>::new();

    if let Some(file_results) = result.file_results.as_mut() {
        let limit_guard = Arc::new(Mutex::new(&mut issue_limit_reached));
        file_results.par_iter_mut().for_each(|file_result| {
            if file_result.issues.len() >= MAX_ISSUES_PER_FILE {
                warn!(
                    "{} on {:?} produced too many results ({} > {}), dropping all issues from file.",
                    plan.plugin_name,
                    file_result.path,
                    file_result.issues.len(),
                    MAX_ISSUES_PER_FILE
                );
                match limit_guard.lock() {
                    Ok(mut limit) => limit.insert(file_result.path.clone().into()),
                    Err(_) => { debug!("Poison error in thread"); false },
                };
                file_result.issues.truncate(MAX_ISSUES_PER_FILE);
                file_result.issues.shrink_to_fit();
                return;
            }

            let mut issues = file_result.issues.clone();
            for transformer in transformers {
                issues = transformer.transform_batch(issues);
            }
            file_result.issues = issues;
        });
    }

    if plan.driver.cache_results {
        result.cache_issues(&cache)?;
    }

    progress.increment(plan.workspace_entries.len() as u64);
    task.clear();

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
