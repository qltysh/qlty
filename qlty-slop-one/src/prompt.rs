//! Compared files as text. When a file declined, the text is a refactoring
//! prompt: guidance for a coding agent around the results, adapted from the
//! trend report's single-file prompt. The two copies of the guidance change
//! independently.

use crate::compare::{Base, Basis, Comparison, ComparisonSummary, Decline, FactorImpact, Verdict};
use crate::features::questions;
use crate::measure::RuleSummary;
use crate::model::Observed;
use crate::report::{excluded_modules_line, format_general, line_ranges, Evaluated, FileOutcome};
use crate::scoring::{format_score, SCORE_CUTOFF};

/// Impacts smaller than this are left to "Other factors".
const MINIMUM_SHOWN_IMPACT: f64 = 0.005;

/// Why the comparison ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framing {
    /// As a Git pre-push hook: a decline blocks the push.
    PrePush,
    /// On request.
    Standalone,
}

/// What the prompt asks a coding agent to do.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptMode {
    Recommend,
    Refactor,
}

/// A pushed commit and the last commit on its line that a remote has.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionPair {
    pub base: String,
    pub pushed: String,
}

/// The versions that were compared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Revisions {
    Pushed(Vec<RevisionPair>),
    WorkingTree {
        upstream: String,
        merge_base: String,
    },
}

#[derive(Clone, Debug)]
pub struct TextOptions {
    /// How many leading factors to show for each declined file.
    pub top: usize,
    pub max_drop: f64,
    pub framing: Framing,
    pub prompt: PromptMode,
    pub revisions: Revisions,
}

/// The whole text output: the prompt and declined files when any declined,
/// then one line for each other scored file, then the summary.
pub fn render_comparisons(comparisons: &[Comparison], options: &TextOptions) -> String {
    let declined: Vec<(&Comparison, &Evaluated, Decline)> = comparisons
        .iter()
        .filter_map(
            |comparison| match (&comparison.outcome, comparison.verdict) {
                (FileOutcome::Evaluated(evaluated), Verdict::Declined(decline)) => {
                    Some((comparison, evaluated, decline))
                }
                _ => None,
            },
        )
        .collect();
    let others: Vec<String> = comparisons.iter().filter_map(other_line).collect();
    let mut sections = vec![];
    if declined.is_empty() {
        if !others.is_empty() {
            sections.push(others.join("\n"));
        }
    } else {
        sections.extend(preamble(declined.len(), options));
        sections.extend(declined.iter().map(|(comparison, evaluated, decline)| {
            declined_section(comparison, evaluated, *decline, options)
        }));
        sections.extend(questions_section(&declined, options.top));
        if !others.is_empty() {
            sections.push(format!("Other changed files:\n{}", others.join("\n")));
        }
    }
    sections.push(summary_line(&ComparisonSummary::of(
        comparisons,
        options.max_drop,
    )));
    sections.join("\n\n")
}

