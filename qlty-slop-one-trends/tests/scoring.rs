//! The scoring stage over a small synthetic run: pending contents are scored
//! from the cached Jev answers, results are written once, error results are
//! recorded and retried on request.

use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use qlty_slop_one::jev::{Jev, JevProvider};
use qlty_slop_one::{content_key, MeasurementCache, Model};
use qlty_slop_one_trends::pipeline::Silent;
use qlty_slop_one_trends::run::{ContentResult, Manifest, RunDir};
use qlty_slop_one_trends::scoring::{score_pending, status};
use qlty_slop_one_trends::Error;
use serde_json::{json, Value};
use tempfile::TempDir;

const SUFFIX: &str = ".py";
const UNCACHED: &str = "def other():\n    return 1\n";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/scoring")
        .join(name)
}

fn cached_text() -> String {
    fs::read_to_string(fixture("simple.py")).unwrap()
}

fn one_job() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}

/// A frozen run with one prepared content per text, plus the offline Jev
/// client and measurement cache that score it.
struct Fixture {
    _root: TempDir,
    run: RunDir,
    manifest: Manifest,
    jev: Jev,
    cache: MeasurementCache,
    keys: Vec<String>,
}

impl Fixture {
    fn new(texts: &[&str]) -> Self {
        let root = tempfile::tempdir().unwrap();
        let run = RunDir::new(root.path().join("run"));
        let cache_dir = root.path().join("cache");
        fs::create_dir_all(cache_dir.join("jev")).unwrap();
        for entry in fs::read_dir(fixture("cache/jev")).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), cache_dir.join("jev").join(entry.file_name())).unwrap();
        }
        let cache = MeasurementCache::try_new(&cache_dir).unwrap();
        let jev = Jev::try_new(cache_dir, 0.0, JevProvider::TypeSafe, true).unwrap();

        let mut versions = serde_json::Map::new();
        let mut keys = Vec::new();
        for (index, text) in texts.iter().enumerate() {
            let key = content_key(SUFFIX, text).unwrap();
            let prepared = run.prepared_path(&key, SUFFIX);
            fs::create_dir_all(prepared.parent().unwrap()).unwrap();
            fs::write(&prepared, text).unwrap();
            versions.insert(
                format!("{index:040}{SUFFIX}"),
                json!({
                    "suffix": SUFFIX,
                    "bytes": text.len(),
                    "source_sha256": "0".repeat(64),
                    "measurement_key": key,
                }),
            );
            keys.push(key);
        }
        let manifest = manifest(cache.identity(), Value::Object(versions));
        manifest.save(&run.manifest_path()).unwrap();
        Self {
            _root: root,
            run,
            manifest,
            jev,
            cache,
            keys,
        }
    }

    fn score(&self, retry_errors: bool) -> qlty_slop_one_trends::scoring::ScoringSummary {
        score_pending(
            &self.run,
            &self.manifest,
            &self.jev,
            &self.cache,
            one_job(),
            retry_errors,
            &Silent,
        )
        .unwrap()
    }

    fn result(&self, index: usize) -> ContentResult {
        self.run.load_result(&self.keys[index]).unwrap()
    }

    fn write_result(&self, index: usize, result: &Value) {
        let path = self.run.result_path(&self.keys[index]);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string(result).unwrap()).unwrap();
    }

    fn error_result(&self, index: usize) -> Value {
        json!({
            "error": "Earlier failure",
            "schema_version": 2,
            "key": self.keys[index],
            "model_sha256": Model::shared().unwrap().sha256(),
            "score_scale": "threshold-centered-1-10-v1",
            "measurement_identity": self.cache.identity(),
        })
    }
}

fn manifest(identity: &str, versions: Value) -> Manifest {
    serde_json::from_value(json!({
        "run_schema_version": 2,
        "interval": "week",
        "repository_path": "/repo",
        "cache_directory": "/cache",
        "since": "2026-01-05",
        "ref": "HEAD",
        "extended_from": null,
        "measurement_identity": identity,
        "created_at": "2026-09-21T00:00:00+00:00",
        "project": "fixture",
        "repository": "https://example.com/fixture",
        "main_commit": "0".repeat(40),
        "first_commit": "0".repeat(40),
        "first_commit_at": "2026-01-01T00:00:00+00:00",
        "timezone": "UTC",
        "first_parent_commits": 1,
        "nonmonotonic_commit_dates": 0,
        "timestamp_policy": "",
        "snapshot_policy": "",
        "change_formula": "",
        "model": {},
        "implementation": {},
        "qlty": {"version": "qlty", "sha256": "", "extractor_profile": "qlty-actual-magnitudes-001"},
        "budget_usd": null,
        "snapshots": [],
        "versions": versions,
        "scope": "",
        "source_exclusion_policy": {},
    }))
    .unwrap()
}

#[test]
fn scores_a_pending_content_and_writes_its_result() {
    let fixture = Fixture::new(&[&cached_text()]);
    let summary = fixture.score(false);
    let result = fixture.result(0);
    assert_eq!(summary.total, 1);
    assert_eq!(summary.errors, 0);
    assert_eq!(result.key, fixture.keys[0]);
    assert_eq!(result.schema_version, 2);
    assert_eq!(result.language.as_deref(), Some("Python"));
    assert_eq!(result.passed, Some(true));
    assert_eq!(result.features.unwrap().len(), 39);
    assert_eq!(result.code_lines, Some(7));
    assert_eq!(result.measurement_identity, fixture.cache.identity());
    assert_eq!(result.error, None);
}

