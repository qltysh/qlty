//! The `build` pipeline: freeze, score, export, explain, render. Every stage
//! is resumable and skips work that is already on disk.

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use git2::Repository;
use qlty_slop_one::content::MeasurementCache;
use qlty_slop_one::jev::{Jev, JevProvider};

use chrono::NaiveDate;
use qlty_slop_one::Usage;

use crate::error::{Error, Result};
use crate::periods::{local_timezone, parse_as_of, Interval};
use crate::run::{Manifest, RunDir};
use crate::scoring::{ScoringSummary, Status};
use crate::snapshot::{self, FreezeOptions};
use crate::{explain, export, render, scoring};

/// Where a run keeps its data and writes its artifacts.
#[derive(Clone, Debug)]
pub struct Locations {
    /// The SlopOne cache root; runs live under `trends/<repository>/<name>/`.
    pub cache_dir: PathBuf,
    /// Where `<name>.html`, `<name>.svg`, and the JSON exports are written.
    pub output_dir: PathBuf,
    pub name: String,
}

impl Locations {
    pub fn run_dir(&self, repository_url: &str) -> RunDir {
        RunDir::new(
            self.cache_dir
                .join("trends")
                .join(repository_slug(repository_url))
                .join(&self.name),
        )
    }

    pub fn artifact(&self, suffix: &str) -> PathBuf {
        self.output_dir.join(format!("{}{suffix}", self.name))
    }

    pub fn output_stem(&self) -> PathBuf {
        self.output_dir.join(&self.name)
    }
}

/// A filesystem-safe form of a repository URL, e.g. `github.com/owner/repo`.
pub fn repository_slug(url: &str) -> String {
    let trimmed = url
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    trimmed
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '-' | '_') {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// The last path segment of a repository URL.
pub fn repository_name(repository_url: &str) -> &str {
    repository_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("repository")
}

/// The default run name: `<repository name>-weekly` or `-monthly`.
pub fn default_name(repository_url: &str, interval: Interval) -> String {
    let interval = match interval {
        Interval::Week => "weekly",
        Interval::Month => "monthly",
    };
    format!("{}-{interval}", repository_name(repository_url))
}

/// The normalized web URL of the repository: an explicit value, else `origin`.
pub fn resolve_repository_url(repo: &Path, explicit: Option<&str>) -> Result<String> {
    if let Some(url) = explicit {
        return snapshot::repository_url(url);
    }
    let repository =
        Repository::discover(repo).map_err(|_| Error::NotARepository(repo.to_path_buf()))?;
    let remote = repository.find_remote("origin").map_err(|_| {
        Error::Configuration(
            "the repository has no origin remote; pass --repository-url".to_owned(),
        )
    })?;
    let url = remote.url().ok_or_else(|| {
        Error::Configuration("the origin remote URL is not valid UTF-8".to_owned())
    })?;
    snapshot::repository_url(url)
}

/// The stages a build passes through, in order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Snapshots,
    Scoring,
    Export,
    Render,
}

/// Progress events for a caller that wants to show them. Every method has a
/// no-op default, and [`Silent`] takes none.
pub trait Observer: Sync {
    fn stage_started(&self, _stage: Stage) {}
    fn snapshots_frozen(&self, _manifest: &Manifest) {}
    /// The contents the scoring stage will score, across all runs.
    fn scoring_started(&self, _pending: usize) {}
    /// A content was scored: from the measurement cache when `cached`.
    fn content_scored(&self, _cached: bool, _usage: &Usage) {}
    fn content_failed(&self, _key: &str, _error: &qlty_slop_one::Error) {}
    /// The report file was written; called before the render stage ends.
    fn report_rendered(&self, _path: &Path) {}
    fn stage_finished(&self, _stage: Stage) {}
}

pub struct Silent;

impl Observer for Silent {}

#[derive(Clone, Debug)]
pub struct BuildOptions {
    pub freeze: FreezeOptions,
    pub jobs: NonZeroUsize,
    /// `None` means no spending cap.
    pub budget_usd: Option<f64>,
    pub provider: JevProvider,
    pub offline: bool,
    pub retry_errors: bool,
    pub render: bool,
    /// The name of a monthly companion run to build alongside a weekly run.
    /// It is frozen at the weekly run's cutoff and joins the rendered
    /// report's interval picker.
    pub monthly: Option<String>,
    /// Freeze an existing run again at these settings instead of refusing
    /// to change it. Scored contents are kept.
    pub refresh: bool,
}

