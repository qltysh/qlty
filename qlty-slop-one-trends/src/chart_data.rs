//! Chart values and drilldown rows derived from the exports.
//!
//! An added file contributes `mass / previous project mass × (score − pass
//! cutoff)`; a removed file contributes the opposite. Matched files,
//! renames, splits, and merges keep the original score-change calculation.
//! This is a change measure, not a difference in project average or
//! maintainable percentage. A port of slopdetect's `weekly_chart_data.py`.

use std::collections::HashMap;

use chrono::{Days, NaiveDate};
use qlty_slop_one::fsum::fsum;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::periods::Interval;
use crate::report::{
    Change, ChangeKind, ChartPeriod, DrilldownPeriod, DrilldownRow, FileChanges, FileRow,
    FilesExport, Member, Period, ReportData, Totals,
};

/// The chart label for a period starting on `start` (an ISO date).
pub fn label(start: &str, interval: Interval) -> Result<String> {
    let date = NaiveDate::parse_from_str(start, "%Y-%m-%d")
        .map_err(|_| Error::Mismatch(format!("period start {start} is not an ISO date")))?;
    match interval {
        Interval::Month => Ok(date.format("%B %Y").to_string()),
        Interval::Week => {
            let end = date
                .checked_add_days(Days::new(6))
                .ok_or_else(|| Error::Mismatch(format!("period start {start} is out of range")))?;
            Ok(end.format("Week ending %B %d, %Y").to_string())
        }
    }
}

/// One added (`direction` 1) or removed (`direction` -1) file's
/// contribution against the pass cutoff.
pub fn file_contribution(row: &FileRow, denominator: f64, cutoff: f64, direction: f64) -> f64 {
    let mass = row.mass.unwrap_or(0.0);
    let score = row.score.unwrap_or(0.0);
    direction * mass / denominator * (score - cutoff)
}

/// The folded-in totals of a set of added or removed files.
pub fn file_changes(
    rows: &[&FileRow],
    denominator: f64,
    cutoff: f64,
    direction: f64,
) -> FileChanges {
    let scored: Vec<&FileRow> = rows
        .iter()
        .copied()
        .filter(|row| row.score.is_some())
        .collect();
    let available = denominator > 0.0;
    let values: Vec<f64> = if available {
        scored
            .iter()
            .map(|row| file_contribution(row, denominator, cutoff, direction))
            .collect()
    } else {
        Vec::new()
    };
    let positive = fsum(values.iter().copied().filter(|value| *value > 0.0));
    let negative = fsum(values.iter().copied().filter(|value| *value < 0.0));
    FileChanges {
        positive: available.then_some(positive),
        negative: available.then_some(negative),
        net: available.then_some(positive + negative),
        files: scored.len(),
        unscored_files: rows.len() - scored.len(),
    }
}

/// One chart bar per period, with file entries and exits folded in.
pub fn chart_periods(data: &ReportData, files: &FilesExport) -> Result<Vec<ChartPeriod>> {
    let model = data.model();
    if model.get("sha256") != files.model.get("sha256") {
        return Err(Error::Mismatch(
            "Chart and file snapshots use different models".to_owned(),
        ));
    }
    for key in ["score_scale", "threshold_score"] {
        if model.get(key) != files.model.get(key) {
            return Err(Error::Mismatch(
                "Chart and file snapshots use different score scales".to_owned(),
            ));
        }
    }
    if data.interval != files.interval {
        return Err(Error::Mismatch(
            "Chart and file snapshots have different intervals".to_owned(),
        ));
    }
    if data.periods.len() != files.periods.len() {
        return Err(Error::Mismatch(
            "Chart and file snapshots have different period counts".to_owned(),
        ));
    }
    let cutoff = threshold_score(model)?;
    let mut result = Vec::with_capacity(data.periods.len());
    let mut previous: HashMap<&str, &FileRow> = HashMap::new();
    for (period, snapshot) in data.periods.iter().zip(&files.periods) {
        if (period.start(), &period.commit) != (snapshot.start(), &snapshot.commit) {
            return Err(Error::Mismatch(
                "Chart and file snapshots have different commits".to_owned(),
            ));
        }
        let current = rows_by_path(snapshot);
        let mut record = ChartPeriod {
            label: label(period.start(), data.interval)?,
            positive: None,
            negative: None,
            net: None,
            changed_groups: None,
            additions: None,
            removals: None,
            with_file_changes: Totals {
                positive: None,
                negative: None,
                net: None,
            },
        };
        if let Some(change) = &period.change {
            record.positive = change.positive;
            record.negative = change.negative;
            record.net = change.net;
            record.changed_groups = Some(change.changed_groups);
            let denominator = change.previous_project_mass;
            // Use the existing matching results; matched moves must not also
            // count as an addition or removal.
            let added = lookup(&current, &change.additions.files)?;
            let removed = lookup(&previous, &change.removals.files)?;
            let additions = file_changes(&added, denominator, cutoff, 1.0);
            let removals = file_changes(&removed, denominator, cutoff, -1.0);
            if denominator > 0.0 {
                let positive = fsum(
                    [change.positive, additions.positive, removals.positive]
                        .map(|value| value.unwrap_or(0.0)),
                );
                let negative = fsum(
                    [change.negative, additions.negative, removals.negative]
                        .map(|value| value.unwrap_or(0.0)),
                );
                record.with_file_changes = Totals {
                    positive: Some(positive),
                    negative: Some(negative),
                    net: Some(positive + negative),
                };
            }
            record.additions = Some(additions);
            record.removals = Some(removals);
        }
        result.push(record);
        previous = current;
    }
    Ok(result)
}

