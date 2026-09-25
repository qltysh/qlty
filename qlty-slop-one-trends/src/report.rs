//! The report data contracts: the summary export (`<name>.json`), the file
//! export (`<name>-files.json.gz`), the exclusion audit, the score
//! explanations (`<name>-explanations.json.gz`), chart values, drilldown
//! rows, and annotations. Names and order follow the slopdetect exports.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::periods::Interval;
use crate::run::PeriodBounds;

/// Aggregate statistics over the scored files of one snapshot (or of its
/// additions or removals).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Summary {
    pub candidate_files: usize,
    pub files: usize,
    pub fails: usize,
    pub errors: usize,
    pub inline_only_skips: usize,
    pub loc: u64,
    pub mass: f64,
    pub maintainable_fraction: Option<f64>,
    pub median: Option<f64>,
    pub loc_weighted_mean: Option<f64>,
}

/// Evidence for how two paths were matched into one group.
pub type MatchEvidence = Map<String, Value>;

/// One matched group's before/after comparison.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Contribution {
    pub before_paths: Vec<String>,
    pub after_paths: Vec<String>,
    pub before_score: f64,
    pub after_score: f64,
    pub before_mass: f64,
    pub after_mass: f64,
    pub previous_mass_share: f64,
    pub score_change: f64,
    pub contribution: f64,
    pub matching: Vec<MatchEvidence>,
    pub changed: bool,
}

/// A group that could not be compared.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UncomparedGroup {
    pub before_paths: Vec<String>,
    pub after_paths: Vec<String>,
    pub reason: String,
}

/// A rejected split/merge inference.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UnresolvedMovement {
    pub before_paths: Vec<String>,
    pub after_paths: Vec<String>,
    pub reason: String,
    pub coverage: f64,
}

/// Files that entered or left the project in a period.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileSet {
    pub summary: Summary,
    pub files: Vec<String>,
}

/// Positive, negative, and net totals.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Totals {
    pub positive: Option<f64>,
    pub negative: Option<f64>,
    pub net: Option<f64>,
}

/// The quality change from the previous snapshot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Change {
    pub positive: Option<f64>,
    pub negative: Option<f64>,
    pub net: Option<f64>,
    pub compared_groups: usize,
    pub changed_groups: usize,
    pub compared_previous_mass: f64,
    pub previous_project_mass: f64,
    pub comparison_coverage_of_retained_scored_mass: Option<f64>,
    pub contributions: Vec<Contribution>,
    pub uncompared_groups: Vec<UncomparedGroup>,
    pub unresolved_movements: Vec<UnresolvedMovement>,
    pub additions: FileSet,
    pub removals: FileSet,
    pub structural_groups: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rename_only_sensitivity: Option<Totals>,
}

/// One file's row in the file export: the snapshot entry plus its measurement.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileRow {
    pub path: String,
    pub blob: String,
    pub suffix: String,
    pub version: String,
    pub source_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_scope: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_scale: Option<String>,
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
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
}

/// The measured fields of a scored file row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    pub score: f64,
    pub passed: bool,
    pub code_lines: u64,
    pub mass: f64,
}

impl FileRow {
    /// The row's measurement, when it was scored.
    pub fn measurement(&self) -> Option<Measurement> {
        Some(Measurement {
            score: self.score?,
            passed: self.passed?,
            code_lines: self.code_lines?,
            mass: self.mass?,
        })
    }
}

/// An analysis error recorded against a path.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AnalysisError {
    pub path: String,
    pub error: String,
}

/// Counts of source exclusions in one snapshot.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SourceExclusionCounts {
    pub files: usize,
    pub by_category: BTreeMap<String, usize>,
}

/// One period in the exports. The summary export omits `files` and
/// `test_exclusions`; the file export includes them.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Period {
    #[serde(flatten)]
    pub bounds: PeriodBounds,
    pub as_of: String,
    pub partial: bool,
    pub commit: String,
    pub committed_at: String,
    pub first_parent_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileRow>>,
    pub summary: Summary,
    pub test_paths_excluded: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_exclusions: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_exclusions: Option<SourceExclusionCounts>,
    pub analysis_errors: Vec<AnalysisError>,
    pub change: Option<Change>,
}

impl Period {
    pub fn start(&self) -> &str {
        self.bounds.start()
    }
}

