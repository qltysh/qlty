//! Predict and explain an average of logistic components.
//!
//! `Model::predict` and `Model::explain` are exact ports of slopdetect's
//! `model.py`: every sum that Python computes with `math.fsum` uses
//! [`fsum`], and every other expression keeps Python's operand order.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::error::{Error, Result};
use crate::features::{feature_names, questions, Field, Rule, FIELDS, RULES};
use crate::fsum::fsum;
use crate::jev::{Excerpt, JevFeatures};
use crate::measure::RuleSummary;
use crate::scoring::{ScoreScale, SCORE_CUTOFF, SCORE_SCALE};

/// The feature extractor profile the packaged model was trained against.
pub const EXTRACTOR_PROFILE: &str = "qlty-actual-magnitudes-001";

/// slopdetect's digest of the extractor version, profile settings, Jev model,
/// chunk size, Qlty configuration, and questions for [`EXTRACTOR_PROFILE`].
const EXTRACTOR_IDENTITY: &str = "5942c7b454d101059f34b3bb54b6ad6cd343559fa768e1ceb265e4a45f22a767";

const SCORE_MEANING: &str =
    "Relative maintainability score; higher is better. Not a calibrated human grade.";

const EXPLANATION_METHOD: &str = "Original additive model contributions along a straight feature path, rescaled between the reference and file scores. Describes the model, not causal effects of edits.";

const EXPLANATION_NOTES: [&str; 3] = [
    "Each Qlty factor combines four related measurements of its findings. Their contributions sum to the factor contribution; they are not four independent defects.",
    "Different smell factors can cover the same source lines.",
    "Score margin is score minus cutoff. Its size is not a calibrated confidence estimate.",
];

const MAX_COMPONENTS: usize = 2048;

const MODEL_JSON: &str = include_str!("assets/model.json");

#[derive(Clone, Debug)]
enum Component {
    Constant(f64),
    Logistic { weights: Vec<f64>, intercept: f64 },
}

/// A validated SlopOne model: an average of logistic components over the 39
/// features, with the risk cutoff and the reference vector that explanations
/// are measured from.
#[derive(Clone, Debug)]
pub struct Model {
    id: String,
    sha256: String,
    extractor_profile: String,
    threshold: f64,
    score_scale: ScoreScale,
    reference: Vec<f64>,
    components: Vec<Component>,
    training: Value,
    measurement_policy: Value,
}

/// `Model::info()` in slopdetect's JSON shape and key order.
#[derive(Clone, Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub sha256: String,
    pub extractor_profile: String,
    pub components: usize,
    pub features: usize,
    pub threshold_risk: f64,
    pub threshold_score: f64,
    pub score_scale: &'static str,
    pub training: Value,
    pub measurement_policy: Value,
    pub score_meaning: &'static str,
}

/// `Model.predict` output in slopdetect's key order.
#[derive(Clone, Debug, Serialize)]
pub struct Prediction {
    pub passed: bool,
    pub score: f64,
    pub risk: f64,
    pub baseline_score: f64,
    pub feature_contributions: Vec<f64>,
    pub threshold_score: f64,
    pub threshold_risk: f64,
    pub score_scale: &'static str,
    pub score_margin: f64,
    pub explanation_residual: f64,
}

/// `Model.explain` output in slopdetect's key order.
#[derive(Clone, Debug, Serialize)]
pub struct Explanation {
    pub passed: bool,
    pub score: f64,
    pub risk: f64,
    pub baseline_score: f64,
    pub threshold_score: f64,
    pub threshold_risk: f64,
    pub score_scale: &'static str,
    pub score_margin: f64,
    pub explanation_residual: f64,
    pub factors: Vec<Factor>,
    pub explanation_method: &'static str,
    pub explanation_notes: Vec<&'static str>,
}

/// One of the 15 factors (8 Qlty smells, 7 Jev questions), sorted by
/// contribution in an [`Explanation`].
#[derive(Clone, Debug, Serialize)]
pub struct Factor {
    pub id: String,
    pub name: &'static str,
    pub score_contribution: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurements: Option<Vec<MeasurementContribution>>,
    pub observed: Observed,
}

