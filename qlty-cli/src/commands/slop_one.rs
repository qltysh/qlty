use crate::git_hook::{self, RefUpdate};
use crate::{Arguments, CommandError, CommandSuccess, Trigger};
use anyhow::anyhow;
use chrono::{Months, NaiveDate, Utc};
use clap::{Args, Subcommand};
use qlty_config::Library;
use qlty_slop_one::{
    render_comparisons, render_summary, render_text, Change, ComparisonSummary, Document,
    Evaluator, Framing, JevProvider, Options, PromptMode, Revisions, TextOptions,
};
use qlty_slop_one_trends::periods::local_timezone;
use qlty_slop_one_trends::pipeline::{self, ReportOptions};
use std::collections::HashSet;
use std::env;
use std::num::NonZeroUsize;
use std::path::PathBuf;

mod changes;
mod trends;
mod ui;

use ui::{allow_push, explain_scoring, explain_trends, FileProgress, ReportProgress};

const DEFAULT_TOP: usize = 3;
const DEFAULT_COMPARISON_TOP: usize = 5;
const DEFAULT_SCORING_BUDGET_USD: f64 = 1.0;
const DEFAULT_MAX_DROP: f64 = 1.0;

/// With no files, builds the trends report for the current repository and
/// opens it. With files, scores them and prints the results. With
/// `--upstream` or `--upstream-from-pre-push`, compares changed files with
/// their earlier versions and fails when one declined.
#[derive(Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SlopOne {
    #[command(subcommand)]
    pub command: Option<SlopOneCommand>,

    /// Source files to score; with none, build the trends report for this repository
    pub files: Vec<PathBuf>,

    /// Use existing cached analyses; make no API calls
    #[arg(long)]
    pub offline: bool,

    /// Maximum estimated API spend for this invocation in USD (default: 1 when scoring files, no cap for the report)
    #[arg(long)]
    pub budget: Option<f64>,

    /// Jev provider: typesafe (direct), vercel (AI Gateway), or openrouter
    #[arg(long, value_enum, default_value_t = Provider::Typesafe)]
    pub provider: Provider,

    /// Directory for cached analyses and run data (default: ~/.qlty/cache/slop-one)
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,

    /// Number of files or contents to analyze concurrently
    #[arg(long, default_value_t = NonZeroUsize::new(16).expect("16 is nonzero"))]
    pub jobs: NonZeroUsize,

    /// Start of the report, YYYY-MM-DD (default: one year ago)
    #[arg(long, help_heading = "Report options")]
    pub since: Option<NaiveDate>,

    /// Directory for the report (default: the working directory)
    #[arg(long, help_heading = "Report options")]
    pub output: Option<PathBuf>,

    /// Write the report without opening it in the browser
    #[arg(long, help_heading = "Report options")]
    pub no_open: bool,

    /// Emit one JSON result object
    #[arg(long, help_heading = "Scoring options")]
    pub json: bool,

    /// Include test files and Rust inline test modules (excluded by default)
    #[arg(long, help_heading = "Scoring options")]
    pub include_tests: bool,

    /// Include generated, dependency, documentation, and test files; also include inline tests
    #[arg(long, help_heading = "Scoring options")]
    pub include_excluded: bool,

    /// Show model metadata without evaluating files
    #[arg(long, help_heading = "Scoring options")]
    pub model_info: bool,

    /// Number of positive and negative factors in text output (default: 3, or 5 when comparing)
    #[arg(long, help_heading = "Scoring options")]
    pub top: Option<usize>,

    /// Compare the files changed since the merge base with REF against their versions there
    #[arg(
        long,
        value_name = "REF",
        help_heading = "Comparison options",
        conflicts_with_all = ["files", "upstream_from_pre_push"]
    )]
    pub upstream: Option<String>,

    /// Compare the commits being pushed, read from a Git pre-push hook's stdin
    #[arg(long, help_heading = "Comparison options", conflicts_with = "files")]
    pub upstream_from_pre_push: bool,

    /// Largest score drop allowed before a changed file counts as declined (default: 1)
    #[arg(long, value_name = "POINTS", help_heading = "Comparison options")]
    pub max_drop: Option<f64>,

    /// What runs the comparison; pre-push reports a blocked push, lets errors through, and can be skipped with Enter
    #[arg(
        long,
        value_enum,
        default_value = "manual",
        help_heading = "Comparison options"
    )]
    pub trigger: Trigger,

    /// What the prompt printed for declined files asks a coding agent to do (default: recommend)
    #[arg(
        long,
        value_enum,
        value_name = "MODE",
        help_heading = "Comparison options"
    )]
    pub prompt: Option<Prompt>,
}

