use crate::ui::ApplyMode;
use crate::ui::ErrorsFormatter;
use crate::ui::Steps;
use crate::ui::TextFormatter;
use crate::{Arguments, CommandError, CommandSuccess, Trigger};
use anyhow::bail;
use anyhow::Result;
use clap::Args;
use console::{style, Emoji};
use qlty_check::planner::Plan;
use qlty_check::{planner::Planner, CheckFilter, Executor, Processor, Report, Settings};
use qlty_cloud::format::JsonFormatter;
use qlty_cloud::load_or_retrieve_auth_token;
use qlty_config::Workspace;
use qlty_types::analysis::v1::ExecutionVerb;
use qlty_types::analysis::v1::Level;
use std::io::BufRead as _;
use std::io::{self, Read};
use std::path::PathBuf;
use std::thread;
use tracing::debug;
use tracing::warn;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static THINKING: Emoji<'_, '_> = Emoji("🤔  ", "");
static FORMATTING: Emoji<'_, '_> = Emoji("📝  ", "");

#[derive(Args, Clone, Debug)]
pub struct Check {
    /// Check all files, not just changed
    #[arg(short, long, conflicts_with = "upstream")]
    pub all: bool,

    /// Apply all auto-fix suggestions
    #[arg(long, conflicts_with = "no_fix")]
    pub fix: bool,

    /// Do not apply auto-fix suggestions
    #[arg(long, conflicts_with = "fix")]
    pub no_fix: bool,

    /// Generate AI fixes (requires OpenAI API key)
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

    /// Minimum level of issues to show
    #[arg(long, value_enum, default_value = "note")]
    pub level: Level,

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

    /// Print a summary of issues
    #[arg(long)]
    pub summary: bool,

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

    #[arg(long, hide = true)]
    upstream_from_pre_push: bool,

    /// Files to analyze
    pub paths: Vec<PathBuf>,
}

impl Check {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        self.validate_options()?;

        let workspace = Workspace::require_initialized()?;
        workspace.fetch_sources()?;

        let settings = self.build_settings(&workspace)?;
        if settings.ai {
            // load the token early so that we can ask user to login first
            load_or_retrieve_auth_token()?;
        }

        let mut counter = 0;
        let mut dirty = true;

        while dirty {
            if counter > 0 {
                eprintln!("{}", style("Changes applied. Re-checking...").bold());
                eprintln!();
            }

            counter += 1;

            let num_steps = if settings.fix { 3 } else { 1 };
            let mut steps = Steps::new(self.no_progress, num_steps);

            if self.verbose >= 1 {
                steps.start(THINKING, "Planning... ");
            }

            let plan = Planner::new(ExecutionVerb::Check, &settings)?.compute()?;

            if self.verbose >= 1 {
                steps.start(LOOKING_GLASS, format!("Analyzing{}...", plan.description()));
                eprintln!();
            }

            if self.trigger == Trigger::PreCommit || self.trigger == Trigger::PrePush {
                self.spawn_exit_on_enter_thread();
            }

            let executor = Executor::new(&plan);
            let results = executor.install_and_invoke()?;

            let mut processor = Processor::new(&plan, results);
            let report = processor.compute()?;

            if !report.fixed.is_empty() {
                if self.verbose >= 1 {
                    steps.start(FORMATTING, "Formatting...");
                }

                self.format_after_fix(&settings, &report)?;
            }

            dirty = self.write_stdout(&report, &plan, &settings)?;
            self.write_stderr(&report)?;

            if !dirty {
                if !self.no_error && !self.skip_errored_plugins && report.has_errors() {
                    return Err(CommandError::Lint);
                } else {
                    return Ok(CommandSuccess {
                        trigger: Some(self.trigger),
                        unformatted_count: if self.no_formatters {
                            None
                        } else {
                            Some(report.unformatted_count())
                        },
                        issues_count: Some(report.counts.total_issues),
                        security_issues_count: Some(report.counts.total_security_issues),
                        fixed_count: report.fixed.len(),
                        fixable_count: report.fixable.len(),
                        fail: report.is_failure(),
                    });
                }
            }
        }