/// The evidence behind a factor: the smell summary or the Jev answers.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Observed {
    Qlty(RuleSummary),
    Jev(JevObserved),
}

#[derive(Clone, Debug, Serialize)]
pub struct JevObserved {
    pub value: f64,
    pub excerpts: Vec<Excerpt>,
}

/// One of a smell factor's four measurements and its share of the contribution.
#[derive(Clone, Debug, Serialize)]
pub struct MeasurementContribution {
    pub id: &'static str,
    pub name: String,
    pub transform: &'static str,
    pub feature_value: f64,
    pub reference_feature_value: f64,
    pub score_contribution: f64,
}

pub(crate) fn sigmoid(value: f64) -> f64 {
    if value >= 0.0 {
        return 1.0 / (1.0 + (-value).exp());
    }
    let exponential = value.exp();
    exponential / (1.0 + exponential)
}

impl Model {
    /// The packaged model, parsed once per process.
    pub fn shared() -> Result<&'static Model> {
        static SHARED: OnceLock<std::result::Result<Model, String>> = OnceLock::new();
        SHARED
            .get_or_init(|| Model::embedded().map_err(|error| error.to_string()))
            .as_ref()
            .map_err(|message| Error::InvalidModel(message.clone()))
    }

    /// Parses and validates the packaged `model.json`.
    pub fn embedded() -> Result<Model> {
        let document: Value = serde_json::from_str(MODEL_JSON)
            .map_err(|error| Error::InvalidModel(error.to_string()))?;
        Self::try_new(document)
    }

    /// Validates a model document exactly as slopdetect's `Model.__init__` does.
    pub fn try_new(document: Value) -> Result<Model> {
        let schema_version = field(&document, "schema_version")?;
        let features: Vec<String> = serde_json::from_value(field(&document, "features")?.clone())
            .map_err(|_| invalid("Unexpected model schema or features"))?;
        if schema_version != &Value::from(1) || features != feature_names() {
            return Err(invalid("Unexpected model schema or features"));
        }

        let extractor_profile = document
            .get("extractor_profile")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let extractor_identity = field(&document, "extractor_identity")?;
        if extractor_profile != EXTRACTOR_PROFILE
            || extractor_identity != &Value::from(EXTRACTOR_IDENTITY)
        {
            return Err(invalid("Model and feature extractor versions differ"));
        }

        let threshold = finite(field(&document, "threshold")?)
            .filter(|threshold| (0.0..=1.0).contains(threshold))
            .ok_or_else(|| invalid("Invalid cutoff"))?;
        let score_scale = ScoreScale::try_new(threshold)?;

        let size = features.len();
        let reference = finite_vector(field(&document, "reference")?)
            .filter(|reference| reference.len() == size)
            .ok_or_else(|| invalid("Invalid reference vector"))?;

        let components = field(&document, "components")?
            .as_array()
            .filter(|components| (1..=MAX_COMPONENTS).contains(&components.len()))
            .ok_or_else(|| invalid("Invalid component count"))?
            .iter()
            .map(|component| parse_component(component, size))
            .collect::<Result<Vec<Component>>>()?;

        let id = field(&document, "model_id")?
            .as_str()
            .ok_or_else(|| invalid("Invalid model id"))?
            .to_string();
        let training = document
            .get("training")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));
        let measurement_policy = document
            .get("measurement_policy")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));

        Ok(Model {
            id,
            sha256: canonical_digest(&document),
            extractor_profile: extractor_profile.to_string(),
            threshold,
            score_scale,
            reference,
            components,
            training,
            measurement_policy,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// slopdetect's digest of the model document: SHA-256 over the compact,
    /// key-sorted JSON that Python's `json.dumps` produces.
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// The risk cutoff. Risks below it pass.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// The feature vector explanations are measured from.
    pub fn reference(&self) -> &[f64] {
        &self.reference
    }

    pub fn score_scale(&self) -> &ScoreScale {
        &self.score_scale
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn info(&self) -> ModelInfo {
        ModelInfo {
            id: self.id.clone(),
            sha256: self.sha256.clone(),
            extractor_profile: self.extractor_profile.clone(),
            components: self.components.len(),
            features: self.reference.len(),
            threshold_risk: self.threshold,
            threshold_score: SCORE_CUTOFF,
            score_scale: SCORE_SCALE,
            training: self.training.clone(),
            measurement_policy: self.measurement_policy.clone(),
            score_meaning: SCORE_MEANING,
        }
    }

    /// Scores a 39-value feature vector and attributes the difference from
    /// the reference score to each feature.
    pub fn predict(&self, values: &[f64]) -> Result<Prediction> {
        if values.len() != self.reference.len() || !values.iter().all(|value| value.is_finite()) {
            return Err(Error::InvalidInput);
        }

        let count = self.components.len();
        let mut risks = Vec::with_capacity(count);
        let mut bases = Vec::with_capacity(count);
        let mut contributions: Vec<Vec<f64>> = Vec::with_capacity(count);
        for component in &self.components {
            match component {
                Component::Constant(constant) => {
                    risks.push(*constant);
                    bases.push(*constant);
                    contributions.push(vec![0.0; values.len()]);
                }
                Component::Logistic { weights, intercept } => {
                    let z0 =
                        intercept + fsum(weights.iter().zip(&self.reference).map(|(w, x)| w * x));
                    let z1 = intercept + fsum(weights.iter().zip(values).map(|(w, x)| w * x));
                    let risk0 = sigmoid(z0);
                    let risk1 = sigmoid(z1);
                    let difference = z1 - z0;
                    let slope = if difference.abs() > 1e-8 {
                        (risk1 - risk0) / difference
                    } else {
                        let middle = sigmoid((z0 + z1) / 2.0);
                        middle * (1.0 - middle)
                    };
                    contributions.push(
                        weights
                            .iter()
                            .zip(values)
                            .zip(&self.reference)
                            .map(|((w, x), base)| -9.0 * w * (x - base) * slope)
                            .collect(),
                    );
                    risks.push(risk1);
                    bases.push(risk0);
                }
            }
        }

        let divisor = count as f64;
        let risk = fsum(risks) / divisor;
        let base_risk = fsum(bases) / divisor;
        let score = self.score_scale.from_risk(risk);
        let baseline = self.score_scale.from_risk(base_risk);
        let multiplier = self
            .score_scale
            .secant(10.0 - 9.0 * base_risk, 10.0 - 9.0 * risk);
        let deltas: Vec<f64> = (0..values.len())
            .map(|index| {
                multiplier * fsum(contributions.iter().map(|component| component[index])) / divisor
            })
            .collect();
        let residual = score - baseline - fsum(deltas.iter().copied());
        if residual.abs() > 1e-8 {
            return Err(Error::Unreconciled);
        }

        Ok(Prediction {
            passed: risk < self.threshold,
            score,
            risk,
            baseline_score: baseline,
            feature_contributions: deltas,
            threshold_score: SCORE_CUTOFF,
            threshold_risk: self.threshold,
            score_scale: SCORE_SCALE,
            score_margin: score - SCORE_CUTOFF,
            explanation_residual: residual,
        })
    }

    /// Groups a prediction's contributions into the 15 factors with their
    /// evidence, sorted from the most harmful contribution upward.
    pub fn explain(
        &self,
        values: &[f64],
        qlty: &BTreeMap<Rule, RuleSummary>,
        jev: &JevFeatures,
    ) -> Result<Explanation> {
        let prediction = self.predict(values)?;
        let contributions = &prediction.feature_contributions;

        let mut factors = Vec::with_capacity(RULES.len() + questions().len());
        for (index, rule) in RULES.iter().enumerate() {
            let start = index * FIELDS.len();
            let summary = qlty.get(rule).ok_or(Error::InvalidInput)?;
            let measurements = FIELDS
                .iter()
                .enumerate()
                .map(|(offset, field)| MeasurementContribution {
                    id: field.id(),
                    name: measurement_name(*field, summary),
                    transform: field.transform(),
                    feature_value: values[start + offset],
                    reference_feature_value: self.reference[start + offset],
                    score_contribution: contributions[start + offset],
                })
                .collect();
            factors.push(Factor {
                id: rule.key().to_string(),
                name: rule.label(),
                score_contribution: fsum(
                    contributions[start..start + FIELDS.len()].iter().copied(),
                ),
                measurements: Some(measurements),
                observed: Observed::Qlty(summary.clone()),
            });
        }
        for (index, question) in questions().iter().enumerate() {
            let value = *jev.means.get(&question.id).ok_or(Error::InvalidInput)?;
            let excerpts = jev
                .assessments
                .get(&question.id)
                .ok_or(Error::InvalidInput)?
                .clone();
            factors.push(Factor {
                id: question.id.clone(),
                name: question.label(),
                score_contribution: contributions[RULES.len() * FIELDS.len() + index],
                measurements: None,
                observed: Observed::Jev(JevObserved { value, excerpts }),
            });
        }
        factors.sort_by(|a, b| {
            a.score_contribution
                .partial_cmp(&b.score_contribution)
                .unwrap_or(Ordering::Equal)
        });

        Ok(Explanation {
            passed: prediction.passed,
            score: prediction.score,
            risk: prediction.risk,
            baseline_score: prediction.baseline_score,
            threshold_score: prediction.threshold_score,
            threshold_risk: prediction.threshold_risk,
            score_scale: prediction.score_scale,
            score_margin: prediction.score_margin,
            explanation_residual: prediction.explanation_residual,
            factors,
            explanation_method: EXPLANATION_METHOD,
            explanation_notes: EXPLANATION_NOTES.to_vec(),
        })
    }
}

fn measurement_name(field: Field, summary: &RuleSummary) -> String {
    if field == Field::LogMaxMagnitude && !summary.magnitude_unit.is_empty() {
        return format!("Maximum {}", summary.magnitude_unit.to_lowercase());
    }
    field.label().to_string()
}

fn invalid(message: &str) -> Error {
    Error::InvalidModel(message.to_string())
}

fn field<'a>(document: &'a Value, key: &str) -> Result<&'a Value> {
    document
        .get(key)
        .ok_or_else(|| invalid(&format!("Missing '{key}'")))
}

