//! The report data contracts: `<name>.json` (run metadata and per-period
//! summaries and changes), `<name>-files.json.gz` (the same periods with
//! every file row), and `<name>-exclusions.json` (the exclusion audit).
//! A port of slopdetect's `weekly_report.export`.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::change::{summary, History};
use crate::error::{Error, Result};
use crate::periods::Interval;
use crate::report::{
    AnalysisError, FileRow, FilesExport, Period, ReportData, SourceExclusionCounts, Totals,
};
use crate::run::{
    write_json, write_json_gz, ContentResult, ExcludedPath, FileEntry, Manifest, RunDir, Snapshot,
    Version,
};

const MATCHING_POLICY: &str = "Same path; Git renames at 50% similarity; unambiguous directory/crate renames supported by >=3 Git-confirmed peers; then conservative exact-line split/merge inference: >=12 uniquely attributable nontrivial lines, >=65% target overlap, >=2x runner-up evidence. Endpoints without a path/rename match must retain >=65% line coverage across the proposed group, on both sides; partial mappings are rejected and listed under unresolved_movements. Matching evidence is saved per group. Copies from unchanged files remain additions. Rename-only sensitivity uses same paths and Git-confirmed renames only.";

/// The files one export run wrote, with the data they hold.
#[derive(Clone, Debug)]
pub struct Exported {
    pub data: ReportData,
    pub files: FilesExport,
    pub data_path: PathBuf,
    pub files_path: PathBuf,
    pub exclusions_path: Option<PathBuf>,
}

/// Writes `<name>.json`, `<name>-files.json.gz`, and, when the run records a
/// source exclusion policy, `<name>-exclusions.json` into `output_dir`.
///
/// Fails with [`Error::Pending`] while any measurement is missing, so a
/// partial report is never exported.
pub fn export(
    run: &RunDir,
    manifest: &Manifest,
    output_dir: &Path,
    name: &str,
) -> Result<Exported> {
    let keys = manifest.measurement_keys();
    let pending = keys.iter().filter(|key| !run.has_result(key)).count();
    if pending > 0 {
        return Err(Error::Pending(pending));
    }
    let measurements: HashMap<&str, ContentResult> = keys
        .iter()
        .map(|key| Ok((*key, run.load_result(key)?)))
        .collect::<Result<_>>()?;
    let model_sha256 = manifest
        .model
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Mismatch("the run manifest records no model sha256".to_owned()))?;
    let audited = !manifest.source_exclusion_policy.is_null();
    let history = History::new(run, &manifest.repository_path);

    let mut periods: Vec<Period> = Vec::with_capacity(manifest.snapshots.len());
    for snapshot in &manifest.snapshots {
        let rows = snapshot
            .files
            .iter()
            .map(|entry| file_row(entry, manifest, &measurements, model_sha256))
            .collect::<Result<Vec<_>>>()?;
        let mut period = period(snapshot, rows, audited);
        if let Some(previous) = periods.last() {
            let mut change = history.change(previous, &period, true)?;
            let simple = history.change(previous, &period, false)?;
            change.rename_only_sensitivity = Some(Totals {
                positive: simple.positive,
                negative: simple.negative,
                net: simple.net,
            });
            period.change = Some(change);
        }
        tracing::info!(
            interval = manifest.interval.as_str(),
            start = period.start(),
            files = period.summary.files,
            net = ?period.change.as_ref().and_then(|change| change.net),
            "compared snapshot"
        );
        periods.push(period);
    }

    let manifest_sha256 = hex_sha256(&fs::read(run.manifest_path())?);
    let mut metadata = metadata(run, manifest, &measurements, keys.len(), &manifest_sha256)?;
    let mut exclusions_path = None;
    if audited {
        let audit_name = format!("{name}-exclusions.json");
        metadata.insert(
            "source_exclusion_audit".to_owned(),
            Value::from(audit_name.as_str()),
        );
        let path = output_dir.join(&audit_name);
        write_json(&path, &exclusion_audit(manifest, &manifest_sha256))?;
        exclusions_path = Some(path);
    }

    let data = ReportData {
        interval: manifest.interval,
        metadata,
        periods: periods.iter().map(without_files).collect(),
    };
    let files = FilesExport {
        model: manifest.model.clone(),
        interval: manifest.interval,
        periods,
    };
    let data_path = output_dir.join(format!("{name}.json"));
    let files_path = output_dir.join(format!("{name}-files.json.gz"));
    write_json(&data_path, &data.to_value()?)?;
    write_json_gz(&files_path, &files.to_value()?)?;
    tracing::info!(
        periods = files.periods.len(),
        interval = manifest.interval.as_str(),
        "exported"
    );
    Ok(Exported {
        data,
        files,
        data_path,
        files_path,
        exclusions_path,
    })
}

