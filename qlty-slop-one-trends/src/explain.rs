//! Score explanations for the drilldown rows, from saved measurements only.
//!
//! An exact port of slopdetect's `weekly_explanations.py`: no Qlty execution,
//! Jev requests, or retraining. A changed group is explained along the
//! straight before-to-after feature path (`Model::explain_between`),
//! mass-weighted across its member pairs; an added or removed file is
//! explained against the model's training reference. Every sum that Python
//! computed with `math.fsum` uses [`fsum`].

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::rc::Rc;

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};

use qlty_slop_one::features::{questions, Rule, QLTY_FEATURE_COUNT, RULES};
use qlty_slop_one::fsum::fsum;
use qlty_slop_one::jev::digest;
use qlty_slop_one::pylines::line_count;
use qlty_slop_one::scoring::SCORE_SCALE;
use qlty_slop_one::Model;

use crate::chart_data::drilldown_periods;
use crate::error::{Error, Result};
use crate::report::{
    DrilldownPeriod, DrilldownRow, Evidence, Explanation, ExplanationPeriod, ExplanationsExport,
    Factor, FileRow, FilesExport, Member, Quantity, ReportData,
};
use crate::run::{read_json_gz, write_json_gz, RunDir};

/// The `method` text of the explanation export, verbatim from slopdetect.
pub const METHOD: &str = "Original straight-path feature contributions rescaled between each pair of endpoint scores; mass-weighted pairs for groups. Added/removed files use the model training reference.";

const SCHEMA_VERSION: u32 = 3;
const AI_ASSESSMENT: &str = "AI assessment";
const CYCLOMATIC_COMPLEXITY: &str = "Cyclomatic complexity";
const MEASUREMENT_TOLERANCE: f64 = 1e-9;
const RECONCILIATION_TOLERANCE: f64 = 1e-8;
const SHOWN_FACTOR_MINIMUM: f64 = 0.005;
const SHOWN_FACTOR_COUNT: usize = 5;

/// The fields of a saved smell summary that evidence uses.
#[derive(Clone, Debug, Deserialize)]
struct SmellSummary {
    count: u64,
    max_magnitude: f64,
    covered_lines: u64,
    #[serde(default = "legacy_basis")]
    magnitude_basis: String,
}

fn legacy_basis() -> String {
    "legacy".to_owned()
}

/// A frozen measurement whose score the packaged model reproduces.
#[derive(Clone, Debug)]
struct Measurement {
    values: Vec<f64>,
    summary: BTreeMap<Rule, SmellSummary>,
    source_lines: u64,
    score: f64,
}

/// A group member: a verified measurement with the file's snapshot mass.
#[derive(Clone, Debug)]
struct GroupFile {
    measurement: Rc<Measurement>,
    mass: f64,
}

impl GroupFile {
    fn smell(&self, rule: Rule) -> &SmellSummary {
        self.measurement
            .summary
            .get(&rule)
            .expect("Measurement::read should keep a summary for every rule")
    }
}

/// Loads and verifies the saved measurements of a run, once per content key.
pub struct Measurements<'a> {
    model: &'a Model,
    run: &'a RunDir,
    cache: HashMap<String, Rc<Measurement>>,
}

impl<'a> Measurements<'a> {
    pub fn new(model: &'a Model, run: &'a RunDir) -> Self {
        Self {
            model,
            run,
            cache: HashMap::new(),
        }
    }

    /// The number of distinct measurements loaded so far.
    pub fn loaded(&self) -> usize {
        self.cache.len()
    }

    fn load(&mut self, key: &str, suffix: &str) -> Result<Rc<Measurement>> {
        if let Some(measurement) = self.cache.get(key) {
            return Ok(Rc::clone(measurement));
        }
        let measurement = Rc::new(self.read(key, suffix)?);
        self.cache.insert(key.to_owned(), Rc::clone(&measurement));
        Ok(measurement)
    }