/// The drilldown rows of one period: matched groups keep their contribution;
/// each scored added or removed file gets its own row.
pub fn drilldown_rows(
    change: Option<&Change>,
    before: &HashMap<&str, &FileRow>,
    after: &HashMap<&str, &FileRow>,
    cutoff: f64,
) -> Result<Vec<DrilldownRow>> {
    let Some(change) = change else {
        return Ok(Vec::new());
    };
    if change.previous_project_mass <= 0.0 {
        return Ok(Vec::new());
    }
    let mut rows = Vec::new();
    for contribution in &change.contributions {
        let old_paths = &contribution.before_paths;
        let new_paths = &contribution.after_paths;
        let kind = if old_paths.len() > 1 && new_paths.len() > 1 {
            ChangeKind::Reorganized
        } else if new_paths.len() > 1 {
            ChangeKind::Split
        } else if old_paths.len() > 1 {
            ChangeKind::Merged
        } else if old_paths != new_paths {
            ChangeKind::Renamed
        } else {
            ChangeKind::Changed
        };
        rows.push(DrilldownRow {
            kind,
            before: members(old_paths, before)?,
            after: members(new_paths, after)?,
            before_score: Some(contribution.before_score),
            after_score: Some(contribution.after_score),
            before_mass: Some(contribution.before_mass),
            after_mass: Some(contribution.after_mass),
            contribution: contribution.contribution,
            diff_anchor: None,
            explanation: None,
        });
    }
    let file_sets = [
        (ChangeKind::Added, &change.additions.files, after, 1.0),
        (ChangeKind::Removed, &change.removals.files, before, -1.0),
    ];
    for (kind, paths, files, direction) in file_sets {
        for path in paths {
            let row = find(files, path)?;
            if row.score.is_none() {
                continue;
            }
            let member = members(std::slice::from_ref(path), files)?;
            let (before_row, after_row) = match kind {
                ChangeKind::Added => (None, Some(row)),
                _ => (Some(row), None),
            };
            rows.push(DrilldownRow {
                kind,
                before: before_row.map(|_| member.clone()).unwrap_or_default(),
                after: after_row.map(|_| member.clone()).unwrap_or_default(),
                before_score: before_row.and_then(|row| row.score),
                after_score: after_row.and_then(|row| row.score),
                before_mass: before_row.and_then(|row| row.mass),
                after_mass: after_row.and_then(|row| row.mass),
                contribution: file_contribution(
                    row,
                    change.previous_project_mass,
                    cutoff,
                    direction,
                ),
                diff_anchor: None,
                explanation: None,
            });
        }
    }
    rows.sort_by(|a, b| {
        b.contribution
            .abs()
            .total_cmp(&a.contribution.abs())
            .then_with(|| a.primary_path().cmp(b.primary_path()))
    });
    Ok(rows)
}

/// The drilldown rows for every period.
pub fn drilldown_periods(data: &ReportData, files: &FilesExport) -> Result<Vec<DrilldownPeriod>> {
    if data.interval != files.interval {
        return Err(Error::Mismatch(
            "Report and file snapshots have different intervals".to_owned(),
        ));
    }
    if data.periods.len() != files.periods.len() {
        return Err(Error::Mismatch(
            "Report and file snapshots have different period counts".to_owned(),
        ));
    }
    let cutoff = threshold_score(data.model())?;
    let mut result = Vec::with_capacity(data.periods.len());
    let mut previous: HashMap<&str, &FileRow> = HashMap::new();
    let mut previous_commit: Option<String> = None;
    for (period, snapshot) in data.periods.iter().zip(&files.periods) {
        if (period.start(), &period.commit) != (snapshot.start(), &snapshot.commit) {
            return Err(Error::Mismatch(
                "Report and file snapshots have different commits".to_owned(),
            ));
        }
        let current = rows_by_path(snapshot);
        result.push(DrilldownPeriod {
            before_commit: previous_commit,
            commit: period.commit.clone(),
            rows: drilldown_rows(period.change.as_ref(), &previous, &current, cutoff)?,
            period_start: None,
        });
        previous = current;
        previous_commit = Some(period.commit.clone());
    }
    Ok(result)
}