/// Python's `finite`: a non-boolean number that is finite.
fn finite(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| number.is_finite())
}

fn finite_vector(value: &Value) -> Option<Vec<f64>> {
    value.as_array()?.iter().map(finite).collect()
}

fn parse_component(component: &Value, size: usize) -> Result<Component> {
    if let Some(constant) = component.get("constant") {
        return finite(constant)
            .filter(|constant| (0.0..=1.0).contains(constant))
            .map(Component::Constant)
            .ok_or_else(|| invalid("Invalid constant prediction"));
    }
    let weights = component
        .get("weights")
        .and_then(finite_vector)
        .filter(|weights| weights.len() == size);
    let intercept = component.get("intercept").and_then(finite);
    match (weights, intercept) {
        (Some(weights), Some(intercept)) => Ok(Component::Logistic { weights, intercept }),
        _ => Err(invalid("Invalid logistic component")),
    }
}

/// SHA-256 of the document as Python's
/// `json.dumps(value, sort_keys=True, separators=(',', ':'))` renders it.
fn canonical_digest(document: &Value) -> String {
    let mut text = String::new();
    write_canonical(document, &mut text);
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => write_python_number(number, out),
        Value::String(text) => write_python_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            out.push('{');
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_python_string(key, out);
                out.push(':');
                write_canonical(item, out);
            }
            out.push('}');
        }
    }
}

