use crate::ui::{ApplyMode, TextFormatter};
use crate::{Arguments, CommandError, CommandSuccess, Trigger};
use anyhow::Result;
use clap::Args;
use duct::cmd;
use qlty_check::{planner::Planner, CheckFilters, Executor, Processor, Settings};
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Fmt {
    /// Check all files, not just changed
    #[arg(short, long)]
    pub all: bool,

    /// Disable progress bar
    #[arg(long)]
    pub no_progress: bool,

    /// Exit successfully regardless of linter errors
    #[arg(long)]
    pub no_error: bool,

    /// Sample results from a number of files for each linter
    #[arg(long)]
    pub sample: Option<usize>,

    /// Maximum number of concurrent jobs
    #[arg(long)]
    pub jobs: Option<u32>,

    /// Only run these formatter plugins (comma-separated)
    ///
    /// Examples:
    ///   --filter prettier                     Run only prettier
    ///   --filter prettier,black               Run only prettier and black
    ///
    /// Use with --skip to remove plugins from what --filter selects.
    #[arg(long, verbatim_doc_comment)]
    filter: Option<String>,

    /// Don't run these formatter plugins (comma-separated)
    ///
    /// Examples:
    ///   --skip prettier                       Run every formatter except prettier
    ///   --skip prettier,black                 Run every formatter except prettier and black
    ///
    /// With --filter, --filter picks what runs first and --skip removes from it.
    /// When both match, --skip wins:
    ///   --filter prettier,black --skip black  Run only prettier
    ///
    /// Every plugin named must be enabled, and skipping every plugin is an error.
    #[arg(long, verbatim_doc_comment)]
    skip: Option<String>,

    /// Run as this trigger, so only formatters whose `triggers` include it run
    ///
    /// Each plugin's `triggers` list in qlty.toml names the triggers it runs
    /// on. A plugin with no `triggers` list runs on every trigger. Qlty's git
    /// hooks pass `pre-commit` to `qlty fmt` and `pre-push` to `qlty check`.
    ///
    /// Examples:
    ///   --trigger pre-commit                  Run only formatters that run on pre-commit
    ///
    /// To leave one formatter out of a single run, use --skip instead.
    #[arg(value_enum, long, verbatim_doc_comment, default_value = "manual")]
    trigger: Trigger,

    /// Print verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(long)]
    pub upstream: Option<String>,

    /// Format files in the Git index
    #[arg(long, conflicts_with = "index_file")]
    pub index: bool,

    /// Format files in the specified Git index file
    #[arg(long, conflicts_with = "index")]
    pub index_file: Option<PathBuf>,

    /// Files to analyze
    pub paths: Vec<PathBuf>,

    /// Skip fetching sources before formatting
    #[arg(long)]
    skip_source_fetch: bool,
}

impl Fmt {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.validate_options()?;

        let workspace = Workspace::require_initialized()?;
        workspace.prepare_sources(self.skip_source_fetch)?;

        let settings = self.build_settings()?;
        let plan = Planner::new(ExecutionVerb::Fmt, &settings)?.compute()?;
        let executor = Executor::new(&plan);
        let results = executor.install_and_invoke()?;

        let mut processor = Processor::new(&plan, results);
        let report = processor.compute()?;

        if self.index || self.index_file.is_some() {
            let paths: Vec<PathBuf> = report.formatted.iter().map(|f| f.path.clone()).collect();
            self.git_add(&paths)?;
        }

        let mut formatter =
            TextFormatter::new(&report, &plan.workspace, &settings, false, ApplyMode::None);
        formatter.write_to(&mut std::io::stdout())?;

        if !self.no_error && report.has_errors() {
            Err(CommandError::Lint)
        } else {
            Ok(CommandSuccess {
                trigger: Some(self.trigger),
                issues_count: if self.verbose >= 1 && report.formatted.is_empty() {
                    Some(0)
                } else {
                    None // Skip printing summary
                },
                ..Default::default()
            })
        }
    }

    fn validate_options(&self) -> Result<(), CommandError> {
        for path in &self.paths {
            if !path.exists() {
                let message = format!("path '{}' does not exist", path.display());
                return Err(CommandError::InvalidOptions { message });
            }
        }

        Ok(())
    }

    fn git_add(&self, paths: &[PathBuf]) -> Result<()> {
        let mut args = vec!["add"];

        for path in paths {
            if let Some(path_str) = path.to_str() {
                args.push(path_str);
            }
        }

        if args.len() > 1 {
            cmd("git", &args).run()?;
        }

        Ok(())
    }

    fn build_settings(&self) -> Result<Settings> {
        let mut settings = Settings::default();
        settings.root = Workspace::assert_within_git_directory()?;
        settings.verbose = self.verbose as usize;
        settings.sample = self.sample;
        settings.all = (self.sample.unwrap_or(0) > 0) || self.all;
        settings.jobs = self.jobs;
        settings.progress = !self.no_progress;
        settings.check_filters =
            CheckFilters::from_optional_lists(self.filter.clone(), self.skip.clone());
        settings.upstream = self.upstream.clone();
        settings.index = self.index;
        settings.index_file = self.index_file.clone();
        settings.paths = self.paths.clone();
        settings.trigger = self.trigger.into();

        Ok(settings)
    }
}