fn threshold_score(model: &Value) -> Result<f64> {
    model
        .get("threshold_score")
        .and_then(Value::as_f64)
        .ok_or_else(|| Error::Mismatch("the model records no threshold score".to_owned()))
}

fn rows_by_path(period: &Period) -> HashMap<&str, &FileRow> {
    period
        .files
        .iter()
        .flatten()
        .map(|row| (row.path.as_str(), row))
        .collect()
}

fn find<'a>(files: &HashMap<&str, &'a FileRow>, path: &str) -> Result<&'a FileRow> {
    files
        .get(path)
        .copied()
        .ok_or_else(|| Error::Mismatch(format!("file {path} is missing from its snapshot")))
}

fn lookup<'a>(files: &HashMap<&str, &'a FileRow>, paths: &[String]) -> Result<Vec<&'a FileRow>> {
    paths.iter().map(|path| find(files, path)).collect()
}

fn members(paths: &[String], files: &HashMap<&str, &FileRow>) -> Result<Vec<Member>> {
    Ok(lookup(files, paths)?
        .into_iter()
        .map(|row| Member {
            path: row.path.clone(),
            score: row.score,
            mass: row.mass,
            passed: None,
            language: None,
            code_lines: None,
            cyclomatic: None,
            source_scope: None,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(path: &str, score: Option<f64>, mass: f64) -> FileRow {
        FileRow {
            path: path.to_owned(),
            blob: String::new(),
            suffix: ".rs".to_owned(),
            version: String::new(),
            source_sha256: String::new(),
            source_scope: None,
            score,
            score_scale: None,
            passed: None,
            language: None,
            code_lines: None,
            cyclomatic: None,
            mass: Some(mass),
            error: None,
            measurement_key: None,
            skipped: None,
        }
    }

    fn approx(a: Option<f64>, b: f64) -> bool {
        a.is_some_and(|a| (a - b).abs() < 1e-12)
    }

    #[test]
    fn week_labels_name_the_sunday() {
        assert_eq!(
            label("2025-06-30", Interval::Week).unwrap(),
            "Week ending July 06, 2025"
        );
    }

    #[test]
    fn month_labels_name_the_month() {
        assert_eq!(label("2025-07-01", Interval::Month).unwrap(), "July 2025");
    }

    #[test]
    fn additions_and_removals_reverse_quality_effects() {
        let rows = [row("a", Some(8.0), 10.0), row("b", Some(3.0), 20.0)];
        let refs: Vec<&FileRow> = rows.iter().collect();
        let added = file_changes(&refs, 100.0, 5.0, 1.0);
        let removed = file_changes(&refs, 100.0, 5.0, -1.0);
        assert!(approx(added.positive, 0.3));
        assert!(approx(added.negative, -0.4));
        assert!(approx(added.net, -0.1));
        assert!(approx(removed.positive, 0.4));
        assert!(approx(removed.negative, -0.3));
        assert!(approx(removed.net, 0.1));
    }

    #[test]
    fn unscored_files_are_not_treated_as_zero_quality() {
        let rows = [
            row("a", None, 1000.0),
            row("b", Some(5.0), 10.0),
            row("c", Some(1.0), 0.0),
        ];
        let refs: Vec<&FileRow> = rows.iter().collect();
        let result = file_changes(&refs, 100.0, 5.0, 1.0);
        assert_eq!(result.positive, Some(0.0));
        assert_eq!(result.negative, Some(0.0));
        assert_eq!(result.net, Some(0.0));
        assert_eq!(result.files, 2);
        assert_eq!(result.unscored_files, 1);
    }

    #[test]
    fn zero_previous_mass_is_unavailable() {
        let rows = [row("a", Some(8.0), 10.0)];
        let refs: Vec<&FileRow> = rows.iter().collect();
        let result = file_changes(&refs, 0.0, 5.0, 1.0);
        assert_eq!(result.net, None);
        assert_eq!(result.positive, None);
        assert_eq!(result.negative, None);
    }

    #[test]
    fn no_comparison_when_previous_mass_is_missing() {
        let empty = HashMap::new();
        assert!(drilldown_rows(None, &empty, &empty, 5.0)
            .unwrap()
            .is_empty());
    }
}
