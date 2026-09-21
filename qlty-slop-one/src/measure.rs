//! In-process Qlty smell measurement for one file.
//!
//! This reproduces what slopdetect computed from `qlty smells --sarif`: the
//! structure and duplication executors run on the text with SlopOne's fixed
//! thresholds, and the issues are folded into the 32 features exactly as
//! slopdetect's `parse_sarif` and `parse_measurements` folded the SARIF.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

use qlty_analysis::code::File;
use qlty_config::config::smells::{
    BooleanLogic, FileComplexity, FunctionComplexity, FunctionParameters, IdenticalCode,
    NestedControlFlow, ReturnStatements, SimilarCode, Smells,
};
use qlty_config::config::Builder;
use qlty_config::QltyConfig;
use qlty_smells::{duplication, structure};
use qlty_types::analysis::v1::Issue;
use serde::Serialize;

use crate::error::{Error, Result};
use crate::features::{Rule, FIELDS, RULES};
use crate::language::Language;
use crate::pylines;

/// An inclusive 1-based line range in the analyzed (or, after restoration,
/// the original) file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct LineSpan {
    pub start_line: u32,
    pub end_line: u32,
}

/// The evidence behind one smell factor, in slopdetect's JSON shape.
#[derive(Clone, Debug, Serialize)]
pub struct RuleSummary {
    pub count: usize,
    pub max_magnitude: f64,
    pub covered_lines: usize,
    pub locations: Vec<LineSpan>,
    pub max_actual: f64,
    pub threshold: usize,
    pub magnitude_basis: &'static str,
    pub magnitude_unit: &'static str,
}

/// The 32 Qlty features and their evidence for one analyzed file.
#[derive(Clone, Debug)]
pub struct Measurement {
    /// Feature values in `features::RULES` × `features::FIELDS` order.
    pub values: Vec<f64>,
    pub summaries: BTreeMap<Rule, RuleSummary>,
    pub language: Language,
    /// Physical line count of the analyzed text, at least 1.
    pub source_lines: usize,
}

impl Measurement {
    /// Detects the language from the file suffix (or shebang) and measures.
    pub fn analyze(text: &str, suffix: &str) -> Result<Self> {
        let language = Language::detect(suffix, text)?;
        Self::measure(text, language)
    }

    /// Runs Qlty's structure and duplication smells on `text` as one file of
    /// `language` and folds the findings into the feature vector.
    pub fn measure(text: &str, language: Language) -> Result<Self> {
        let source_lines = pylines::line_count(text).max(1);
        let issues = smell_issues(text, language)?;
        let findings = Findings::from_issues(&issues, source_lines, language)?;

        let mut values = Vec::with_capacity(RULES.len() * FIELDS.len());
        let mut summaries = BTreeMap::new();

        for rule in RULES {
            let summary = findings.summary(rule, language);
            values.extend(summary.values(source_lines));
            summaries.insert(rule, summary);
        }

        Ok(Self {
            values,
            summaries,
            language,
            source_lines,
        })
    }
}

impl RuleSummary {
    /// The four feature encodings, in `features::FIELDS` order.
    fn values(&self, source_lines: usize) -> [f64; 4] {
        let count = self.count as f64;
        let source_lines = source_lines as f64;
        [
            count.ln_1p(),
            (100.0 * count) / source_lines,
            self.max_actual.ln_1p(),
            self.covered_lines as f64 / source_lines,
        ]
    }
}

