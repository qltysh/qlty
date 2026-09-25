//! Parity of the in-process measurement with slopdetect's `qlty.scan`.
//!
//! The committed fixtures were generated with the reference Python
//! implementation and its pinned Qlty build. The ignored benchmark test
//! compares against the 304 training files saved in a slopdetect checkout.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use qlty_slop_one::features::Rule;
use qlty_slop_one::language::Language;
use qlty_slop_one::measure::{LineSpan, Measurement, RuleSummary};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    suffix: String,
    language: String,
    text: String,
    values: Vec<f64>,
    summary: BTreeMap<String, ExpectedSummary>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ExpectedSpan {
    start_line: u32,
    end_line: u32,
}

impl From<LineSpan> for ExpectedSpan {
    fn from(span: LineSpan) -> Self {
        Self {
            start_line: span.start_line,
            end_line: span.end_line,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct ExpectedSummary {
    count: usize,
    max_magnitude: f64,
    covered_lines: usize,
    locations: Vec<ExpectedSpan>,
    max_actual: f64,
    threshold: usize,
    magnitude_basis: String,
    magnitude_unit: String,
}

impl ExpectedSummary {
    fn from_summary(summary: &RuleSummary) -> Self {
        Self {
            count: summary.count,
            max_magnitude: summary.max_magnitude,
            covered_lines: summary.covered_lines,
            locations: summary.locations.iter().copied().map(Into::into).collect(),
            max_actual: summary.max_actual,
            threshold: summary.threshold,
            magnitude_basis: summary.magnitude_basis.to_string(),
            magnitude_unit: summary.magnitude_unit.to_string(),
        }
    }
}

fn fixture(name: &str) -> Fixture {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/measure")
        .join(format!("{name}.json"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn actual_summaries(measurement: &Measurement) -> BTreeMap<String, ExpectedSummary> {
    measurement
        .summaries
        .iter()
        .map(|(rule, summary)| {
            (
                rule.key().to_string(),
                ExpectedSummary::from_summary(summary),
            )
        })
        .collect()
}

fn assert_fixture_parity(name: &str) {
    let fixture = fixture(name);
    let measurement = Measurement::analyze(&fixture.text, &fixture.suffix).unwrap();
    assert_eq!(measurement.language.display_name(), fixture.language);
    assert_eq!(bits(&measurement.values), bits(&fixture.values));
    assert_eq!(actual_summaries(&measurement), fixture.summary);
}

#[test]
fn rust_boolean_depth_matches_reference() {
    assert_fixture_parity("rust_boolean_depth_rs");
}

#[test]
fn rust_nested_control_matches_reference() {
    assert_fixture_parity("rust_nested_control_rs");
}

#[test]
fn rust_duplicated_matches_reference() {
    assert_fixture_parity("rust_duplicated_rs");
}

#[test]
fn rust_clean_matches_reference() {
    assert_fixture_parity("rust_clean_rs");
}

#[test]
fn python_duplication_matches_reference() {
    assert_fixture_parity("python_duplication_py");
}

#[test]
fn python_shebang_script_matches_reference() {
    assert_fixture_parity("python_shebang");
}

#[test]
fn java_mixed_matches_reference() {
    assert_fixture_parity("java_mixed_java");
}

#[test]
fn java_identical_matches_reference() {
    assert_fixture_parity("java_identical_java");
}

#[test]
fn javascript_mixed_matches_reference() {
    assert_fixture_parity("javascript_mixed_js");
}

#[test]
fn rust_boolean_fixture_finds_depth_five_and_six() {
    let fixture = fixture("rust_boolean_depth_rs");
    let measurement = Measurement::measure(&fixture.text, Language::Rust).unwrap();
    let summary = &measurement.summaries[&Rule::BooleanLogic];
    assert_eq!(summary.count, 2);
    assert_eq!(summary.max_actual, 6.0);
    assert_eq!(summary.threshold, 5);
}

#[derive(Deserialize)]
struct TrainingFeatures {
    rows: Vec<TrainingRow>,
    records: Vec<TrainingRecord>,
}

#[derive(Deserialize)]
struct TrainingRow {
    id: String,
}

#[derive(Deserialize)]
struct TrainingRecord {
    values: Vec<f64>,
    summary: BTreeMap<String, ExpectedSummary>,
}

/// slopdetect's `common.py::source_text`: strip a UTF-8 BOM and normalize
/// line endings to `\n`.
fn source_text(path: &Path) -> String {
    let bytes = fs::read(path).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    text.strip_prefix('\u{feff}')
        .unwrap_or(&text)
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

fn training_source(slopdetect_dir: &Path, id: &str) -> PathBuf {
    let directory = slopdetect_dir.join(".research/sources").join(id);
    let mut entries: Vec<PathBuf> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(entries.len(), 1);
    entries.remove(0)
}

fn training_mismatches(slopdetect_dir: &Path) -> Vec<String> {
    let path = slopdetect_dir.join(".ai/local/actual-magnitudes-001/training-features.json");
    let features: TrainingFeatures =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(features.rows.len(), 304);
    assert_eq!(features.records.len(), features.rows.len());

    let mut mismatches = Vec::new();
    for (row, record) in features.rows.iter().zip(&features.records) {
        let source = training_source(slopdetect_dir, &row.id);
        let text = source_text(&source);
        let measurement = match Measurement::analyze(&text, ".java") {
            Ok(measurement) => measurement,
            Err(error) => {
                mismatches.push(format!("{}: {error}", source.display()));
                continue;
            }
        };
        if bits(&measurement.values) != bits(&record.values) {
            mismatches.push(format!(
                "{}: values {:?} != {:?}",
                source.display(),
                measurement.values,
                record.values
            ));
        }
        let summaries = actual_summaries(&measurement);
        if summaries != record.summary {
            mismatches.push(format!(
                "{}: summary {:?} != {:?}",
                source.display(),
                summaries,
                record.summary
            ));
        }
    }
    mismatches
}

#[test]
#[ignore = "needs SLOPDETECT_DIR pointing at a slopdetect checkout with the benchmark data"]
fn training_files_match_slopdetect_features() {
    let Some(slopdetect_dir) = std::env::var_os("SLOPDETECT_DIR").map(PathBuf::from) else {
        eprintln!("skipping: set SLOPDETECT_DIR");
        return;
    };
    let mismatches = training_mismatches(&slopdetect_dir);
    assert_eq!(mismatches, Vec::<String>::new());
}