/// One run's frozen manifest and scoring summary.
#[derive(Debug)]
pub struct RunOutcome {
    pub manifest: Manifest,
    pub scoring: ScoringSummary,
    pub run_dir: RunDir,
}

#[derive(Debug)]
pub struct BuildOutcome {
    pub run: RunOutcome,
    pub monthly: Option<RunOutcome>,
}

/// Runs every stage in order: both runs are frozen first, then scored,
/// then exported. With a monthly companion, both runs share one cutoff,
/// Jev budget, and measurement cache, so the companion only scores
/// contents the weekly run never saw.
pub fn build(
    locations: &Locations,
    options: &BuildOptions,
    observer: &dyn Observer,
) -> Result<BuildOutcome> {
    if options.monthly.is_some() && options.freeze.interval != Interval::Week {
        return Err(Error::Configuration(
            "a monthly companion is built alongside a weekly run".to_owned(),
        ));
    }
    let repository_url = resolve_repository_url(
        &options.freeze.repo,
        options.freeze.repository_url.as_deref(),
    )?;
    let freeze = FreezeOptions {
        repository_url: Some(repository_url.clone()),
        cache_directory: locations.cache_dir.clone(),
        ..options.freeze.clone()
    };

    observer.stage_started(Stage::Snapshots);
    let mut runs = vec![Frozen::freeze(
        locations,
        &repository_url,
        &freeze,
        options,
    )?];
    observer.snapshots_frozen(&runs[0].manifest);
    if let Some(name) = &options.monthly {
        let companion = Locations {
            name: name.clone(),
            ..locations.clone()
        };
        let companion_freeze = FreezeOptions {
            interval: Interval::Month,
            as_of: Some(parse_as_of(&runs[0].manifest.created_at)?),
            ..freeze.clone()
        };
        runs.push(Frozen::freeze(
            &companion,
            &repository_url,
            &companion_freeze,
            options,
        )?);
        observer.snapshots_frozen(&runs[1].manifest);
    }
    observer.stage_finished(Stage::Snapshots);

    observer.stage_started(Stage::Scoring);
    let jev = Jev::try_new(
        locations.cache_dir.clone(),
        options.budget_usd.unwrap_or(f64::MAX),
        options.provider,
        options.offline,
    )?;
    let cache = MeasurementCache::try_new(&locations.cache_dir)?;
    let mut pending = 0;
    for frozen in &runs {
        pending += scoring::pending_count(&frozen.run, &frozen.manifest, options.retry_errors)?;
    }
    observer.scoring_started(pending);
    let mut outcomes = Vec::with_capacity(runs.len());
    for frozen in runs {
        let scoring = scoring::score_pending(
            &frozen.run,
            &frozen.manifest,
            &jev,
            &cache,
            options.jobs,
            options.retry_errors,
            observer,
        )?;
        outcomes.push((frozen, scoring));
    }
    observer.stage_finished(Stage::Scoring);

    observer.stage_started(Stage::Export);
    let mut results = Vec::with_capacity(outcomes.len());
    for (frozen, scoring) in outcomes {
        export::export(
            &frozen.run,
            &frozen.manifest,
            &frozen.locations.output_dir,
            &frozen.locations.name,
        )?;
        explain::export(
            &frozen.run,
            &frozen.locations.output_dir,
            &frozen.locations.name,
        )?;
        results.push(RunOutcome {
            manifest: frozen.manifest,
            scoring,
            run_dir: frozen.run,
        });
    }
    observer.stage_finished(Stage::Export);

    if options.render {
        observer.stage_started(Stage::Render);
        render_report(locations, options.monthly.as_deref())?;
        observer.stage_finished(Stage::Render);
    }
    let mut results = results.into_iter();
    let run = results.next().ok_or(Error::TooFewSnapshots)?;
    Ok(BuildOutcome {
        run,
        monthly: results.next(),
    })
}

/// One run after freezing, before scoring.
struct Frozen {
    locations: Locations,
    run: RunDir,
    manifest: Manifest,
}

impl Frozen {
    fn freeze(
        locations: &Locations,
        repository_url: &str,
        freeze: &FreezeOptions,
        options: &BuildOptions,
    ) -> Result<Self> {
        let run = locations.run_dir(repository_url);
        let manifest = if options.refresh {
            snapshot::refresh(&run, freeze)?
        } else {
            snapshot::freeze(&run, freeze)?
        };
        Ok(Self {
            locations: locations.clone(),
            run,
            manifest,
        })
    }
}