/// SlopOne's smell configuration: Qlty's `default.toml` with the bundled
/// slopdetect `qlty.toml` layered on top.
fn config() -> &'static QltyConfig {
    static CONFIG: OnceLock<QltyConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let mut config =
            Builder::default_config().expect("the bundled default.toml should be valid");
        config.exclude_patterns = Vec::new();
        config.test_patterns = Vec::new();

        let smells = config.smells.get_or_insert_with(Smells::default);
        smells
            .duplication
            .get_or_insert_with(Default::default)
            .nodes_threshold = Some(64);
        smells.boolean_logic = Some(BooleanLogic {
            enabled: true,
            threshold: Some(4),
        });
        smells.nested_control_flow = Some(NestedControlFlow {
            enabled: true,
            threshold: Some(5),
        });
        smells.function_parameters = Some(FunctionParameters {
            enabled: true,
            threshold: Some(6),
        });
        smells.return_statements = Some(ReturnStatements {
            enabled: true,
            threshold: Some(6),
        });
        smells.file_complexity = Some(FileComplexity {
            enabled: true,
            threshold: Some(50),
        });
        smells.function_complexity = Some(FunctionComplexity {
            enabled: true,
            threshold: Some(18),
        });
        smells.identical_code = Some(IdenticalCode {
            enabled: true,
            threshold: Some(15),
        });
        smells.similar_code = Some(SimilarCode {
            enabled: true,
            threshold: Some(15),
        });

        let rust = config
            .language
            .get_mut(Language::Rust.qlty_name())
            .expect("the bundled default.toml should configure rust");
        rust.smells
            .get_or_insert_with(Smells::default)
            .boolean_logic = Some(BooleanLogic {
            enabled: true,
            threshold: Some(5),
        });

        config
    })
}

fn smell_issues(text: &str, language: Language) -> Result<Vec<Issue>> {
    let config = config();
    let file = Arc::new(File::from_string(language.qlty_name(), text));

    let structure_plan = structure::Planner::new(config, vec![file.clone()])
        .and_then(|planner| planner.compute())
        .map_err(Error::Qlty)?;
    let mut structure_executor = structure::Executor::new(&structure_plan);
    structure_executor.execute();
    let mut issues = structure_executor.issues;

    let settings = duplication::Settings {
        paths: Vec::new(),
        include_tests: true,
    };
    let duplication_plan = duplication::Planner::new(config, &settings, vec![file])
        .and_then(|planner| planner.compute())
        .map_err(Error::Qlty)?;
    let mut duplication_executor = duplication::Executor::new(&duplication_plan);
    duplication_executor.execute();
    issues.extend(duplication_executor.report().issues);

    Ok(issues)
}

/// One finding's exact SARIF region, used only for the dedup identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ExactLocation {
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Identity {
    rule: Rule,
    locations: Vec<ExactLocation>,
    magnitude_bits: u64,
}

/// One issue's line intervals (primary plus related locations), with
/// slopdetect's exclusive-endpoint adjustment applied.
#[derive(Clone, Debug)]
struct Finding {
    intervals: Vec<LineSpan>,
}

/// The deduplicated findings and the raw maxima for every rule.
#[derive(Debug, Default)]
struct Findings {
    by_rule: BTreeMap<Rule, Vec<Finding>>,
    max_actual: BTreeMap<Rule, f64>,
}

impl Findings {
    fn from_issues(issues: &[Issue], source_lines: usize, language: Language) -> Result<Self> {
        let mut findings = Self::default();
        let mut seen = BTreeSet::new();

        for issue in issues {
            let rule = Rule::from_key(&issue.rule_key).ok_or_else(|| {
                Error::Analysis(format!("Unknown Qlty smell rule: {}", issue.rule_key))
            })?;
            let (finding, identity) = Finding::from_issue(issue, rule, source_lines)?;
            if seen.insert(identity) {
                findings.by_rule.entry(rule).or_default().push(finding);
            }
        }

        for issue in issues {
            let rule = Rule::from_key(&issue.rule_key).ok_or_else(|| {
                Error::Analysis(format!("Unknown Qlty smell rule: {}", issue.rule_key))
            })?;
            let actual = validated_actual(issue, rule, language)?;
            let maximum = findings.max_actual.entry(rule).or_insert(0.0);
            *maximum = maximum.max(actual);
        }

        Ok(findings)
    }

    fn summary(&self, rule: Rule, language: Language) -> RuleSummary {
        let findings = self.by_rule.get(&rule).map(Vec::as_slice).unwrap_or(&[]);
        let intervals: Vec<LineSpan> = findings
            .iter()
            .flat_map(|finding| finding.intervals.iter().copied())
            .collect();
        let max_actual = self.max_actual.get(&rule).copied().unwrap_or(0.0);

        RuleSummary {
            count: findings.len(),
            max_magnitude: max_actual,
            covered_lines: covered_lines(&intervals),
            locations: intervals
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            max_actual,
            threshold: rule.threshold(language),
            magnitude_basis: "actual",
            magnitude_unit: rule.magnitude_unit(),
        }
    }
}

