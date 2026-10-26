//! Parity with slopdetect's Python exports of the frozen Interface runs.
//!
//! These tests are ignored by default. Run them with
//! `SLOPDETECT_DIR=<slopdetect checkout> INTERFACE_REPO=<interface checkout>
//! cargo test -p qlty-slop-one-trends --test change_parity -- --ignored`.

mod common;

use std::env;
use std::path::PathBuf;

use common::{assert_same_json, json_differences};
use qlty_slop_one_trends::chart_data::{chart_periods, drilldown_periods};
use qlty_slop_one_trends::export::export;
use qlty_slop_one_trends::report::{FilesExport, ReportData};
use qlty_slop_one_trends::run::{read_json, read_json_gz, RunDir};
use serde_json::{Map, Value};

const METADATA_KEYS: [&str; 7] = [
    "project",
    "repository",
    "main_commit",
    "timezone",
    "since",
    "unique_raw_versions",
    "unique_analyzed_contents",
];

/// Fields whose Python value depends on the hash-seeded iteration order of
/// same-path groups (a sequential `+=` over groups), so they only agree to
/// rounding.
const ORDER_DEPENDENT_SUMS: [&str; 2] = [
    "compared_previous_mass",
    "comparison_coverage_of_retained_scored_mass",
];

struct Fixture {
    slopdetect: PathBuf,
    interface: PathBuf,
}

impl Fixture {
    fn from_env() -> Option<Self> {
        let slopdetect = env::var_os("SLOPDETECT_DIR")?;
        let interface =
            env::var_os("INTERFACE_REPO").expect("INTERFACE_REPO points at the Interface checkout");
        Some(Self {
            slopdetect: PathBuf::from(slopdetect),
            interface: PathBuf::from(interface),
        })
    }

    fn run(&self, name: &str) -> RunDir {
        RunDir::new(self.slopdetect.join(".ai/local").join(name))
    }

    fn result(&self, file: &str) -> PathBuf {
        self.slopdetect.join(".ai/results").join(file)
    }

    fn python_export(&self, name: &str) -> (Value, Value) {
        let data: Value = read_json(&self.result(&format!("{name}.json"))).unwrap();
        let files: Value = read_json_gz(&self.result(&format!("{name}-files.json.gz"))).unwrap();
        (data, files)
    }
}

fn records_key(data: &Value) -> &'static str {
    match data.get("interval").and_then(Value::as_str) {
        Some("month") => "periods",
        _ => "weeks",
    }
}

fn export_matches_python(name: &str) {
    let Some(fixture) = Fixture::from_env() else {
        eprintln!("skipping {name}: set SLOPDETECT_DIR");
        return;
    };
    let run = fixture.run(name);
    let mut manifest = run.load_manifest().unwrap();
    manifest.repository_path = fixture.interface.to_string_lossy().into_owned();
    let output = tempfile::tempdir().unwrap();
    let exported = export(&run, &manifest, output.path(), name).unwrap();

    let (expected_data, expected_files) = fixture.python_export(name);
    let actual_data: Value = read_json(&exported.data_path).unwrap();
    let actual_files: Value = read_json_gz(&exported.files_path).unwrap();
    let key = records_key(&expected_data);

    for field in METADATA_KEYS {
        assert_eq!(expected_data[field], actual_data[field], "metadata {field}");
    }

    let expected_periods = expected_data[key].as_array().unwrap();
    let actual_periods = actual_data[key].as_array().unwrap();
    assert_eq!(expected_periods.len(), actual_periods.len(), "period count");
    for (index, (expected, actual)) in expected_periods.iter().zip(actual_periods).enumerate() {
        let context = format!("{name} period {index} ({})", expected["commit"]);
        for field in [
            "summary",
            "test_paths_excluded",
            "source_exclusions",
            "analysis_errors",
        ] {
            assert_same_json(
                &format!("{context} {field}"),
                &expected[field],
                &actual[field],
            );
        }
        compare_change(&context, &expected["change"], &actual["change"]);
    }

    let expected_rows = expected_files[key].as_array().unwrap();
    let actual_rows = actual_files[key].as_array().unwrap();
    assert_eq!(
        expected_rows.len(),
        actual_rows.len(),
        "file export period count"
    );
    for (index, (expected, actual)) in expected_rows.iter().zip(actual_rows).enumerate() {
        let context = format!("{name} file rows of period {index}");
        assert_same_json(&context, &expected["files"], &actual["files"]);
        assert_same_json(
            &format!("{context} test exclusions"),
            &expected["test_exclusions"],
            &actual["test_exclusions"],
        );
    }
    assert_same_json(
        &format!("{name} model"),
        &expected_files["model"],
        &actual_files["model"],
    );
}

