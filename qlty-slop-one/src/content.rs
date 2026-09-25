//! Score one prepared source text by content, with a content-keyed cache.
//! Used by the trend reports and by the file scorer.
//!
//! A content is a prepared text plus its file suffix. Its key is slopdetect's
//! `digest([suffix, text])`, so keys match the frozen slopdetect runs. Scores
//! are cached under `<cache_dir>/measurements/<identity>/<key>.json`, where the
//! identity fixes everything that can change a score: the model, the Qlty
//! build, the extractor profile, and the Jev questions.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use qlty_analysis::code::File;
use qlty_analysis::workspace_entries::TargetMode;
use qlty_config::version::{GIT_COMMIT_QLTY, QLTY_VERSION};
use qlty_smells::metrics::{Executor, Planner, Settings};
use qlty_types::analysis::v1::ComponentType;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::error::{Error, Result};
use crate::features::questions;
use crate::jev::{digest, Jev, JevFeatures};
use crate::language::Language;
use crate::measure::{self, Measurement};
use crate::model::{Model, EXTRACTOR_PROFILE};

/// The largest file Qlty's workspace walker reads, in bytes
/// (`qlty_analysis::walker::MAX_FILE_SIZE`). `qlty metrics` never reported a
/// larger file, so slopdetect recorded it as unparsed.
const QLTY_MAX_FILE_BYTES: usize = 2_098_000;

/// A scored content: everything the trend reports need per unique text.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContentScore {
    pub score: f64,
    pub passed: bool,
    pub language: String,
    pub code_lines: u64,
    pub cyclomatic: u64,
    /// `cyclomatic × √code_lines`
    pub mass: f64,
    /// The 39 model inputs in feature order.
    pub features: Vec<f64>,
    /// Per-rule smell summaries keyed by rule key.
    pub summary: serde_json::Map<String, serde_json::Value>,
    /// Physical line count of the analyzed text, at least 1.
    pub source_lines: usize,
}

/// A content's score together with the evidence an explanation needs.
#[derive(Clone, Debug)]
pub struct ScoredContent {
    pub score: ContentScore,
    pub measurement: Measurement,
    pub jev: JevFeatures,
}

/// slopdetect's content key: `digest([suffix, text])`.
pub fn content_key(suffix: &str, text: &str) -> Result<String> {
    digest(&json!([suffix, text]))
}

/// `cyclomatic × √code_lines`, in Python's operand order.
pub fn mass(cyclomatic: u64, code_lines: u64) -> f64 {
    cyclomatic as f64 * (code_lines as f64).sqrt()
}

/// Scores a prepared text as a file with `suffix`.
pub fn score_content(text: &str, suffix: &str, jev: &Jev) -> Result<ContentScore> {
    Ok(evaluate_content(text, suffix, jev)?.score)
}

/// Scores a prepared text and keeps the measurement and Jev evidence.
pub fn evaluate_content(text: &str, suffix: &str, jev: &Jev) -> Result<ScoredContent> {
    if text.len() > QLTY_MAX_FILE_BYTES {
        return Err(Error::Unparsed);
    }
    let language = Language::detect(suffix, text)?;
    let measurement = Measurement::measure(text, language)?;
    let jev_features = jev.extract(text, language)?;
    let mut features = measurement.values.clone();
    features.extend_from_slice(&jev_features.values);
    let prediction = Model::shared()?.predict(&features)?;
    let metrics = FileMetrics::measure(text, language)?;
    let summary = summary_json(&measurement)?;
    let score = ContentScore {
        score: prediction.score,
        passed: prediction.passed,
        language: language.display_name().to_owned(),
        code_lines: metrics.code_lines,
        cyclomatic: metrics.cyclomatic,
        mass: mass(metrics.cyclomatic, metrics.code_lines),
        features,
        summary,
        source_lines: measurement.source_lines,
    };
    Ok(ScoredContent {
        score,
        measurement,
        jev: jev_features,
    })
}