    fn read(&self, key: &str, suffix: &str) -> Result<Measurement> {
        let saved = self.run.load_result(key)?;
        if saved.score_scale != SCORE_SCALE {
            return Err(mismatch(
                "Saved measurement uses an unsupported score scale",
            ));
        }
        if saved.model_sha256 != self.model.sha256() {
            return Err(mismatch("Saved measurement uses a different model"));
        }
        let text = fs::read_to_string(self.run.prepared_path(key, suffix))?;
        if digest(&json!([suffix, text]))? != key {
            return Err(mismatch("Prepared source identity mismatch"));
        }
        if saved.schema_version != 2 {
            return Err(mismatch(
                "This run needs self-contained measurements; re-evaluate with the generic pipeline",
            ));
        }
        let values = saved
            .features
            .ok_or_else(|| mismatch("Saved measurement has no features"))?;
        let summaries = saved
            .summary
            .ok_or_else(|| mismatch("Saved measurement has no summary"))?;
        let summary = RULES
            .iter()
            .map(|rule| {
                let value = summaries
                    .get(rule.key())
                    .ok_or_else(|| mismatch(&format!("Saved summary lacks {}", rule.key())))?;
                Ok((*rule, serde_json::from_value(value.clone())?))
            })
            .collect::<Result<BTreeMap<Rule, SmellSummary>>>()?;
        let source_lines = line_count(&text).max(1);
        if saved.source_lines != Some(source_lines) {
            return Err(mismatch("Measurement source length differs"));
        }
        let predicted = self.model.predict(&values)?.score;
        let saved_score = saved
            .score
            .ok_or_else(|| mismatch("Saved measurement has no score"))?;
        if !is_close(predicted, saved_score, 0.0, MEASUREMENT_TOLERANCE) {
            return Err(mismatch(
                "Reconstructed score differs from the frozen measurement",
            ));
        }
        Ok(Measurement {
            values,
            summary,
            source_lines: source_lines as u64,
            score: predicted,
        })
    }

    fn members(
        &mut self,
        entries: &[Member],
        files: &HashMap<&str, &FileRow>,
    ) -> Result<Vec<GroupFile>> {
        entries
            .iter()
            .map(|member| {
                let row = files
                    .get(member.path.as_str())
                    .ok_or_else(|| mismatch(&format!("{} is not in the snapshot", member.path)))?;
                let key = row
                    .measurement_key
                    .as_deref()
                    .ok_or_else(|| mismatch(&format!("{} has no measurement", member.path)))?;
                let measurement = self.load(key, &row.suffix)?;
                let score = member
                    .score
                    .ok_or_else(|| mismatch(&format!("{} has no score", member.path)))?;
                let mass = member
                    .mass
                    .ok_or_else(|| mismatch(&format!("{} has no mass", member.path)))?;
                if !is_close(measurement.score, score, 0.0, MEASUREMENT_TOLERANCE) {
                    return Err(mismatch(
                        "Snapshot score differs from the frozen measurement",
                    ));
                }
                Ok(GroupFile { measurement, mass })
            })
            .collect()
    }
}

fn mismatch(message: &str) -> Error {
    Error::Mismatch(message.to_owned())
}

/// Python's `math.isclose`.
fn is_close(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if a == b {
        return true;
    }
    if a.is_infinite() || b.is_infinite() {
        return false;
    }
    let difference = (b - a).abs();
    difference <= (rel_tol * b).abs() || difference <= (rel_tol * a).abs() || difference <= abs_tol
}

fn weighted_members(files: &[GroupFile]) -> Result<Vec<(&GroupFile, f64)>> {
    let total = fsum(files.iter().map(|file| file.mass));
    if total <= 0.0 {
        return Err(mismatch("Cannot explain a group with no mass"));
    }
    Ok(files.iter().map(|file| (file, file.mass / total)).collect())
}

/// Averages the pairwise straight paths, using each side's existing mass
/// weights. This does not claim a one-to-one match between split or merged
/// members; the weighted pair score differences sum to the exact group score
/// difference.
fn group_changes(model: &Model, before: &[GroupFile], after: &[GroupFile]) -> Result<Vec<f64>> {
    let mut pairs = Vec::with_capacity(before.len() * after.len());
    for (a, a_weight) in weighted_members(before)? {
        for (b, b_weight) in weighted_members(after)? {
            let changes = model.explain_between(&a.measurement.values, &b.measurement.values)?;
            pairs.push((a_weight * b_weight, changes));
        }
    }
    Ok((0..model.reference().len())
        .map(|index| fsum(pairs.iter().map(|(weight, values)| weight * values[index])))
        .collect())
}

