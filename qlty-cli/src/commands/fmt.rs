use crate::ui::format::TextFormatter;
use crate::{Arguments, CommandError, CommandSuccess, Trigger};
use anyhow::Result;
use clap::Args;
use duct::cmd;
use qlty_check::{planner::Planner, CheckFilter, Executor, Processor, Settings};
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

    /// Filter by plugin or check
    #[arg(long)]
    filter: Option<String>,

    #[arg(value_enum, long, default_value = "manual")]
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
}

impl Fmt {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.validate_options()?;

        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let settings = self.build_settings()?;
        let plan = Planner::new(ExecutionVerb::Fmt, &settings)?.compute()?;
        let executor = Executor::new(&plan);
        let results = executor.install_and_invoke()?;

        let mut processor = Processor::new(&plan, results);
        let report = processor.compute()?;

        if self.index || self.index_file.is_some() {
            self.git_add(&report.formatted)?;
        }

        let formatter = TextFormatter::new(&report, settings.verbose);
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
        settings.filters = CheckFilter::from_optional_list(self.filter.clone());
        settings.upstream = self.upstream.clone();
        settings.index = self.index;
        settings.index_file = self.index_file.clone();
        settings.paths = self.paths.clone();
        settings.trigger = self.trigger.into();

        Ok(settings)
    }
}