        CommandSuccess::ok()
    }

    fn spawn_exit_on_enter_thread(&self) {
        eprintln!("Tap {} to skip...", style("enter").bold(),);

        thread::spawn(move || loop {
            let mut input = String::new();

            if let Ok(tty) = std::fs::File::open("/dev/tty") {
                let mut tty_reader = io::BufReader::new(tty);
                tty_reader.read_line(&mut input).ok();

                if input == "\n" {
                    std::process::exit(0);
                }
            }
        });
    }

    fn format_after_fix(&self, settings: &Settings, report: &Report) -> Result<Report> {
        debug!("Format after fix: {:?}", report.fixed);
        let mut settings = settings.clone();
        settings.filters = vec![];
        settings.paths = report
            .fixed
            .iter()
            .map(|f| settings.root.join(f.location.path.clone()))
            .collect();
        let plan = Planner::new(ExecutionVerb::Fmt, &settings)?.compute()?;
        let executor = Executor::new(&plan);
        let results = executor.install_and_invoke()?;
        let mut processor = Processor::new(&plan, results);
        processor.compute()
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

    fn build_settings(&self, workspace: &Workspace) -> Result<Settings> {
        let mut settings = Settings::default();
        settings.root = Workspace::assert_within_git_directory()?;
        settings.verbose = self.verbose as usize;
        settings.sample = self.sample;
        settings.all = self.all;
        settings.fix = self.fix;
        settings.ai = self.ai;
        settings.r#unsafe = self.r#unsafe;
        settings.jobs = self.jobs;
        settings.progress = !self.no_progress;
        settings.formatters = !self.no_formatters;
        settings.filters = CheckFilter::from_optional_list(self.filter.clone());
        settings.upstream = self.compute_upstream(&workspace)?;
        settings.level = self.level;
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

    fn compute_upstream(&self, workspace: &Workspace) -> Result<Option<String>> {
        if self.upstream_from_pre_push {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;

            // https://git-scm.com/docs/githooks#_pre_push
            //
            // <local-ref> SP <local-object-name> SP <remote-ref> SP <remote-object-name> LF
            let parts: Vec<&str> = buffer.split_whitespace().collect();
            let remote_commit_id = parts.get(3).unwrap_or(&"");

            if remote_commit_id.is_empty() {
                bail!("Missing remote commit ID from Git pre-push input")
            }

            // When pushing a new branch, the remote object name is 40 zeros.
            // In this case, revert to the upstream branch.
            if remote_commit_id.chars().all(|c| c == '0') {
                Ok(self.upstream.clone())
            } else {
                // Check if the remote commit ID exists in the repository
                let remote_commit_present_locally =
                    workspace.repo()?.revparse_single(remote_commit_id).is_ok();

                // If the remote commit ID is not present locally, revert to the upstream branch.
                if remote_commit_present_locally {
                    Ok(Some(remote_commit_id.to_string()))
                } else {
                    warn!(
                        "Remote commit ID {} is not present locally, reverting to upstream branch",
                        remote_commit_id
                    );
                    Ok(self.upstream.clone())
                }
            }
        } else {
            Ok(self.upstream.clone())
        }
    }

    fn write_stdout(&self, report: &Report, plan: &Plan, settings: &Settings) -> Result<bool> {
        if self.json {
            let formatter = JsonFormatter::new(report.issues.clone());
            formatter.write_to(&mut std::io::stdout())?;
            Ok(false)
        } else {
            let apply_mode = if self.fix {
                ApplyMode::All
            } else if self.no_fix {
                ApplyMode::None
            } else {
                ApplyMode::Ask
            };

            let mut formatter =
                TextFormatter::new(report, &plan.workspace, settings, self.summary, apply_mode);

            formatter.write_to(&mut std::io::stdout())
        }
    }

    fn write_stderr(&self, report: &Report) -> Result<()> {
        if self.print_errors {
            let formatter = ErrorsFormatter::new(report);
            formatter.write_to(&mut std::io::stderr())?;
        }

        Ok(())
    }
}