/// [`score_content`] through `cache`: a valid cached score is returned as
/// is, and a fresh score is stored before it is returned.
pub fn score_content_cached(
    text: &str,
    suffix: &str,
    jev: &Jev,
    cache: &MeasurementCache,
) -> Result<ContentScore> {
    score_content_via_cache(text, suffix, jev, cache).map(|(score, _)| score)
}

/// [`score_content_cached`], also saying whether the score came from the
/// cache (`true`) or was computed now.
pub fn score_content_via_cache(
    text: &str,
    suffix: &str,
    jev: &Jev,
    cache: &MeasurementCache,
) -> Result<(ContentScore, bool)> {
    let key = content_key(suffix, text)?;
    if let Some(score) = cache.read(&key)? {
        return Ok((score, true));
    }
    let score = score_content(text, suffix, jev)?;
    cache.write(&key, &score)?;
    Ok((score, false))
}

fn summary_json(measurement: &Measurement) -> Result<Map<String, Value>> {
    let mut summary = Map::new();
    for (rule, rule_summary) in &measurement.summaries {
        summary.insert(rule.key().to_owned(), serde_json::to_value(rule_summary)?);
    }
    Ok(summary)
}

/// The two file metrics `qlty metrics --all --json` reported for one file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileMetrics {
    code_lines: u64,
    cyclomatic: u64,
}

impl FileMetrics {
    /// Runs Qlty's file-level metrics executor as `qlty metrics --all` did:
    /// without test-syntax filters.
    fn measure(text: &str, language: Language) -> Result<Self> {
        let file = Arc::new(File::from_string(language.qlty_name(), text));
        let settings = Settings {
            functions: false,
            exclude_tests: false,
            target_mode: TargetMode::All,
        };
        let plan = Planner::new(measure::config(), &settings, vec![file])
            .compute()
            .map_err(Error::Qlty)?;
        let results = Executor::new(&plan).execute();
        let stats = results
            .stats
            .into_iter()
            .find(|stats| stats.kind == i32::from(ComponentType::File))
            .ok_or(Error::Unparsed)?;
        let (Some(code_lines), Some(cyclomatic)) = (stats.code_lines, stats.cyclomatic) else {
            return Err(Error::Analysis("Qlty reported no file metrics.".to_owned()));
        };
        Ok(Self {
            code_lines: u64::from(code_lines),
            cyclomatic: u64::from(cyclomatic),
        })
    }
}

static TEMPORARY_FILES: AtomicU64 = AtomicU64::new(0);

/// One cache entry as stored on disk, with the identity it is valid for.
#[derive(Deserialize, Serialize)]
struct CacheEntry {
    key: String,
    measurement_identity: String,
    #[serde(flatten)]
    score: ContentScore,
}

/// The content-keyed score cache under `<cache_dir>/measurements/<identity>/`.
#[derive(Clone, Debug)]
pub struct MeasurementCache {
    dir: PathBuf,
    identity: String,
}

impl MeasurementCache {
    /// A cache for the current model and Qlty build under `cache_dir`.
    pub fn try_new(cache_dir: &Path) -> Result<Self> {
        let identity = measurement_identity()?;
        Ok(Self {
            dir: cache_dir.join("measurements").join(&identity),
            identity,
        })
    }

    /// The digest of everything that decides a score: the model, the Qlty
    /// version and commit, the extractor profile, and the Jev questions.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The cached score for `key`, or `None` when there is none. An entry
    /// that cannot be parsed or that was written for another key or identity
    /// is an error.
    pub fn read(&self, key: &str) -> Result<Option<ContentScore>> {
        let path = self.path(key);
        if !path.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(&path).map_err(|_| unreadable(&path))?;
        let entry: CacheEntry = serde_json::from_str(&text).map_err(|_| unreadable(&path))?;
        if entry.key != key || entry.measurement_identity != self.identity {
            return Err(Error::Cache(
                "Measurement cache identity mismatch.".to_owned(),
            ));
        }
        validate_score(&entry.score).map_err(|_| unreadable(&path))?;
        Ok(Some(entry.score))
    }