fn preamble(declined: usize, options: &TextOptions) -> Vec<String> {
    let files = plural(declined, "changed file");
    let opening = match options.framing {
        Framing::PrePush => {
            format!("qlty slop-one blocked this push: {files} declined in maintainability.")
        }
        Framing::Standalone => {
            format!("qlty slop-one found {files} that declined in maintainability.")
        }
    };
    let task = match options.prompt {
        PromptMode::Recommend => "This project was analyzed by qlty slop-one. Review the declined files below and recommend refactorings that improve maintainability for human software engineers.",
        PromptMode::Refactor => "This project was analyzed by qlty slop-one. Refactor the declined files below to improve maintainability for human software engineers.",
    };
    let mut guidance = vec![
        "Consider all code quality improvements, not only the listed findings. Do not over-fit recommendations to the quality model or assume that a FAIL status proves a defect. A passing file may also have worthwhile improvements.",
        "Improvements may extend beyond the declined files, including reorganizing files and folders, splitting large files along clear responsibilities, or structural changes across related files when justified.",
        match options.prompt {
            PromptMode::Recommend => "Avoid extraneous indirection and never make the code worse just to improve the analysis. Preserve intended behavior. It is valid to recommend no change.",
            PromptMode::Refactor => "Avoid extraneous indirection and never make the code worse just to improve the analysis. Preserve intended behavior. It is valid to make no change.",
        },
        "Treat this like a greenfield app where the top priorities are elegance, quality, and long-term maintainability for human software engineers.",
    ];
    if options.framing == Framing::PrePush {
        guidance.push("Do not skip this check with git push --no-verify yourself. If you conclude that a decline is acceptable, say so and leave that decision to the developer.");
    }
    let outcome = match options.prompt {
        PromptMode::Recommend => "Return concrete recommendations, in priority order. For each, explain the source-level problem, the proposed refactoring, which files would change, why it helps readers, and how to verify behavior. Include useful file and symbol references. Explain which findings you agree or disagree with and why. If no refactoring is justified, say so.".to_owned(),
        PromptMode::Refactor => format!(
            "Make the refactorings, in priority order. Then report each one: the source-level problem, the change you made, which files changed, why it helps readers, and how you verified behavior. Explain which findings you agree or disagree with and why. If no refactoring is justified, change nothing and say so.\nRun the project's tests after refactoring. You can run `qlty slop-one --upstream {}` to see the new scores, but never change code only to raise a score.",
            rescore_reference(&options.revisions)
        ),
    };
    let version = match options.revisions {
        Revisions::Pushed(_) => "the pushed version",
        Revisions::WorkingTree { .. } => "the working tree",
    };
    let how_to_read = format!(
        "How to read the results: scores run from 1 to 10, and higher is better. A file passes when its score is above {SCORE_CUTOFF:.2}. A file declines when it passed before and now scores {SCORE_CUTOFF:.2} or below, when its score drops by more than {:.2} points, or when it is new and does not pass. Factor impacts are in score points: positive impact raises the score, and negative impact lowers it. They are model attributions, not proof of a code defect or estimates of refactoring benefit. A smell's impact can change even when its findings did not, because its density and coverage depend on the file's length. The results list summary measurements, not every finding; inspect the source to locate problems. Missing evidence is not a zero value. AI assessments use a 0–100 scale. Line numbers refer to {version}.",
        options.max_drop
    );
    let mut sections = vec![opening, task.to_owned()];
    sections.extend(revisions_section(&options.revisions));
    sections.push(guidance.join("\n"));
    sections.push(outcome);
    sections.push(how_to_read);
    sections
}

/// The revision line and the working context, or nothing when no pushed
/// ref had a base.
fn revisions_section(revisions: &Revisions) -> Vec<String> {
    match revisions {
        Revisions::WorkingTree {
            upstream,
            merge_base,
        } => vec![
            format!("Compared revisions: {merge_base} (merge base with {upstream}) → working tree"),
            format!("Use the current checkout as the working context. The analysis covers the working tree, including uncommitted changes. See what changed with `git diff {merge_base} -- <path>`. Do not reset the checkout."),
        ],
        Revisions::Pushed(pairs) => match pairs.as_slice() {
            [] => vec![],
            [pair] => vec![
                format!(
                    "Compared revisions: {} (already on a remote) → {} (being pushed)",
                    pair.base, pair.pushed
                ),
                format!("Use the current checkout as the working context. The analysis covers the commits being pushed, not uncommitted changes. See what the push changed with `git diff {} {} -- <path>`. Do not reset the checkout.", pair.base, pair.pushed),
            ],
            pairs => {
                let lines: Vec<String> = pairs
                    .iter()
                    .map(|pair| {
                        format!(
                            "  {} (already on a remote) → {} (being pushed)",
                            pair.base, pair.pushed
                        )
                    })
                    .collect();
                vec![
                    format!("Compared revisions:\n{}", lines.join("\n")),
                    "Use the current checkout as the working context. The analysis covers the commits being pushed, not uncommitted changes. See what the push changed with `git diff <base> <pushed> -- <path>`, using the revisions above. Do not reset the checkout.".to_owned(),
                ]
            }
        },
    }
}

/// What to pass to `--upstream` to score the refactored working tree.
fn rescore_reference(revisions: &Revisions) -> &str {
    match revisions {
        Revisions::WorkingTree { upstream, .. } => upstream,
        Revisions::Pushed(pairs) => match pairs.as_slice() {
            [pair] => &pair.base,
            _ => "<base>",
        },
    }
}

