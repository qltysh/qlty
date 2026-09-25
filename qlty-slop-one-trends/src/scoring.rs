//! Score every unique content of a frozen run with SlopOne.
//!
//! Ports `WeeklyRun.run`, `evaluate`, and `status` from slopdetect's
//! `weekly_scan.py`: pending contents are scored on a bounded pool, each
//! result (or analysis error) is written to `results/<key>.json`, and the
//! content-keyed measurement cache makes a repeated or extended run cheap.

use std::collections::BTreeSet;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use qlty_slop_one::content::score_content_via_cache;
use qlty_slop_one::jev::Jev;
use qlty_slop_one::scoring::SCORE_SCALE;
use qlty_slop_one::source::source_text;
use qlty_slop_one::{content_key, ContentScore, MeasurementCache, Model, Usage};
use rayon::prelude::*;
use serde_json::Map;
use tracing::{info, warn};

use crate::error::{Error, Result};
use crate::pipeline::Observer;
use crate::run::{write_json, ContentResult, Manifest, RunDir};

/// How often a progress line is logged, in scored contents.
const PROGRESS_EVERY: usize = 50;

/// What one scoring pass did.
#[derive(Clone, Debug)]
pub struct ScoringSummary {
    /// Contents that were pending when the pass started.
    pub total: usize,
    /// Contents whose analysis failed and were recorded as error results.
    pub errors: usize,
    pub usage: Usage,
}

/// How far a run's scoring has come.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {
    /// Contents with a result file, including error results.
    pub measured: usize,
    /// Distinct analyzed contents in the manifest.
    pub total: usize,
    /// Result files that record an analysis error.
    pub errors: usize,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "STATUS {} / {} measured; {} analysis errors",
            self.measured, self.total, self.errors
        )
    }
}

/// One content still to score.
#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingContent {
    key: String,
    suffix: String,
}

/// The outcome of analyzing one content: a score, from the measurement
/// cache when `cached`, or the analysis error to record.
enum Analysis {
    Scored(ContentScore, bool),
    Failed(qlty_slop_one::Error),
}

/// How many contents [`score_pending`] would score.
pub fn pending_count(run: &RunDir, manifest: &Manifest, retry_errors: bool) -> Result<usize> {
    Ok(pending_contents(run, manifest, Model::shared()?, retry_errors)?.len())
}

/// Scores every content without a result (or with an error result when
/// `retry_errors`), on `jobs` worker threads, writing `results/<key>.json`
/// as each finishes and `usage.json` at the end. A missing credential or an
/// exhausted budget stops the pass; everything scored so far is kept.
pub fn score_pending(
    run: &RunDir,
    manifest: &Manifest,
    jev: &Jev,
    cache: &MeasurementCache,
    jobs: NonZeroUsize,
    retry_errors: bool,
    observer: &dyn Observer,
) -> Result<ScoringSummary> {
    if manifest.measurement_identity != cache.identity() {
        return Err(Error::Frozen(format!(
            "measurement identity {} differs from the current tooling {}",
            manifest.measurement_identity,
            cache.identity()
        )));
    }
    let model = Model::shared()?;
    let pending = pending_contents(run, manifest, model, retry_errors)?;
    info!("SCORING {} contents with {} workers", pending.len(), jobs);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs.get())
        .build()
        .map_err(|error| {
            Error::Configuration(format!("Could not start {jobs} worker threads: {error}"))
        })?;
    let progress = Progress::new(pending.len());
    let outcome = pool.install(|| {
        pending
            .par_iter()
            .map(|content| {
                let analysis = analyze(run, content, jev, cache)?;
                match &analysis {
                    Analysis::Scored(_, cached) => observer.content_scored(*cached, &jev.usage()),
                    Analysis::Failed(error) => observer.content_failed(&content.key, error),
                }
                let result = content_result(&content.key, analysis, model, cache.identity());
                write_json(&run.result_path(&content.key), &result)?;
                progress.record(&content.key, &result, jev);
                Ok(())
            })
            .collect::<Result<()>>()
    });
    let usage = jev.usage();
    write_json(&run.usage_path(), &usage)?;
    outcome?;
    Ok(ScoringSummary {
        total: pending.len(),
        errors: progress.errors(),
        usage,
    })
}

/// Counts the run's result files and logs a `STATUS` line.
pub fn status(run: &RunDir) -> Result<Status> {
    let manifest = run.load_manifest()?;
    let keys = manifest.measurement_keys();
    let mut measured = 0;
    let mut errors = 0;
    for key in &keys {
        if !run.has_result(key) {
            continue;
        }
        measured += 1;
        if run.load_result(key)?.is_error() {
            errors += 1;
        }
    }
    let status = Status {
        measured,
        total: keys.len(),
        errors,
    };
    info!("{status}");
    Ok(status)
}