/// The five observable measurements of a smell over a set of files, in the
/// order evidence lists them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Measure {
    Findings,
    Maximum,
    Lines,
    Density,
    Coverage,
}

const MEASURES: [Measure; 5] = [
    Measure::Findings,
    Measure::Maximum,
    Measure::Lines,
    Measure::Density,
    Measure::Coverage,
];

#[derive(Clone, Copy, Debug)]
struct Observed {
    findings: u64,
    maximum: f64,
    lines: u64,
    density: f64,
    coverage: f64,
}

impl Observed {
    fn of(files: &[GroupFile], rule: Rule) -> Self {
        let source_lines: u64 = files.iter().map(|file| file.measurement.source_lines).sum();
        let findings: u64 = files.iter().map(|file| file.smell(rule).count).sum();
        let lines: u64 = files
            .iter()
            .map(|file| file.smell(rule).covered_lines)
            .sum();
        let maximum = files
            .iter()
            .map(|file| file.smell(rule).max_magnitude)
            .fold(
                f64::NEG_INFINITY,
                |best, value| {
                    if value > best {
                        value
                    } else {
                        best
                    }
                },
            );
        Self {
            findings,
            maximum,
            lines,
            density: (100 * findings) as f64 / source_lines as f64,
            coverage: (100 * lines) as f64 / source_lines as f64,
        }
    }

    fn quantity(&self, measure: Measure) -> Quantity {
        match measure {
            Measure::Findings => Quantity::Count(self.findings),
            Measure::Maximum => Quantity::Amount(self.maximum),
            Measure::Lines => Quantity::Count(self.lines),
            Measure::Density => Quantity::Amount(self.density),
            Measure::Coverage => Quantity::Amount(self.coverage),
        }
    }

    fn has_findings(&self) -> bool {
        self.findings != 0
    }
}

/// The evidence labels of a smell; `None` hides a measurement.
#[derive(Clone, Copy, Debug)]
struct Labels {
    findings: Option<&'static str>,
    maximum: Option<&'static str>,
    lines: Option<&'static str>,
    density: Option<&'static str>,
    coverage: Option<&'static str>,
}

impl Labels {
    fn get(&self, measure: Measure) -> Option<&'static str> {
        match measure {
            Measure::Findings => self.findings,
            Measure::Maximum => self.maximum,
            Measure::Lines => self.lines,
            Measure::Density => self.density,
            Measure::Coverage => self.coverage,
        }
    }
}

fn measurement_labels(rule: Rule, basis: &str) -> Labels {
    let actual = basis == "actual";
    let maximum = match rule {
        Rule::BooleanLogic if actual => Some("Max. Boolean depth"),
        Rule::BooleanLogic => Some("Longest finding (lines)"),
        Rule::NestedControlFlow => Some("Max. nesting depth"),
        Rule::FunctionParameters => Some("Max. flagged parameters"),
        Rule::ReturnStatements => Some("Max. flagged returns"),
        Rule::FileComplexity | Rule::FunctionComplexity => Some("Max. cyclomatic complexity"),
        Rule::IdenticalCode | Rule::SimilarCode if actual => Some("Longest duplicate (lines)"),
        Rule::IdenticalCode | Rule::SimilarCode => None,
    };
    Labels {
        findings: Some("Findings"),
        maximum,
        lines: rule.is_duplication().then_some("Duplicated lines"),
        density: Some("Findings / 100 lines"),
        coverage: Some("Lines affected (%)"),
    }
}

fn magnitude_basis<'f>(
    files: impl IntoIterator<Item = &'f GroupFile>,
    rule: Rule,
) -> Result<&'f str> {
    let bases: BTreeSet<&str> = files
        .into_iter()
        .map(|file| file.smell(rule).magnitude_basis.as_str())
        .collect();
    let mut bases = bases.into_iter();
    match (bases.next(), bases.next()) {
        (Some(basis), None) => Ok(basis),
        _ => Err(mismatch(
            "Cannot compare different magnitude measurement methods",
        )),
    }
}