fn compare_change(context: &str, expected: &Value, actual: &Value) {
    if expected.is_null() {
        assert!(
            actual.is_null(),
            "{context}: expected the baseline to have no change"
        );
        return;
    }
    let (Value::Object(expected), Value::Object(actual)) = (expected, actual) else {
        panic!("{context}: change is not an object");
    };
    let mut expected = expected.clone();
    let mut actual = actual.clone();
    for field in ORDER_DEPENDENT_SUMS {
        let expected_value = expected.remove(field).and_then(|value| value.as_f64());
        let actual_value = actual.remove(field).and_then(|value| value.as_f64());
        match (expected_value, actual_value) {
            (Some(expected_value), Some(actual_value)) => {
                assert!(
                    (expected_value - actual_value).abs() <= 1e-9 * expected_value.abs().max(1.0),
                    "{context} {field}: {expected_value} != {actual_value}"
                );
                if expected_value != actual_value {
                    eprintln!("{context} {field} differs in rounding only: {expected_value} vs {actual_value}");
                }
            }
            (None, None) => {}
            _ => panic!("{context} {field}: {expected_value:?} != {actual_value:?}"),
        }
    }
    sort_groups(&mut expected, "uncompared_groups");
    sort_groups(&mut actual, "uncompared_groups");
    sort_tied_contributions(&mut expected);
    sort_tied_contributions(&mut actual);
    assert_same_json(
        &format!("{context} change"),
        &Value::Object(expected),
        &Value::Object(actual),
    );
}

/// Python breaks ties between equal absolute contributions by its
/// hash-seeded group order; order ties by path on both sides instead.
fn sort_tied_contributions(change: &mut Map<String, Value>) {
    if let Some(Value::Array(contributions)) = change.get_mut("contributions") {
        contributions.sort_by(|a, b| {
            let magnitude = |record: &Value| record["contribution"].as_f64().unwrap_or(0.0).abs();
            magnitude(b)
                .total_cmp(&magnitude(a))
                .then_with(|| {
                    a["before_paths"]
                        .to_string()
                        .cmp(&b["before_paths"].to_string())
                })
                .then_with(|| {
                    a["after_paths"]
                        .to_string()
                        .cmp(&b["after_paths"].to_string())
                })
        });
    }
}

/// Python lists uncompared groups in hash-seeded set order; compare them as a set.
fn sort_groups(change: &mut Map<String, Value>, field: &str) {
    if let Some(Value::Array(groups)) = change.get_mut(field) {
        groups.sort_by_key(|group| group.to_string());
    }
}

fn chart_matches_verification(name: &str) {
    let Some(fixture) = Fixture::from_env() else {
        eprintln!("skipping {name}: set SLOPDETECT_DIR");
        return;
    };
    let (data, files) = fixture.python_export(name);
    let verification: Value =
        read_json(&fixture.result(&format!("{name}-verification.json"))).unwrap();
    let data = ReportData::from_value(data).unwrap();
    let files = FilesExport::from_value(files).unwrap();
    let chart = chart_periods(&data, &files).unwrap();
    let periods = verification["periods"].as_array().unwrap();
    assert_eq!(chart.len(), periods.len(), "{name}: chart period count");
    let mut differences = Vec::new();
    for (index, (record, expected)) in chart.iter().zip(periods).enumerate() {
        let existing = serde_json::json!({
            "positive": record.positive,
            "negative": record.negative,
            "net": record.net,
        });
        differences.extend(
            json_differences(&expected["existing_files"], &existing)
                .into_iter()
                .map(|difference| format!("period {index} existing files {difference}")),
        );
        differences.extend(
            json_differences(
                &expected["with_added_removed_files"],
                &serde_json::to_value(&record.with_file_changes).unwrap(),
            )
            .into_iter()
            .map(|difference| format!("period {index} with file changes {difference}")),
        );
    }
    assert!(differences.is_empty(), "{name}: {}", differences.join("\n"));
}

fn drilldown_counts_match_explanations(name: &str) {
    let Some(fixture) = Fixture::from_env() else {
        eprintln!("skipping {name}: set SLOPDETECT_DIR");
        return;
    };
    let (data, files) = fixture.python_export(name);
    let explanations: Value =
        read_json_gz(&fixture.result(&format!("{name}-explanations.json.gz"))).unwrap();
    let key = records_key(&explanations);
    let data = ReportData::from_value(data).unwrap();
    let files = FilesExport::from_value(files).unwrap();
    let drilldown = drilldown_periods(&data, &files).unwrap();
    let expected = explanations[key].as_array().unwrap();
    assert_eq!(
        drilldown.len(),
        expected.len(),
        "{name}: drilldown period count"
    );
    for (index, (period, explained)) in drilldown.iter().zip(expected).enumerate() {
        assert_eq!(
            period.commit, explained["commit"],
            "{name}: period {index} commit"
        );
        assert_eq!(
            period.rows.len(),
            explained["rows"].as_array().unwrap().len(),
            "{name}: period {index} row count"
        );
    }
}

#[test]
#[ignore = "needs SLOPDETECT_DIR and INTERFACE_REPO"]
fn weekly_export_matches_python() {
    export_matches_python("interface-weekly-003");
}

#[test]
#[ignore = "needs SLOPDETECT_DIR and INTERFACE_REPO"]
fn monthly_export_matches_python() {
    export_matches_python("interface-monthly-001");
}

#[test]
#[ignore = "needs SLOPDETECT_DIR"]
fn weekly_chart_matches_verification() {
    chart_matches_verification("interface-weekly-003");
}

#[test]
#[ignore = "needs SLOPDETECT_DIR"]
fn monthly_chart_matches_verification() {
    chart_matches_verification("interface-monthly-001");
}

#[test]
#[ignore = "needs SLOPDETECT_DIR"]
fn weekly_drilldown_counts_match_explanations() {
    drilldown_counts_match_explanations("interface-weekly-003");
}

#[test]
#[ignore = "needs SLOPDETECT_DIR"]
fn monthly_drilldown_counts_match_explanations() {
    drilldown_counts_match_explanations("interface-monthly-001");
}