impl Finding {
    fn from_issue(issue: &Issue, rule: Rule, source_lines: usize) -> Result<(Self, Identity)> {
        let locations = issue.location.iter().chain(&issue.other_locations);
        let mut intervals = Vec::new();
        let mut exact_locations = Vec::new();

        for location in locations {
            let range = location
                .range
                .as_ref()
                .ok_or_else(|| Error::Analysis("Finding lacks source locations".to_string()))?;
            let start = range.start_line;
            let mut end = range.end_line;
            // Qlty uses inclusive line endpoints for duplication (column 0),
            // and exclusive character endpoints for structural findings.
            if end > start && range.end_column == 1 {
                end -= 1;
            }
            if !(1 <= start && start <= end && end as usize <= source_lines) {
                return Err(Error::Analysis(format!(
                    "Invalid source span: {start}:{end} of {source_lines}"
                )));
            }
            intervals.push(LineSpan {
                start_line: start,
                end_line: end,
            });
            exact_locations.push(ExactLocation {
                start_line: start,
                start_column: range.start_column,
                end_line: end,
                end_column: range.end_column,
            });
        }

        if intervals.is_empty() {
            return Err(Error::Analysis(
                "Finding lacks source locations".to_string(),
            ));
        }

        let magnitude = legacy_magnitude(issue, rule, &intervals)?;
        if !magnitude.is_finite() || magnitude < 0.0 {
            return Err(Error::Analysis("Invalid finding magnitude".to_string()));
        }

        exact_locations.sort();
        let identity = Identity {
            rule,
            locations: exact_locations,
            magnitude_bits: magnitude.to_bits(),
        };

        Ok((Self { intervals }, identity))
    }
}

/// The magnitude slopdetect's legacy parser derived per finding. It only
/// takes part in the dedup identity; features use `properties.actual`.
fn legacy_magnitude(issue: &Issue, rule: Rule, intervals: &[LineSpan]) -> Result<f64> {
    if rule.is_duplication() {
        return property_number(issue, "mass")
            .ok_or_else(|| Error::Analysis("Missing duplication mass".to_string()));
    }

    if rule == Rule::BooleanLogic {
        let longest = intervals
            .iter()
            .map(|span| span.end_line - span.start_line + 1)
            .max()
            .unwrap_or(0);
        return Ok(f64::from(longest));
    }

    message_magnitude(&issue.message)
        .ok_or_else(|| Error::Analysis(format!("Missing structural magnitude: {}", rule.key())))
}

/// The number in `(count = N)` or `(level = N)` in a structural message.
fn message_magnitude(message: &str) -> Option<f64> {
    ["(count = ", "(level = "].iter().find_map(|prefix| {
        let start = message.find(prefix)? + prefix.len();
        let rest = &message[start..];
        let end = rest.find(')')?;
        let digits = &rest[..end];
        let well_formed = !digits.is_empty()
            && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
            && !digits.starts_with('.')
            && !digits.ends_with('.')
            && digits.matches('.').count() <= 1;
        if well_formed {
            digits.parse().ok()
        } else {
            None
        }
    })
}

/// `properties.actual`, checked the way slopdetect's `parse_measurements`
/// checked it: present, finite, non-negative, at or above a threshold that
/// equals the configured one for this rule and language.
fn validated_actual(issue: &Issue, rule: Rule, language: Language) -> Result<f64> {
    let invalid = || {
        Error::Analysis(format!(
            "Missing, invalid, or incompatible Qlty measurements: {}",
            rule.key()
        ))
    };
    let actual = property_number(issue, "actual").ok_or_else(invalid)?;
    let threshold = property_number(issue, "threshold").ok_or_else(invalid)?;
    let expected = rule.threshold(language) as f64;

    if !actual.is_finite()
        || !threshold.is_finite()
        || actual < 0.0
        || actual < threshold
        || threshold != expected
    {
        return Err(invalid());
    }

    Ok(actual)
}