/// The summary export `<name>.json`: run metadata plus one period per snapshot.
/// Metadata keys round-trip through `metadata`; the period array is keyed by
/// the interval (`weeks` or `periods`).
#[derive(Clone, Debug, PartialEq)]
pub struct ReportData {
    pub interval: Interval,
    pub metadata: Map<String, Value>,
    pub periods: Vec<Period>,
}

impl ReportData {
    pub fn from_value(value: Value) -> Result<Self, serde_json::Error> {
        let Value::Object(mut map) = value else {
            return Err(serde::de::Error::custom("report data must be an object"));
        };
        let interval = match map.get("interval") {
            Some(value) => serde_json::from_value(value.clone())?,
            None => Interval::Week,
        };
        let periods = map
            .remove(interval.records_key())
            .ok_or_else(|| serde::de::Error::custom("report data has no periods"))?;
        Ok(Self {
            interval,
            metadata: map,
            periods: serde_json::from_value(periods)?,
        })
    }

    pub fn to_value(&self) -> Result<Value, serde_json::Error> {
        let mut map = self.metadata.clone();
        map.insert(
            self.interval.records_key().to_owned(),
            serde_json::to_value(&self.periods)?,
        );
        Ok(Value::Object(map))
    }

    pub fn model(&self) -> &Value {
        self.metadata.get("model").unwrap_or(&Value::Null)
    }

    pub fn repository(&self) -> &str {
        self.metadata
            .get("repository")
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    pub fn project(&self) -> &str {
        self.metadata
            .get("project")
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    pub fn threshold_score(&self) -> f64 {
        self.model()
            .get("threshold_score")
            .and_then(Value::as_f64)
            .unwrap_or(5.0)
    }
}

/// The file export `<name>-files.json.gz`.
#[derive(Clone, Debug, PartialEq)]
pub struct FilesExport {
    pub model: Value,
    pub interval: Interval,
    pub periods: Vec<Period>,
}

impl FilesExport {
    pub fn from_value(value: Value) -> Result<Self, serde_json::Error> {
        let Value::Object(mut map) = value else {
            return Err(serde::de::Error::custom("file export must be an object"));
        };
        let interval: Interval = match map.get("interval") {
            Some(value) => serde_json::from_value(value.clone())?,
            None => Interval::Week,
        };
        let periods = map
            .remove(interval.records_key())
            .ok_or_else(|| serde::de::Error::custom("file export has no periods"))?;
        Ok(Self {
            model: map.remove("model").unwrap_or(Value::Null),
            interval,
            periods: serde_json::from_value(periods)?,
        })
    }

    pub fn to_value(&self) -> Result<Value, serde_json::Error> {
        let mut map = Map::new();
        map.insert("model".to_owned(), self.model.clone());
        map.insert("interval".to_owned(), serde_json::to_value(self.interval)?);
        map.insert(
            self.interval.records_key().to_owned(),
            serde_json::to_value(&self.periods)?,
        );
        Ok(Value::Object(map))
    }
}

/// File entries and exits folded into a period, for the chart toggle.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FileChanges {
    pub positive: Option<f64>,
    pub negative: Option<f64>,
    pub net: Option<f64>,
    pub files: usize,
    pub unscored_files: usize,
}

/// One bar of the chart.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChartPeriod {
    pub label: String,
    pub positive: Option<f64>,
    pub negative: Option<f64>,
    pub net: Option<f64>,
    pub changed_groups: Option<usize>,
    pub additions: Option<FileChanges>,
    pub removals: Option<FileChanges>,
    pub with_file_changes: Totals,
}

/// A file in a drilldown row.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Member {
    pub path: String,
    pub score: Option<f64>,
    pub mass: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_lines: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cyclomatic: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_scope: Option<Value>,
}

/// How a drilldown row's files relate across the period.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ChangeKind {
    Changed,
    Renamed,
    Split,
    Merged,
    Reorganized,
    Added,
    Removed,
}

/// One row of the period drilldown.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DrilldownRow {
    pub kind: ChangeKind,
    pub before: Vec<Member>,
    pub after: Vec<Member>,
    pub before_score: Option<f64>,
    pub after_score: Option<f64>,
    pub before_mass: Option<f64>,
    pub after_mass: Option<f64>,
    pub contribution: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_anchor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<Value>,
}

