use super::check::autofix::autofix;
use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::Result;
use clap::Args;
use git2::Repository;
use itertools::Itertools;
use pbjson_types::Timestamp;
use qlty_analysis::{
    code::File,
    workspace_entries::{TargetMode, WorkspaceEntryFinderBuilder},
    Report,
};
use qlty_cloud::export::AnalysisExport;
use qlty_config::{QltyConfig, Workspace};
use qlty_types::analysis::v1::{AnalysisResult, ExecutionVerb, Metadata};
use rayon::prelude::*;
use std::{env, path::PathBuf, sync::Arc};
use time::OffsetDateTime;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Args, Debug, Default)]
pub struct Build {
    /// Upstream base ref to compare against
    #[arg(long)]
    pub upstream: Option<String>,

    /// Exit successfully regardless of linter errors
    #[arg(long)]
    pub no_error: bool,

    /// Use the commit timestamp as the build timestamp
    #[arg(long)]
    pub backfill: bool,

    /// Generate AI fixes (requires OpenAI API key)
    #[arg(long)]
    pub ai: bool,

    /// Use the issues cache (defaults to disabled)
    #[arg(long)]
    pub cache: bool,

    #[arg(long)]
    pub no_plugins: bool,

    #[arg(long)]
    pub print: bool,

    #[arg(long)]
    pub skip_errored_plugins: bool,

    #[arg(long)]
    output_path: Option<PathBuf>,
}

impl Build {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let config = workspace.config()?;

        let mut report = Report {
            metadata: self.build_metadata(&config),
            ..Default::default()
        };

        info!("Starting build: {}", report.metadata.build_id);
        info!("Identifying workspace files...");
        let mut workspace_entry_finder_builder = WorkspaceEntryFinderBuilder {
            mode: self.target_mode(),
            config: config.clone(),
            ..Default::default()
        };

        let mut workspace_entry_finder = workspace_entry_finder_builder.build()?;
        let workspace_entry_files = workspace_entry_finder.files()?;

        report.metadata.files_analyzed = Some(workspace_entry_files.len().try_into().unwrap());

        enum ReportFunctions {
            Structure,
            Duplication,
            Metrics,
            Check,
        }
        let results = [
            ReportFunctions::Structure,
            ReportFunctions::Duplication,
            ReportFunctions::Metrics,
            ReportFunctions::Check,
        ]
        .par_iter()
        .map(|op| match op {
            ReportFunctions::Structure => {
                info!(
                    "Checking structure of {} files... ",
                    workspace_entry_files.len()
                );
                self.run_structure(&config, &workspace_entry_files)
            }
            ReportFunctions::Duplication => {
                info!("Looking for duplication across all files... ");
                self.run_duplication(&config, &workspace_entry_files)
            }
            ReportFunctions::Metrics => {
                info!(
                    "Computing metrics for {} files...",
                    workspace_entry_files.len()
                );
                self.run_metrics(&config, &workspace_entry_files)
            }
            ReportFunctions::Check => {
                if self.no_plugins {
                    info!("Skipping plugins...");
                    Ok(Report::default())
                } else {
                    info!("Running plugins...");
                    self.run_check()
                }
            }
        })
        .collect::<Vec<Result<Report>>>();
        for result in results {
            report.merge(&result?);
        }

        debug!("Found {} issues (before filters)", report.issues.len());

        // TODO: Extract this into a Transformer + Processor pattern
        debug!("Transforming issues...");
        let diff_line_filter = workspace_entry_finder_builder.diff_line_filter()?;
        report.transform_issues(diff_line_filter);
        report.relativeize_paths(&workspace.root);
        report.attach_metadata();

        debug!("Finishing analysis...");
        report.finish();

        info!("Reporting {} issues...", report.issues.len());
        info!("Reporting {} stats...", report.stats.len());

        if self.print {
            info!("Printing report...");
            report.issues = report
                .issues
                .iter()
                .sorted_by_key(|issue| {
                    (
                        &issue.tool,
                        &issue.driver,
                        &issue.rule_key,
                        issue.path().unwrap(),
                        issue.range().unwrap_or_default().start_line,
                    )
                })
                .cloned()
                .collect();

            report.stats = report
                .stats
                .iter()
                .sorted_by_key(|stat| (&stat.kind, &stat.fully_qualified_name))
                .cloned()
                .collect();

            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        } else {
            let output_path = match self.output_path {
                Some(ref path) => path.clone(),
                None => PathBuf::from(".qlty/builds").join(report.metadata.build_id.clone()),
            };

            AnalysisExport::new(&report, &output_path, false).export()?;
        }

