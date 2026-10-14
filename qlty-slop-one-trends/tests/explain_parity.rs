//! Parity of the explanation export with slopdetect's `weekly_explanations.py`.
//!
//! The committed fixture is a two-snapshot run of synthetic measurements over
//! small prepared texts, with `explain-fixture-explanations.json.gz` produced
//! by the Python module. The Interface tests are ignored by default; run them
//! with `SLOPDETECT_DIR=<slopdetect checkout> cargo test -p qlty-slop-one-trends
//! --test explain_parity -- --ignored`. The checked-in slopdetect explanation
//! exports are stale relative to its own module, so the test prefers exports
//! regenerated into `.ai/local/slop-one-parity/results` when that directory
//! exists. `SLOPDETECT_RESULTS` points them at a
//! directory of regenerated Python exports instead of `.ai/results`.

mod common;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use common::{assert_same_json, json_differences};
use qlty_slop_one_trends::explain::export;
use qlty_slop_one_trends::run::{read_json_gz, RunDir};
use qlty_slop_one_trends::Error;
use serde_json::Value;

const FIXTURE: &str = "explain-fixture";

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/explain")
}

/// Copies the two report exports into `output`, so `export` never writes
/// next to the reference files.
fn stage_inputs(source: &Path, output: &Path, name: &str) {
    for file in [format!("{name}.json"), format!("{name}-files.json.gz")] {
        fs::copy(source.join(&file), output.join(&file)).unwrap();
    }
}

fn export_to_temp(run: &RunDir, source: &Path, name: &str) -> (tempfile::TempDir, Value) {
    let output = tempfile::tempdir().unwrap();
    stage_inputs(source, output.path(), name);
    export(run, output.path(), name).unwrap();
    let produced: Value =
        read_json_gz(&output.path().join(format!("{name}-explanations.json.gz"))).unwrap();
    (output, produced)
}

fn records_key(export: &Value) -> &'static str {
    match export.get("interval").and_then(Value::as_str) {
        Some("month") => "periods",
        _ => "weeks",
    }
}

fn explained_rows(export: &Value) -> usize {
    export[records_key(export)]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|period| period["rows"].as_array().unwrap())
        .filter(|row| !row.is_null())
        .count()
}

#[test]
fn fixture_export_matches_python() {
    let run = RunDir::new(fixture_dir().join("run"));
    let (_output, produced) = export_to_temp(&run, &fixture_dir(), FIXTURE);
    let expected: Value =
        read_json_gz(&fixture_dir().join(format!("{FIXTURE}-explanations.json.gz"))).unwrap();
    assert_same_json("fixture explanations", &expected, &produced);
}

#[test]
fn fixture_export_explains_the_visible_rows_only() {
    let run = RunDir::new(fixture_dir().join("run"));
    let (_output, produced) = export_to_temp(&run, &fixture_dir(), FIXTURE);
    let rows = produced["weeks"][1]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(explained_rows(&produced), 4);
    assert!(rows[4].is_null());
    assert_eq!(produced["schema_version"], 3);
    assert_eq!(produced["weeks"][0]["rows"], Value::Array(Vec::new()));
}

#[test]
fn fixture_export_rejects_a_report_from_another_model() {
    let run = RunDir::new(fixture_dir().join("run"));
    let output = tempfile::tempdir().unwrap();
    stage_inputs(&fixture_dir(), output.path(), FIXTURE);
    let data_path = output.path().join(format!("{FIXTURE}.json"));
    let mut data: Value = serde_json::from_str(&fs::read_to_string(&data_path).unwrap()).unwrap();
    data["model"]["sha256"] = Value::from("0".repeat(64));
    fs::write(&data_path, serde_json::to_string(&data).unwrap()).unwrap();
    let error = export(&run, output.path(), FIXTURE).unwrap_err();
    assert!(matches!(error, Error::Mismatch(_)));
}

#[test]
fn fixture_export_rejects_a_missing_measurement() {
    let run = RunDir::new(fixture_dir().join("missing-run"));
    let output = tempfile::tempdir().unwrap();
    stage_inputs(&fixture_dir(), output.path(), FIXTURE);
    let error = export(&run, output.path(), FIXTURE).unwrap_err();
    assert!(matches!(error, Error::Io(_)));
}

fn interface_export_matches_python(name: &str) {
    let Some(slopdetect) = env::var_os("SLOPDETECT_DIR").map(PathBuf::from) else {
        eprintln!("skipping {name}: set SLOPDETECT_DIR");
        return;
    };
    let run = RunDir::new(slopdetect.join(".ai/local").join(name));
    let regenerated = slopdetect.join(".ai/local/slop-one-parity/results");
    let results = env::var_os("SLOPDETECT_RESULTS").map_or_else(
        || {
            if regenerated.is_dir() {
                regenerated.clone()
            } else {
                slopdetect.join(".ai/results")
            }
        },
        PathBuf::from,
    );
    let (_output, produced) = export_to_temp(&run, &results, name);
    let expected: Value =
        read_json_gz(&results.join(format!("{name}-explanations.json.gz"))).unwrap();
    let differences = json_differences(&expected, &produced);
    println!(
        "{name}: {} rows explained by Python, {} by Rust, {} differences",
        explained_rows(&expected),
        explained_rows(&produced),
        differences.len()
    );
    for difference in differences.iter().take(40) {
        println!("  {difference}");
    }
    assert_eq!(explained_rows(&expected), explained_rows(&produced));
    assert!(differences.is_empty());
}

#[test]
#[ignore = "needs a slopdetect checkout with the frozen Interface runs"]
fn interface_weekly_explanations_match_python() {
    interface_export_matches_python("interface-weekly-003");
}

#[test]
#[ignore = "needs a slopdetect checkout with the frozen Interface runs"]
fn interface_monthly_explanations_match_python() {
    interface_export_matches_python("interface-monthly-001");
}