/// The measurements of `rule` that changed between the two file sets, at
/// most two, with a maximum blanked on a side without findings.
fn evidence(before: &[GroupFile], after: &[GroupFile], rule: Rule) -> Result<Vec<Evidence>> {
    let a = Observed::of(before, rule);
    let b = Observed::of(after, rule);
    let basis = magnitude_basis(before.iter().chain(after), rule)?;
    let mut labels = measurement_labels(rule, basis);
    let single_file_complexity =
        rule == Rule::FileComplexity && before.len() == 1 && after.len() == 1;
    if single_file_complexity {
        labels.findings = None;
        labels.maximum = Some(CYCLOMATIC_COMPLEXITY);
    }
    let mut changes: Vec<Evidence> = MEASURES
        .into_iter()
        .filter_map(|measure| {
            let label = labels.get(measure)?;
            let (before, after) = (a.quantity(measure), b.quantity(measure));
            if is_close(before.as_f64(), after.as_f64(), 1e-9, 1e-9) {
                return None;
            }
            let blank_missing = measure == Measure::Maximum;
            Some(Evidence::Change {
                label: label.to_owned(),
                decimals: None,
                scale: None,
                before: (!blank_missing || a.has_findings()).then_some(before),
                after: (!blank_missing || b.has_findings()).then_some(after),
            })
        })
        .collect();
    let leads_with_complexity = changes
        .first()
        .is_some_and(|change| change.label() == CYCLOMATIC_COMPLEXITY);
    if single_file_complexity && leads_with_complexity {
        changes.truncate(1);
        return Ok(changes);
    }
    changes.truncate(2);
    Ok(changes)
}

/// The measurements of `rule` for one file, at most two; only the finding
/// count is shown when there are no findings.
fn score_evidence(file: &GroupFile, rule: Rule) -> Vec<Evidence> {
    let values = Observed::of(std::slice::from_ref(file), rule);
    if rule == Rule::FileComplexity {
        return vec![Evidence::Value {
            label: CYCLOMATIC_COMPLEXITY.to_owned(),
            decimals: None,
            scale: None,
            value: values
                .has_findings()
                .then_some(Quantity::Amount(values.maximum)),
        }];
    }
    let labels = measurement_labels(rule, &file.smell(rule).magnitude_basis);
    MEASURES
        .into_iter()
        .filter_map(|measure| {
            let label = labels.get(measure)?;
            if measure != Measure::Findings && !values.has_findings() {
                return None;
            }
            Some(Evidence::Value {
                label: label.to_owned(),
                decimals: None,
                scale: None,
                value: Some(values.quantity(measure)),
            })
        })
        .take(2)
        .collect()
}

fn assessment_value(value: f64) -> Evidence {
    Evidence::Value {
        label: AI_ASSESSMENT.to_owned(),
        decimals: Some(0),
        scale: Some(100),
        value: Some(Quantity::Amount(value)),
    }
}

fn assessment_change(before: f64, after: f64) -> Evidence {
    Evidence::Change {
        label: AI_ASSESSMENT.to_owned(),
        decimals: Some(0),
        scale: Some(100),
        before: Some(Quantity::Amount(before)),
        after: Some(Quantity::Amount(after)),
    }
}

/// Keeps the largest factors: sorted by absolute change, at least 0.005 in
/// size, at most five. The remainder of `expected` is `other`.
fn leading_factors(mut factors: Vec<Factor>, expected: f64) -> Result<(Vec<Factor>, f64)> {
    let total = fsum(factors.iter().map(|factor| factor.change));
    if !is_close(total, expected, 0.0, RECONCILIATION_TOLERANCE) {
        return Err(Error::Unreconciled(format!(
            "factors sum to {total} but the score changed by {expected}"
        )));
    }
    factors.sort_by(|a, b| {
        b.change
            .abs()
            .partial_cmp(&a.change.abs())
            .unwrap_or(Ordering::Equal)
    });
    let shown: Vec<Factor> = factors
        .into_iter()
        .filter(|factor| factor.change.abs() >= SHOWN_FACTOR_MINIMUM)
        .take(SHOWN_FACTOR_COUNT)
        .collect();
    let other = expected - fsum(shown.iter().map(|factor| factor.change));
    Ok((shown, other))
}