fn declined_section(
    comparison: &Comparison,
    evaluated: &Evaluated,
    decline: Decline,
    options: &TextOptions,
) -> String {
    let explanation = &evaluated.explanation;
    let mut lines = vec![
        format!(
            "DECLINED {}  {}; pass above {:.2}; {})",
            evaluated.path,
            scores(comparison, evaluated),
            explanation.threshold_score,
            evaluated.language
        ),
        match (decline, &comparison.base) {
            (Decline::FellBelowCutoff, _) => {
                "  The file passed before and now falls below the cutoff.".to_owned()
            }
            (Decline::DroppedTooFar, Some(Base::Scored { score, .. })) => format!(
                "  The score dropped by {:.2} points; the most allowed is {:.2}.",
                score - explanation.score,
                options.max_drop
            ),
            (Decline::DroppedTooFar, _) => format!(
                "  The score dropped by more than the {:.2} points allowed.",
                options.max_drop
            ),
            (Decline::NewBelowCutoff, _) => "  New files must score above the cutoff.".to_owned(),
        },
    ];
    if let Some(Base::Scored { path, .. }) = &comparison.base {
        if *path != evaluated.path {
            lines.push(format!("  Renamed from {path}."));
        }
    }
    lines.extend(excluded_modules_line(evaluated));
    if let Some(impacts) = &comparison.impacts {
        lines.push(match impacts.basis {
            Basis::Change => format!(
                "  Factors behind the change (they sum to {:+.2}):",
                impacts.total
            ),
            Basis::Reference(reference) => format!(
                "  This file is new, so there is no earlier score. Its factors explain the score against the model's reference ({reference:.2}), not a decline.\n  Leading factors:"
            ),
        });
        let (shown, other) = leading(&impacts.factors, impacts.total, options.top);
        lines.extend(shown.iter().map(|factor| {
            format!(
                "    {:+.2}  {} ({})",
                factor.impact,
                factor.name,
                evidence(factor)
            )
        }));
        if other.abs() >= MINIMUM_SHOWN_IMPACT {
            lines.push(format!("    {other:+.2}  Other factors"));
        }
    }
    lines.join("\n")
}

/// The largest impacts, and what the rest add up to.
fn leading(factors: &[FactorImpact], total: f64, top: usize) -> (Vec<&FactorImpact>, f64) {
    let shown: Vec<&FactorImpact> = factors
        .iter()
        .filter(|factor| factor.impact.abs() >= MINIMUM_SHOWN_IMPACT)
        .take(top)
        .collect();
    let other = total - crate::fsum::fsum(shown.iter().map(|factor| factor.impact));
    (shown, other)
}

fn evidence(factor: &FactorImpact) -> String {
    match (&factor.before, &factor.after) {
        (before, Observed::Qlty(after)) => {
            let before = match before {
                Some(Observed::Qlty(before)) => Some(before),
                _ => None,
            };
            smell_evidence(before, after)
        }
        (before, Observed::Jev(after)) => {
            let after_text = assessment(after.value);
            match before {
                Some(Observed::Jev(before)) if assessment(before.value) != after_text => {
                    format!("AI assessment {} → {after_text}", assessment(before.value))
                }
                _ => format!("AI assessment {after_text}"),
            }
        }
    }
}

fn smell_evidence(before: Option<&RuleSummary>, after: &RuleSummary) -> String {
    let mut parts = vec![match before {
        Some(before) if before.count != after.count => {
            format!("findings {} → {}", before.count, after.count)
        }
        _ => format!("findings {}", after.count),
    }];
    if after.count > 0 {
        let unit = after.magnitude_unit.to_lowercase();
        parts.push(match before {
            Some(before) if before.count > 0 && before.max_actual != after.max_actual => format!(
                "max {unit} {} → {}",
                format_general(before.max_actual),
                format_general(after.max_actual)
            ),
            _ => format!("max {unit} {}", format_general(after.max_actual)),
        });
        if let Some(lines) = line_ranges(&after.locations) {
            parts.push(format!("lines {lines}"));
        }
    }
    parts.join("; ")
}

fn assessment(value: f64) -> String {
    format!("{:.0}", value * 100.0)
}

/// The exact question behind each AI factor shown for a declined file.
fn questions_section(
    declined: &[(&Comparison, &Evaluated, Decline)],
    top: usize,
) -> Option<String> {
    let mut ids: Vec<&str> = vec![];
    for (comparison, _, _) in declined {
        let Some(impacts) = &comparison.impacts else {
            continue;
        };
        let (shown, _) = leading(&impacts.factors, impacts.total, top);
        for factor in shown {
            if matches!(factor.after, Observed::Jev(_)) && !ids.contains(&factor.id.as_str()) {
                ids.push(&factor.id);
            }
        }
    }
    let lines: Vec<String> = questions()
        .iter()
        .filter(|question| ids.contains(&question.id.as_str()))
        .map(|question| format!("  {}: {}", question.label(), question.instructions))
        .collect();
    (!lines.is_empty()).then(|| format!("AI assessment questions:\n{}", lines.join("\n")))
}