        if !self.no_error && report.metadata.result == AnalysisResult::Error as i32 {
            Err(CommandError::Lint)
        } else {
            CommandSuccess::ok()
        }
    }

    fn run_structure(&self, config: &QltyConfig, files: &[Arc<File>]) -> Result<Report> {
        let planner = qlty_smells::structure::Planner::new(config, files.to_vec())?;
        let plan = planner.compute()?;
        let mut executor = qlty_smells::structure::Executor::new(&plan);
        executor.execute();
        Ok(executor.report())
    }

    fn run_duplication(&self, config: &QltyConfig, files: &[Arc<File>]) -> Result<Report> {
        // No need to run duplication if there are no files to analyze
        if files.is_empty() {
            return Ok(Report::default());
        }

        // Duplication must run against the whole program
        let mut workspace_entry_finder_builder = WorkspaceEntryFinderBuilder {
            mode: TargetMode::All,
            config: config.clone(),
            ..Default::default()
        };

        let settings = qlty_smells::duplication::Settings {
            paths: files.iter().map(|file| file.path.clone()).collect(),
            include_tests: false,
        };
        let planner = qlty_smells::duplication::Planner::new(
            config,
            &settings,
            workspace_entry_finder_builder.build()?.files()?.to_vec(),
        )?;
        let plan = planner.compute()?;
        let mut executor = qlty_smells::duplication::Executor::new(&plan);
        executor.execute();
        Ok(executor.report())
    }

    fn run_metrics(&self, config: &QltyConfig, files: &[Arc<File>]) -> Result<Report> {
        let settings = qlty_smells::metrics::Settings::default();
        let planner = qlty_smells::metrics::Planner::new(config, &settings, files.to_vec());
        let plan = planner.compute()?;
        let mut executor = qlty_smells::metrics::Executor::new(&plan);
        let results = executor.execute();
        let mut processor = qlty_smells::metrics::Processor::new(results);
        processor.compute()
    }

    fn run_check(&self) -> Result<Report> {
        let settings = self.build_check_settings()?;
        let mut planner = qlty_check::Planner::new(ExecutionVerb::Check, &settings)?;
        let plan = planner.compute()?;

        let executor = qlty_check::Executor::new(&plan);

        let results = executor.install_and_invoke()?;
        let results = autofix(&results, &settings, &plan.staging_area, None)?;
        let mut processor = qlty_check::Processor::new(&plan, results.clone());
        let report = processor.compute()?;

        let invocations = results
            .invocations
            .iter()
            .map(|ir| ir.invocation.clone())
            .collect::<Vec<_>>();

        Ok(qlty_analysis::Report {
            invocations,
            messages: results.messages,
            issues: report.issues.clone(),
            metadata: Metadata {
                result: if !self.skip_errored_plugins && report.has_errors() {
                    AnalysisResult::Error.into()
                } else {
                    AnalysisResult::Success.into()
                },
                ..Default::default()
            },
            ..Default::default()
        })
    }

    fn build_check_settings(&self) -> Result<qlty_check::Settings> {
        Ok(qlty_check::Settings {
            root: Workspace::assert_within_git_directory()?,
            all: self.upstream.is_none(),
            progress: false,
            upstream: self.upstream.clone(),
            fail_level: None,
            cache: self.cache,
            ai: self.ai,
            r#unsafe: self.ai, // When AI is enabled, we also enable unsafe fixes
            skip_errored_plugins: self.skip_errored_plugins,
            emit_existing_issues: true,
            ..Default::default()
        })
    }

    fn target_mode(&self) -> TargetMode {
        match self.upstream {
            Some(ref upstream) => TargetMode::UpstreamDiff(upstream.clone()),
            None => TargetMode::All,
        }
    }

    fn build_metadata(&self, config: &QltyConfig) -> Metadata {
        let now = OffsetDateTime::now_utc();
        let branch = env::var("QLTY_BRANCH").unwrap_or_default();
        let pull_request_number = env::var("QLTY_PULL_REQUEST_NUMBER").ok();

        let reference = env::var("QLTY_REFERENCE").unwrap_or_else(|_| {
            if pull_request_number.is_some() && !pull_request_number.as_ref().unwrap().is_empty() {
                format!("refs/pull/{}/head", pull_request_number.as_ref().unwrap())
            } else if !branch.is_empty() {
                format!("refs/heads/{}", branch)
            } else {
                "".to_string()
            }
        });

        let mut metadata = Metadata {
            workspace_id: env::var("QLTY_WORKSPACE_ID").unwrap_or_default(),
            project_id: match config.project_id {
                Some(ref id) => id.clone(),
                None => env::var("QLTY_PROJECT_ID").unwrap_or_default(),
            },
            build_id: env::var("QLTY_BUILD_ID").unwrap_or_else(|_| {
                let uuid = Uuid::new_v4().to_string();
                warn!("QLTY_BUILD_ID is unset, generated: {}", uuid);
                uuid.to_string()
            }),
            start_time: Some(Timestamp {
                seconds: now.unix_timestamp(),
                nanos: now.nanosecond() as i32,
            }),
            reference,
            backfill: self.backfill,
            revision_oid: env::var("QLTY_REVISION_OID").unwrap_or_default(),
            branch,
            repository_clone_url: env::var("QLTY_REPOSITORY_CLONE_URL").unwrap_or_default(),
            pull_request_number,
            tracked_branch_id: env::var("QLTY_TRACKED_BRANCH_ID").ok(),
            result: AnalysisResult::Success.into(),
            ..Default::default()
        };

        self.append_commit_metadata(&mut metadata);
        metadata
    }

    fn append_commit_metadata(&self, metadata: &mut Metadata) {
        match env::current_dir() {
            Ok(root) => match Repository::open(root) {
                Ok(repository) => match repository.head() {
                    Ok(head) => match head.peel_to_commit() {
                        Ok(head_commit) => {
                            let author = head_commit.author();
                            let committer = head_commit.committer();

                            metadata.commit_message =
                                head_commit.message().unwrap_or_default().to_string();

                            metadata.committed_at = Some(Timestamp {
                                seconds: committer.when().seconds(),
                                nanos: 0,
                            });

                            metadata.authored_at = Some(Timestamp {
                                seconds: author.when().seconds(),
                                nanos: 0,
                            });

                            metadata.committer_email =
                                committer.email().unwrap_or_default().to_string();
                            metadata.committer_name =
                                committer.name().unwrap_or_default().to_string();

                            metadata.author_email = author.email().unwrap_or_default().to_string();
                            metadata.author_name = author.name().unwrap_or_default().to_string();
                        }
                        Err(e) => warn!("Failed to get head commit: {}", e),
                    },
                    Err(e) => warn!("Failed to get head commit: {}", e),
                },
                Err(e) => warn!("Failed to open repository: {}", e),
            },
            Err(e) => warn!("Failed to get current directory: {}", e),
        }
    }
}
