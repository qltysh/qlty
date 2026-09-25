use chrono::{DateTime, FixedOffset, NaiveDate};
use chrono_tz::Tz;
use qlty_slop_one_trends::periods::{label, select_snapshots, History, Interval, SelectedPeriod};

fn stamp(text: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(text).unwrap()
}

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
}

fn select(
    commits: &[(&str, &str)],
    since: Option<&str>,
    as_of: &str,
    interval: Interval,
) -> Vec<SelectedPeriod> {
    select_in(
        commits,
        since,
        as_of,
        chrono_tz::America::New_York,
        interval,
    )
}

fn select_in(
    commits: &[(&str, &str)],
    since: Option<&str>,
    as_of: &str,
    tz: Tz,
    interval: Interval,
) -> Vec<SelectedPeriod> {
    let history = History::from_commits(
        commits
            .iter()
            .map(|(sha, when)| ((*sha).to_owned(), stamp(when))),
    );
    select_snapshots(&history, since.map(date), stamp(as_of), tz, interval)
}

fn starts(rows: &[SelectedPeriod]) -> Vec<&str> {
    rows.iter().map(|row| row.bounds.start()).collect()
}

fn commits(rows: &[SelectedPeriod]) -> Vec<&str> {
    rows.iter().map(|row| row.commit.as_str()).collect()
}

const LEAP_YEAR: [(&str, &str); 6] = [
    ("origin", "2023-12-20T12:00:00-05:00"),
    ("january", "2024-01-31T23:59:59-05:00"),
    ("february-start", "2024-02-01T05:00:00+00:00"),
    ("february", "2024-02-29T23:59:59-05:00"),
    ("march", "2024-03-31T23:59:59-04:00"),
    ("april-start", "2024-04-01T04:00:00+00:00"),
];

#[test]
fn monthly_selection_includes_the_preceding_month_as_a_baseline() {
    let rows = select(
        &LEAP_YEAR,
        Some("2024-02-10"),
        "2024-04-01T04:00:00+00:00",
        Interval::Month,
    );
    assert_eq!(starts(&rows), ["2024-01-01", "2024-02-01", "2024-03-01"]);
}

#[test]
fn monthly_selection_takes_the_last_commit_before_each_month_end() {
    let rows = select(
        &LEAP_YEAR,
        Some("2024-02-10"),
        "2024-04-01T04:00:00+00:00",
        Interval::Month,
    );
    assert_eq!(commits(&rows), ["january", "february", "march"]);
}

#[test]
fn month_end_follows_daylight_saving_time() {
    let rows = select(
        &LEAP_YEAR,
        Some("2024-02-10"),
        "2024-04-01T04:00:00+00:00",
        Interval::Month,
    );
    assert_eq!(rows[1].bounds.end_exclusive(), "2024-03-01T00:00:00-05:00");
    assert_eq!(rows[2].bounds.end_exclusive(), "2024-04-01T00:00:00-04:00");
}

#[test]
fn a_month_that_ends_exactly_at_the_as_of_time_is_complete() {
    let rows = select(
        &LEAP_YEAR,
        Some("2024-02-10"),
        "2024-04-01T04:00:00+00:00",
        Interval::Month,
    );
    assert!(rows.iter().all(|row| !row.partial));
}

const YEAR_ROLLOVER: [(&str, &str); 3] = [
    ("origin", "2024-11-01T12:00:00-04:00"),
    ("december", "2024-12-28T12:00:00-05:00"),
    ("january", "2025-01-20T12:00:00-05:00"),
];