    /// Stores `score` for `key` atomically: a temporary file in the same
    /// directory, then a rename.
    pub fn write(&self, key: &str, score: &ContentScore) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        let path = self.path(key);
        let sequence = TEMPORARY_FILES.fetch_add(1, Ordering::Relaxed);
        let temporary = self
            .dir
            .join(format!("{key}.json.{}.{sequence}.tmp", process::id()));
        let entry = CacheEntry {
            key: key.to_owned(),
            measurement_identity: self.identity.clone(),
            score: score.clone(),
        };
        let mut text = serde_json::to_string_pretty(&entry)
            .map_err(|_| Error::Cache(format!("Cannot write JSON file: {}", path.display())))?;
        text.push('\n');
        let outcome = fs::write(&temporary, text).and_then(|()| fs::rename(&temporary, &path));
        if outcome.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        Ok(outcome?)
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.json"))
    }
}

fn measurement_identity() -> Result<String> {
    let model = Model::shared()?;
    digest(&json!({
        "model_sha256": model.sha256(),
        "qlty": { "version": QLTY_VERSION, "commit": GIT_COMMIT_QLTY },
        "extractor_profile": EXTRACTOR_PROFILE,
        "questions": questions(),
    }))
}

/// Rejects a stored score whose numbers could not have come from the model.
fn validate_score(score: &ContentScore) -> Result<()> {
    let model = Model::shared()?;
    let finite = score.score.is_finite()
        && score.mass.is_finite()
        && score.features.iter().all(|value| value.is_finite());
    if !finite || score.features.len() != model.reference().len() || score.source_lines == 0 {
        return Err(Error::Cache("Invalid cached measurement.".to_owned()));
    }
    Ok(())
}

