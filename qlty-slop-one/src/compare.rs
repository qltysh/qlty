//! Compare changed files with their earlier versions, as a pre-push hook or
//! an upstream diff does. A file declines when it passed the cutoff and no
//! longer does, when its score drops by more than the allowed amount, or
//! when it is new and does not pass.

use std::num::NonZeroUsize;
use std::path::PathBuf;

use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;

use crate::error::{Error, Result};
use crate::evaluate::{thread_pool, Evaluator};
use crate::features::{questions, FIELDS, RULES};
use crate::fsum::fsum;
use crate::jev::Usage;
use crate::model::{Model, ModelInfo, Observed};
use crate::report::{Document, Evaluated, FileOutcome, Skipped};

/// Where the current version of a changed file comes from.
#[derive(Clone, Debug)]
pub enum Contents {
    /// The file on disk at the change's path.
    OnDisk,
    /// These bytes, such as a blob from a commit being pushed.
    InMemory(Vec<u8>),
}

impl Contents {
    fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::OnDisk => None,
            Self::InMemory(bytes) => Some(bytes),
        }
    }
}

/// The version of a changed file to compare against.
#[derive(Clone, Debug)]
pub struct BaseVersion {
    /// The file's path in the base commit; it differs after a rename.
    pub path: PathBuf,
    pub commit: String,
    pub contents: Vec<u8>,
}

/// A changed file and, unless it is new, its earlier version.
#[derive(Clone, Debug)]
pub struct Change {
    pub path: PathBuf,
    pub contents: Contents,
    pub base: Option<BaseVersion>,
}

/// Why a file declined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Decline {
    /// The file passed the cutoff before and does not now.
    FellBelowCutoff,
    /// The score dropped by more than the allowed amount.
    DroppedTooFar,
    /// A new file does not pass the cutoff.
    NewBelowCutoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Verdict {
    Ok,
    Declined(Decline),
    /// Excluded or not source code; never judged.
    Skipped,
    /// The file or its earlier version could not be scored.
    Error,
}

/// The earlier version's result, known only once the current version scored.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Base {
    /// Nothing earlier could be scored: the file is new, or its earlier
    /// version was excluded or was not source code. It is judged as new.
    New,
    Scored {
        path: String,
        commit: String,
        score: f64,
        passed: bool,
    },
    Failed {
        path: String,
        commit: String,
        error: String,
    },
}

/// What factor impacts are measured against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Basis {
    /// The earlier version's score; the impacts add up to the change.
    Change,
    /// The model's reference score, for a file with no earlier score.
    Reference(f64),
}

/// One of the 15 factors' share of a score change or of a score, with the
/// measurements behind it.
#[derive(Clone, Debug)]
pub struct FactorImpact {
    /// The rule key or Jev question id.
    pub id: String,
    pub name: &'static str,
    pub impact: f64,
    /// The earlier measurement, when the basis is the change.
    pub before: Option<Observed>,
    pub after: Observed,
}

/// All 15 factor impacts, largest first.
#[derive(Clone, Debug)]
pub struct FactorImpacts {
    pub basis: Basis,
    /// The score change or the distance from the reference; the impacts
    /// add up to it.
    pub total: f64,
    pub factors: Vec<FactorImpact>,
}

#[derive(Clone, Debug)]
pub struct Comparison {
    pub outcome: FileOutcome,
    /// `None` when the current version was not scored.
    pub base: Option<Base>,
    pub verdict: Verdict,
    /// Why a declined file scored as it did; `None` for other verdicts.
    pub impacts: Option<FactorImpacts>,
}

impl Verdict {
    pub fn judge(outcome: &FileOutcome, base: Option<&Base>, max_drop: f64) -> Self {
        let head = match outcome {
            FileOutcome::Skipped(_) => return Self::Skipped,
            FileOutcome::Failed(_) => return Self::Error,
            FileOutcome::Evaluated(evaluated) => &evaluated.explanation,
        };
        match base {
            None | Some(Base::Failed { .. }) => Self::Error,
            Some(Base::New) if head.passed => Self::Ok,
            Some(Base::New) => Self::Declined(Decline::NewBelowCutoff),
            Some(Base::Scored { passed, .. }) if *passed && !head.passed => {
                Self::Declined(Decline::FellBelowCutoff)
            }
            Some(Base::Scored { score, .. }) if score - head.score > max_drop => {
                Self::Declined(Decline::DroppedTooFar)
            }
            Some(Base::Scored { .. }) => Self::Ok,
        }
    }
}

