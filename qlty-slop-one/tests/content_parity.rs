//! Parity of `score_content` with the frozen slopdetect Interface run: a
//! deterministic sample of its saved results is re-scored offline from the
//! same prepared sources and Jev cache and compared bit for bit.
//!
//! The one exception is `score`: slopdetect's `migrate_actual_magnitudes.py`
//! wrote the saved scores from a numpy matrix product and only checked them
//! against `Model.predict` within 1e-10. `Model.predict` itself (and this
//! port, which matches it bit for bit) differs from those saved values by a
//! few ULPs on about one content in seven, so scores are compared within the
//! migration's own tolerance.
//!
//! Unix only because the two Jev caches are merged with symbolic links.

#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qlty_slop_one::jev::{Jev, JevProvider};
use qlty_slop_one::source::source_text;
use qlty_slop_one::{content_key, score_content, ContentScore};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use tempfile::TempDir;

const RUN: &str = ".ai/local/interface-weekly-003";
/// The Interface run's Jev answers were collected under two cache
/// directories; the test reads them through one merged directory.
const JEV_CACHES: [&str; 2] = [".ai/local/lithos-llm-cache", ".ai/local/weekly-cache"];
const SAMPLE_STRIDE: usize = 30;
const COMMON_LANGUAGES: [&str; 3] = ["JavaScript", "TypeScript", "Tsx"];
/// The tolerance `migrate_actual_magnitudes.py` accepted between its saved
/// scores and `Model.predict`.
const SCORE_TOLERANCE: f64 = 1e-10;

fn slopdetect_dir() -> Option<PathBuf> {
    std::env::var_os("SLOPDETECT_DIR").map(PathBuf::from)
}

/// A saved schema 2 result, scored or failed.
#[derive(Deserialize)]
struct SavedResult {
    key: String,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    passed: Option<bool>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    code_lines: Option<u64>,
    #[serde(default)]
    cyclomatic: Option<u64>,
    #[serde(default)]
    mass: Option<f64>,
    #[serde(default)]
    features: Option<Vec<f64>>,
    #[serde(default)]
    summary: Option<Value>,
    #[serde(default)]
    source_lines: Option<usize>,
}