fn property_number(issue: &Issue, key: &str) -> Option<f64> {
    let properties = serde_json::to_value(issue.properties.as_ref()?).ok()?;
    properties.get(key)?.as_f64()
}

/// The number of distinct lines covered by the union of inclusive intervals.
fn covered_lines(intervals: &[LineSpan]) -> usize {
    let mut sorted = intervals.to_vec();
    sorted.sort();

    let mut count = 0usize;
    let mut stop = 0usize;
    for span in sorted {
        let start = span.start_line as usize;
        let end = span.end_line as usize;
        count += (end + 1).saturating_sub(start.max(stop + 1));
        stop = stop.max(end);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use qlty_types::analysis::v1::{Location, Range};

    fn issue(rule: &str, message: &str, ranges: &[(u32, u32, u32, u32)]) -> Issue {
        let mut locations = ranges.iter().map(|&(sl, sc, el, ec)| Location {
            path: "STRING".to_string(),
            range: Some(Range {
                start_line: sl,
                start_column: sc,
                end_line: el,
                end_column: ec,
                ..Default::default()
            }),
        });
        Issue {
            rule_key: rule.to_string(),
            message: message.to_string(),
            location: locations.next(),
            other_locations: locations.collect(),
            ..Default::default()
        }
    }

    fn measured(rule: &str, message: &str, ranges: &[(u32, u32, u32, u32)]) -> Issue {
        let mut issue = issue(rule, message, ranges);
        issue.set_property_number("threshold", 5.0);
        issue.set_property_number("actual", 6.0);
        issue
    }

    #[test]
    fn covered_lines_unions_overlapping_intervals() {
        let spans = [
            LineSpan {
                start_line: 5,
                end_line: 9,
            },
            LineSpan {
                start_line: 2,
                end_line: 6,
            },
            LineSpan {
                start_line: 20,
                end_line: 20,
            },
        ];
        assert_eq!(covered_lines(&spans), 9);
    }

    #[test]
    fn covered_lines_ignores_contained_intervals() {
        let spans = [
            LineSpan {
                start_line: 1,
                end_line: 10,
            },
            LineSpan {
                start_line: 3,
                end_line: 4,
            },
        ];
        assert_eq!(covered_lines(&spans), 10);
    }

    #[test]
    fn exclusive_endpoint_at_column_one_drops_the_last_line() {
        let issue = measured(
            "nested-control-flow",
            "Deeply nested control flow (level = 6)",
            &[(2, 3, 5, 1)],
        );
        let (finding, _) = Finding::from_issue(&issue, Rule::NestedControlFlow, 10).unwrap();
        assert_eq!(
            finding.intervals,
            [LineSpan {
                start_line: 2,
                end_line: 4
            }]
        );
    }

    #[test]
    fn single_line_finding_ending_at_column_one_is_kept() {
        let issue = measured(
            "nested-control-flow",
            "Deeply nested control flow (level = 6)",
            &[(2, 1, 2, 1)],
        );
        let (finding, _) = Finding::from_issue(&issue, Rule::NestedControlFlow, 10).unwrap();
        assert_eq!(
            finding.intervals,
            [LineSpan {
                start_line: 2,
                end_line: 2
            }]
        );
    }

    #[test]
    fn duplication_column_zero_endpoints_are_inclusive() {
        let mut issue = issue(
            "identical-code",
            "Found 20 lines",
            &[(5, 0, 24, 0), (30, 0, 49, 0)],
        );
        issue.set_property_number("mass", 120.0);
        let (finding, _) = Finding::from_issue(&issue, Rule::IdenticalCode, 60).unwrap();
        assert_eq!(finding.intervals[0].end_line, 24);
        assert_eq!(finding.intervals[1].start_line, 30);
    }

    #[test]
    fn span_outside_the_source_is_an_error() {
        let issue = measured(
            "nested-control-flow",
            "Deeply nested control flow (level = 6)",
            &[(2, 3, 12, 4)],
        );
        assert!(Finding::from_issue(&issue, Rule::NestedControlFlow, 10).is_err());
    }

    #[test]
    fn repeated_structural_findings_collapse() {
        let issue = measured(
            "nested-control-flow",
            "Deeply nested control flow (level = 6)",
            &[(2, 3, 5, 1)],
        );
        let findings =
            Findings::from_issues(&[issue.clone(), issue], 10, Language::Python).unwrap();
        assert_eq!(findings.by_rule[&Rule::NestedControlFlow].len(), 1);
    }

    #[test]
    fn duplication_group_members_collapse_into_one_finding() {
        let mut first = issue("identical-code", "Found", &[(5, 0, 24, 0), (30, 0, 49, 0)]);
        let mut second = issue("identical-code", "Found", &[(30, 0, 49, 0), (5, 0, 24, 0)]);
        for issue in [&mut first, &mut second] {
            issue.set_property_number("mass", 120.0);
            issue.set_property_number("threshold", 15.0);
            issue.set_property_number("actual", 20.0);
        }
        let findings = Findings::from_issues(&[first, second], 60, Language::Java).unwrap();
        let summary = findings.summary(Rule::IdenticalCode, Language::Java);
        assert_eq!(summary.count, 1);
        assert_eq!(summary.covered_lines, 40);
        assert_eq!(
            summary.locations,
            [
                LineSpan {
                    start_line: 5,
                    end_line: 24
                },
                LineSpan {
                    start_line: 30,
                    end_line: 49
                }
            ]
        );
    }

    #[test]
    fn different_mass_keeps_both_findings() {
        let mut first = issue("similar-code", "Found", &[(5, 0, 24, 0), (30, 0, 49, 0)]);
        let mut second = first.clone();
        for (issue, mass) in [(&mut first, 120.0), (&mut second, 121.0)] {
            issue.set_property_number("mass", mass);
            issue.set_property_number("threshold", 15.0);
            issue.set_property_number("actual", 20.0);
        }
        let findings = Findings::from_issues(&[first, second], 60, Language::Java).unwrap();
        assert_eq!(findings.by_rule[&Rule::SimilarCode].len(), 2);
    }

    #[test]
    fn maximum_uses_actual_over_all_raw_issues() {
        let mut first = measured(
            "boolean-logic",
            "Complex binary expression",
            &[(1, 1, 2, 3)],
        );
        first.set_property_number("threshold", 4.0);
        first.set_property_number("actual", 8.0);
        let mut second = first.clone();
        second.set_property_number("actual", 9.0);
        let findings = Findings::from_issues(&[first, second], 50, Language::Java).unwrap();
        let summary = findings.summary(Rule::BooleanLogic, Language::Java);
        assert_eq!(summary.count, 1);
        assert_eq!(summary.max_actual, 9.0);
        assert_eq!(summary.max_magnitude, 9.0);
    }

    #[test]
    fn zero_findings_give_zero_values() {
        let findings = Findings::from_issues(&[], 50, Language::Rust).unwrap();
        let summary = findings.summary(Rule::FileComplexity, Language::Rust);
        assert_eq!(summary.values(50), [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(summary.count, 0);
        assert_eq!(summary.threshold, 50);
        assert!(summary.locations.is_empty());
    }

    #[test]
    fn values_follow_the_python_formulas() {
        let summary = RuleSummary {
            count: 1,
            max_magnitude: 6.0,
            covered_lines: 3,
            locations: Vec::new(),
            max_actual: 6.0,
            threshold: 5,
            magnitude_basis: "actual",
            magnitude_unit: "Nesting depth",
        };
        let values = summary.values(10);
        assert_eq!(values[0], 2f64.ln());
        assert_eq!(values[1], 10.0);
        assert_eq!(values[2], 7f64.ln());
        assert_eq!(values[3], 0.3);
    }

    #[test]
    fn threshold_mismatch_is_an_error() {
        let mut issue = measured(
            "boolean-logic",
            "Complex binary expression",
            &[(1, 1, 1, 9)],
        );
        issue.set_property_number("threshold", 5.0);
        issue.set_property_number("actual", 8.0);
        let error = Findings::from_issues(&[issue], 50, Language::Java).unwrap_err();
        assert!(error.to_string().contains("incompatible"));
    }

    #[test]
    fn rust_boolean_threshold_five_is_accepted() {
        let mut issue = measured(
            "boolean-logic",
            "Complex binary expression",
            &[(1, 1, 1, 9)],
        );
        issue.set_property_number("threshold", 5.0);
        issue.set_property_number("actual", 8.0);
        assert!(Findings::from_issues(&[issue], 50, Language::Rust).is_ok());
    }

    #[test]
    fn missing_actual_is_an_error() {
        let issue = issue(
            "boolean-logic",
            "Complex binary expression",
            &[(1, 1, 1, 9)],
        );
        assert!(Findings::from_issues(&[issue], 50, Language::Java).is_err());
    }

    #[test]
    fn actual_below_threshold_is_an_error() {
        let mut issue = measured(
            "function-parameters",
            "Function with many parameters (count = 3): f",
            &[(1, 1, 1, 9)],
        );
        issue.set_property_number("threshold", 6.0);
        issue.set_property_number("actual", 3.0);
        assert!(Findings::from_issues(&[issue], 50, Language::Java).is_err());
    }

    #[test]
    fn boolean_actual_is_a_number_not_a_bool() {
        let mut issue = measured(
            "boolean-logic",
            "Complex binary expression",
            &[(1, 1, 1, 9)],
        );
        issue.set_property_number("threshold", 4.0);
        issue.set_property_bool("actual", true);
        assert!(Findings::from_issues(&[issue], 50, Language::Java).is_err());
    }

    #[test]
    fn structural_magnitude_comes_from_the_message() {
        assert_eq!(
            message_magnitude("Function with many returns (count = 35): f"),
            Some(35.0)
        );
        assert_eq!(
            message_magnitude("Deeply nested control flow (level = 5)"),
            Some(5.0)
        );
        assert_eq!(message_magnitude("Complex binary expression"), None);
        assert_eq!(message_magnitude("(count = )"), None);
    }

    #[test]
    fn missing_structural_magnitude_is_an_error() {
        let issue = measured(
            "return-statements",
            "Function with many returns",
            &[(1, 1, 3, 1)],
        );
        assert!(Finding::from_issue(&issue, Rule::ReturnStatements, 10).is_err());
    }

    #[test]
    fn unknown_rule_is_an_error() {
        let issue = measured("mystery", "?", &[(1, 1, 1, 2)]);
        assert!(Findings::from_issues(&[issue], 10, Language::Java).is_err());
    }

    #[test]
    fn measures_a_boolean_chain_in_java() {
        let text = "class Logic { boolean logic() { return a || b || c || d || e; } }\n";
        let measurement = Measurement::measure(text, Language::Java).unwrap();
        assert_eq!(measurement.summaries[&Rule::BooleanLogic].count, 1);
        assert_eq!(measurement.summaries[&Rule::BooleanLogic].max_actual, 4.0);
        assert_eq!(measurement.values.len(), 32);
        assert_eq!(measurement.source_lines, 1);
    }

    #[test]
    fn rust_needs_five_operators() {
        let four = "fn logic() -> bool { a || b || c || d || e }\n";
        let five = "fn logic() -> bool { a || b || c || d || e || f }\n";
        assert_eq!(
            Measurement::measure(four, Language::Rust)
                .unwrap()
                .summaries[&Rule::BooleanLogic]
                .count,
            0
        );
        assert_eq!(
            Measurement::measure(five, Language::Rust)
                .unwrap()
                .summaries[&Rule::BooleanLogic]
                .count,
            1
        );
    }

    #[test]
    fn analyze_detects_the_language() {
        let measurement =
            Measurement::analyze("def double(value):\n    return value * 2\n", ".py").unwrap();
        assert_eq!(measurement.language, Language::Python);
        assert!(measurement.values.iter().all(|value| *value == 0.0));
    }
}