impl FactorImpacts {
    /// Splits the change from `base` to `head` along the straight feature
    /// path, as the trend report explains a changed file.
    fn between(base: &Evaluated, head: &Evaluated) -> Result<Self> {
        let deltas = Model::shared()?.explain_between(&base.features, &head.features)?;
        let jev_start = RULES.len() * FIELDS.len();
        let rules = RULES.iter().enumerate().map(|(index, rule)| {
            let start = index * FIELDS.len();
            (
                rule.key(),
                fsum(deltas[start..start + FIELDS.len()].iter().copied()),
            )
        });
        let jev = questions()
            .iter()
            .enumerate()
            .map(|(index, question)| (question.id.as_str(), deltas[jev_start + index]));
        let factors = rules
            .chain(jev)
            .map(|(id, impact)| {
                let after = factor(head, id)?;
                Ok(FactorImpact {
                    id: id.to_owned(),
                    name: after.name,
                    impact,
                    before: Some(factor(base, id)?.observed.clone()),
                    after: after.observed.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self::sorted(
            Basis::Change,
            head.explanation.score - base.explanation.score,
            factors,
        ))
    }

    /// Splits `head`'s distance from the model's reference score.
    fn reference(head: &Evaluated) -> Self {
        let explanation = &head.explanation;
        let factors = explanation
            .factors
            .iter()
            .map(|factor| FactorImpact {
                id: factor.id.clone(),
                name: factor.name,
                impact: factor.score_contribution,
                before: None,
                after: factor.observed.clone(),
            })
            .collect();
        Self::sorted(
            Basis::Reference(explanation.baseline_score),
            explanation.score - explanation.baseline_score,
            factors,
        )
    }

    fn sorted(basis: Basis, total: f64, mut factors: Vec<FactorImpact>) -> Self {
        factors.sort_by(|a, b| b.impact.abs().total_cmp(&a.impact.abs()));
        Self {
            basis,
            total,
            factors,
        }
    }
}

fn factor<'a>(evaluated: &'a Evaluated, id: &str) -> Result<&'a crate::model::Factor> {
    evaluated
        .explanation
        .factors
        .iter()
        .find(|factor| factor.id == id)
        .ok_or(Error::InvalidInput)
}

impl Evaluator {
    /// Compare changes on up to `jobs` worker threads, calling `on_change` as
    /// each finishes. Results keep the input order. The earlier version is
    /// scored only when the current one was.
    pub fn compare_all_with(
        &self,
        changes: &[Change],
        jobs: NonZeroUsize,
        max_drop: f64,
        on_change: &(dyn Fn(&Comparison) + Sync),
    ) -> Result<Vec<Comparison>> {
        thread_pool(jobs)?.install(|| {
            changes
                .par_iter()
                .map(|change| {
                    let comparison = self.compare(change, max_drop)?;
                    on_change(&comparison);
                    Ok(comparison)
                })
                .collect()
        })
    }

    fn compare(&self, change: &Change, max_drop: f64) -> Result<Comparison> {
        let outcome = self.current_outcome(change)?;
        let (base, base_evaluated) = match (&outcome, &change.base) {
            (FileOutcome::Evaluated(_), Some(base)) => {
                let (base, evaluated) = self.base_result(base)?;
                (Some(base), evaluated)
            }
            (FileOutcome::Evaluated(_), None) => (Some(Base::New), None),
            _ => (None, None),
        };
        let verdict = Verdict::judge(&outcome, base.as_ref(), max_drop);
        let impacts = match (&outcome, verdict, base_evaluated) {
            (FileOutcome::Evaluated(head), Verdict::Declined(_), Some(base)) => {
                Some(FactorImpacts::between(&base, head)?)
            }
            (FileOutcome::Evaluated(head), Verdict::Declined(_), None) => {
                Some(FactorImpacts::reference(head))
            }
            _ => None,
        };
        Ok(Comparison {
            outcome,
            base,
            verdict,
            impacts,
        })
    }

    fn current_outcome(&self, change: &Change) -> Result<FileOutcome> {
        match self.evaluate_file(&change.path, change.contents.bytes()) {
            Ok(outcome) => Ok(outcome),
            Err(error) if error.is_fatal() => Err(error),
            Err(error) if error.is_unsupported_source() => Ok(FileOutcome::Skipped(Skipped {
                path: change.path.display().to_string(),
                language: None,
                passed: None,
                score: None,
                skipped: true,
                reason: format!("Not a supported source file: {error}"),
                source_scope: json!({"policy": "unsupported-source-file"}),
            })),
            Err(error) => Ok(FileOutcome::failed(&change.path, &error)),
        }
    }