fn saved_results(run: &Path) -> Vec<SavedResult> {
    let mut paths: Vec<PathBuf> = fs::read_dir(run.join("results"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    paths
        .par_iter()
        .map(|path| serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap())
        .collect()
}

/// Content key to file suffix, from the prepared directory listing.
fn suffixes(run: &Path) -> BTreeMap<String, String> {
    fs::read_dir(run.join("prepared"))
        .unwrap()
        .map(|entry| {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            let (key, suffix) = name.split_at(64);
            (key.to_owned(), suffix.to_owned())
        })
        .collect()
}

/// Every 30th key in sorted order, every result in an uncommon language,
/// and every error result.
fn sample(results: &[SavedResult]) -> Vec<&SavedResult> {
    results
        .iter()
        .enumerate()
        .filter(|(index, result)| {
            index % SAMPLE_STRIDE == 0
                || result.error.is_some()
                || result
                    .language
                    .as_deref()
                    .is_some_and(|language| !COMMON_LANGUAGES.contains(&language))
        })
        .map(|(_, result)| result)
        .collect()
}

fn prepared_path(run: &Path, key: &str, suffix: &str) -> PathBuf {
    run.join("prepared").join(format!("{key}{suffix}"))
}

/// One directory of symbolic links to every cached Jev answer, so one
/// offline client sees both caches.
fn merged_jev_cache(root: &Path) -> TempDir {
    let merged = tempfile::tempdir().unwrap();
    let jev = merged.path().join("jev");
    fs::create_dir(&jev).unwrap();
    for cache in JEV_CACHES {
        for entry in fs::read_dir(root.join(cache).join("jev")).unwrap() {
            let entry = entry.unwrap();
            let link = jev.join(entry.file_name());
            if !link.exists() {
                std::os::unix::fs::symlink(entry.path(), link).unwrap();
            }
        }
    }
    merged
}

fn bits_differ(expected: Option<f64>, actual: f64) -> bool {
    expected.map(f64::to_bits) != Some(actual.to_bits())
}

/// The fields of `actual` that differ from the saved result, with both values.
fn mismatched_fields(expected: &SavedResult, actual: &ContentScore) -> Vec<String> {
    let mut fields = Vec::new();
    let score_close = expected
        .score
        .is_some_and(|score| (score - actual.score).abs() <= SCORE_TOLERANCE);
    if !score_close {
        fields.push(format!("score {:?} != {:?}", expected.score, actual.score));
    }
    if expected.passed != Some(actual.passed) {
        fields.push(format!("passed {:?} != {}", expected.passed, actual.passed));
    }
    if expected.language.as_deref() != Some(actual.language.as_str()) {
        fields.push(format!(
            "language {:?} != {}",
            expected.language, actual.language
        ));
    }
    if expected.code_lines != Some(actual.code_lines) {
        fields.push(format!(
            "code_lines {:?} != {}",
            expected.code_lines, actual.code_lines
        ));
    }
    if expected.cyclomatic != Some(actual.cyclomatic) {
        fields.push(format!(
            "cyclomatic {:?} != {}",
            expected.cyclomatic, actual.cyclomatic
        ));
    }
    if bits_differ(expected.mass, actual.mass) {
        fields.push(format!("mass {:?} != {:?}", expected.mass, actual.mass));
    }
    let features_match = expected.features.as_ref().is_some_and(|features| {
        features.len() == actual.features.len()
            && features
                .iter()
                .zip(&actual.features)
                .all(|(a, b)| a.to_bits() == b.to_bits())
    });
    if !features_match {
        fields.push(format!(
            "features {:?} != {:?}",
            expected.features, actual.features
        ));
    }
    if expected.summary.as_ref() != Some(&Value::Object(actual.summary.clone())) {
        fields.push("summary".to_owned());
    }
    if expected.source_lines != Some(actual.source_lines) {
        fields.push(format!(
            "source_lines {:?} != {}",
            expected.source_lines, actual.source_lines
        ));
    }
    fields
}

#[test]
#[ignore = "requires SLOPDETECT_DIR with the frozen Interface run and its Jev cache"]
fn sampled_interface_contents_score_bit_exactly() {
    let Some(root) = slopdetect_dir() else {
        eprintln!("skipping: set SLOPDETECT_DIR");
        return;
    };
    let run = root.join(RUN);
    let results = saved_results(&run);
    let suffixes = suffixes(&run);
    let jev_cache = merged_jev_cache(&root);
    let jev = Jev::try_new(
        jev_cache.path().to_path_buf(),
        0.0,
        JevProvider::TypeSafe,
        true,
    )
    .unwrap();
    let sample = sample(&results);
    assert!(sample.len() >= 400);

    let mismatches: Vec<String> = sample
        .par_iter()
        .flat_map_iter(|expected| {
            let suffix = &suffixes[&expected.key];
            let text = source_text(&prepared_path(&run, &expected.key, suffix)).unwrap();
            let mut problems = Vec::new();
            if content_key(suffix, &text).unwrap() != expected.key {
                problems.push(format!("{} content_key", expected.key));
            }
            match (score_content(&text, suffix, &jev), &expected.error) {
                (Ok(_), Some(error)) => {
                    problems.push(format!("{} expected error: {error}", expected.key))
                }
                (Err(error), None) => {
                    problems.push(format!("{} unexpected error: {error}", expected.key))
                }
                (Err(_), Some(_)) => {}
                (Ok(actual), None) => problems.extend(
                    mismatched_fields(expected, &actual)
                        .into_iter()
                        .map(|field| format!("{} {field}", expected.key)),
                ),
            }
            problems
        })
        .collect();

    assert_eq!(mismatches, Vec::<String>::new());
}
