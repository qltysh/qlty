pub mod autofix;

use crate::ui::format::ErrorsFormatter;
use crate::ui::format::TextFormatter;
use crate::ui::Steps;
use crate::{Arguments, CommandError, CommandSuccess, Trigger};
use anyhow::Result;
use autofix::autofix;
use clap::Args;
use console::{style, Emoji};
use qlty_check::{planner::Planner, CheckFilter, Executor, Processor, Report, Settings};
use qlty_cloud::format::JsonFormatter;
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;
use qlty_types::analysis::v1::Level;
use qlty_types::level_from_str;
use std::path::PathBuf;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static THINKING: Emoji<'_, '_> = Emoji("🤔  ", "");

#[derive(Args, Clone, Debug)]
pub struct Check {
    /// Check all files, not just changed
    #[arg(short, long, conflicts_with = "upstream")]
    pub all: bool,

    /// Apply all auto-fix suggestions
    #[arg(long)]
    pub fix: bool,

    /// Generate AI fixes (requires OpenAI API key)
    #[cfg(feature = "llm")]
    #[arg(long)]
    pub ai: bool,

    /// Allow unsafe fixes
    #[arg(long)]
    pub r#unsafe: bool,

    /// Disable formatter checks
    #[arg(long)]
    pub no_formatters: bool,

    /// Disable progress bar
    #[arg(long)]
    pub no_progress: bool,

    /// Exit successfully regardless of what issues are found
    #[arg(long, conflicts_with = "fail_level")]
    pub no_fail: bool,

    /// Exit successfully regardless of linter errors
    #[arg(long)]
    pub no_error: bool,

    /// Sample results from a number of files for each linter
    #[arg(long, conflicts_with = "all")]
    pub sample: Option<usize>,

    /// Minimum level of issues to show (high, medium, low)
    #[arg(long)]
    pub level: Option<String>,

    /// Maximum number of concurrent jobs
    #[arg(short, long)]
    pub jobs: Option<u32>,

    /// Filter by plugin or check
    #[arg(long)]
    filter: Option<String>,

    #[arg(value_enum, long, hide = true, default_value = "manual")]
    trigger: Trigger,

    /// Print verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Upstream base ref to compare against
    #[arg(long)]
    pub upstream: Option<String>,

    /// Disable caching issues
    #[arg(long)]
    pub no_cache: bool,

    /// Print errors to stderr
    #[arg(long)]
    pub print_errors: bool,

    /// Minimium level of issues to fail on
    #[arg(long, value_enum, default_value = "fmt")]
    fail_level: Level,

    /// JSON output
    #[arg(long, hide = true)]
    json: bool,

    /// Allow individual plugins to be skipped if they fail or crash
    #[arg(hide = true, long, conflicts_with = "fail_level")]
    skip_errored_plugins: bool,

    /// Files to analyze
    pub paths: Vec<PathBuf>,
}

impl Check {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.validate_options()?;

        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let mut steps = Steps::new(self.no_progress, 1);
        steps.start(THINKING, "Planning... ");

        let settings = self.build_settings()?;
        let plan = Planner::new(ExecutionVerb::Check, &settings)?.compute()?;

        steps.start(LOOKING_GLASS, format!("Analyzing{}...", plan.description()));
        eprintln!();

        let executor = Executor::new(&plan);
        let results = executor.install_and_invoke()?;
        let results = autofix(
            &results,
            &settings,
            &workspace,
            &plan.staging_area,
            Some(&mut steps),
        )?;

        let mut processor = Processor::new(&plan, results);
        let report = processor.compute()?;

        self.write_stdout(&report, &settings)?;
        self.write_stderr(&report)?;

        if !self.no_error && !self.skip_errored_plugins && report.has_errors() {
            Err(CommandError::Lint)
        } else {
            Ok(CommandSuccess {
                trigger: Some(self.trigger),
                issues_count: Some(report.counts.total_issues),
                fixed_count: report.fixed.len(),
                fixable_count: report.fixable.len(),
                fail: report.is_failure(),
            })
        }
    }

    fn validate_options(&self) -> Result<(), CommandError> {
        if self.all && !self.paths.is_empty() {
            let message = format!(
                "the argument '{}' cannot be used with specified {}",
                style("--all").yellow(),
                style("[PATHS]").yellow()
            );

            return Err(CommandError::InvalidOptions { message });
        }

        if self.sample.is_some() && !self.paths.is_empty() {
            let message = format!(
                "the argument '{}' cannot be used with specified {}",
                style("--sample").yellow(),
                style("[PATHS]").yellow()
            );

            return Err(CommandError::InvalidOptions { message });
        }

        if self.upstream.is_some() && !self.paths.is_empty() {
            let message = format!(
                "the argument '{}' cannot be used with specified {}",
                style("--upstream").yellow(),
                style("[PATHS]").yellow()
            );

            return Err(CommandError::InvalidOptions { message });
        }

        for path in &self.paths {
            if !path.exists() {
                let message = format!("path '{}' does not exist", path.display());
                return Err(CommandError::InvalidOptions { message });
            }
        }

        Ok(())
    }

    fn build_settings(&self) -> Result<Settings> {
        let mut settings = Settings::default();
        settings.root = Workspace::assert_within_git_directory()?;
        settings.verbose = self.verbose as usize;
        settings.sample = self.sample;
        settings.all = self.all;
        settings.fix = self.fix;
        #[cfg(feature = "llm")]
        {
            settings.ai = self.ai;
        }
        settings.r#unsafe = self.r#unsafe;
        settings.jobs = self.jobs;
        settings.progress = !self.no_progress;
        settings.formatters = !self.no_formatters;
        settings.filters = CheckFilter::from_optional_list(self.filter.clone());
        settings.upstream = self.upstream.clone();
        settings.level = level_from_str(&self.level.clone().unwrap_or("".to_string()));
        settings.fail_level = if self.no_fail {
            None
        } else {
            Some(self.fail_level)
        };
        settings.cache = !self.no_cache;
        settings.paths = self.paths.clone();
        settings.trigger = self.trigger.into();
        settings.skip_errored_plugins = self.skip_errored_plugins;

        Ok(settings)
    }

    fn write_stdout(&self, report: &Report, settings: &Settings) -> Result<()> {
        let formatter = if self.json {
            JsonFormatter::new(report.issues.clone())
        } else {
            TextFormatter::new(report, settings.verbose)
        };

        formatter.write_to(&mut std::io::stdout())
    }

    fn write_stderr(&self, report: &Report) -> Result<()> {
        if self.print_errors {
            let formatter = ErrorsFormatter::new(report);
            formatter.write_to(&mut std::io::stderr())?;
        }

        Ok(())
    }
}
