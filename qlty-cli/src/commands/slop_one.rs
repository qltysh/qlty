use crate::{Arguments, CommandError, CommandSuccess};
use anyhow::anyhow;
use chrono::{Months, NaiveDate, Utc};
use clap::{Args, Subcommand};
use qlty_config::Library;
use qlty_slop_one::{render_summary, render_text, Document, Evaluator, JevProvider, Options};
use qlty_slop_one_trends::periods::local_timezone;
use qlty_slop_one_trends::pipeline::{self, ReportOptions};
use std::collections::HashSet;
use std::env;
use std::num::NonZeroUsize;
use std::path::PathBuf;

mod trends;
mod ui;

use ui::{explain_scoring, explain_trends, FileProgress, ReportProgress};

const DEFAULT_TOP: usize = 3;
const DEFAULT_SCORING_BUDGET_USD: f64 = 1.0;

/// With no files, builds the trends report for the current repository and
/// opens it. With files, scores them and prints the results.
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

    /// Number of positive and negative factors in text output (default: 3)
    #[arg(long, help_heading = "Scoring options")]
    pub top: Option<usize>,
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
        if self.since.is_some() || self.output.is_some() || self.no_open {
            return Err(CommandError::InvalidOptions {
                message: "--since, --output, and --no-open apply to the trends report".to_owned(),
            });
        }
        let top = self.top.unwrap_or(DEFAULT_TOP);
        if !(1..=15).contains(&top) {
            return Err(CommandError::InvalidOptions {
                message: "--top must be between 1 and 15".to_string(),
            });
        }
        let evaluator = Evaluator::new(Options {
            cache_dir: self.cache_dir()?,
            budget_usd: self.budget.unwrap_or(DEFAULT_SCORING_BUDGET_USD),
            provider: self.provider.into(),
            offline: self.offline,
            include_tests: self.include_tests,
            include_excluded: self.include_excluded,
        })
        .map_err(explain_scoring)?;
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
