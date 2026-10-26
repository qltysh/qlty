//! Calendar boundaries and snapshot selection, ported from slopdetect's
//! `history_periods.py`.
//!
//! Periods start on Monday 00:00 or on the first of the month in the run
//! timezone. Each period takes the last first-parent commit before its end,
//! measured by cumulative committer time so backdated commits cannot reverse
//! ancestry. The current period ends at the frozen as-of time.

use chrono::{
    DateTime, Datelike, Days, FixedOffset, LocalResult, NaiveDate, Offset as _, TimeZone, Utc,
};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::run::PeriodBounds;

/// The calendar interval between snapshots.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Interval {
    #[default]
    Week,
    Month,
}

impl Interval {
    /// The key the exports use for their snapshot array.
    pub fn records_key(self) -> &'static str {
        match self {
            Self::Week => "weeks",
            Self::Month => "periods",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Month => "month",
        }
    }

    /// The start of the period that contains `date`.
    pub fn period_start(self, date: NaiveDate) -> NaiveDate {
        match self {
            Self::Week => date - Days::new(u64::from(date.weekday().num_days_from_monday())),
            Self::Month => date.with_day(1).unwrap_or(date),
        }
    }

    /// The start of the period after the one starting at `start`.
    pub fn next_start(self, start: NaiveDate) -> NaiveDate {
        match self {
            Self::Week => start + Days::new(7),
            Self::Month => month_after(start),
        }
    }

    fn bounds(self, start: NaiveDate, end: &DateTime<Tz>) -> PeriodBounds {
        let end = isoformat(end);
        match self {
            Self::Week => PeriodBounds::Week {
                week_start: start.to_string(),
                week_end_exclusive: end,
            },
            Self::Month => PeriodBounds::Month {
                period_start: start.to_string(),
                period_end_exclusive: end,
            },
        }
    }
}

/// One first-parent commit with its committer time and the cumulative
/// maximum committer time along its ancestry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitStamp {
    sha: String,
    committed_at: DateTime<FixedOffset>,
    cumulative_at: DateTime<Utc>,
}

impl CommitStamp {
    pub fn sha(&self) -> &str {
        &self.sha
    }

    pub fn committed_at(&self) -> DateTime<FixedOffset> {
        self.committed_at
    }

    /// The committer time, or the latest earlier committer time when the
    /// ancestry is not monotonic.
    pub fn cumulative_at(&self) -> DateTime<Utc> {
        self.cumulative_at
    }
}

/// First-parent history, oldest first, with monotonic ancestry times.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct History {
    stamps: Vec<CommitStamp>,
    inversions: usize,
}

impl History {
    /// Builds the history from commits in ancestry order, oldest first.
    pub fn from_commits<I>(commits: I) -> Self
    where
        I: IntoIterator<Item = (String, DateTime<FixedOffset>)>,
    {
        let mut history = Self::default();
        for (sha, committed_at) in commits {
            history.push(sha, committed_at);
        }
        history
    }

    pub fn push(&mut self, sha: String, committed_at: DateTime<FixedOffset>) {
        let committed = committed_at.with_timezone(&Utc);
        let latest = self.stamps.last().map(CommitStamp::cumulative_at);
        if latest.is_some_and(|latest| committed < latest) {
            self.inversions += 1;
        }
        let cumulative_at = latest.map_or(committed, |latest| latest.max(committed));
        self.stamps.push(CommitStamp {
            sha,
            committed_at,
            cumulative_at,
        });
    }

    pub fn stamps(&self) -> &[CommitStamp] {
        &self.stamps
    }

    pub fn first(&self) -> Option<&CommitStamp> {
        self.stamps.first()
    }

    pub fn len(&self) -> usize {
        self.stamps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stamps.is_empty()
    }

    /// How many commits carry a committer time earlier than an ancestor's.
    pub fn inversions(&self) -> usize {
        self.inversions
    }
}