fn smell_factor(rule: Rule, deltas: &[f64], evidence: Vec<Evidence>) -> Factor {
    let start = RULES
        .iter()
        .position(|candidate| *candidate == rule)
        .expect("RULES should contain every rule")
        * 4;
    Factor {
        id: rule.key().to_owned(),
        name: rule.label().to_owned(),
        change: fsum(deltas[start..start + 4].iter().copied()),
        evidence,
    }
}

/// Explains an existing score against the model's training reference.
///
/// Used for added and removed files, where no historical score comparison
/// exists. These factors do not describe the file's period contribution.
fn explain_score(model: &Model, file: &GroupFile) -> Result<Explanation> {
    let prediction = model.predict(&file.measurement.values)?;
    let deltas = &prediction.feature_contributions;
    let mut factors: Vec<Factor> = RULES
        .iter()
        .map(|rule| smell_factor(*rule, deltas, score_evidence(file, *rule)))
        .collect();
    for (offset, question) in questions().iter().enumerate() {
        let index = QLTY_FEATURE_COUNT + offset;
        factors.push(Factor {
            id: question.id.clone(),
            name: question.label().to_owned(),
            change: deltas[index],
            evidence: vec![assessment_value(100.0 * file.measurement.values[index])],
        });
    }
    let expected = prediction.score - prediction.baseline_score;
    let (factors, other) = leading_factors(factors, expected)?;
    Ok(Explanation::Score {
        mode: "score".to_owned(),
        score: prediction.score,
        reference_score: prediction.baseline_score,
        factors,
        other,
    })
}

fn assessment_mean(files: &[GroupFile], index: usize) -> Result<f64> {
    let members = weighted_members(files)?;
    Ok(100.0
        * fsum(
            members
                .iter()
                .map(|(file, weight)| file.measurement.values[index] * weight),
        ))
}

/// Explains a compared group's score change along the mass-weighted
/// before-to-after paths.
fn explain_row(
    model: &Model,
    before: &[GroupFile],
    after: &[GroupFile],
    row: &DrilldownRow,
) -> Result<Explanation> {
    let deltas = group_changes(model, before, after)?;
    let mut factors = Vec::with_capacity(RULES.len() + questions().len());
    for rule in RULES {
        factors.push(smell_factor(rule, &deltas, evidence(before, after, rule)?));
    }
    for (offset, question) in questions().iter().enumerate() {
        let index = QLTY_FEATURE_COUNT + offset;
        factors.push(Factor {
            id: question.id.clone(),
            name: question.label().to_owned(),
            change: deltas[index],
            evidence: vec![assessment_change(
                assessment_mean(before, index)?,
                assessment_mean(after, index)?,
            )],
        });
    }
    let (Some(before_score), Some(after_score)) = (row.before_score, row.after_score) else {
        return Err(mismatch("A compared row needs both scores"));
    };
    let expected = after_score - before_score;
    let (factors, other) = leading_factors(factors, expected)?;
    Ok(Explanation::Change {
        factors,
        other,
        score_change: expected,
    })
}

/// Whether a row's contribution is visible at three decimals; other rows
/// carry no explanation.
fn is_visible(row: &DrilldownRow) -> bool {
    format!("{:.3}", row.contribution.abs()) != "0.000"
}

fn explain_drilldown_row(
    model: &Model,
    measurements: &mut Measurements,
    row: &DrilldownRow,
    previous: &HashMap<&str, &FileRow>,
    current: &HashMap<&str, &FileRow>,
) -> Result<Option<Explanation>> {
    if !is_visible(row) {
        return Ok(None);
    }
    if !row.before.is_empty() && !row.after.is_empty() {
        let before = measurements.members(&row.before, previous)?;
        let after = measurements.members(&row.after, current)?;
        return explain_row(model, &before, &after, row).map(Some);
    }
    let files = if row.after.is_empty() {
        measurements.members(&row.before, previous)?
    } else {
        measurements.members(&row.after, current)?
    };
    let file = files
        .first()
        .ok_or_else(|| mismatch("A drilldown row has no files"))?;
    explain_score(model, file).map(Some)
}

