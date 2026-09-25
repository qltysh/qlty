use std::env;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use anyhow::anyhow;
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use qlty_config::Library;
use qlty_slop_one_trends::periods::{local_timezone, parse_as_of, parse_timezone, Interval};
use qlty_slop_one_trends::pipeline::{
    self, default_name, resolve_repository_url, BuildOptions, Locations, Silent,
};
use qlty_slop_one_trends::snapshot::FreezeOptions;

use super::Provider;
use crate::{Arguments, CommandError, CommandSuccess};

#[derive(Args, Debug)]
pub struct Trends {
    #[command(subcommand)]
    pub command: TrendsCommand,
}

#[derive(Subcommand, Debug)]
pub enum TrendsCommand {
    /// Freeze snapshots, score contents, export data, explain changes, and render the report
    Build(Build),
    /// Render the HTML report from previously exported data
    Render(Render),
    /// Show how many contents of a run are scored
    Status(Status),
}

#[derive(Args, Debug)]
pub struct Common {
    /// Run and output filename stem (default: <repository>-weekly or -monthly)
    #[arg(long)]
    pub name: Option<String>,

    /// Repository checkout (default: the repository containing the working directory)
    #[arg(long)]
    pub repo: Option<PathBuf>,

    /// Web URL for repository links (default: from the origin remote)
    #[arg(long)]
    pub repository_url: Option<String>,

    /// Directory for report artifacts (default: the working directory)
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Directory for run data and caches (default: ~/.qlty/cache/slop-one)
    #[arg(long)]
    pub cache_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum IntervalArg {
    Week,
    Month,
}

impl From<IntervalArg> for Interval {
    fn from(interval: IntervalArg) -> Self {
        match interval {
            IntervalArg::Week => Self::Week,
            IntervalArg::Month => Self::Month,
        }
    }
}

#[derive(Args, Debug)]
pub struct Build {
    #[command(flatten)]
    pub common: Common,

    /// Start date, YYYY-MM-DD; monthly runs also include the preceding month-end baseline
    #[arg(long)]
    pub since: NaiveDate,

    /// Calendar interval between snapshots
    #[arg(long, value_enum, default_value_t = IntervalArg::Week)]
    pub interval: IntervalArg,

    /// Git ref to analyze (default: origin/HEAD when configured, otherwise HEAD)
    #[arg(long = "ref")]
    pub git_ref: Option<String>,

    /// Freeze at this ISO 8601 timestamp, including its offset (default: now)
    #[arg(long)]
    pub as_of: Option<String>,

    /// Calendar boundary timezone, an IANA name (default: the machine's local timezone)
    #[arg(long)]
    pub timezone: Option<String>,

    /// Display name (default: the repository name)
    #[arg(long)]
    pub project: Option<String>,

    /// Number of contents to score concurrently
    #[arg(long, default_value_t = NonZeroUsize::new(16).expect("16 is nonzero"))]
    pub jobs: NonZeroUsize,

    /// Maximum estimated Jev spend for this invocation in USD (default: no cap)
    #[arg(long)]
    pub budget: Option<f64>,

    /// Jev provider
    #[arg(long, value_enum, default_value_t = Provider::Typesafe)]
    pub provider: Provider,

    /// Use cached Jev answers only; make no API calls
    #[arg(long)]
    pub offline: bool,

    /// Re-score contents whose previous analysis failed
    #[arg(long)]
    pub retry_errors: bool,

    /// Build data and explanations without rendering the HTML report
    #[arg(long)]
    pub data_only: bool,

    /// Also build a monthly run frozen at the same cutoff, and add the interval picker to the report
    #[arg(long)]
    pub monthly: bool,

    /// Run name for the monthly companion (default: <repository>-monthly)
    #[arg(long, requires = "monthly")]
    pub monthly_name: Option<String>,
}

#[derive(Args, Debug)]
pub struct Render {
    #[command(flatten)]
    pub common: Common,

    /// Monthly companion run name to include in the report's interval picker
    #[arg(long)]
    pub monthly: Option<String>,

    /// Calendar interval of the run named by --name (default: week)
    #[arg(long, value_enum, default_value_t = IntervalArg::Week)]
    pub interval: IntervalArg,
}

#[derive(Args, Debug)]
pub struct Status {
    #[command(flatten)]
    pub common: Common,

    /// Calendar interval of the run named by --name (default: week)
    #[arg(long, value_enum, default_value_t = IntervalArg::Week)]
    pub interval: IntervalArg,
}

impl Trends {
    pub fn execute(&self, _args: &Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            TrendsCommand::Build(build) => build.execute(),
            TrendsCommand::Render(render) => render.execute(),
            TrendsCommand::Status(status) => status.execute(),
        }
    }
}