fn write_python_number(number: &serde_json::Number, out: &mut String) {
    if number.is_f64() {
        let _ = write!(out, "{}", python_float_repr(number.as_f64().unwrap_or(0.0)));
        return;
    }
    let _ = write!(out, "{number}");
}

/// Python's `repr(float)`: the shortest round-trip digits, in fixed notation
/// when the decimal exponent is in `-4..16` and otherwise in `e±XX` notation.
fn python_float_repr(value: f64) -> String {
    let scientific = format!("{value:e}");
    let (sign, unsigned) = match scientific.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", scientific.as_str()),
    };
    let (mantissa, exponent) = unsigned.split_once('e').unwrap_or((unsigned, "0"));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let decimal_point = exponent + 1;

    if decimal_point <= -4 || decimal_point > 16 {
        let (first, rest) = digits.split_at(1);
        let fraction = if rest.is_empty() {
            String::new()
        } else {
            format!(".{rest}")
        };
        let exponent_sign = if exponent < 0 { '-' } else { '+' };
        return format!(
            "{sign}{first}{fraction}e{exponent_sign}{:02}",
            exponent.abs()
        );
    }
    if decimal_point <= 0 {
        let zeros = "0".repeat(decimal_point.unsigned_abs() as usize);
        return format!("{sign}0.{zeros}{digits}");
    }
    let point = decimal_point as usize;
    if point >= digits.len() {
        let zeros = "0".repeat(point - digits.len());
        return format!("{sign}{digits}{zeros}.0");
    }
    let (whole, fraction) = digits.split_at(point);
    format!("{sign}{whole}.{fraction}")
}