/// One line for a file that did not decline; nothing for a skipped file.
fn other_line(comparison: &Comparison) -> Option<String> {
    match (&comparison.outcome, comparison.verdict, &comparison.base) {
        (FileOutcome::Skipped(_), _, _) | (_, Verdict::Declined(_), _) => None,
        (FileOutcome::Failed(failed), _, _) => {
            Some(format!("ERROR {}: {}", failed.path, failed.error))
        }
        (
            FileOutcome::Evaluated(evaluated),
            Verdict::Error,
            Some(Base::Failed { path, error, .. }),
        ) => Some(format!(
            "ERROR {}: the earlier version at {path} could not be scored: {error}",
            evaluated.path
        )),
        (FileOutcome::Evaluated(evaluated), _, _) => Some(format!(
            "OK {}  {})",
            evaluated.path,
            scores(comparison, evaluated)
        )),
    }
}

/// `8.00 → 7.50/10 (-0.50` or `7.50/10 (new file`, left open for the caller
/// to finish.
fn scores(comparison: &Comparison, evaluated: &Evaluated) -> String {
    let explanation = &evaluated.explanation;
    let score = format_score(
        Some(explanation.score),
        2,
        Some(explanation.passed),
        explanation.threshold_score,
    );
    match &comparison.base {
        Some(Base::Scored {
            score: base_score,
            passed,
            ..
        }) => format!(
            "{} → {score}/10 ({:+.2}",
            format_score(
                Some(*base_score),
                2,
                Some(*passed),
                explanation.threshold_score
            ),
            explanation.score - base_score
        ),
        _ => format!("{score}/10 (new file"),
    }
}

fn summary_line(summary: &ComparisonSummary) -> String {
    format!(
        "Compared {} with their earlier versions: {} declined, {} OK, {} skipped, {}. Largest allowed drop: {:.2} points.",
        plural(summary.files.len(), "changed file"),
        summary.declined_files,
        summary.ok_files,
        summary.skipped_files,
        plural(summary.error_files, "error"),
        summary.max_drop
    )
}

fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::tests::{comparison, scored};
    use crate::compare::FactorImpacts;
    use crate::jev::Excerpt;
    use crate::measure::LineSpan;
    use crate::model::JevObserved;

    fn options(framing: Framing, prompt: PromptMode) -> TextOptions {
        TextOptions {
            top: 5,
            max_drop: 1.0,
            framing,
            prompt,
            revisions: Revisions::Pushed(vec![RevisionPair {
                base: "aaaaaaa".to_owned(),
                pushed: "bbbbbbb".to_owned(),
            }]),
        }
    }

    fn summary(count: usize, max_actual: f64, locations: Vec<LineSpan>) -> RuleSummary {
        RuleSummary {
            count,
            max_magnitude: max_actual,
            covered_lines: 0,
            locations,
            max_actual,
            threshold: 15,
            magnitude_basis: "actual",
            magnitude_unit: "Duplicated lines",
        }
    }

    fn jev(value: f64) -> Observed {
        Observed::Jev(JevObserved {
            value,
            excerpts: Vec::<Excerpt>::new(),
        })
    }

    fn smell_impact(impact: f64) -> FactorImpact {
        FactorImpact {
            id: "similar-code".to_owned(),
            name: "Similar code",
            impact,
            before: Some(Observed::Qlty(summary(0, 0.0, vec![]))),
            after: Observed::Qlty(summary(
                1,
                18.0,
                vec![
                    LineSpan {
                        start_line: 17,
                        end_line: 34,
                    },
                    LineSpan {
                        start_line: 37,
                        end_line: 54,
                    },
                ],
            )),
        }
    }

    fn jev_impact(impact: f64) -> FactorImpact {
        FactorImpact {
            id: "duplicated_logic".to_owned(),
            name: "Duplicated logic",
            impact,
            before: Some(jev(0.26)),
            after: jev(0.62),
        }
    }

    fn declined() -> Comparison {
        let mut declined = comparison(6.5, scored(8.0));
        declined.impacts = Some(FactorImpacts {
            basis: Basis::Change,
            total: -1.5,
            factors: vec![smell_impact(-1.0), jev_impact(-0.45)],
        });
        declined
    }

    fn render(comparisons: &[Comparison], framing: Framing, prompt: PromptMode) -> String {
        render_comparisons(comparisons, &options(framing, prompt))
    }

    #[test]
    fn without_a_decline_only_results_print() {
        assert_eq!(
            render(
                &[comparison(7.5, scored(8.0))],
                Framing::PrePush,
                PromptMode::Recommend
            ),
            "OK lib/file.py  8.00 → 7.50/10 (-0.50)\n\nCompared 1 changed file with their earlier versions: 0 declined, 1 OK, 0 skipped, 0 errors. Largest allowed drop: 1.00 points."
        );
    }

    #[test]
    fn a_pre_push_decline_opens_with_the_blocked_push() {
        let text = render(&[declined()], Framing::PrePush, PromptMode::Recommend);
        assert_eq!(
            text.lines().next().unwrap(),
            "qlty slop-one blocked this push: 1 changed file declined in maintainability."
        );
    }

    #[test]
    fn a_standalone_decline_opens_with_the_finding() {
        let text = render(&[declined()], Framing::Standalone, PromptMode::Recommend);
        assert_eq!(
            text.lines().next().unwrap(),
            "qlty slop-one found 1 changed file that declined in maintainability."
        );
    }

    #[test]
    fn only_a_pre_push_prompt_mentions_no_verify() {
        let text = render(&[declined()], Framing::Standalone, PromptMode::Recommend);
        assert!(!text.contains("--no-verify"));
    }

    #[test]
    fn the_default_prompt_asks_for_recommendations() {
        let text = render(&[declined()], Framing::PrePush, PromptMode::Recommend);
        assert!(text.contains("Return concrete recommendations, in priority order."));
    }

    #[test]
    fn the_refactor_prompt_asks_for_changes() {
        let text = render(&[declined()], Framing::PrePush, PromptMode::Refactor);
        assert!(
            text.contains("You can run `qlty slop-one --upstream aaaaaaa` to see the new scores")
        );
    }

    #[test]
    fn a_declined_file_lists_what_changed() {
        let text = render(&[declined()], Framing::PrePush, PromptMode::Recommend);
        let section: Vec<&str> = text
            .split("\n\n")
            .find(|section| section.starts_with("DECLINED"))
            .unwrap()
            .lines()
            .collect();
        assert_eq!(
            section,
            [
                "DECLINED lib/file.py  8.00 → 6.50/10 (-1.50; pass above 5.00; Python)",
                "  The score dropped by 1.50 points; the most allowed is 1.00.",
                "  Factors behind the change (they sum to -1.50):",
                "    -1.00  Similar code (findings 0 → 1; max duplicated lines 18; lines 17–34, 37–54)",
                "    -0.45  Duplicated logic (AI assessment 26 → 62)",
                "    -0.05  Other factors",
            ]
        );
    }

    #[test]
    fn the_questions_behind_shown_ai_factors_are_listed() {
        let text = render(&[declined()], Framing::PrePush, PromptMode::Recommend);
        assert!(text.contains(
            "AI assessment questions:\n  Duplicated logic: How much of the supplied code"
        ));
    }

    #[test]
    fn other_files_follow_the_declined_ones() {
        let text = render(
            &[comparison(7.5, scored(8.0)), declined()],
            Framing::PrePush,
            PromptMode::Recommend,
        );
        assert!(text.contains("Other changed files:\nOK lib/file.py  8.00 → 7.50/10 (-0.50)"));
    }

    #[test]
    fn a_renamed_file_names_its_old_path() {
        let mut renamed = declined();
        renamed.base = Some(Base::Scored {
            path: "lib/old.py".to_owned(),
            commit: "abc123".to_owned(),
            score: 8.0,
            passed: true,
        });
        let text = render(&[renamed], Framing::PrePush, PromptMode::Recommend);
        assert!(text.contains("\n  Renamed from lib/old.py.\n"));
    }

    #[test]
    fn a_new_file_explains_its_score_against_the_reference() {
        let mut new_file = comparison(4.0, Base::New);
        new_file.impacts = Some(FactorImpacts {
            basis: Basis::Reference(7.0),
            total: -3.0,
            factors: vec![smell_impact(-3.0)],
        });
        let text = render(&[new_file], Framing::PrePush, PromptMode::Recommend);
        assert!(text.contains("Its factors explain the score against the model's reference (7.00), not a decline.\n  Leading factors:\n    -3.00  Similar code"));
    }

    #[test]
    fn leading_factors_leave_small_impacts_to_the_rest() {
        let factors = [smell_impact(-1.0), jev_impact(-0.001)];
        let (shown, other) = leading(&factors, -1.001, 5);
        assert_eq!((shown.len(), (other * 1000.0).round()), (1, -1.0));
    }
}