#[test]
fn a_scored_content_is_also_in_the_measurement_cache() {
    let fixture = Fixture::new(&[&cached_text()]);
    fixture.score(false);
    let cached = fixture.cache.read(&fixture.keys[0]).unwrap().unwrap();
    assert_eq!(Some(cached.score), fixture.result(0).score);
}

#[test]
fn writes_usage_for_the_run() {
    let fixture = Fixture::new(&[&cached_text()]);
    fixture.score(false);
    let usage: Value =
        serde_json::from_str(&fs::read_to_string(fixture.run.usage_path()).unwrap()).unwrap();
    assert_eq!(usage["requests"], 0);
}

#[test]
fn skips_contents_that_already_have_a_result() {
    let fixture = Fixture::new(&[&cached_text()]);
    let mut saved = fixture.error_result(0);
    saved["error"] = Value::Null;
    saved.as_object_mut().unwrap().remove("error");
    saved["score"] = json!(9.5);
    saved["sentinel"] = json!(true);
    fixture.write_result(0, &saved);
    let summary = fixture.score(false);
    assert_eq!(summary.total, 0);
    assert_eq!(fixture.result(0).extra["sentinel"], json!(true));
}

#[test]
fn records_an_error_result_when_a_content_cannot_be_scored() {
    let fixture = Fixture::new(&[UNCACHED]);
    let summary = fixture.score(false);
    let result = fixture.result(0);
    assert_eq!(summary.errors, 1);
    assert!(result.is_error());
    assert_eq!(
        result.error.as_deref(),
        Some("No cached Jev answers for this file; run once without --offline.")
    );
    assert_eq!(result.score, None);
    assert_eq!(result.key, fixture.keys[0]);
}

#[test]
fn error_results_are_kept_unless_retried() {
    let fixture = Fixture::new(&[&cached_text()]);
    fixture.write_result(0, &fixture.error_result(0));
    let summary = fixture.score(false);
    assert_eq!(summary.total, 0);
    assert_eq!(fixture.result(0).error.as_deref(), Some("Earlier failure"));
}

#[test]
fn retry_errors_rescores_error_results() {
    let fixture = Fixture::new(&[&cached_text()]);
    fixture.write_result(0, &fixture.error_result(0));
    let summary = fixture.score(true);
    let result = fixture.result(0);
    assert_eq!(summary.total, 1);
    assert_eq!(result.error, None);
    assert!(result.score.is_some());
}

#[test]
fn rejects_a_result_scored_by_another_model() {
    let fixture = Fixture::new(&[&cached_text()]);
    let mut saved = fixture.error_result(0);
    saved["model_sha256"] = json!("f".repeat(64));
    fixture.write_result(0, &saved);
    let error = score_pending(
        &fixture.run,
        &fixture.manifest,
        &fixture.jev,
        &fixture.cache,
        one_job(),
        false,
        &Silent,
    )
    .unwrap_err();
    assert!(matches!(error, Error::Mismatch(_)));
}

#[test]
fn rejects_a_manifest_frozen_under_another_identity() {
    let fixture = Fixture::new(&[&cached_text()]);
    let stale = manifest("stale", Value::Object(serde_json::Map::new()));
    let error = score_pending(
        &fixture.run,
        &stale,
        &fixture.jev,
        &fixture.cache,
        one_job(),
        false,
        &Silent,
    )
    .unwrap_err();
    assert!(matches!(error, Error::Frozen(_)));
}

#[test]
fn rejects_a_prepared_file_that_does_not_match_its_key() {
    let fixture = Fixture::new(&[&cached_text()]);
    fs::write(
        fixture.run.prepared_path(&fixture.keys[0], SUFFIX),
        "x = 2\n",
    )
    .unwrap();
    let error = score_pending(
        &fixture.run,
        &fixture.manifest,
        &fixture.jev,
        &fixture.cache,
        one_job(),
        false,
        &Silent,
    )
    .unwrap_err();
    assert!(matches!(error, Error::Mismatch(_)));
}

#[test]
fn scores_several_contents_in_parallel() {
    let fixture = Fixture::new(&[&cached_text(), UNCACHED]);
    let summary = score_pending(
        &fixture.run,
        &fixture.manifest,
        &fixture.jev,
        &fixture.cache,
        NonZeroUsize::new(4).unwrap(),
        false,
        &Silent,
    )
    .unwrap();
    assert_eq!(summary.total, 2);
    assert_eq!(summary.errors, 1);
    assert!(fixture.result(0).score.is_some());
    assert!(fixture.result(1).is_error());
}

#[test]
fn status_counts_results_and_errors() {
    let fixture = Fixture::new(&[&cached_text(), UNCACHED, "y = 3\n"]);
    fixture.write_result(1, &fixture.error_result(1));
    fixture.write_result(0, &fixture.error_result(0));
    let mut scored = fixture.error_result(0);
    scored.as_object_mut().unwrap().remove("error");
    scored["score"] = json!(7.0);
    fixture.write_result(0, &scored);
    let status = status(&fixture.run).unwrap();
    assert_eq!(status.measured, 2);
    assert_eq!(status.total, 3);
    assert_eq!(status.errors, 1);
}