fn unreadable(path: &Path) -> Error {
    Error::Cache(format!("Cannot read JSON file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jev::JevProvider;

    const KEY: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn score() -> ContentScore {
        ContentScore {
            score: 4.158994576673668,
            passed: false,
            language: "TypeScript".to_owned(),
            code_lines: 198,
            cyclomatic: 18,
            mass: mass(18, 198),
            features: vec![0.5; 39],
            summary: json!({"boolean-logic": {"count": 0, "locations": []}})
                .as_object()
                .cloned()
                .unwrap(),
            source_lines: 241,
        }
    }

    fn offline_jev(dir: &Path) -> Jev {
        Jev::try_new(dir.to_path_buf(), 0.0, JevProvider::TypeSafe, true).unwrap()
    }

    #[test]
    fn content_key_matches_python_digest_for_ascii() {
        assert_eq!(
            content_key(".py", "x = 1\n").unwrap(),
            "255631b64073072f4dcb0b4ba1f1c90d57d860c63b52484497ab1ac61b267cd3"
        );
    }

    #[test]
    fn content_key_matches_python_digest_for_rust() {
        assert_eq!(
            content_key(".rs", "fn main() {}\n").unwrap(),
            "6144c1470b4854785583f25ca4f38cea59afd0e274776fb998b7f8d68a7a5907"
        );
    }

    #[test]
    fn content_key_matches_python_digest_for_non_ascii() {
        assert_eq!(
            content_key(".ts", "const λ = \"日本\";\n").unwrap(),
            "619b0160dcd385ab7087f8b9288502b852adb52437ff5ead0c9c7e0ba5c646c3"
        );
    }

    #[test]
    fn content_key_matches_python_digest_for_an_empty_suffix_and_controls() {
        assert_eq!(
            content_key("", "plain\ttext\r\n").unwrap(),
            "35838120818e25cd774d2607b38c3b7bfdd925b87cff147c919b5936e58f5376"
        );
    }

    #[test]
    fn mass_is_cyclomatic_times_root_of_code_lines() {
        assert_eq!(mass(18, 198), 18.0 * 198f64.sqrt());
        assert_eq!(mass(18, 198), 253.28245103046518);
    }

    #[test]
    fn mass_of_an_empty_file_is_zero() {
        assert_eq!(mass(0, 0), 0.0);
        assert_eq!(mass(3, 0), 0.0);
    }

    #[test]
    fn file_metrics_count_code_lines_and_branches() {
        let text = "def f(a):\n    # comment\n    if a:\n        return 1\n    return 2\n";
        let metrics = FileMetrics::measure(text, Language::Python).unwrap();
        assert_eq!(
            metrics,
            FileMetrics {
                code_lines: 4,
                cyclomatic: 2,
            }
        );
    }

    #[test]
    fn file_metrics_keep_test_code() {
        let text = "def test_f():\n    if True:\n        assert f(1) == 1\n";
        let metrics = FileMetrics::measure(text, Language::Python).unwrap();
        assert_eq!(metrics.cyclomatic, 2);
    }

    #[test]
    fn oversized_text_is_unparsed_like_qlty_metrics() {
        let dir = tempfile::tempdir().unwrap();
        let text = format!("x = 1\n{}", "# padding\n".repeat(QLTY_MAX_FILE_BYTES / 10));
        let error = score_content(&text, ".py", &offline_jev(dir.path())).unwrap_err();
        assert!(matches!(error, Error::Unparsed));
    }

    #[test]
    fn identity_is_a_sha256_hex_digest() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        assert_eq!(cache.identity().len(), 64);
        assert!(cache
            .identity()
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn missing_entry_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        assert_eq!(cache.read(KEY).unwrap(), None);
    }

    #[test]
    fn round_trips_a_score() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        cache.write(KEY, &score()).unwrap();
        assert_eq!(cache.read(KEY).unwrap(), Some(score()));
    }

    #[test]
    fn stores_entries_under_the_identity_directory() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        cache.write(KEY, &score()).unwrap();
        let path = dir
            .path()
            .join("measurements")
            .join(cache.identity())
            .join(format!("{KEY}.json"));
        let document: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(document["key"], KEY);
        assert_eq!(document["measurement_identity"], cache.identity());
        assert_eq!(document["cyclomatic"], 18);
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn rejects_an_entry_written_for_another_identity() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        cache.write(KEY, &score()).unwrap();
        let path = cache.path(KEY);
        let text = fs::read_to_string(&path)
            .unwrap()
            .replace(cache.identity(), "stale");
        fs::write(&path, text).unwrap();
        assert_eq!(
            cache.read(KEY).unwrap_err().to_string(),
            "Measurement cache identity mismatch."
        );
    }

    #[test]
    fn rejects_an_entry_whose_key_does_not_match() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        let other = "1".repeat(64);
        cache.write(&other, &score()).unwrap();
        fs::rename(cache.path(&other), cache.path(KEY)).unwrap();
        assert_eq!(
            cache.read(KEY).unwrap_err().to_string(),
            "Measurement cache identity mismatch."
        );
    }

    #[test]
    fn rejects_an_unparsable_entry() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        fs::create_dir_all(&cache.dir).unwrap();
        fs::write(cache.path(KEY), "nope").unwrap();
        assert!(matches!(cache.read(KEY), Err(Error::Cache(_))));
    }

    #[test]
    fn rejects_an_entry_with_the_wrong_feature_count() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        let mut short = score();
        short.features.truncate(38);
        cache.write(KEY, &short).unwrap();
        assert!(matches!(cache.read(KEY), Err(Error::Cache(_))));
    }

    #[test]
    fn cached_scoring_misses_then_hits() {
        let dir = tempfile::tempdir().unwrap();
        let cache = MeasurementCache::try_new(dir.path()).unwrap();
        let jev = offline_jev(dir.path());
        let text = "x = 1\n";
        assert!(matches!(
            score_content_cached(text, ".py", &jev, &cache),
            Err(Error::Offline)
        ));
        let key = content_key(".py", text).unwrap();
        cache.write(&key, &score()).unwrap();
        assert_eq!(
            score_content_cached(text, ".py", &jev, &cache).unwrap(),
            score()
        );
    }
}