/// A snapshot entry joined with its version and measurement.
fn file_row(
    entry: &FileEntry,
    manifest: &Manifest,
    measurements: &HashMap<&str, ContentResult>,
    model_sha256: &str,
) -> Result<FileRow> {
    let version: &Version = manifest.versions.get(&entry.version).ok_or_else(|| {
        Error::Mismatch(format!(
            "snapshot entry {} refers to unknown version {}",
            entry.path, entry.version
        ))
    })?;
    let mut row = FileRow {
        path: entry.path.clone(),
        blob: entry.blob.clone(),
        suffix: entry.suffix.clone(),
        version: entry.version.clone(),
        source_sha256: version.source_sha256.clone(),
        source_scope: version.source_scope.clone(),
        score: None,
        score_scale: None,
        passed: None,
        language: None,
        code_lines: None,
        cyclomatic: None,
        mass: None,
        error: None,
        measurement_key: None,
        skipped: None,
    };
    if let Some(key) = &version.measurement_key {
        let measured = measurements
            .get(key.as_str())
            .ok_or_else(|| Error::Mismatch(format!("measurement {key} was not loaded")))?;
        if measured.model_sha256 != model_sha256 {
            return Err(Error::Mismatch(format!(
                "measurement {key} was scored with model {} instead of {model_sha256}",
                measured.model_sha256
            )));
        }
        row.score = measured.score;
        row.score_scale = Some(measured.score_scale.clone());
        row.passed = measured.passed;
        row.language = measured.language.clone();
        row.code_lines = measured.code_lines;
        row.cyclomatic = measured.cyclomatic;
        row.mass = measured.mass;
        row.error = measured.error.clone();
        row.measurement_key = Some(key.clone());
    } else if let Some(error) = &version.error {
        row.error = Some(error.clone());
    } else {
        row.skipped = version.skipped.clone();
    }
    Ok(row)
}

/// The period record for a snapshot, before its change is attached.
fn period(snapshot: &Snapshot, rows: Vec<FileRow>, audited: bool) -> Period {
    let analysis_errors = rows
        .iter()
        .filter_map(|row| {
            row.error.as_ref().map(|error| AnalysisError {
                path: row.path.clone(),
                error: error.clone(),
            })
        })
        .collect();
    Period {
        bounds: snapshot.bounds.clone(),
        as_of: snapshot.as_of.clone(),
        partial: snapshot.partial,
        commit: snapshot.commit.clone(),
        committed_at: snapshot.committed_at.clone(),
        first_parent_index: snapshot.first_parent_index,
        summary: summary(&rows),
        files: Some(rows),
        test_paths_excluded: snapshot.skipped.len(),
        test_exclusions: Some(
            snapshot
                .skipped
                .iter()
                .map(|skipped| serde_json::to_value(skipped).unwrap_or(Value::Null))
                .collect(),
        ),
        source_exclusions: audited.then(|| source_exclusion_counts(&snapshot.source_excluded)),
        analysis_errors,
        change: None,
    }
}