fn write_python_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            c => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::measure::LineSpan;

    const PACKAGED_SHA256: &str =
        "75abf3e26f868eb12b567bbe2af45ef97fd6a3fbaf27eed0bc303f822c8a4d35";

    fn document() -> Value {
        let count = feature_names().len();
        let mut first = vec![0.0; count];
        first[0] = 2.0;
        first[1] = -1.0;
        let mut second = vec![0.0; count];
        second[0] = -0.2;
        second[1] = 0.7;
        json!({
            "schema_version": 1,
            "model_id": "test",
            "extractor_profile": EXTRACTOR_PROFILE,
            "extractor_identity": EXTRACTOR_IDENTITY,
            "features": feature_names(),
            "threshold": 0.5,
            "reference": vec![0.0; count],
            "components": [
                {"weights": first, "intercept": -0.3},
                {"weights": second, "intercept": 0.8},
                {"constant": 0.2},
            ],
            "training": {},
        })
    }

    fn document_with(changes: Value) -> Value {
        let mut document = document();
        for (key, value) in changes.as_object().unwrap() {
            document[key] = value.clone();
        }
        document
    }

    fn vector(a: f64, b: f64) -> Vec<f64> {
        let mut values = vec![0.0; feature_names().len()];
        values[0] = a;
        values[1] = b;
        values
    }

    fn assert_matches_independent_formula(a: f64, b: f64) {
        let model = Model::try_new(document()).unwrap();
        let expected = (sigmoid(2.0 * a - b - 0.3) + sigmoid(-0.2 * a + 0.7 * b + 0.8) + 0.2) / 3.0;
        let result = model.predict(&vector(a, b)).unwrap();
        assert!((result.risk - expected).abs() <= 1e-14);
        let expected_score = if expected < 0.5 {
            10.0 - 10.0 * expected
        } else {
            1.0 + 8.0 * (1.0 - expected)
        };
        assert!((result.score - expected_score).abs() <= 1e-12);
        let reconstructed =
            result.baseline_score + fsum(result.feature_contributions.iter().copied());
        assert!((reconstructed - result.score).abs() <= 1e-10);
        assert_eq!(result.passed, expected < 0.5);
    }

    #[test]
    fn probability_average_and_additive_explanation_match_independent_formula() {
        assert_matches_independent_formula(0.0, 0.0);
        assert_matches_independent_formula(1.0, 0.0);
        assert_matches_independent_formula(0.0, 1.0);
        assert_matches_independent_formula(4.0, 2.0);
        assert_matches_independent_formula(1000.0, 1.0);
        assert_matches_independent_formula(1e-12, 1e-12);
    }

    struct Lcg(u64);

    impl Lcg {
        fn uniform(&mut self, low: f64, high: f64) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = (self.0 >> 11) as f64 / (1u64 << 53) as f64;
            low + (high - low) * unit
        }

        fn vector(&mut self, low: f64, high: f64) -> Vec<f64> {
            (0..feature_names().len())
                .map(|_| self.uniform(low, high))
                .collect()
        }
    }

    #[test]
    fn explanations_sum_with_many_mixed_sign_models_and_reference() {
        let mut rng = Lcg(194);
        let components: Vec<Value> = (0..64)
            .map(|_| json!({"weights": rng.vector(-3.0, 3.0), "intercept": rng.uniform(-3.0, 3.0)}))
            .collect();
        let document = document_with(json!({
            "reference": rng.vector(0.0, 1.0),
            "components": components,
        }));
        let model = Model::try_new(document).unwrap();
        let results: Vec<Prediction> = (0..20)
            .map(|_| model.predict(&rng.vector(0.0, 5.0)).unwrap())
            .collect();
        assert!(results
            .iter()
            .all(|result| result.explanation_residual.abs() < 1e-10));
        assert!(results
            .iter()
            .all(|result| (1.0..=10.0).contains(&result.score)));
    }

    fn summary() -> RuleSummary {
        RuleSummary {
            count: 1,
            max_magnitude: 3.0,
            covered_lines: 3,
            locations: vec![LineSpan {
                start_line: 2,
                end_line: 4,
            }],
            max_actual: 3.0,
            threshold: 4,
            magnitude_basis: "actual",
            magnitude_unit: "Boolean depth",
        }
    }

    fn jev_features() -> JevFeatures {
        let excerpt = Excerpt {
            start_line: 1,
            end_line: 10,
            value: 0.5,
            weight_bytes: 100,
            source_ranges: None,
        };
        JevFeatures {
            values: vec![0.5; questions().len()],
            means: questions().iter().map(|q| (q.id.clone(), 0.5)).collect(),
            assessments: questions()
                .iter()
                .map(|q| (q.id.clone(), vec![excerpt.clone()]))
                .collect(),
            model: "jev-1.13.0".to_string(),
            excerpts: 1,
        }
    }

    fn explanation_of_ones() -> (Explanation, Prediction) {
        let model = Model::try_new(document()).unwrap();
        let values = vec![1.0; feature_names().len()];
        let qlty: BTreeMap<Rule, RuleSummary> =
            RULES.iter().map(|rule| (*rule, summary())).collect();
        let explanation = model.explain(&values, &qlty, &jev_features()).unwrap();
        let prediction = model.predict(&values).unwrap();
        (explanation, prediction)
    }

    #[test]
    fn grouped_factors_preserve_total_contribution() {
        let (explanation, _) = explanation_of_ones();
        assert_eq!(explanation.factors.len(), 15);
        let total = explanation.baseline_score
            + explanation
                .factors
                .iter()
                .map(|factor| factor.score_contribution)
                .sum::<f64>();
        assert!((total - explanation.score).abs() <= 1e-10);
        assert_eq!(
            explanation.score_margin,
            explanation.score - explanation.threshold_score
        );
    }

    #[test]
    fn grouped_factors_carry_smell_evidence() {
        let (explanation, _) = explanation_of_ones();
        let factor = explanation
            .factors
            .iter()
            .find(|factor| factor.id == "boolean-logic")
            .unwrap();
        let Observed::Qlty(observed) = &factor.observed else {
            panic!("boolean-logic should carry a rule summary");
        };
        assert_eq!(observed.locations[0].start_line, 2);
        assert_eq!(factor.name, "Boolean logic");
    }

    #[test]
    fn grouped_factor_measurements_match_raw_contributions() {
        let (explanation, raw) = explanation_of_ones();
        let checks: Vec<bool> = RULES
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                let factor = explanation
                    .factors
                    .iter()
                    .find(|factor| factor.id == rule.key())
                    .unwrap();
                let measurements = factor.measurements.as_ref().unwrap();
                let ids: Vec<&str> = measurements.iter().map(|m| m.id).collect();
                let contributions: Vec<f64> =
                    measurements.iter().map(|m| m.score_contribution).collect();
                let transforms: Vec<&str> = measurements.iter().map(|m| m.transform).collect();
                ids == [
                    "log_count",
                    "count_per_100_source_lines",
                    "log_max_magnitude",
                    "line_coverage_fraction",
                ] && contributions == raw.feature_contributions[index * 4..index * 4 + 4]
                    && fsum(contributions.iter().copied()) == factor.score_contribution
                    && transforms == ["log1p", "identity", "log1p", "identity"]
                    && measurements.iter().all(|m| m.feature_value == 1.0)
                    && measurements
                        .iter()
                        .all(|m| m.reference_feature_value == 0.0)
            })
            .collect();
        assert_eq!(checks, vec![true; RULES.len()]);
    }

    #[test]
    fn magnitude_measurement_is_named_after_the_unit() {
        let (explanation, _) = explanation_of_ones();
        let factor = explanation
            .factors
            .iter()
            .find(|factor| factor.id == "boolean-logic")
            .unwrap();
        let names: Vec<&str> = factor
            .measurements
            .as_ref()
            .unwrap()
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "Finding count",
                "Findings per 100 source lines",
                "Maximum boolean depth",
                "Source-line coverage"
            ]
        );
    }

    #[test]
    fn jev_factors_have_no_measurements_and_carry_answers() {
        let (explanation, _) = explanation_of_ones();
        let jev_factors: Vec<&Factor> = explanation
            .factors
            .iter()
            .filter(|factor| Rule::from_key(&factor.id).is_none())
            .collect();
        assert_eq!(jev_factors.len(), 7);
        assert!(jev_factors
            .iter()
            .all(|factor| factor.measurements.is_none()));
        assert!(jev_factors.iter().all(
            |factor| matches!(&factor.observed, Observed::Jev(observed) if observed.value == 0.5)
        ));
        assert!(explanation.explanation_notes[2].contains("not a calibrated confidence"));
    }

    #[test]
    fn factors_are_sorted_by_contribution() {
        let (explanation, _) = explanation_of_ones();
        assert!(explanation
            .factors
            .windows(2)
            .all(|pair| pair[0].score_contribution <= pair[1].score_contribution));
    }

    #[test]
    fn explanation_serializes_in_python_key_order() {
        let (explanation, _) = explanation_of_ones();
        let value = serde_json::to_value(&explanation).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            [
                "passed",
                "score",
                "risk",
                "baseline_score",
                "threshold_score",
                "threshold_risk",
                "score_scale",
                "score_margin",
                "explanation_residual",
                "factors",
                "explanation_method",
                "explanation_notes"
            ]
        );
        let factor_keys: Vec<&str> = value["factors"][0]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert!(factor_keys.starts_with(&["id", "name", "score_contribution"]));
        assert_eq!(factor_keys.last().copied(), Some("observed"));
    }

    #[test]
    fn score_and_boolean_boundaries_are_consistent() {
        let document = document_with(json!({"components": [{"constant": 0.5}]}));
        let model = Model::try_new(document).unwrap();
        let result = model.predict(&vec![0.0; feature_names().len()]).unwrap();
        assert_eq!(result.score, 5.0);
        assert_eq!(result.threshold_score, 5.0);
        assert!(!result.passed);
        assert_eq!(result.score_margin, 0.0);
        assert_eq!(sigmoid(1000.0), 1.0);
        assert_eq!(sigmoid(-1000.0), 0.0);
    }

    fn assert_rejected(changes: Value) {
        let error = Model::try_new(document_with(changes)).unwrap_err();
        assert!(matches!(error, Error::InvalidModel(_)));
    }

    #[test]
    fn rejects_non_numeric_threshold() {
        assert_rejected(json!({"threshold": null}));
        assert_rejected(json!({"threshold": "0.5"}));
        assert_rejected(json!({"threshold": true}));
    }

    #[test]
    fn rejects_threshold_outside_the_open_unit_interval() {
        assert_rejected(json!({"threshold": 2.0}));
        assert_rejected(json!({"threshold": 0.0}));
        assert_rejected(json!({"threshold": 1.0}));
    }

    #[test]
    fn rejects_wrong_sized_reference() {
        assert_rejected(json!({"reference": []}));
    }

    #[test]
    fn rejects_wrong_extractor_identity() {
        assert_rejected(json!({"extractor_identity": "wrong"}));
        assert_rejected(json!({"extractor_profile": "qlty-actual-depth-001"}));
    }

    #[test]
    fn rejects_wrong_schema_or_features() {
        assert_rejected(json!({"schema_version": 2}));
        assert_rejected(json!({"features": ["a"]}));
    }

    #[test]
    fn rejects_empty_components() {
        assert_rejected(json!({"components": []}));
    }

    #[test]
    fn rejects_invalid_constant_component() {
        assert_rejected(json!({"components": [{"constant": null}]}));
        assert_rejected(json!({"components": [{"constant": 1.5}]}));
    }

    #[test]
    fn rejects_short_logistic_component() {
        assert_rejected(json!({"components": [{"weights": [1.0], "intercept": 0.0}]}));
    }

    #[test]
    fn rejects_missing_keys() {
        let mut document = document();
        document.as_object_mut().unwrap().remove("reference");
        assert!(matches!(
            Model::try_new(document),
            Err(Error::InvalidModel(_))
        ));
    }

    #[test]
    fn rejects_wrong_length_input() {
        let model = Model::try_new(document()).unwrap();
        assert!(matches!(model.predict(&[]), Err(Error::InvalidInput)));
    }

    #[test]
    fn rejects_non_finite_input() {
        let model = Model::try_new(document()).unwrap();
        let values = vec![f64::NAN; feature_names().len()];
        assert!(matches!(model.predict(&values), Err(Error::InvalidInput)));
    }

    #[test]
    fn packaged_digest_matches_python_canonical_json() {
        let model = Model::embedded().unwrap();
        assert_eq!(model.sha256(), PACKAGED_SHA256);
    }

    #[test]
    fn shared_model_is_the_packaged_model() {
        let model = Model::shared().unwrap();
        assert_eq!(model.id(), "maintainability-003-actual-magnitudes");
        assert_eq!(model.sha256(), PACKAGED_SHA256);
    }

    #[test]
    fn info_serializes_in_python_key_order() {
        let info = Model::embedded().unwrap().info();
        let value = serde_json::to_value(&info).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            [
                "id",
                "sha256",
                "extractor_profile",
                "components",
                "features",
                "threshold_risk",
                "threshold_score",
                "score_scale",
                "training",
                "measurement_policy",
                "score_meaning"
            ]
        );
        assert_eq!(value["components"], 640);
        assert_eq!(value["features"], 39);
        assert_eq!(value["training"]["files"], 304);
    }

    #[test]
    fn python_float_repr_matches_cpython() {
        assert_eq!(python_float_repr(1e16), "1e+16");
        assert_eq!(python_float_repr(1e15), "1000000000000000.0");
        assert_eq!(
            python_float_repr(5.1736575202213965e-05),
            "5.1736575202213965e-05"
        );
        assert_eq!(python_float_repr(0.0001), "0.0001");
        assert_eq!(python_float_repr(0.00001), "1e-05");
        assert_eq!(
            python_float_repr(123456789012345678.0),
            "1.2345678901234568e+17"
        );
        assert_eq!(python_float_repr(0.0), "0.0");
        assert_eq!(python_float_repr(-0.0), "-0.0");
        assert_eq!(python_float_repr(-2.5), "-2.5");
        assert_eq!(python_float_repr(100.0), "100.0");
        assert_eq!(python_float_repr(0.6073781128742494), "0.6073781128742494");
    }

    #[test]
    fn canonical_json_matches_python_dumps() {
        let mut text = String::new();
        write_canonical(
            &json!({"b": [1, 2.0, null, true], "a": "q\"\\\n\u{e9}\u{1f600}", "c": {"z": 1e-7, "y": -3}}),
            &mut text,
        );
        assert_eq!(
            text,
            r#"{"a":"q\"\\\n\u00e9\ud83d\ude00","b":[1,2.0,null,true],"c":{"y":-3,"z":1e-07}}"#
        );
    }
}