/// Explains every visible drilldown row of every period, aligned with the
/// drilldown rows (`None` where a row is not visible).
pub fn explain_periods(
    model: &Model,
    measurements: &mut Measurements,
    files: &FilesExport,
    drilldowns: &[DrilldownPeriod],
) -> Result<Vec<ExplanationPeriod>> {
    if files.periods.len() != drilldowns.len() {
        return Err(mismatch(
            "Drilldown and file snapshots have different period counts",
        ));
    }
    let mut result = Vec::with_capacity(drilldowns.len());
    let mut previous: HashMap<&str, &FileRow> = HashMap::new();
    for (period, snapshot) in drilldowns.iter().zip(&files.periods) {
        if period.commit != snapshot.commit {
            return Err(mismatch(
                "Drilldown and file snapshots have different commits",
            ));
        }
        let rows = snapshot
            .files
            .as_deref()
            .ok_or_else(|| mismatch("The file export has a period without files"))?;
        let current: HashMap<&str, &FileRow> =
            rows.iter().map(|row| (row.path.as_str(), row)).collect();
        let explanations = period
            .rows
            .iter()
            .map(|row| explain_drilldown_row(model, measurements, row, &previous, &current))
            .collect::<Result<Vec<Option<Explanation>>>>()?;
        tracing::debug!(
            period = snapshot.start(),
            explained = explanations.iter().flatten().count(),
            "explained drilldown rows"
        );
        result.push(ExplanationPeriod {
            commit: period.commit.clone(),
            rows: explanations,
        });
        previous = current;
    }
    Ok(result)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn model_sha256(model: &Value) -> Option<&str> {
    model.get("sha256").and_then(Value::as_str)
}

/// Writes `<name>-explanations.json.gz` next to the `<name>.json` and
/// `<name>-files.json.gz` exports in `output_dir`, from the run's saved
/// measurements.
pub fn export(run: &RunDir, output_dir: &Path, name: &str) -> Result<()> {
    let data_path = output_dir.join(format!("{name}.json"));
    let files_path = output_dir.join(format!("{name}-files.json.gz"));
    let data_bytes = fs::read(&data_path)?;
    let files_bytes = fs::read(&files_path)?;
    let data = ReportData::from_value(serde_json::from_slice(&data_bytes)?)?;
    let files = FilesExport::from_value(read_json_gz(&files_path)?)?;
    let model = Model::shared()?;
    if model_sha256(data.model()) != Some(model.sha256())
        || model_sha256(&files.model) != Some(model.sha256())
    {
        return Err(mismatch("Model does not match the frozen report"));
    }

    let mut measurements = Measurements::new(model, run);
    let drilldowns = drilldown_periods(&data, &files)?;
    let periods = explain_periods(model, &mut measurements, &files, &drilldowns)?;
    tracing::info!(
        explained = periods
            .iter()
            .flat_map(|period| &period.rows)
            .flatten()
            .count(),
        measurements = measurements.loaded(),
        "exported verified explanations"
    );

    let export = ExplanationsExport {
        schema_version: SCHEMA_VERSION,
        model_sha256: model.sha256().to_owned(),
        score_scale: SCORE_SCALE.to_owned(),
        data_sha256: sha256_hex(&data_bytes),
        files_sha256: sha256_hex(&files_bytes),
        method: METHOD.to_owned(),
        questions: questions()
            .iter()
            .map(|question| (question.id.clone(), question.instructions.clone()))
            .collect(),
        interval: data.interval,
        periods,
    };
    write_json_gz(
        &output_dir.join(format!("{name}-explanations.json.gz")),
        &export.to_value()?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smell(count: u64, max_magnitude: f64, covered_lines: u64) -> SmellSummary {
        SmellSummary {
            count,
            max_magnitude,
            covered_lines,
            magnitude_basis: "actual".to_owned(),
        }
    }

    fn file(source_lines: u64, mass: f64, smells: &[(Rule, SmellSummary)]) -> GroupFile {
        let mut summary: BTreeMap<Rule, SmellSummary> =
            RULES.iter().map(|rule| (*rule, smell(0, 0.0, 0))).collect();
        summary.extend(smells.iter().cloned());
        GroupFile {
            measurement: Rc::new(Measurement {
                values: vec![0.0; 39],
                summary,
                source_lines,
                score: 5.0,
            }),
            mass,
        }
    }

    fn change(label: &str, before: Option<Quantity>, after: Option<Quantity>) -> Evidence {
        Evidence::Change {
            label: label.to_owned(),
            decimals: None,
            scale: None,
            before,
            after,
        }
    }

    fn value(label: &str, value: Option<Quantity>) -> Evidence {
        Evidence::Value {
            label: label.to_owned(),
            decimals: None,
            scale: None,
            value,
        }
    }

    fn factor(id: &str, change: f64) -> Factor {
        Factor {
            id: id.to_owned(),
            name: id.to_owned(),
            change,
            evidence: Vec::new(),
        }
    }

    #[test]
    fn is_close_matches_python_defaults() {
        assert!(is_close(1.0, 1.0 + 5e-10, 1e-9, 1e-9));
        assert!(!is_close(1.0, 1.0 + 5e-9, 1e-9, 1e-9));
        assert!(is_close(1e9, 1e9 + 0.5, 1e-9, 1e-9));
        assert!(!is_close(1e9, 1e9 + 0.5, 0.0, 1e-9));
        assert!(!is_close(f64::INFINITY, 1.0, 1e-9, 1e-9));
    }

    #[test]
    fn evidence_lists_at_most_two_changed_measurements() {
        let before = [file(100, 1.0, &[(Rule::BooleanLogic, smell(2, 5.0, 10))])];
        let after = [file(200, 1.0, &[(Rule::BooleanLogic, smell(1, 4.0, 3))])];
        assert_eq!(
            evidence(&before, &after, Rule::BooleanLogic).unwrap(),
            [
                change(
                    "Findings",
                    Some(Quantity::Count(2)),
                    Some(Quantity::Count(1))
                ),
                change(
                    "Max. Boolean depth",
                    Some(Quantity::Amount(5.0)),
                    Some(Quantity::Amount(4.0))
                ),
            ]
        );
    }

    #[test]
    fn evidence_blanks_the_maximum_on_a_side_without_findings() {
        let before = [file(100, 1.0, &[])];
        let after = [file(
            100,
            1.0,
            &[(Rule::NestedControlFlow, smell(1, 6.0, 8))],
        )];
        assert_eq!(
            evidence(&before, &after, Rule::NestedControlFlow).unwrap(),
            [
                change(
                    "Findings",
                    Some(Quantity::Count(0)),
                    Some(Quantity::Count(1))
                ),
                change("Max. nesting depth", None, Some(Quantity::Amount(6.0))),
            ]
        );
    }

    #[test]
    fn evidence_skips_unchanged_measurements() {
        let before = [file(100, 1.0, &[(Rule::IdenticalCode, smell(1, 20.0, 20))])];
        let after = [file(100, 1.0, &[(Rule::IdenticalCode, smell(1, 20.0, 25))])];
        assert_eq!(
            evidence(&before, &after, Rule::IdenticalCode).unwrap(),
            [
                change(
                    "Duplicated lines",
                    Some(Quantity::Count(20)),
                    Some(Quantity::Count(25))
                ),
                change(
                    "Lines affected (%)",
                    Some(Quantity::Amount(20.0)),
                    Some(Quantity::Amount(25.0))
                ),
            ]
        );
    }

    #[test]
    fn single_file_complexity_shows_only_the_complexity() {
        let before = [file(
            100,
            1.0,
            &[(Rule::FileComplexity, smell(1, 60.0, 100))],
        )];
        let after = [file(
            100,
            1.0,
            &[(Rule::FileComplexity, smell(1, 72.0, 100))],
        )];
        assert_eq!(
            evidence(&before, &after, Rule::FileComplexity).unwrap(),
            [change(
                "Cyclomatic complexity",
                Some(Quantity::Amount(60.0)),
                Some(Quantity::Amount(72.0))
            )]
        );
    }

    #[test]
    fn grouped_file_complexity_keeps_the_generic_labels() {
        let before = [file(
            100,
            1.0,
            &[(Rule::FileComplexity, smell(1, 60.0, 100))],
        )];
        let after = [
            file(50, 1.0, &[(Rule::FileComplexity, smell(1, 55.0, 50))]),
            file(50, 1.0, &[]),
        ];
        assert_eq!(
            evidence(&before, &after, Rule::FileComplexity).unwrap(),
            [
                change(
                    "Max. cyclomatic complexity",
                    Some(Quantity::Amount(60.0)),
                    Some(Quantity::Amount(55.0))
                ),
                change(
                    "Lines affected (%)",
                    Some(Quantity::Amount(100.0)),
                    Some(Quantity::Amount(50.0))
                ),
            ]
        );
    }

    #[test]
    fn evidence_rejects_mixed_magnitude_bases() {
        let before = [file(100, 1.0, &[])];
        let mut legacy = smell(1, 3.0, 3);
        legacy.magnitude_basis = "legacy".to_owned();
        let after = [file(100, 1.0, &[(Rule::BooleanLogic, legacy)])];
        assert!(matches!(
            evidence(&before, &after, Rule::BooleanLogic),
            Err(Error::Mismatch(_))
        ));
    }

    #[test]
    fn legacy_duplication_has_no_longest_duplicate_label() {
        let labels = measurement_labels(Rule::SimilarCode, "legacy");
        assert_eq!(labels.maximum, None);
        assert_eq!(labels.lines, Some("Duplicated lines"));
        assert_eq!(
            measurement_labels(Rule::BooleanLogic, "legacy").maximum,
            Some("Longest finding (lines)")
        );
        assert_eq!(
            measurement_labels(Rule::ReturnStatements, "actual").lines,
            None
        );
    }

    #[test]
    fn absent_findings_do_not_claim_zero_complexity() {
        let subject = file(20, 1.0, &[]);
        assert_eq!(
            score_evidence(&subject, Rule::FunctionComplexity),
            [value("Findings", Some(Quantity::Count(0)))]
        );
        assert_eq!(
            score_evidence(&subject, Rule::FileComplexity),
            [value("Cyclomatic complexity", None)]
        );
    }

    #[test]
    fn score_evidence_shows_the_count_and_maximum_of_findings() {
        let subject = file(20, 1.0, &[(Rule::FunctionParameters, smell(3, 8.0, 12))]);
        assert_eq!(
            score_evidence(&subject, Rule::FunctionParameters),
            [
                value("Findings", Some(Quantity::Count(3))),
                value("Max. flagged parameters", Some(Quantity::Amount(8.0))),
            ]
        );
    }

    #[test]
    fn leading_factors_keeps_the_five_largest_and_the_remainder() {
        let factors = vec![
            factor("a", 0.01),
            factor("b", -0.5),
            factor("c", 0.004),
            factor("d", 0.2),
            factor("e", -0.2),
            factor("f", 0.03),
            factor("g", 0.02),
            factor("h", 0.1),
        ];
        let expected = fsum(factors.iter().map(|factor| factor.change));
        let (shown, other) = leading_factors(factors, expected).unwrap();
        let ids: Vec<&str> = shown.iter().map(|factor| factor.id.as_str()).collect();
        assert_eq!(ids, ["b", "d", "e", "h", "f"]);
        assert!((other - 0.034).abs() < 1e-12);
    }

    #[test]
    fn leading_factors_rejects_an_unreconciled_total() {
        let factors = vec![factor("a", 0.5)];
        assert!(matches!(
            leading_factors(factors, 0.6),
            Err(Error::Unreconciled(_))
        ));
    }

    #[test]
    fn rows_below_a_thousandth_are_not_explained() {
        let row = |contribution: f64| DrilldownRow {
            kind: crate::report::ChangeKind::Changed,
            before: Vec::new(),
            after: Vec::new(),
            before_score: None,
            after_score: None,
            before_mass: None,
            after_mass: None,
            contribution,
            diff_anchor: None,
            explanation: None,
        };
        assert!(!is_visible(&row(0.0004999)));
        assert!(is_visible(&row(-0.0006)));
        assert!(is_visible(&row(0.001)));
    }

    #[test]
    fn weighted_members_rejects_a_massless_group() {
        assert!(matches!(
            weighted_members(&[file(10, 0.0, &[])]),
            Err(Error::Mismatch(_))
        ));
    }
}