fn source_exclusion_counts(excluded: &[ExcludedPath]) -> SourceExclusionCounts {
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for path in excluded {
        let category = path
            .exclusion
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or("");
        *by_category.entry(category.to_owned()).or_default() += 1;
    }
    SourceExclusionCounts {
        files: excluded.len(),
        by_category,
    }
}

fn without_files(period: &Period) -> Period {
    Period {
        files: None,
        test_exclusions: None,
        ..period.clone()
    }
}

/// The manifest without its snapshots and versions, plus the export's own
/// provenance, usage, counts, and the matching policy and limitations text.
fn metadata(
    run: &RunDir,
    manifest: &Manifest,
    measurements: &HashMap<&str, ContentResult>,
    analyzed_contents: usize,
    manifest_sha256: &str,
) -> Result<Map<String, Value>> {
    let Value::Object(mut metadata) = serde_json::to_value(manifest)? else {
        return Err(Error::Mismatch(
            "the run manifest is not an object".to_owned(),
        ));
    };
    metadata.remove("snapshots");
    metadata.remove("versions");
    let frequency = manifest.interval.as_str();
    let schema_version = match manifest.interval {
        Interval::Week => 1,
        Interval::Month => 2,
    };
    metadata.insert("schema_version".to_owned(), Value::from(schema_version));
    metadata.insert(
        "generated_at".to_owned(),
        Value::from(Utc::now().to_rfc3339_opts(SecondsFormat::Micros, false)),
    );
    metadata.insert("manifest_sha256".to_owned(), Value::from(manifest_sha256));
    metadata.insert(
        "report_tool_sha256".to_owned(),
        Value::from(hex_sha256(env!("CARGO_PKG_VERSION").as_bytes())),
    );
    metadata.insert("usage".to_owned(), usage(run, manifest)?);
    metadata.insert(
        "unique_raw_versions".to_owned(),
        Value::from(manifest.versions.len()),
    );
    metadata.insert(
        "unique_analyzed_contents".to_owned(),
        Value::from(analyzed_contents),
    );
    metadata.insert(
        "reused_scores".to_owned(),
        Value::from(count_with(measurements, "reused_from")),
    );
    metadata.insert(
        "reused_assessments".to_owned(),
        Value::from(count_with(measurements, "assessment_source")),
    );
    metadata.insert("matching_policy".to_owned(), Value::from(MATCHING_POLICY));
    let mut limitations = vec![
        "Retrospective evaluation with one fixed model. These scores were not available at the historical commit dates.".to_owned(),
        "Positive/negative bars describe retained code; additions/removals are reported separately. Their net does not equal the change in project average or maintainable percentage.".to_owned(),
        format!("First snapshot is the baseline. Current {frequency} is partial. Period endpoints do not count changes reverted within the period."),
        "Model trained on Java; accuracy for languages other than Java is unvalidated. Cached Jev results hold unchanged content stable; no inference-noise deadband has been fitted.".to_owned(),
        "Analysis errors and zero-mass groups are explicitly excluded from comparisons, not assigned zero quality.".to_owned(),
        "Split/merge matching is a conservative heuristic. Rename-only sensitivity is included; unmatched movements may remain additions/removals.".to_owned(),
        "Local score improvements can include delegation or moving implementation into dependencies. This report does not measure the quality of the full dependency graph.".to_owned(),
        "No extra generated/vendor/example exclusions. Default test exclusions can leave test infrastructure in production-shaped paths.".to_owned(),
    ];
    if !manifest.source_exclusion_policy.is_null() {
        limitations.retain(|note| !note.starts_with("No extra generated/"));
        limitations.push("Exclusions use conventions, explicit metadata and reviewed project rules; they do not prove authorship. Every exclusion is recorded per snapshot.".to_owned());
        limitations.push("The chart toggle adds mass / previous project mass * (score - pass cutoff) for added files and the opposite for removed files. This is not a difference in project average.".to_owned());
    }
    metadata.insert("limitations".to_owned(), Value::from(limitations));
    Ok(metadata)
}