    fn base_result(&self, base: &BaseVersion) -> Result<(Base, Option<Evaluated>)> {
        let path = base.path.display().to_string();
        match self.evaluate_file(&base.path, Some(&base.contents)) {
            Ok(FileOutcome::Evaluated(evaluated)) => Ok((
                Base::Scored {
                    path,
                    commit: base.commit.clone(),
                    score: evaluated.explanation.score,
                    passed: evaluated.explanation.passed,
                },
                Some(evaluated),
            )),
            Ok(FileOutcome::Skipped(_)) => Ok((Base::New, None)),
            Ok(FileOutcome::Failed(failed)) => Ok((
                Base::Failed {
                    path,
                    commit: base.commit.clone(),
                    error: failed.error,
                },
                None,
            )),
            Err(error) if error.is_fatal() => Err(error),
            Err(error) if error.is_unsupported_source() => Ok((Base::New, None)),
            Err(error) => Ok((
                Base::Failed {
                    path,
                    commit: base.commit.clone(),
                    error: error.to_string(),
                },
                None,
            )),
        }
    }
}

/// One compared file in the JSON document.
#[derive(Clone, Debug, Serialize)]
pub struct ComparedFile {
    pub path: String,
    pub verdict: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decline: Option<Decline>,
    pub score: Option<f64>,
    pub base: Option<Base>,
    /// The current score minus the earlier score.
    pub change: Option<f64>,
}

/// The comparison part of the JSON document.
#[derive(Clone, Debug, Serialize)]
pub struct ComparisonSummary {
    pub max_drop: f64,
    /// No file declined.
    pub passed: bool,
    pub declined_files: usize,
    pub ok_files: usize,
    pub skipped_files: usize,
    pub error_files: usize,
    pub files: Vec<ComparedFile>,
}

impl ComparisonSummary {
    pub fn of(comparisons: &[Comparison], max_drop: f64) -> Self {
        let count = |wanted: fn(&Verdict) -> bool| {
            comparisons
                .iter()
                .filter(|comparison| wanted(&comparison.verdict))
                .count()
        };
        let declined_files = count(|verdict| matches!(verdict, Verdict::Declined(_)));
        Self {
            max_drop,
            passed: declined_files == 0,
            declined_files,
            ok_files: count(|verdict| *verdict == Verdict::Ok),
            skipped_files: count(|verdict| *verdict == Verdict::Skipped),
            error_files: count(|verdict| *verdict == Verdict::Error),
            files: comparisons.iter().map(ComparedFile::of).collect(),
        }
    }
}

impl ComparedFile {
    fn of(comparison: &Comparison) -> Self {
        let score = current_score(&comparison.outcome);
        let base_score = match &comparison.base {
            Some(Base::Scored { score, .. }) => Some(*score),
            _ => None,
        };
        let (verdict, decline) = match comparison.verdict {
            Verdict::Ok => ("ok", None),
            Verdict::Declined(decline) => ("declined", Some(decline)),
            Verdict::Skipped => ("skipped", None),
            Verdict::Error => ("error", None),
        };
        Self {
            path: comparison.outcome.path().to_owned(),
            verdict,
            decline,
            score,
            base: comparison.base.clone(),
            change: score.zip(base_score).map(|(score, base)| score - base),
        }
    }
}

fn current_score(outcome: &FileOutcome) -> Option<f64> {
    match outcome {
        FileOutcome::Evaluated(evaluated) => Some(evaluated.explanation.score),
        FileOutcome::Skipped(_) | FileOutcome::Failed(_) => None,
    }
}

impl Document {
    /// The document for compared files: the usual results for the current
    /// versions, plus the comparison.
    pub fn compared(
        model: ModelInfo,
        comparisons: &[Comparison],
        usage: Usage,
        max_drop: f64,
    ) -> Self {
        let results = comparisons
            .iter()
            .map(|comparison| comparison.outcome.clone())
            .collect();
        let mut document = Self::new(model, results, usage);
        document.comparison = Some(ComparisonSummary::of(comparisons, max_drop));
        document
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::Explanation;
    use crate::report::Failed;
    use serde_json::Value;

    pub(crate) fn evaluated(score: f64) -> FileOutcome {
        FileOutcome::Evaluated(Evaluated {
            path: "lib/file.py".to_owned(),
            language: "Python",
            source_scope: Value::Null,
            features: vec![],
            explanation: Explanation {
                passed: score > 5.0,
                score,
                risk: 0.0,
                baseline_score: 7.0,
                threshold_score: 5.0,
                threshold_risk: 0.6,
                score_scale: "threshold-centered-1-10-v1",
                score_margin: score - 5.0,
                explanation_residual: 0.0,
                factors: vec![],
                explanation_method: "",
                explanation_notes: vec![],
            },
        })
    }

    pub(crate) fn scored(score: f64) -> Base {
        Base::Scored {
            path: "lib/file.py".to_owned(),
            commit: "abc123".to_owned(),
            score,
            passed: score > 5.0,
        }
    }

    pub(crate) fn comparison(score: f64, base: Base) -> Comparison {
        let outcome = evaluated(score);
        let verdict = Verdict::judge(&outcome, Some(&base), 1.0);
        Comparison {
            outcome,
            base: Some(base),
            verdict,
            impacts: None,
        }
    }

    #[test]
    fn a_small_drop_is_ok() {
        assert_eq!(
            Verdict::judge(&evaluated(7.2), Some(&scored(8.0)), 1.0),
            Verdict::Ok
        );
    }

    #[test]
    fn a_drop_of_exactly_the_maximum_is_ok() {
        assert_eq!(
            Verdict::judge(&evaluated(7.0), Some(&scored(8.0)), 1.0),
            Verdict::Ok
        );
    }

    #[test]
    fn a_drop_beyond_the_maximum_declines() {
        assert_eq!(
            Verdict::judge(&evaluated(6.5), Some(&scored(8.0)), 1.0),
            Verdict::Declined(Decline::DroppedTooFar)
        );
    }

    #[test]
    fn falling_below_the_cutoff_declines_even_for_a_small_drop() {
        assert_eq!(
            Verdict::judge(&evaluated(4.9), Some(&scored(5.2)), 1.0),
            Verdict::Declined(Decline::FellBelowCutoff)
        );
    }

    #[test]
    fn a_file_already_below_the_cutoff_may_stay_there() {
        assert_eq!(
            Verdict::judge(&evaluated(2.6), Some(&scored(3.0)), 1.0),
            Verdict::Ok
        );
    }

    #[test]
    fn a_file_already_below_the_cutoff_may_not_drop_too_far() {
        assert_eq!(
            Verdict::judge(&evaluated(1.5), Some(&scored(3.0)), 1.0),
            Verdict::Declined(Decline::DroppedTooFar)
        );
    }

    #[test]
    fn a_new_file_that_passes_is_ok() {
        assert_eq!(
            Verdict::judge(&evaluated(6.0), Some(&Base::New), 1.0),
            Verdict::Ok
        );
    }

    #[test]
    fn a_new_file_below_the_cutoff_declines() {
        assert_eq!(
            Verdict::judge(&evaluated(4.0), Some(&Base::New), 1.0),
            Verdict::Declined(Decline::NewBelowCutoff)
        );
    }

    #[test]
    fn an_unscored_earlier_version_is_an_error() {
        let base = Base::Failed {
            path: "lib/file.py".to_owned(),
            commit: "abc123".to_owned(),
            error: "boom".to_owned(),
        };
        assert_eq!(
            Verdict::judge(&evaluated(4.0), Some(&base), 1.0),
            Verdict::Error
        );
    }

    #[test]
    fn a_failed_file_is_an_error() {
        let outcome = FileOutcome::Failed(Failed {
            path: "lib/file.py".to_owned(),
            passed: None,
            score: None,
            error: "boom".to_owned(),
        });
        assert_eq!(Verdict::judge(&outcome, None, 1.0), Verdict::Error);
    }

    #[test]
    fn summary_counts_verdicts() {
        let summary = ComparisonSummary::of(
            &[comparison(8.0, scored(8.1)), comparison(4.0, scored(6.0))],
            1.0,
        );
        assert_eq!(
            (summary.declined_files, summary.ok_files, summary.passed),
            (1, 1, false)
        );
    }

    #[test]
    fn summary_reports_the_change() {
        let summary = ComparisonSummary::of(&[comparison(7.5, scored(8.0))], 1.0);
        assert_eq!(summary.files[0].change, Some(-0.5));
    }

    #[test]
    fn a_declined_file_serializes_its_reason() {
        let summary = ComparisonSummary::of(&[comparison(6.5, scored(8.0))], 1.0);
        assert_eq!(
            serde_json::to_value(&summary.files[0]).unwrap()["decline"],
            "dropped-too-far"
        );
    }
}
