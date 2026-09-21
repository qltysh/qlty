//! Bit-for-bit parity of `Model::predict` with slopdetect's Python model.
//!
//! The fixtures were produced by `slopdetect_cli.model.Model.predict` on
//! macOS arm64, the reference platform. There every value must match
//! exactly. On other platforms the same comparisons use an absolute
//! tolerance of 1e-12 as a diagnostic only.

use std::path::PathBuf;

use serde::Deserialize;

use qlty_slop_one::Model;

const THRESHOLD: f64 = 0.6073781128742494;

#[derive(Debug, Deserialize)]
struct Expected {
    #[serde(default)]
    name: String,
    #[serde(default)]
    values: Vec<f64>,
    passed: bool,
    score: f64,
    risk: f64,
    baseline_score: f64,
    feature_contributions: Vec<f64>,
    score_margin: f64,
    explanation_residual: f64,
}

fn fixture_cases() -> Vec<Expected> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model/predictions.json");
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn assert_same(what: &str, actual: f64, expected: f64) {
    assert_eq!(
        (what, actual, actual.to_bits()),
        (what, expected, expected.to_bits())
    );
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
fn assert_same(what: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "{what}: {actual:?} != {expected:?}"
    );
}

fn assert_prediction_matches(model: &Model, name: &str, values: &[f64], expected: &Expected) {
    let prediction = model.predict(values).unwrap();
    assert_eq!((name, prediction.passed), (name, expected.passed));
    assert_same(&format!("{name}: risk"), prediction.risk, expected.risk);
    assert_same(&format!("{name}: score"), prediction.score, expected.score);
    assert_same(
        &format!("{name}: baseline_score"),
        prediction.baseline_score,
        expected.baseline_score,
    );
    assert_same(
        &format!("{name}: score_margin"),
        prediction.score_margin,
        expected.score_margin,
    );
    assert_same(
        &format!("{name}: explanation_residual"),
        prediction.explanation_residual,
        expected.explanation_residual,
    );
    assert_eq!(
        prediction.feature_contributions.len(),
        expected.feature_contributions.len()
    );
    prediction
        .feature_contributions
        .iter()
        .zip(&expected.feature_contributions)
        .enumerate()
        .for_each(|(index, (actual, expected))| {
            assert_same(&format!("{name}: contribution {index}"), *actual, *expected)
        });
}

#[test]
fn embedded_model_matches_the_packaged_document() {
    let model = Model::embedded().unwrap();
    assert_eq!(model.id(), "maintainability-003-actual-magnitudes");
    assert_eq!(model.component_count(), 640);
    assert_eq!(model.reference().len(), 39);
    assert_eq!(model.threshold(), THRESHOLD);
    assert_eq!(model.info().features, 39);
}

#[test]
fn fixture_vectors_cover_both_sides_of_the_cutoff() {
    let cases = fixture_cases();
    assert_eq!(cases.len(), 42);
    assert_eq!(cases.iter().filter(|case| case.passed).count(), 12);
    assert!(cases
        .iter()
        .any(|case| case.name.ends_with("_pass") && case.risk == THRESHOLD.next_down()));
    assert!(cases
        .iter()
        .any(|case| case.name.ends_with("_fail") && case.risk == THRESHOLD));
}

#[test]
fn predictions_match_python_for_every_fixture_vector() {
    let model = Model::embedded().unwrap();
    fixture_cases()
        .iter()
        .for_each(|case| assert_prediction_matches(&model, &case.name, &case.values, case));
}

#[test]
fn reference_vector_has_zero_contributions_and_residual() {
    let model = Model::embedded().unwrap();
    let prediction = model.predict(model.reference()).unwrap();
    assert_eq!(prediction.score, prediction.baseline_score);
    assert_eq!(prediction.explanation_residual, 0.0);
    assert!(prediction
        .feature_contributions
        .iter()
        .all(|contribution| *contribution == 0.0));
}

#[derive(Debug, Deserialize)]
struct TrainingFeatures {
    values: Vec<Vec<f64>>,
}

/// Run with:
/// `SLOPDETECT_DIR=~/p/lithoscomputer/slopdetect cargo test -p qlty-slop-one --test model_parity -- --ignored`
#[test]
#[ignore = "needs a slopdetect checkout with the 304-file benchmark data"]
fn predictions_match_python_for_the_304_benchmark_files() {
    let root = PathBuf::from(std::env::var("SLOPDETECT_DIR").unwrap());
    let features: TrainingFeatures = serde_json::from_str(
        &std::fs::read_to_string(
            root.join(".ai/local/actual-magnitudes-001/training-features.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let expected: Vec<Expected> = serde_json::from_str(
        &std::fs::read_to_string(root.join(".ai/local/slop-one-parity/model-304.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(features.values.len(), 304);
    assert_eq!(expected.len(), 304);
    let model = Model::embedded().unwrap();
    features
        .values
        .iter()
        .zip(&expected)
        .enumerate()
        .for_each(|(index, (values, expected))| {
            assert_prediction_matches(&model, &format!("benchmark file {index}"), values, expected)
        });
}
