//! A frozen run: its manifest, per-content results, and directory layout.
//! Field names and order follow the slopdetect run schema (version 2).

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::Result;
use crate::periods::Interval;

/// A snapshot's calendar bounds; the key names depend on the interval.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PeriodBounds {
    Week {
        week_start: String,
        week_end_exclusive: String,
    },
    Month {
        period_start: String,
        period_end_exclusive: String,
    },
}

impl PeriodBounds {
    /// The ISO date the period starts on.
    pub fn start(&self) -> &str {
        match self {
            Self::Week { week_start, .. } => week_start,
            Self::Month { period_start, .. } => period_start,
        }
    }

    pub fn end_exclusive(&self) -> &str {
        match self {
            Self::Week {
                week_end_exclusive, ..
            } => week_end_exclusive,
            Self::Month {
                period_end_exclusive,
                ..
            } => period_end_exclusive,
        }
    }
}

/// A tracked source file in one snapshot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub blob: String,
    pub suffix: String,
    /// `blob + suffix`, the key into `Manifest::versions`.
    pub version: String,
}

/// A path skipped as a test file.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SkippedPath {
    pub path: String,
    pub reason: Value,
}

/// A path excluded as generated, dependency, documentation, or by project rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExcludedPath {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    pub exclusion: Value,
}

/// One period's frozen commit and the files it contains.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Snapshot {
    #[serde(flatten)]
    pub bounds: PeriodBounds,
    pub as_of: String,
    pub partial: bool,
    pub commit: String,
    pub committed_at: String,
    pub first_parent_index: usize,
    pub files: Vec<FileEntry>,
    pub skipped: Vec<SkippedPath>,
    pub source_excluded: Vec<ExcludedPath>,
    pub other_extensions: BTreeMap<String, usize>,
    pub excluded_modes: BTreeMap<String, usize>,
}

/// One unique blob version and how it was prepared for analysis.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Version {
    pub suffix: String,
    pub bytes: usize,
    pub source_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_scope: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_bytes: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpts: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The measurement tooling recorded in a run.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct QltyIdentity {
    pub version: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,
    pub extractor_profile: String,
}

/// The frozen run manifest. Unknown metadata keys round-trip through `extra`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Manifest {
    pub run_schema_version: u32,
    #[serde(default)]
    pub interval: Interval,
    pub repository_path: String,
    pub cache_directory: String,
    pub since: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub extended_from: Option<Value>,
    pub measurement_identity: String,
    pub created_at: String,
    pub project: String,
    pub repository: String,
    pub main_commit: String,
    pub first_commit: String,
    pub first_commit_at: String,
    pub timezone: String,
    pub first_parent_commits: usize,
    pub nonmonotonic_commit_dates: usize,
    pub timestamp_policy: String,
    pub snapshot_policy: String,
    pub change_formula: String,
    pub model: Value,
    pub implementation: Value,
    pub qlty: QltyIdentity,
    pub budget_usd: Option<f64>,
    pub snapshots: Vec<Snapshot>,
    pub versions: BTreeMap<String, Version>,
    pub scope: String,
    pub source_exclusion_policy: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self> {
        read_json(path)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        write_json(path, self)
    }

    /// Distinct analyzed contents, in key order.
    pub fn measurement_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self
            .versions
            .values()
            .filter_map(|version| version.measurement_key.as_deref())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        keys
    }
}

/// A scored content, or its analysis error. Schema version 2.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContentResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_lines: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cyclomatic: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_lines: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub schema_version: u32,
    pub key: String,
    pub model_sha256: String,
    pub score_scale: String,
    pub measurement_identity: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl ContentResult {
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// The on-disk layout of one run.
#[derive(Clone, Debug)]
pub struct RunDir {
    root: PathBuf,
}

impl RunDir {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.root.join("manifest.json")
    }

    pub fn result_path(&self, key: &str) -> PathBuf {
        self.root.join("results").join(format!("{key}.json"))
    }

    pub fn prepared_path(&self, key: &str, suffix: &str) -> PathBuf {
        self.root.join("prepared").join(format!("{key}{suffix}"))
    }

    pub fn source_path(&self, version: &str) -> PathBuf {
        self.root.join("sources").join(version)
    }

    pub fn usage_path(&self) -> PathBuf {
        self.root.join("usage.json")
    }

    pub fn load_manifest(&self) -> Result<Manifest> {
        Manifest::load(&self.manifest_path())
    }

    pub fn load_result(&self, key: &str) -> Result<ContentResult> {
        read_json(&self.result_path(key))
    }

    pub fn has_result(&self, key: &str) -> bool {
        self.result_path(key).is_file()
    }
}

/// Reads a JSON file into `T`.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

/// Writes pretty JSON with a trailing newline, atomically.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    let temporary = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}

/// Writes compact JSON, gzip-compressed with a zero mtime so output is reproducible.
pub fn write_json_gz<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_vec(value)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&payload)?;
    fs::write(path, encoder.finish()?)?;
    Ok(())
}

/// Reads gzip-compressed JSON.
pub fn read_json_gz<T: DeserializeOwned>(path: &Path) -> Result<T> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut text = String::new();
    GzDecoder::new(fs::File::open(path)?).read_to_string(&mut text)?;
    Ok(serde_json::from_str(&text)?)
}