#[derive(Subcommand, Debug)]
pub enum SlopOneCommand {
    /// Build and render weekly or monthly code quality trend reports for a repository
    #[command(hide = true)]
    Trends(trends::Trends),
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Provider {
    Typesafe,
    Vercel,
    Openrouter,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Prompt {
    Recommend,
    Refactor,
}

impl From<Prompt> for PromptMode {
    fn from(prompt: Prompt) -> Self {
        match prompt {
            Prompt::Recommend => Self::Recommend,
            Prompt::Refactor => Self::Refactor,
        }
    }
}

impl From<Provider> for JevProvider {
    fn from(provider: Provider) -> Self {
        match provider {
            Provider::Typesafe => Self::TypeSafe,
            Provider::Vercel => Self::Vercel,
            Provider::Openrouter => Self::OpenRouter,
        }
    }
}

impl SlopOne {
    pub fn execute(&self, args: &Arguments) -> Result<CommandSuccess, CommandError> {
        if let Some(SlopOneCommand::Trends(trends)) = &self.command {
            return trends.execute(args);
        }
        if let Some(budget) = self.budget {
            if !budget.is_finite() || budget < 0.0 {
                return Err(CommandError::InvalidOptions {
                    message: "--budget must be a finite, nonnegative dollar amount".to_owned(),
                });
            }
        }
        if self.upstream.is_some() || self.upstream_from_pre_push {
            return self.compare();
        }
        if self.max_drop.is_some() || self.prompt.is_some() || self.trigger != Trigger::Manual {
            return Err(CommandError::InvalidOptions {
                message:
                    "--max-drop, --prompt, and --trigger apply with --upstream or --upstream-from-pre-push"
                        .to_owned(),
            });
        }
        if self.files.is_empty() && !self.model_info {
            self.report()
        } else {
            self.score()
        }
    }

    fn cache_dir(&self) -> Result<PathBuf, CommandError> {
        Ok(match &self.cache_dir {
            Some(dir) => dir.clone(),
            None => Library::global_cache_root()?.join("slop-one"),
        })
    }

    fn report(&self) -> Result<CommandSuccess, CommandError> {
        if self.json || self.include_tests || self.include_excluded || self.top.is_some() {
            return Err(CommandError::InvalidOptions {
                message: "--json, --include-tests, --include-excluded, and --top apply when scoring files".to_owned(),
            });
        }
        let cwd = env::current_dir()?;
        let since = match self.since {
            Some(since) => since,
            None => default_since()?,
        };
        let progress = ReportProgress::new();
        let outcome = pipeline::report(
            &ReportOptions {
                repo: cwd.clone(),
                since,
                output_dir: self.output.clone().unwrap_or(cwd),
                cache_dir: self.cache_dir()?,
                jobs: self.jobs,
                budget_usd: self.budget,
                provider: self.provider.into(),
                offline: self.offline,
            },
            &progress,
        )
        .map_err(|error| {
            progress.fail();
            explain_trends(error)
        })?;
        if let Some(line) = ui::sparkline_line(&outcome.recent_weekly_net) {
            eprintln!();
            eprintln!("{line}");
        }
        eprintln!();
        if self.no_open {
            println!("Report: {}", outcome.html.display());
        } else {
            let url = format!("file://{}", std::path::absolute(&outcome.html)?.display());
            match webbrowser::open(&url) {
                Ok(()) => println!("Opened {} in your browser.", outcome.html.display()),
                Err(error) => {
                    eprintln!("Could not open the report in a browser: {error}");
                    println!("Report: {}", outcome.html.display());
                }
            }
        }
        CommandSuccess::ok()
    }

    fn score(&self) -> Result<CommandSuccess, CommandError> {
        self.reject_report_options()?;
        let top = self.top(DEFAULT_TOP)?;
        let evaluator = self.evaluator()?;
        if self.model_info && self.files.is_empty() {
            println!("{}", serde_json::to_string_pretty(&evaluator.model_info())?);
            return CommandSuccess::ok();
        }
        let mut seen = HashSet::new();
        let mut paths = vec![];
        for path in &self.files {
            let normalized = std::path::absolute(path)?;
            if seen.insert(normalized) {
                paths.push(path.clone());
            }
        }
        let progress = FileProgress::new(paths.len());
        let results = evaluator
            .evaluate_all_with(&paths, self.jobs, &|_| progress.file_done())
            .map_err(|error| {
                progress.finish();
                explain_scoring(error)
            })?;
        progress.finish();
        let document = Document::new(evaluator.model_info(), results, evaluator.usage());
        if self.json {
            println!("{}", serde_json::to_string_pretty(&document)?);
        } else {
            let mut sections: Vec<String> = document
                .results
                .iter()
                .map(|result| render_text(result, top))
                .collect();
            sections.push(render_summary(&document.summary));
            println!("{}", sections.join("\n\n"));
        }
        if document.errors > 0 {
            return Err(CommandError::Unknown {
                source: anyhow!("{} file(s) could not be evaluated", document.errors),
            });
        }
        Ok(CommandSuccess {
            fail: document.passed == Some(false),
            ..Default::default()
        })
    }

    fn reject_report_options(&self) -> Result<(), CommandError> {
        if self.since.is_some() || self.output.is_some() || self.no_open {
            return Err(CommandError::InvalidOptions {
                message: "--since, --output, and --no-open apply to the trends report".to_owned(),
            });
        }
        Ok(())
    }

    fn top(&self, default: usize) -> Result<usize, CommandError> {
        let top = self.top.unwrap_or(default);
        if !(1..=15).contains(&top) {
            return Err(CommandError::InvalidOptions {
                message: "--top must be between 1 and 15".to_string(),
            });
        }
        Ok(top)
    }

    fn evaluator(&self) -> Result<Evaluator, CommandError> {
        Evaluator::new(Options {
            cache_dir: self.cache_dir()?,
            budget_usd: self.budget.unwrap_or(DEFAULT_SCORING_BUDGET_USD),
            provider: self.provider.into(),
            offline: self.offline,
            include_tests: self.include_tests,
            include_excluded: self.include_excluded,
        })
        .map_err(explain_scoring)
    }

    /// Checks the options first, so a mistyped hook still fails loudly. After
    /// that, as a pre-push hook, nothing but a declined file blocks the push.
    fn compare(&self) -> Result<CommandSuccess, CommandError> {
        self.reject_report_options()?;
        if self.model_info {
            return Err(CommandError::InvalidOptions {
                message: "--model-info does not apply when comparing changes".to_owned(),
            });
        }
        let top = self.top(DEFAULT_COMPARISON_TOP)?;
        let max_drop = self.max_drop.unwrap_or(DEFAULT_MAX_DROP);
        if !max_drop.is_finite() || max_drop < 0.0 {
            return Err(CommandError::InvalidOptions {
                message: "--max-drop must be a finite, nonnegative number of points".to_owned(),
            });
        }
        let framing = match self.trigger {
            Trigger::PrePush => Framing::PrePush,
            _ => Framing::Standalone,
        };
        let result = self.compare_changes(top, max_drop, framing);
        match (result, framing) {
            (Err(error), Framing::PrePush) => Ok(allow_push(error)),
            (result, _) => result,
        }
    }

    fn compare_changes(
        &self,
        top: usize,
        max_drop: f64,
        framing: Framing,
    ) -> Result<CommandSuccess, CommandError> {
        let cwd = env::current_dir()?;
        let (changes, revisions) = match &self.upstream {
            Some(upstream) => {
                if framing == Framing::PrePush {
                    git_hook::exit_on_enter();
                }
                let found = changes::since_upstream(&cwd, upstream)?;
                let revisions = Revisions::WorkingTree {
                    upstream: upstream.clone(),
                    merge_base: found.merge_base,
                };
                (found.changes, revisions)
            }
            None => {
                let Some(input) = git_hook::read_pre_push_stdin()? else {
                    return CommandSuccess::ok();
                };
                if framing == Framing::PrePush {
                    git_hook::exit_on_enter();
                }
                let pushed = changes::being_pushed(&cwd, &RefUpdate::parse_all(&input)?)?;
                for local_ref in &pushed.without_base {
                    eprintln!(
                        "SlopOne skipped {local_ref}: none of its commits are on a remote yet, so there is nothing to compare with."
                    );
                }
                (pushed.changes, Revisions::Pushed(pushed.revisions))
            }
        };
        let options = TextOptions {
            top,
            max_drop,
            framing,
            prompt: self.prompt.map_or(PromptMode::Recommend, PromptMode::from),
            revisions,
        };
        self.report_comparisons(&changes, &options)
    }

    fn report_comparisons(
        &self,
        changes: &[Change],
        options: &TextOptions,
    ) -> Result<CommandSuccess, CommandError> {
        let evaluator = self.evaluator()?;
        let progress = FileProgress::new(changes.len());
        let comparisons = evaluator
            .compare_all_with(changes, self.jobs, options.max_drop, &|_| {
                progress.file_done()
            })
            .map_err(|error| {
                progress.finish();
                explain_scoring(error)
            })?;
        progress.finish();
        let summary = ComparisonSummary::of(&comparisons, options.max_drop);
        if self.json {
            let document = Document::compared(
                evaluator.model_info(),
                &comparisons,
                evaluator.usage(),
                options.max_drop,
            );
            println!("{}", serde_json::to_string_pretty(&document)?);
        } else {
            println!("{}", render_comparisons(&comparisons, options));
        }
        if options.framing == Framing::PrePush {
            if summary.error_files > 0 {
                eprintln!(
                    "SlopOne could not score {}; errors do not block the push.",
                    plural(summary.error_files, "changed file")
                );
            }
            if summary.declined_files > 0 {
                eprintln!(
                    "Push blocked: {} declined. The output above is a refactoring prompt for your coding agent.",
                    plural(summary.declined_files, "changed file")
                );
                eprintln!(
                    "Only a developer should decide to skip this check with git push --no-verify."
                );
            }
        } else if summary.error_files > 0 {
            return Err(CommandError::Unknown {
                source: anyhow!("{} file(s) could not be evaluated", summary.error_files),
            });
        }
        Ok(CommandSuccess {
            fail: !summary.passed,
            ..Default::default()
        })
    }
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

/// One year before today in the machine's timezone.
fn default_since() -> Result<NaiveDate, CommandError> {
    let today = Utc::now().with_timezone(&local_timezone()?).date_naive();
    today
        .checked_sub_months(Months::new(12))
        .ok_or_else(|| CommandError::Unknown {
            source: anyhow!("cannot compute a start date one year before {today}"),
        })
}
