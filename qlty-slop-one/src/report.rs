//! Result documents, summaries, and text rendering, in slopdetect's shapes.

use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::features::Field;
use crate::jev::Usage;
use crate::model::{Explanation, ModelInfo, Observed};
use crate::scoring::format_score;

/// One evaluated, skipped, or failed file, serialized exactly as slopdetect did.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum FileOutcome {
    Evaluated(Evaluated),
    Skipped(Skipped),
    Failed(Failed),
}

#[derive(Clone, Debug, Serialize)]
pub struct Evaluated {
    pub path: String,
    pub language: &'static str,
    pub source_scope: Value,
    #[serde(flatten)]
    pub explanation: Explanation,
}

#[derive(Clone, Debug, Serialize)]
pub struct Skipped {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'static str>,
    pub passed: Option<bool>,
    pub score: Option<f64>,
    pub skipped: bool,
    pub reason: String,
    pub source_scope: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct Failed {
    pub path: String,
    pub passed: Option<bool>,
    pub score: Option<f64>,
    pub error: String,
}

impl FileOutcome {
    pub fn path(&self) -> &str {
        match self {
            Self::Evaluated(outcome) => &outcome.path,
            Self::Skipped(outcome) => &outcome.path,
            Self::Failed(outcome) => &outcome.path,
        }
    }

    pub fn failed(path: &Path, error: &crate::Error) -> Self {
        Self::Failed(Failed {
            path: path.display().to_string(),
            passed: None,
            score: None,
            error: error.to_string(),
        })
    }