/// The Jev usage ledger summary saved with the run, or zeros when the run
/// made no requests.
fn usage(run: &RunDir, manifest: &Manifest) -> Result<Value> {
    let path = run.usage_path();
    if path.is_file() {
        return Ok(serde_json::from_str(&fs::read_to_string(path)?)?);
    }
    let mut usage = Map::new();
    usage.insert("attempts".to_owned(), Value::from(0));
    usage.insert("cost_upper_bound_usd".to_owned(), Value::from(0));
    usage.insert("reported_input_tokens".to_owned(), Value::from(0));
    usage.insert("unconfirmed_attempts".to_owned(), Value::from(0));
    usage.insert(
        "budget_usd".to_owned(),
        serde_json::to_value(manifest.budget_usd)?,
    );
    Ok(Value::Object(usage))
}

fn count_with(measurements: &HashMap<&str, ContentResult>, key: &str) -> usize {
    measurements
        .values()
        .filter(|result| result.extra.contains_key(key))
        .count()
}

/// Every exclusion of every snapshot, for review.
fn exclusion_audit(manifest: &Manifest, manifest_sha256: &str) -> Value {
    let start_key = match manifest.interval {
        Interval::Week => "week_start",
        Interval::Month => "period_start",
    };
    let records: Vec<Value> = manifest
        .snapshots
        .iter()
        .map(|snapshot| {
            let mut record = Map::new();
            record.insert(start_key.to_owned(), Value::from(snapshot.bounds.start()));
            record.insert("commit".to_owned(), Value::from(snapshot.commit.as_str()));
            record.insert(
                "test_exclusions".to_owned(),
                serde_json::to_value(&snapshot.skipped).unwrap_or(Value::Null),
            );
            record.insert(
                "source_exclusions".to_owned(),
                serde_json::to_value(&snapshot.source_excluded).unwrap_or(Value::Null),
            );
            record.insert(
                "other_extensions".to_owned(),
                serde_json::to_value(&snapshot.other_extensions).unwrap_or(Value::Null),
            );
            record.insert(
                "excluded_modes".to_owned(),
                serde_json::to_value(&snapshot.excluded_modes).unwrap_or(Value::Null),
            );
            Value::Object(record)
        })
        .collect();
    let mut audit = Map::new();
    audit.insert(
        "policy".to_owned(),
        manifest.source_exclusion_policy.clone(),
    );
    audit.insert("manifest_sha256".to_owned(), Value::from(manifest_sha256));
    audit.insert(
        "interval".to_owned(),
        Value::from(manifest.interval.as_str()),
    );
    audit.insert(
        manifest.interval.records_key().to_owned(),
        Value::from(records),
    );
    Value::Object(audit)
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_lowercase_hex() {
        assert_eq!(
            hex_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn exclusion_counts_group_by_category() {
        let excluded = vec![
            ExcludedPath {
                path: "a".to_owned(),
                blob: None,
                suffix: None,
                version: None,
                source_sha256: None,
                exclusion: serde_json::json!({"category": "test"}),
            },
            ExcludedPath {
                path: "b".to_owned(),
                blob: None,
                suffix: None,
                version: None,
                source_sha256: None,
                exclusion: serde_json::json!({"category": "generated"}),
            },
            ExcludedPath {
                path: "c".to_owned(),
                blob: None,
                suffix: None,
                version: None,
                source_sha256: None,
                exclusion: serde_json::json!({"category": "test"}),
            },
        ];
        let counts = source_exclusion_counts(&excluded);
        assert_eq!(counts.files, 3);
        assert_eq!(
            counts.by_category,
            BTreeMap::from([("generated".to_owned(), 1), ("test".to_owned(), 2)])
        );
    }
}