/// A period's selected commit before any files are listed. The strings are
/// Python `isoformat()` renderings, as the run schema records them.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedPeriod {
    pub bounds: PeriodBounds,
    pub as_of: String,
    pub partial: bool,
    pub commit: String,
    pub committed_at: String,
    pub first_parent_index: usize,
}

/// Port of `select_snapshots`: strict end-of-period commits from
/// ancestry-ordered history.
///
/// Monthly requests include the preceding month as a baseline when history
/// exists. Weekly requests start at the Monday of the requested week.
pub fn select_snapshots(
    history: &History,
    since: Option<NaiveDate>,
    as_of: DateTime<FixedOffset>,
    tz: Tz,
    interval: Interval,
) -> Vec<SelectedPeriod> {
    let Some(first) = history.first() else {
        return Vec::new();
    };
    let created = first.committed_at().with_timezone(&tz).date_naive();
    let requested = since.unwrap_or(created);
    let requested_start = match interval {
        Interval::Month => interval.period_start(interval.period_start(requested) - Days::new(1)),
        Interval::Week => interval.period_start(requested),
    };
    let mut start = interval.period_start(created).max(requested_start);
    let as_of_utc = as_of.with_timezone(&Utc);
    let timestamps: Vec<DateTime<Utc>> = history
        .stamps()
        .iter()
        .map(CommitStamp::cumulative_at)
        .collect();
    let mut snapshots = Vec::new();
    while local_midnight(start, tz).with_timezone(&Utc) < as_of_utc {
        let next_start = interval.next_start(start);
        let end = local_midnight(next_start, tz);
        let end_utc = end.with_timezone(&Utc);
        let partial = end_utc > as_of_utc;
        let cutoff = end_utc.min(as_of_utc);
        let before_cutoff = timestamps.partition_point(|stamp| *stamp < cutoff);
        if let Some(index) = before_cutoff.checked_sub(1) {
            let stamp = &history.stamps()[index];
            snapshots.push(SelectedPeriod {
                bounds: interval.bounds(start, &end),
                as_of: if partial {
                    isoformat(&as_of)
                } else {
                    isoformat(&end)
                },
                partial,
                commit: stamp.sha().to_owned(),
                committed_at: isoformat(&stamp.committed_at()),
                first_parent_index: index,
            });
        }
        start = next_start;
    }
    snapshots
}

/// The chart label for a period: "Week ending July 06, 2025" or "July 2025".
pub fn label(start: NaiveDate, interval: Interval) -> String {
    match interval {
        Interval::Week => (start + Days::new(6))
            .format("Week ending %B %d, %Y")
            .to_string(),
        Interval::Month => start.format("%B %Y").to_string(),
    }
}