#[test]
fn monthly_selection_rolls_over_the_year() {
    let rows = select(
        &YEAR_ROLLOVER,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    assert_eq!(starts(&rows), ["2024-12-01", "2025-01-01", "2025-02-01"]);
}

#[test]
fn a_current_month_without_commits_repeats_the_previous_commit() {
    let rows = select(
        &YEAR_ROLLOVER,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    assert_eq!(commits(&rows), ["december", "january", "january"]);
}

#[test]
fn the_current_month_is_partial_and_ends_at_the_as_of_time() {
    let rows = select(
        &YEAR_ROLLOVER,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    let current = rows.last().unwrap();
    assert!(current.partial);
    assert_eq!(current.as_of, "2025-02-15T12:00:00-05:00");
    assert_eq!(current.bounds.end_exclusive(), "2025-03-01T00:00:00-05:00");
}

#[test]
fn complete_periods_record_their_end_as_the_as_of_time() {
    let rows = select(
        &YEAR_ROLLOVER,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    assert_eq!(rows[0].as_of, "2025-01-01T00:00:00-05:00");
}

const BACKDATED: [(&str, &str); 3] = [
    ("origin", "2025-01-10T12:00:00-05:00"),
    ("parent", "2025-02-02T12:00:00-05:00"),
    ("backdated-child", "2025-01-20T12:00:00-05:00"),
];

#[test]
fn the_baseline_does_not_invent_history() {
    let rows = select(
        &BACKDATED,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    assert_eq!(starts(&rows), ["2025-01-01", "2025-02-01"]);
}

#[test]
fn commit_dates_cannot_reverse_ancestry() {
    let rows = select(
        &BACKDATED,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    assert_eq!(commits(&rows), ["origin", "backdated-child"]);
}

#[test]
fn backdated_commits_keep_their_own_committed_at() {
    let rows = select(
        &BACKDATED,
        Some("2025-01-01"),
        "2025-02-15T12:00:00-05:00",
        Interval::Month,
    );
    assert_eq!(rows[1].committed_at, "2025-01-20T12:00:00-05:00");
    assert_eq!(rows[1].first_parent_index, 2);
}

const WEEKLY: [(&str, &str); 2] = [
    ("origin", "2025-06-28T12:00:00+00:00"),
    ("change", "2025-07-09T12:00:00+00:00"),
];

#[test]
fn weekly_selection_starts_on_the_monday_of_the_requested_week() {
    let rows = select(
        &WEEKLY,
        Some("2025-07-01"),
        "2025-07-15T12:00:00+00:00",
        Interval::Week,
    );
    assert_eq!(starts(&rows), ["2025-06-30", "2025-07-07", "2025-07-14"]);
}

#[test]
fn weekly_selection_repeats_the_last_commit_into_quiet_weeks() {
    let rows = select(
        &WEEKLY,
        Some("2025-07-01"),
        "2025-07-15T12:00:00+00:00",
        Interval::Week,
    );
    assert_eq!(commits(&rows), ["origin", "change", "change"]);
    assert!(rows.last().unwrap().partial);
}

#[test]
fn weekly_bounds_use_the_run_timezone() {
    let rows = select(
        &WEEKLY,
        Some("2025-07-01"),
        "2025-07-15T12:00:00+00:00",
        Interval::Week,
    );
    assert_eq!(rows[0].bounds.end_exclusive(), "2025-07-07T00:00:00-04:00");
}

#[test]
fn the_partial_period_keeps_the_as_of_offset() {
    let rows = select(
        &WEEKLY,
        Some("2025-07-01"),
        "2025-07-15T12:00:00.250000+00:00",
        Interval::Week,
    );
    assert_eq!(
        rows.last().unwrap().as_of,
        "2025-07-15T12:00:00.250000+00:00"
    );
}

#[test]
fn without_since_the_history_starts_at_the_first_commit() {
    let rows = select(&WEEKLY, None, "2025-07-15T12:00:00+00:00", Interval::Week);
    assert_eq!(
        starts(&rows),
        ["2025-06-23", "2025-06-30", "2025-07-07", "2025-07-14"]
    );
}

#[test]
fn a_sunday_night_commit_belongs_to_the_week_of_its_local_timezone() {
    let commits = [
        ("origin", "2025-06-20T12:00:00+00:00"),
        ("late-sunday", "2025-07-06T23:30:00-04:00"),
    ];
    let new_york = select_in(
        &commits,
        Some("2025-06-30"),
        "2025-07-10T12:00:00+00:00",
        chrono_tz::America::New_York,
        Interval::Week,
    );
    let utc = select_in(
        &commits,
        Some("2025-06-30"),
        "2025-07-10T12:00:00+00:00",
        chrono_tz::UTC,
        Interval::Week,
    );
    assert_eq!(new_york[0].commit, "late-sunday");
    assert_eq!(utc[0].commit, "origin");
}

#[test]
fn no_snapshots_before_the_first_commit() {
    let rows = select(
        &[("only", "2025-07-09T12:00:00+00:00")],
        Some("2025-06-01"),
        "2025-07-15T12:00:00+00:00",
        Interval::Week,
    );
    assert_eq!(starts(&rows), ["2025-07-07", "2025-07-14"]);
}

#[test]
fn an_as_of_before_the_first_period_selects_nothing() {
    let rows = select(
        &WEEKLY,
        Some("2025-07-01"),
        "2025-06-01T00:00:00+00:00",
        Interval::Week,
    );
    assert!(rows.is_empty());
}

#[test]
fn weekly_labels_name_the_sunday() {
    assert_eq!(
        label(date("2025-07-07"), Interval::Week),
        "Week ending July 13, 2025"
    );
}

#[test]
fn monthly_labels_name_the_month() {
    assert_eq!(label(date("2024-12-01"), Interval::Month), "December 2024");
}