/// What the everyday report needs: everything else takes its default.
#[derive(Clone, Debug)]
pub struct ReportOptions {
    pub repo: PathBuf,
    pub since: NaiveDate,
    /// Where `<repository>-trends.html` is written.
    pub output_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub jobs: NonZeroUsize,
    /// `None` means no spending cap.
    pub budget_usd: Option<f64>,
    pub provider: JevProvider,
    pub offline: bool,
}

#[derive(Debug)]
pub struct ReportOutcome {
    pub html: PathBuf,
    pub build: BuildOutcome,
    /// The net quality change the chart draws for the last weeks, oldest
    /// first; `None` where there was no earlier week to compare.
    pub recent_weekly_net: Vec<Option<f64>>,
}

/// How many weeks the outcome's recent net changes cover.
pub const RECENT_WEEKS: usize = 12;

/// Builds the weekly and monthly runs up to now, keeps their data under the
/// cache, and writes one report. The runs are refreshed on every call, so
/// the report always reflects the repository as it is; scored contents are
/// reused from the caches.
pub fn report(options: &ReportOptions, observer: &dyn Observer) -> Result<ReportOutcome> {
    let repository_url = resolve_repository_url(&options.repo, None)?;
    let monthly = default_name(&repository_url, Interval::Month);
    let exports = options
        .cache_dir
        .join("trends")
        .join(repository_slug(&repository_url))
        .join("exports");
    let locations = Locations {
        cache_dir: options.cache_dir.clone(),
        output_dir: exports.clone(),
        name: default_name(&repository_url, Interval::Week),
    };
    let build = build(
        &locations,
        &BuildOptions {
            freeze: FreezeOptions {
                repo: options.repo.clone(),
                since: options.since,
                interval: Interval::Week,
                timezone: local_timezone()?,
                git_ref: None,
                as_of: None,
                project: None,
                repository_url: Some(repository_url.clone()),
                cache_directory: options.cache_dir.clone(),
            },
            jobs: options.jobs,
            budget_usd: options.budget_usd,
            provider: options.provider,
            offline: options.offline,
            retry_errors: false,
            render: false,
            monthly: Some(monthly.clone()),
            refresh: true,
        },
        observer,
    )?;
    let stem = options
        .output_dir
        .join(format!("{}-trends", repository_name(&repository_url)));
    observer.stage_started(Stage::Render);
    let values = render::html::render(
        &locations.artifact(".json"),
        &stem,
        Some(&exports.join(format!("{monthly}.json"))),
    )?;
    observer.report_rendered(&stem.with_extension("html"));
    observer.stage_finished(Stage::Render);
    let skipped = values.len().saturating_sub(RECENT_WEEKS);
    let recent_weekly_net = values
        .iter()
        .skip(skipped)
        .map(|period| period.with_file_changes.net)
        .collect();
    Ok(ReportOutcome {
        html: stem.with_extension("html"),
        build,
        recent_weekly_net,
    })
}

/// Renders `<name>.html` from the exported data.
pub fn render_report(locations: &Locations, monthly_companion: Option<&str>) -> Result<()> {
    let data = locations.artifact(".json");
    let monthly = monthly_companion.map(|name| locations.output_dir.join(format!("{name}.json")));
    render::html::render(&data, &locations.output_stem(), monthly.as_deref())?;
    Ok(())
}

/// How many of a run's contents are scored.
pub fn status(locations: &Locations, repository_url: &str) -> Result<Status> {
    scoring::status(&locations.run_dir(repository_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_strips_scheme_and_keeps_path_characters() {
        assert_eq!(
            repository_slug("https://github.com/veniceai/interface"),
            "github.com/veniceai/interface"
        );
    }

    #[test]
    fn slug_replaces_unsafe_characters() {
        assert_eq!(repository_slug("https://host:8443/a b"), "host-8443/a-b");
    }

    #[test]
    fn repository_name_is_the_last_segment() {
        assert_eq!(
            repository_name("https://github.com/veniceai/interface/"),
            "interface"
        );
        assert_eq!(repository_name(""), "repository");
    }

    #[test]
    fn default_name_uses_repository_and_interval() {
        assert_eq!(
            default_name("https://github.com/veniceai/interface", Interval::Week),
            "interface-weekly"
        );
        assert_eq!(
            default_name("https://github.com/veniceai/interface/", Interval::Month),
            "interface-monthly"
        );
    }
}