impl DrilldownRow {
    /// The path the row is listed under: the first after-path, else before.
    pub fn primary_path(&self) -> &str {
        self.after
            .first()
            .or(self.before.first())
            .map_or("", |member| member.path.as_str())
    }
}

/// The drilldown for one period.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DrilldownPeriod {
    pub before_commit: Option<String>,
    pub commit: String,
    pub rows: Vec<DrilldownRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_start: Option<String>,
}

/// A measured quantity in explanation evidence. Counts (findings, duplicated
/// lines) serialize as integers, as Python's `sum` of integers does; every
/// other quantity is a float.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Quantity {
    Count(u64),
    Amount(f64),
}

impl Quantity {
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Count(count) => count as f64,
            Self::Amount(amount) => amount,
        }
    }
}

/// Evidence attached to an explanation factor: a single value, or a
/// before/after pair. A `null` quantity means the measurement does not apply
/// (for example the maximum of no findings). The AI assessment rows carry
/// `decimals: 0, scale: 100` so the reader renders them as percentages.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Evidence {
    Value {
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decimals: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scale: Option<u32>,
        value: Option<Quantity>,
    },
    Change {
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        decimals: Option<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scale: Option<u32>,
        before: Option<Quantity>,
        after: Option<Quantity>,
    },
}

impl Evidence {
    pub fn label(&self) -> &str {
        match self {
            Self::Value { label, .. } | Self::Change { label, .. } => label,
        }
    }
}

/// One leading factor of a score explanation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Factor {
    pub id: String,
    pub name: String,
    pub change: f64,
    pub evidence: Vec<Evidence>,
}

/// A score explanation: either against the model reference (added and
/// removed files) or a before-to-after change.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Explanation {
    Score {
        mode: String,
        score: f64,
        reference_score: f64,
        factors: Vec<Factor>,
        other: f64,
    },
    Change {
        factors: Vec<Factor>,
        other: f64,
        score_change: f64,
    },
}

/// Explanations for one period, aligned with its drilldown rows.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExplanationPeriod {
    pub commit: String,
    pub rows: Vec<Option<Explanation>>,
}

/// The explanation export `<name>-explanations.json.gz`, schema version 3.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplanationsExport {
    pub schema_version: u32,
    pub model_sha256: String,
    pub score_scale: String,
    pub data_sha256: String,
    pub files_sha256: String,
    pub method: String,
    pub questions: BTreeMap<String, String>,
    pub interval: Interval,
    pub periods: Vec<ExplanationPeriod>,
}

impl ExplanationsExport {
    pub fn from_value(value: Value) -> Result<Self, serde_json::Error> {
        #[derive(Deserialize)]
        struct Head {
            schema_version: u32,
            model_sha256: String,
            score_scale: String,
            data_sha256: String,
            files_sha256: String,
            method: String,
            questions: BTreeMap<String, String>,
            #[serde(default)]
            interval: Interval,
        }
        let head: Head = serde_json::from_value(value.clone())?;
        let periods = value
            .get(head.interval.records_key())
            .cloned()
            .ok_or_else(|| serde::de::Error::custom("explanations have no periods"))?;
        Ok(Self {
            schema_version: head.schema_version,
            model_sha256: head.model_sha256,
            score_scale: head.score_scale,
            data_sha256: head.data_sha256,
            files_sha256: head.files_sha256,
            method: head.method,
            questions: head.questions,
            interval: head.interval,
            periods: serde_json::from_value(periods)?,
        })
    }

    pub fn to_value(&self) -> Result<Value, serde_json::Error> {
        let mut map = Map::new();
        map.insert("schema_version".into(), self.schema_version.into());
        map.insert("model_sha256".into(), self.model_sha256.clone().into());
        map.insert("score_scale".into(), self.score_scale.clone().into());
        map.insert("data_sha256".into(), self.data_sha256.clone().into());
        map.insert("files_sha256".into(), self.files_sha256.clone().into());
        map.insert("method".into(), self.method.clone().into());
        map.insert("questions".into(), serde_json::to_value(&self.questions)?);
        map.insert("interval".into(), serde_json::to_value(self.interval)?);
        map.insert(
            self.interval.records_key().to_owned(),
            serde_json::to_value(&self.periods)?,
        );
        Ok(Value::Object(map))
    }
}

/// A reviewed chart annotation from `<name>-annotations.json`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Annotation {
    pub period_start: String,
    pub commit: String,
    pub title: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