struct Resolved {
    repo: PathBuf,
    repository_url: String,
    locations: Locations,
}

impl Common {
    fn resolve(&self, interval: Interval) -> Result<Resolved, CommandError> {
        let repo = match &self.repo {
            Some(repo) => repo.clone(),
            None => env::current_dir()?,
        };
        let repository_url = resolve_repository_url(&repo, self.repository_url.as_deref())?;
        let cache_dir = match &self.cache_dir {
            Some(dir) => dir.clone(),
            None => Library::global_cache_root()?.join("slop-one"),
        };
        let output_dir = match &self.output {
            Some(dir) => dir.clone(),
            None => env::current_dir()?,
        };
        let name = self
            .name
            .clone()
            .unwrap_or_else(|| default_name(&repository_url, interval));
        validate_name(&name, "--name")?;
        Ok(Resolved {
            repo,
            repository_url,
            locations: Locations {
                cache_dir,
                output_dir,
                name,
            },
        })
    }
}

fn validate_name(name: &str, option: &str) -> Result<(), CommandError> {
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        || name.starts_with('.')
    {
        return Err(CommandError::InvalidOptions {
            message: format!("{option} must use letters, digits, _, -, or ."),
        });
    }
    Ok(())
}

impl Build {
    fn execute(&self) -> Result<CommandSuccess, CommandError> {
        let interval: Interval = self.interval.into();
        let resolved = self.common.resolve(interval)?;
        if self.monthly && interval != Interval::Week {
            return Err(CommandError::InvalidOptions {
                message:
                    "--monthly builds a monthly companion for a weekly run; drop --interval month"
                        .to_owned(),
            });
        }
        let monthly = self
            .monthly
            .then(|| {
                self.monthly_name
                    .clone()
                    .unwrap_or_else(|| default_name(&resolved.repository_url, Interval::Month))
            })
            .map(|name| validate_name(&name, "--monthly-name").map(|()| name))
            .transpose()?;
        let timezone = match &self.timezone {
            Some(name) => parse_timezone(name)?,
            None => local_timezone()?,
        };
        let as_of = self.as_of.as_deref().map(parse_as_of).transpose()?;
        if let Some(budget) = self.budget {
            if !budget.is_finite() || budget < 0.0 {
                return Err(CommandError::InvalidOptions {
                    message: "--budget must be a finite, nonnegative dollar amount".to_owned(),
                });
            }
        }
        let outcome = pipeline::build(
            &resolved.locations,
            &BuildOptions {
                freeze: FreezeOptions {
                    repo: resolved.repo.clone(),
                    since: self.since,
                    interval,
                    timezone,
                    git_ref: self.git_ref.clone(),
                    as_of,
                    project: self.project.clone(),
                    repository_url: Some(resolved.repository_url.clone()),
                    cache_directory: resolved.locations.cache_dir.clone(),
                },
                jobs: self.jobs,
                budget_usd: self.budget,
                provider: self.provider.into(),
                offline: self.offline,
                retry_errors: self.retry_errors,
                render: !self.data_only,
                monthly,
                refresh: false,
            },
            &Silent,
        )?;
        for run in std::iter::once(&outcome.run).chain(outcome.monthly.as_ref()) {
            println!(
                "Built {} {} snapshots of {} ({} contents scored, {} analysis errors).",
                run.manifest.snapshots.len(),
                run.manifest.interval.as_str(),
                run.manifest.project,
                run.scoring.total,
                run.scoring.errors
            );
        }
        if self.data_only {
            println!("Data: {}", resolved.locations.artifact(".json").display());
        } else {
            println!("Report: {}", resolved.locations.artifact(".html").display());
        }
        CommandSuccess::ok()
    }
}

impl Render {
    fn execute(&self) -> Result<CommandSuccess, CommandError> {
        let resolved = self.common.resolve(self.interval.into())?;
        if !resolved.locations.artifact(".json").is_file() {
            return Err(CommandError::Unknown {
                source: anyhow!(
                    "no exported data at {}; run `qlty slop-one trends build` first",
                    resolved.locations.artifact(".json").display()
                ),
            });
        }
        pipeline::render_report(&resolved.locations, self.monthly.as_deref())?;
        println!("Report: {}", resolved.locations.artifact(".html").display());
        CommandSuccess::ok()
    }
}

impl Status {
    fn execute(&self) -> Result<CommandSuccess, CommandError> {
        let resolved = self.common.resolve(self.interval.into())?;
        let status = pipeline::status(&resolved.locations, &resolved.repository_url)?;
        println!("{status}");
        CommandSuccess::ok()
    }
}