/// The first day of the month after the one containing `start`.
pub fn month_after(start: NaiveDate) -> NaiveDate {
    let (year, month) = if start.month() == 12 {
        (start.year() + 1, 1)
    } else {
        (start.year(), start.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(start)
}

/// Midnight at the start of `date` in `tz`. An ambiguous or skipped
/// midnight resolves like Python's `fold=0`: the offset in force before the
/// transition.
pub fn local_midnight(date: NaiveDate, tz: Tz) -> DateTime<Tz> {
    let naive = date.and_hms_opt(0, 0, 0).unwrap_or_default();
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(moment) | LocalResult::Ambiguous(moment, _) => moment,
        LocalResult::None => {
            let day_before = naive - Days::new(1);
            let offset = tz.offset_from_utc_datetime(&day_before).fix();
            tz.from_utc_datetime(&(naive - offset))
        }
    }
}

/// Python's `datetime.isoformat()`: seconds always, microseconds only when
/// non-zero, and a `+HH:MM` offset.
pub fn isoformat<Z: TimeZone>(moment: &DateTime<Z>) -> String
where
    Z::Offset: std::fmt::Display,
{
    let fixed = moment.fixed_offset();
    let micros = fixed.timestamp_subsec_micros();
    if micros == 0 {
        fixed.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
    } else {
        format!(
            "{}.{micros:06}{}",
            fixed.format("%Y-%m-%dT%H:%M:%S"),
            fixed.format("%:z")
        )
    }
}

/// Parses an aware ISO 8601 timestamp such as `2026-09-20T04:30:46+00:00`.
pub fn parse_as_of(text: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(text.trim()).map_err(|_| {
        Error::Configuration(format!(
            "The as-of time must be an ISO 8601 timestamp with a timezone: {text}"
        ))
    })
}

/// An IANA timezone by name, such as `America/New_York`.
pub fn parse_timezone(name: &str) -> Result<Tz> {
    name.trim()
        .parse()
        .map_err(|_| Error::Timezone(name.to_owned()))
}

/// The machine's IANA timezone.
pub fn local_timezone() -> Result<Tz> {
    let name =
        iana_time_zone::get_timezone().map_err(|error| Error::Timezone(error.to_string()))?;
    parse_timezone(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stamp(text: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(text).unwrap()
    }

    fn history(commits: &[(&str, &str)]) -> History {
        History::from_commits(
            commits
                .iter()
                .map(|(sha, when)| ((*sha).to_owned(), stamp(when))),
        )
    }

    #[test]
    fn month_after_rolls_over_the_year() {
        assert_eq!(
            month_after(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
        );
    }

    #[test]
    fn month_after_stays_within_the_year() {
        assert_eq!(
            month_after(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
        );
    }

    #[test]
    fn weekly_label_names_the_sunday_with_a_padded_day() {
        assert_eq!(
            label(
                NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
                Interval::Week
            ),
            "Week ending July 06, 2025"
        );
    }

    #[test]
    fn monthly_label_names_the_month() {
        assert_eq!(
            label(
                NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
                Interval::Month
            ),
            "July 2025"
        );
    }

    #[test]
    fn isoformat_omits_zero_microseconds() {
        assert_eq!(
            isoformat(&stamp("2025-07-07T00:00:00-04:00")),
            "2025-07-07T00:00:00-04:00"
        );
    }

    #[test]
    fn isoformat_writes_six_microsecond_digits() {
        assert_eq!(
            isoformat(&stamp("2026-09-20T04:30:46.620682+00:00")),
            "2026-09-20T04:30:46.620682+00:00"
        );
    }

    #[test]
    fn isoformat_writes_utc_as_a_zero_offset() {
        assert_eq!(
            isoformat(&stamp("2026-09-20T04:30:46Z")),
            "2026-09-20T04:30:46+00:00"
        );
    }

    #[test]
    fn history_counts_inversions_and_keeps_the_cumulative_maximum() {
        let history = history(&[
            ("a", "2025-01-10T12:00:00-05:00"),
            ("b", "2025-02-02T12:00:00-05:00"),
            ("c", "2025-01-20T12:00:00-05:00"),
        ]);
        assert_eq!(history.inversions(), 1);
        assert_eq!(
            history.stamps()[2].cumulative_at(),
            stamp("2025-02-02T12:00:00-05:00")
        );
    }

    #[test]
    fn local_midnight_uses_the_zone_offset() {
        let midnight = local_midnight(
            NaiveDate::from_ymd_opt(2025, 7, 7).unwrap(),
            chrono_tz::America::New_York,
        );
        assert_eq!(isoformat(&midnight), "2025-07-07T00:00:00-04:00");
    }

    #[test]
    fn local_midnight_in_a_gap_uses_the_offset_before_the_transition() {
        let midnight = local_midnight(
            NaiveDate::from_ymd_opt(2024, 3, 10).unwrap(),
            chrono_tz::America::Havana,
        );
        assert_eq!(isoformat(&midnight), "2024-03-10T01:00:00-04:00");
    }

    #[test]
    fn parse_timezone_rejects_unknown_names() {
        assert!(matches!(
            parse_timezone("Mars/Olympus").unwrap_err(),
            Error::Timezone(_)
        ));
    }

    #[test]
    fn parse_as_of_requires_an_offset() {
        assert!(matches!(
            parse_as_of("2026-09-20T04:30:46").unwrap_err(),
            Error::Configuration(_)
        ));
    }
}