    fn evaluated(&self) -> Option<&Evaluated> {
        match self {
            Self::Evaluated(outcome) => Some(outcome),
            Self::Skipped(_) | Self::Failed(_) => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LowestFile {
    pub path: String,
    pub score: f64,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Summary {
    pub scope: &'static str,
    pub evaluated_files: usize,
    pub passed_files: usize,
    pub failed_files: usize,
    pub skipped_files: usize,
    pub error_files: usize,
    pub median_score: Option<f64>,
    pub pass_rate: Option<f64>,
    pub lowest_scoring_files: Vec<LowestFile>,
}

impl Summary {
    pub fn of(results: &[FileOutcome]) -> Self {
        let evaluated: Vec<&Evaluated> =
            results.iter().filter_map(FileOutcome::evaluated).collect();
        let passed = evaluated
            .iter()
            .filter(|outcome| outcome.explanation.passed)
            .count();
        let mut lowest: Vec<&Evaluated> = evaluated.clone();
        lowest.sort_by(|a, b| {
            a.explanation
                .score
                .total_cmp(&b.explanation.score)
                .then_with(|| a.path.cmp(&b.path))
        });
        Self {
            scope: "supplied-files",
            evaluated_files: evaluated.len(),
            passed_files: passed,
            failed_files: evaluated.len() - passed,
            skipped_files: results
                .iter()
                .filter(|r| matches!(r, FileOutcome::Skipped(_)))
                .count(),
            error_files: results
                .iter()
                .filter(|r| matches!(r, FileOutcome::Failed(_)))
                .count(),
            median_score: median(evaluated.iter().map(|outcome| outcome.explanation.score)),
            pass_rate: (!evaluated.is_empty()).then(|| passed as f64 / evaluated.len() as f64),
            lowest_scoring_files: lowest
                .into_iter()
                .take(5)
                .map(|outcome| LowestFile {
                    path: outcome.path.clone(),
                    score: outcome.explanation.score,
                    passed: outcome.explanation.passed,
                })
                .collect(),
        }
    }
}

/// `statistics.median`: the mean of the two middle values for an even count.
fn median(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut values: Vec<f64> = values.collect();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[middle])
    } else {
        Some((values[middle - 1] + values[middle]) / 2.0)
    }
}

/// The whole JSON document for one invocation.
#[derive(Clone, Debug, Serialize)]
pub struct Document {
    pub schema_version: u32,
    pub model: ModelInfo,
    pub passed: Option<bool>,
    pub results: Vec<FileOutcome>,
    pub errors: usize,
    pub skipped: usize,
    pub evaluated: usize,
    pub summary: Summary,
    pub usage: Usage,
}

impl Document {
    pub fn new(model: ModelInfo, results: Vec<FileOutcome>, usage: Usage) -> Self {
        let summary = Summary::of(&results);
        let passed = if summary.error_files > 0 {
            Some(false)
        } else if summary.evaluated_files > 0 {
            Some(summary.failed_files == 0)
        } else {
            None
        };
        Self {
            schema_version: 1,
            model,
            passed,
            errors: summary.error_files,
            skipped: summary.skipped_files,
            evaluated: summary.evaluated_files,
            results,
            summary,
            usage,
        }
    }
}

/// Python's `format(value, 'g')` for the integral magnitudes SlopOne reports.
fn format_general(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e16 {
        format!("{}", value as i64)
    } else {
        let text = format!("{:.6}", value);
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn render_text(outcome: &FileOutcome, top: usize) -> String {
    let result = match outcome {
        FileOutcome::Failed(failed) => return format!("ERROR {}: {}", failed.path, failed.error),
        FileOutcome::Skipped(skipped) => {
            return format!("SKIP {}: {}", skipped.path, skipped.reason)
        }
        FileOutcome::Evaluated(result) => result,
    };
    let explanation = &result.explanation;
    let status = if explanation.passed { "PASS" } else { "FAIL" };
    let mut lines = vec![format!(
        "{status} {}  {}/10 (pass above {:.2}; {})",
        result.path,
        format_score(
            Some(explanation.score),
            2,
            Some(explanation.passed),
            explanation.threshold_score
        ),
        explanation.threshold_score,
        result.language
    )];
    let margin = explanation.score_margin;
    if margin == 0.0 {
        lines.push("  Score is at the cutoff.".to_string());
    } else {
        let direction = if margin > 0.0 { "above" } else { "below" };
        let distance = if margin.abs() >= 0.005 {
            format!("{:.2}", margin.abs())
        } else {
            "less than 0.01".to_string()
        };
        lines.push(format!(
            "  Score is {distance} points {direction} the cutoff."
        ));
    }
    if let Some(modules) = result
        .source_scope
        .get("excluded_modules")
        .and_then(Value::as_array)
    {
        if !modules.is_empty() {
            lines.push(format!(
                "  Excluded {} inline test module(s); analyzed {} of {} lines. Locations refer to the original file.",
                modules.len(),
                result.source_scope["analyzed_lines"],
                result.source_scope["original_lines"]
            ));
        }
    }
    let negative: Vec<_> = explanation
        .factors
        .iter()
        .filter(|factor| factor.score_contribution < -0.005)
        .take(top)
        .collect();
    let mut positive: Vec<_> = explanation
        .factors
        .iter()
        .filter(|factor| factor.score_contribution > 0.005)
        .collect();
    positive.sort_by(|a, b| b.score_contribution.total_cmp(&a.score_contribution));
    positive.truncate(top);
    lines.push(format!(
        "  Reference score: {:.2}; factors show changes from that reference.",
        explanation.baseline_score
    ));
    for (factors, label) in [
        (&negative, "Main factors lowering the score"),
        (&positive, "Main factors raising the score"),
    ] {
        if factors.is_empty() {
            continue;
        }
        lines.push(format!("  {label}:"));
        for factor in factors {
            let detail = match &factor.observed {
                Observed::Qlty(summary) => {
                    let mut detail = format!("{} findings", summary.count);
                    if summary.count > 0 {
                        detail.push_str(&format!(
                            "; max {} {}, threshold {}",
                            summary.magnitude_unit.to_lowercase(),
                            format_general(summary.max_actual),
                            format_general(summary.threshold as f64)
                        ));
                    }
                    let locations: Vec<String> = summary
                        .locations
                        .iter()
                        .take(3)
                        .map(|span| {
                            if span.start_line == span.end_line {
                                span.start_line.to_string()
                            } else {
                                format!("{}–{}", span.start_line, span.end_line)
                            }
                        })
                        .collect();
                    if !locations.is_empty() {
                        detail.push_str("; lines ");
                        detail.push_str(&locations.join(", "));
                    }
                    detail
                }
                Observed::Jev(observed) => format!("Jev assessment {:.2}", observed.value),
            };
            lines.push(format!(
                "    {:+.2}  {} ({detail})",
                factor.score_contribution, factor.name
            ));
            if let Some(measurements) = &factor.measurements {
                let parts: Vec<String> = measurements
                    .iter()
                    .map(|m| {
                        let label = Field::from_id(m.id).map_or(m.id, Field::short_label);
                        format!("{label} {:+.2}", m.score_contribution)
                    })
                    .collect();
                lines.push(format!("      Measurements: {}", parts.join("; ")));
            }
        }
    }
    if negative
        .iter()
        .chain(positive.iter())
        .any(|factor| factor.measurements.is_some())
    {
        lines.push("  Qlty measurements describe related properties of the same findings; their lines can overlap across factors.".to_string());
    }
    lines.join("\n")
}

pub fn render_summary(summary: &Summary) -> String {
    let mut lines = vec![format!(
        "Summary of supplied files: {} evaluated, {} passed, {} failed, {} skipped, {} errors.",
        summary.evaluated_files,
        summary.passed_files,
        summary.failed_files,
        summary.skipped_files,
        summary.error_files
    )];
    match (summary.median_score, summary.pass_rate) {
        (Some(median), Some(rate)) => {
            lines.push(format!(
                "  Median file score: {median:.2}/10; pass rate: {:.1}%.",
                rate * 100.0
            ));
            lines.push("  Lowest-scoring files:".to_string());
            for row in &summary.lowest_scoring_files {
                lines.push(format!(
                    "    {}/10  {}  {}",
                    format_score(
                        Some(row.score),
                        2,
                        Some(row.passed),
                        crate::scoring::SCORE_CUTOFF
                    ),
                    if row.passed { "PASS" } else { "FAIL" },
                    row.path
                ));
            }
        }
        _ => lines.push(
            "  Median score: unavailable; pass rate: unavailable (no evaluated files).".to_string(),
        ),
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluated(path: &str, score: f64, passed: bool) -> FileOutcome {
        FileOutcome::Evaluated(Evaluated {
            path: path.to_string(),
            language: "Python",
            source_scope: Value::Null,
            explanation: Explanation {
                passed,
                score,
                risk: 0.0,
                baseline_score: 6.0,
                threshold_score: 5.0,
                threshold_risk: 0.5,
                score_scale: "threshold-centered-1-10-v1",
                score_margin: score - 5.0,
                explanation_residual: 0.0,
                factors: vec![],
                explanation_method: "",
                explanation_notes: vec![],
            },
        })
    }

    fn skipped(path: &str) -> FileOutcome {
        FileOutcome::Skipped(Skipped {
            path: path.to_string(),
            language: None,
            passed: None,
            score: None,
            skipped: true,
            reason: "Test file excluded".to_string(),
            source_scope: Value::Null,
        })
    }

    #[test]
    fn median_of_even_count_is_the_mean_of_the_middle_values() {
        assert_eq!(median([1.0, 4.0, 2.0, 8.0].into_iter()), Some(3.0));
    }

    #[test]
    fn median_of_odd_count_is_the_middle_value() {
        assert_eq!(median([5.0, 1.0, 3.0].into_iter()), Some(3.0));
    }

    #[test]
    fn summary_uses_only_evaluated_files_and_orders_lowest_by_score_then_path() {
        let results = vec![
            evaluated("b.py", 7.0, true),
            evaluated("a.py", 7.0, true),
            evaluated("c.py", 2.0, false),
            skipped("t.py"),
            FileOutcome::failed(Path::new("e.py"), &crate::Error::UnknownLanguage),
        ];
        let summary = Summary::of(&results);
        assert_eq!(
            (
                summary.evaluated_files,
                summary.passed_files,
                summary.failed_files,
                summary.skipped_files,
                summary.error_files
            ),
            (3, 2, 1, 1, 1)
        );
        assert_eq!(summary.median_score, Some(7.0));
        assert_eq!(summary.pass_rate, Some(2.0 / 3.0));
        let lowest: Vec<&str> = summary
            .lowest_scoring_files
            .iter()
            .map(|row| row.path.as_str())
            .collect();
        assert_eq!(lowest, ["c.py", "a.py", "b.py"]);
    }

    #[test]
    fn no_evaluated_files_has_no_median_or_pass_rate() {
        let summary = Summary::of(&[skipped("t.py")]);
        assert_eq!((summary.median_score, summary.pass_rate), (None, None));
    }

    #[test]
    fn document_passed_is_false_with_errors_and_none_without_evaluations() {
        let usage = Usage {
            requests: 0,
            cost_upper_bound_usd: 0.0,
            input_tokens: 0,
            unconfirmed_requests: 0,
            budget_usd: 1.0,
        };
        let info = || crate::model::Model::shared().unwrap().info();
        let with_error = Document::new(
            info(),
            vec![
                evaluated("a.py", 8.0, true),
                FileOutcome::failed(Path::new("e.py"), &crate::Error::UnknownLanguage),
            ],
            usage.clone(),
        );
        let only_skips = Document::new(info(), vec![skipped("t.py")], usage.clone());
        let all_pass = Document::new(info(), vec![evaluated("a.py", 8.0, true)], usage);
        assert_eq!(
            (with_error.passed, only_skips.passed, all_pass.passed),
            (Some(false), None, Some(true))
        );
    }

    #[test]
    fn summary_text_lists_lowest_files_with_pass_and_fail() {
        let summary = Summary::of(&[evaluated("a.py", 8.25, true), evaluated("b.py", 3.0, false)]);
        let text = render_summary(&summary);
        assert!(text.contains("2 evaluated, 1 passed, 1 failed, 0 skipped, 0 errors."));
        assert!(text.contains("Median file score: 5.62/10; pass rate: 50.0%."));
        assert!(text.contains("    3.00/10  FAIL  b.py"));
    }

    #[test]
    fn text_shows_margin_and_reference_score() {
        let text = render_text(&evaluated("a.py", 4.85, false), 3);
        assert!(text.starts_with("FAIL a.py  4.85/10 (pass above 5.00; Python)"));
        assert!(text.contains("  Score is 0.15 points below the cutoff."));
        assert!(text.contains("  Reference score: 6.00; factors show changes from that reference."));
    }

    #[test]
    fn tiny_margins_are_described_without_rounding_to_zero() {
        let text = render_text(&evaluated("a.py", 5.003, true), 3);
        assert!(text.contains("less than 0.01 points above the cutoff"));
    }

    #[test]
    fn general_format_drops_fractional_zeros() {
        assert_eq!(
            (
                format_general(8.0),
                format_general(20.0),
                format_general(4.5)
            ),
            ("8".to_string(), "20".to_string(), "4.5".to_string())
        );
    }
}