/// The contents to score, in key order. Every saved result must come from
/// the current model.
fn pending_contents(
    run: &RunDir,
    manifest: &Manifest,
    model: &Model,
    retry_errors: bool,
) -> Result<Vec<PendingContent>> {
    let mut pending = Vec::new();
    let mut seen = BTreeSet::new();
    for version in manifest.versions.values() {
        let Some(key) = version.measurement_key.as_deref() else {
            continue;
        };
        if !seen.insert(key) {
            continue;
        }
        if run.has_result(key) {
            let saved = run.load_result(key)?;
            if saved.model_sha256 != model.sha256() {
                return Err(Error::Mismatch(format!(
                    "result {key} was scored by model {}",
                    saved.model_sha256
                )));
            }
            if !retry_errors || !saved.is_error() {
                continue;
            }
        }
        pending.push(PendingContent {
            key: key.to_owned(),
            suffix: version.suffix.clone(),
        });
    }
    pending.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(pending)
}

/// Reads the prepared text and scores it. Analysis failures are recorded;
/// a prepared file whose content no longer matches its key is corrupt and
/// stops the run.
fn analyze(
    run: &RunDir,
    content: &PendingContent,
    jev: &Jev,
    cache: &MeasurementCache,
) -> Result<Analysis> {
    let path = run.prepared_path(&content.key, &content.suffix);
    let text = match source_text(&path) {
        Ok(text) => text,
        Err(error) => return Ok(Analysis::Failed(error)),
    };
    if content_key(&content.suffix, &text)? != content.key {
        return Err(Error::Mismatch(format!(
            "prepared source {} does not match its content key",
            path.display()
        )));
    }
    match score_content_via_cache(&text, &content.suffix, jev, cache) {
        Ok((score, cached)) => Ok(Analysis::Scored(score, cached)),
        Err(error) if error.is_fatal() => Err(Error::Score(error)),
        Err(error) => Ok(Analysis::Failed(error)),
    }
}

/// A schema 2 result row, in slopdetect's shape.
fn content_result(key: &str, analysis: Analysis, model: &Model, identity: &str) -> ContentResult {
    let mut result = ContentResult {
        score: None,
        passed: None,
        language: None,
        code_lines: None,
        cyclomatic: None,
        mass: None,
        features: None,
        summary: None,
        source_lines: None,
        error: None,
        schema_version: 2,
        key: key.to_owned(),
        model_sha256: model.sha256().to_owned(),
        score_scale: SCORE_SCALE.to_owned(),
        measurement_identity: identity.to_owned(),
        extra: Map::new(),
    };
    match analysis {
        Analysis::Scored(score, _) => {
            result.score = Some(score.score);
            result.passed = Some(score.passed);
            result.language = Some(score.language);
            result.code_lines = Some(score.code_lines);
            result.cyclomatic = Some(score.cyclomatic);
            result.mass = Some(score.mass);
            result.features = Some(score.features);
            result.summary = Some(score.summary);
            result.source_lines = Some(score.source_lines);
        }
        Analysis::Failed(error) => result.error = Some(error.to_string()),
    }
    result
}

/// Thread-safe progress counters with slopdetect's log lines.
struct Progress {
    total: usize,
    done: AtomicUsize,
    errors: AtomicUsize,
}

impl Progress {
    fn new(total: usize) -> Self {
        Self {
            total,
            done: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
        }
    }

    fn record(&self, key: &str, result: &ContentResult, jev: &Jev) {
        let done = self.done.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(error) = &result.error {
            self.errors.fetch_add(1, Ordering::SeqCst);
            warn!("ERROR {key} {error}");
        }
        if done.is_multiple_of(PROGRESS_EVERY) || done == self.total {
            let usage = serde_json::to_string(&jev.usage()).unwrap_or_default();
            info!(
                "PROGRESS {done} / {} errors {} usage {usage}",
                self.total,
                self.errors()
            );
        }
    }

    fn errors(&self) -> usize {
        self.errors.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_displays_like_the_python_line() {
        let status = Status {
            measured: 3,
            total: 5,
            errors: 1,
        };
        assert_eq!(
            status.to_string(),
            "STATUS 3 / 5 measured; 1 analysis errors"
        );
    }

    #[test]
    fn an_error_result_carries_only_the_trailer() {
        let model = Model::shared().unwrap();
        let result = content_result(
            "k",
            Analysis::Failed(qlty_slop_one::Error::Jev("boom".to_owned())),
            model,
            "id",
        );
        assert_eq!(result.error.as_deref(), Some("boom"));
        assert_eq!(result.score, None);
        assert_eq!(result.schema_version, 2);
        assert_eq!(result.model_sha256, model.sha256());
        assert_eq!(result.score_scale, SCORE_SCALE);
        assert_eq!(result.measurement_identity, "id");
    }
}
