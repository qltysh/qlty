//! The trend chart, drawn to the design of the original matplotlib figure: a
//! 16 by 9 inch figure at 72 points per inch, so every coordinate is in
//! points and the HTML positions annotation markers from the same numbers.

use std::fmt::Write as _;

use chrono::{Datelike, Days, NaiveDate};

use crate::error::{Error, Result};
use crate::periods::Interval;
use crate::render::escape_html;
use crate::report::{ChartPeriod, ReportData};

const FIGURE_WIDTH: f64 = 1152.0;
const FIGURE_HEIGHT: f64 = 648.0;
const AXES_LEFT: f64 = 0.085 * FIGURE_WIDTH;
const AXES_WIDTH: f64 = 0.88 * FIGURE_WIDTH;
const AXES_RIGHT: f64 = AXES_LEFT + AXES_WIDTH;
const AXES_HEIGHT: f64 = 0.53 * FIGURE_HEIGHT;
const AXES_BOTTOM: f64 = FIGURE_HEIGHT - 0.23 * FIGURE_HEIGHT;
const AXES_TOP: f64 = AXES_BOTTOM - AXES_HEIGHT;
/// Twelve points below the axes plus the ascent of an 11-point DejaVu Sans line.
const DATE_LABEL_BASELINE: f64 = 519.318281;
/// A 14 pt² marker: radius √14 / 2.
const MARKER_RADIUS: f64 = 1.870829;

/// The part of the figure the HTML shows: the axes and date labels, without
/// the title and legend the page renders itself.
pub const VIEW_BOX: &str = "80 144 1050 416";

const BACKGROUND: &str = "#f4f1ee";
const INK: &str = "#252725";
const MUTED: &str = "#777e77";
const GREEN: &str = "#aed9b0";
const RED: &str = "#d9948b";
const NET_GREEN: &str = "#286648";
const NET_RED: &str = "#b23e31";
const ZERO_LINE: &str = "#aeb2a9";
const GRID: &str = "#e2ded8";
const FONT: &str = "font-family: 'DejaVu Sans', sans-serif";

/// The rendered chart.
#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    /// The chart cropped to [`VIEW_BOX`] with hit targets, for the HTML report.
    pub svg: String,
    /// The full figure with title and legend, for the `.svg` file.
    pub standalone_svg: String,
    /// Per-period geometry in figure points.
    pub targets: Vec<Target>,
}

/// Where one period sits in the figure, in points from the top left.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    /// `[x, y, width, height]` of the hover and click target.
    pub bounds: [f64; 4],
    /// The net change marker, when the period has a measured change.
    pub net_point: Option<[f64; 2]>,
    /// The bottom of the reduced-quality bar.
    pub lower_edge: Option<f64>,
    /// The lowest drawn gridline, which reads as the chart's floor.
    pub baseline: f64,
}

/// Draws the chart for `data`'s periods with the folded chart `values`.
pub fn chart(data: &ReportData, values: &[ChartPeriod]) -> Result<Chart> {
    if values.is_empty() {
        return Err(Error::Render(
            "The report has no snapshots to render".to_owned(),
        ));
    }
    if values.len() != data.periods.len() {
        return Err(Error::Mismatch(format!(
            "the chart has {} values for {} periods",
            values.len(),
            data.periods.len()
        )));
    }
    let ends = data
        .periods
        .iter()
        .map(|period| period_end(period.start(), data.interval))
        .collect::<Result<Vec<_>>>()?;
    let series = Series::from_values(values);
    let layout = Layout::new(&ends, &series, data.interval);
    let body = draw(&layout, &series, &date_ticks(&ends, data.interval));
    let targets = hit_targets(&layout, &series);

    let interval_label = match data.interval {
        Interval::Week => "Weekly",
        Interval::Month => "Monthly",
    };
    let mut svg = format!(
        "<svg class=\"chart\" role=\"group\" aria-label=\"{interval_label} code quality. \
         Hover for values; click or press Enter for file details.\" viewBox=\"{VIEW_BOX}\" \
         version=\"1.1\">\n{body}<g id=\"week-targets\">"
    );
    for (index, (target, period)) in targets.iter().zip(values).enumerate() {
        svg.push_str(&hit_target_markup(index, target, period, data.interval));
    }
    svg.push_str("</g></svg>");

    let title = crate::render::html::report_title(data);
    let mut standalone_svg = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{FIGURE_WIDTH}pt\" \
         height=\"{FIGURE_HEIGHT}pt\" viewBox=\"0 0 {FIGURE_WIDTH} {FIGURE_HEIGHT}\" \
         version=\"1.1\">\n<title>{}</title>\n{body}",
        escape_html(&title)
    );
    standalone_svg.push_str(&title_and_legend(&title));
    standalone_svg.push_str("</svg>\n");

    Ok(Chart {
        svg,
        standalone_svg,
        targets,
    })
}

/// The last calendar day of the period starting on `start`.
fn period_end(start: &str, interval: Interval) -> Result<NaiveDate> {
    let start = NaiveDate::parse_from_str(start, "%Y-%m-%d")
        .map_err(|err| Error::Render(format!("invalid period start {start}: {err}")))?;
    let end = match interval {
        Interval::Week => start.checked_add_days(Days::new(6)),
        Interval::Month => month_after(start).and_then(|next| next.checked_sub_days(Days::new(1))),
    };
    end.ok_or_else(|| Error::Render(format!("period end out of range for {start}")))
}

fn month_after(date: NaiveDate) -> Option<NaiveDate> {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1)
}

/// The three plotted series, one entry per period.
struct Series {
    positive: Vec<Option<f64>>,
    negative: Vec<Option<f64>>,
    net: Vec<Option<f64>>,
}

impl Series {
    fn from_values(values: &[ChartPeriod]) -> Self {
        let totals = values.iter().map(|period| &period.with_file_changes);
        Self {
            positive: totals.clone().map(|totals| totals.positive).collect(),
            negative: totals.clone().map(|totals| totals.negative).collect(),
            net: totals.map(|totals| totals.net).collect(),
        }
    }
}

/// Data limits and the mapping from dates and values to figure points.
struct Layout {
    /// Days between period ends: 7 for weeks, 30 for months.
    spacing: f64,
    /// Each period end as a day number.
    days: Vec<f64>,
    day_min: f64,
    day_max: f64,
    value_min: f64,
    value_max: f64,
    /// Value-axis ticks within the limits, ascending.
    ticks: Vec<f64>,
}

impl Layout {
    fn new(ends: &[NaiveDate], series: &Series, interval: Interval) -> Self {
        let spacing = match interval {
            Interval::Week => 7.0,
            Interval::Month => 30.0,
        };
        let days: Vec<f64> = ends
            .iter()
            .map(|end| f64::from(end.num_days_from_ce()))
            .collect();
        let first = days.first().copied().unwrap_or_default();
        let last = days.last().copied().unwrap_or_default();
        let top = series
            .positive
            .iter()
            .flatten()
            .fold(0.01_f64, |top, &value| top.max(value));
        let bottom = series
            .negative
            .iter()
            .flatten()
            .fold(-0.01_f64, |bottom, &value| bottom.min(value));
        let value_min = bottom * 1.20;
        let value_max = top * 1.35;
        let ticks = nice_ticks(value_min, value_max)
            .into_iter()
            .filter(|&tick| value_min <= tick && tick <= value_max)
            .collect();
        Self {
            spacing,
            days,
            day_min: first - spacing * 5.0 / 7.0,
            day_max: last + spacing * 5.0 / 7.0,
            value_min,
            value_max,
            ticks,
        }
    }

    fn px(&self, day: f64) -> f64 {
        AXES_LEFT + (day - self.day_min) / (self.day_max - self.day_min) * AXES_WIDTH
    }

    fn py(&self, value: f64) -> f64 {
        AXES_TOP + (self.value_max - value) / (self.value_max - self.value_min) * AXES_HEIGHT
    }

    fn bar_half_width(&self) -> f64 {
        self.spacing * 4.6 / 7.0 / (self.day_max - self.day_min) * AXES_WIDTH / 2.0
    }

    fn baseline(&self) -> f64 {
        self.ticks
            .first()
            .map_or(AXES_BOTTOM, |&lowest| self.py(lowest))
    }
}

/// Tick locations following matplotlib's `MaxNLocator(nbins=6)`: the smallest
/// step of 1, 1.5, 2, 2.5, 3, 4, 5, 6, or 8 times a power of ten that yields
/// at most six intervals, extended to enclose the limits.
fn nice_ticks(low: f64, high: f64) -> Vec<f64> {
    const BINS: f64 = 6.0;
    const MIN_TICKS: usize = 2;
    const STEPS: [f64; 20] = [
        0.1, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.8, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0,
        10.0, 15.0,
    ];

    let (low, high) = if low > high { (high, low) } else { (low, high) };
    let span = high - low;
    let mean = (high + low) / 2.0;
    let offset = if mean.abs() / span < 100.0 {
        0.0
    } else {
        10_f64.powf(mean.abs().log10().floor()).copysign(mean)
    };
    let scale = 10_f64.powf((span / BINS).log10().floor());
    let low = low - offset;
    let high = high - offset;
    let raw_step = (high - low) / BINS;
    let steps: Vec<f64> = STEPS.iter().map(|step| step * scale).collect();
    let largest = steps
        .iter()
        .position(|&step| step >= raw_step)
        .unwrap_or(steps.len() - 1);
    let mut ticks = Vec::new();
    for &step in steps[..=largest].iter().rev() {
        let start = (low / step).floor() * step;
        let first = edge_at_or_below(low - start, step);
        let last = edge_at_or_above(high - start, step);
        ticks = (first..=last)
            .map(|index| index as f64 * step + start)
            .collect();
        let shown = ticks
            .iter()
            .filter(|&&tick| tick >= low && tick <= high)
            .count();
        if shown >= MIN_TICKS {
            break;
        }
    }
    ticks.into_iter().map(|tick| tick + offset).collect()
}

/// Python's `divmod` for floats: a floored quotient and a remainder with the divisor's sign.
fn divmod(value: f64, divisor: f64) -> (f64, f64) {
    let mut remainder = value % divisor;
    let mut quotient = (value - remainder) / divisor;
    if remainder != 0.0 && (divisor < 0.0) != (remainder < 0.0) {
        remainder += divisor;
        quotient -= 1.0;
    }
    let floored = if quotient == 0.0 {
        0.0_f64.copysign(value / divisor)
    } else {
        let floor = quotient.floor();
        if quotient - floor > 0.5 {
            floor + 1.0
        } else {
            floor
        }
    };
    (floored, remainder)
}

/// The largest multiple of `step` at or below `value`, tolerating rounding
/// just under the next multiple.
fn edge_at_or_below(value: f64, step: f64) -> i64 {
    let (quotient, remainder) = divmod(value, step);
    if (remainder / step - 1.0).abs() < 1e-10 {
        quotient as i64 + 1
    } else {
        quotient as i64
    }
}

/// The smallest multiple of `step` at or above `value`, tolerating rounding
/// just over a multiple.
fn edge_at_or_above(value: f64, step: f64) -> i64 {
    let (quotient, remainder) = divmod(value, step);
    if (remainder / step).abs() < 1e-10 {
        quotient as i64
    } else {
        quotient as i64 + 1
    }
}

/// One labelled date on the horizontal axis.
struct DateTick {
    index: usize,
    label: String,
    anchor: &'static str,
}

/// Picks which period ends get a date label so labels never crowd, and
/// formats them with the year only when the chart spans more than one.
fn date_ticks(ends: &[NaiveDate], interval: Interval) -> Vec<DateTick> {
    let count = ends.len();
    let last = count - 1;
    let spans_years = ends[0].year() != ends[last].year();
    let step = match interval {
        Interval::Week => 4 * last.div_ceil(if spans_years { 24 } else { 32 }).max(1),
        Interval::Month => last.div_ceil(if spans_years { 5 } else { 8 }).max(1),
    };
    let mut indexes: Vec<usize> = (0..count).step_by(step).collect();
    let final_position = indexes.len() - 1;
    let final_tick = indexes[final_position];
    if spans_years && final_position > 0 && (last - final_tick) * 2 < step {
        indexes[final_position] = last;
    } else if last - final_tick >= 2 || (spans_years && final_tick != last) {
        indexes.push(last);
    }
    let date_format = match (spans_years, interval) {
        (true, _) => "%b %Y",
        (false, Interval::Month) => "%b",
        (false, Interval::Week) => "%b %d",
    };
    let label_count = indexes.len();
    indexes
        .into_iter()
        .enumerate()
        .map(|(position, index)| DateTick {
            index,
            label: ends[index].format(date_format).to_string(),
            anchor: match (spans_years, position) {
                (true, 0) => "start",
                (true, position) if position + 1 == label_count => "end",
                _ => "middle",
            },
        })
        .collect()
}

/// Formats a coordinate with up to six decimals, as matplotlib does, so the
/// browser anti-aliases edges the same way.
fn pt(value: f64) -> String {
    let text = format!("{value:.6}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// The axes contents: background, gridlines, date labels, bars, zero line,
/// the net line, and its markers.
fn draw(layout: &Layout, series: &Series, dates: &[DateTick]) -> String {
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        "<rect x=\"0\" y=\"0\" width=\"{FIGURE_WIDTH}\" height=\"{FIGURE_HEIGHT}\" fill=\"{BACKGROUND}\"/>"
    );
    for &tick in &layout.ticks {
        let y = pt(layout.py(tick));
        let _ = writeln!(
            svg,
            "<path d=\"M {left} {y} L {right} {y}\" fill=\"none\" stroke=\"{GRID}\" \
             stroke-width=\"0.8\" stroke-linecap=\"square\"/>",
            left = pt(AXES_LEFT),
            right = pt(AXES_RIGHT),
        );
    }
    for date in dates {
        let _ = writeln!(
            svg,
            "<text x=\"{x}\" y=\"{DATE_LABEL_BASELINE}\" text-anchor=\"{anchor}\" \
             style=\"font-size: 11px; {FONT}; fill: {MUTED}\">{label}</text>",
            x = pt(layout.px(layout.days[date.index])),
            anchor = date.anchor,
            label = escape_html(&date.label),
        );
    }
    let zero = layout.py(0.0);
    let half_width = layout.bar_half_width();
    for (bars, color) in [(&series.positive, GREEN), (&series.negative, RED)] {
        for (&day, &value) in layout.days.iter().zip(bars) {
            let Some(value) = value else {
                continue;
            };
            let edge = layout.py(value);
            let _ = writeln!(
                svg,
                "<rect x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\" fill=\"{color}\"/>",
                x = pt(layout.px(day) - half_width),
                y = pt(edge.min(zero)),
                width = pt(half_width * 2.0),
                height = pt((edge - zero).abs()),
            );
        }
    }
    let _ = writeln!(
        svg,
        "<path d=\"M {left} {zero} L {right} {zero}\" fill=\"none\" stroke=\"{ZERO_LINE}\" \
         stroke-width=\"1\" stroke-linecap=\"square\"/>",
        left = pt(AXES_LEFT),
        right = pt(AXES_RIGHT),
        zero = pt(zero),
    );
    svg.push_str(&net_line(layout, &series.net));
    for (&day, &value) in layout.days.iter().zip(&series.net) {
        let Some(value) = value else {
            continue;
        };
        let _ = writeln!(
            svg,
            "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{MARKER_RADIUS}\" fill=\"{color}\"/>",
            cx = pt(layout.px(day)),
            cy = pt(layout.py(value)),
            color = net_color(value),
        );
    }
    svg
}

fn net_color(value: f64) -> &'static str {
    if value >= 0.0 {
        NET_GREEN
    } else {
        NET_RED
    }
}

/// The net line, split where it crosses zero so each piece takes the color
/// of its sign.
fn net_line(layout: &Layout, net: &[Option<f64>]) -> String {
    let mut svg = String::new();
    let mut segment = |from: (f64, f64), to: (f64, f64), color: &str| {
        let _ = writeln!(
            svg,
            "<path d=\"M {x1} {y1} L {x2} {y2}\" fill=\"none\" stroke=\"{color}\" \
             stroke-width=\"2.5\" stroke-linecap=\"square\"/>",
            x1 = pt(layout.px(from.0)),
            y1 = pt(layout.py(from.1)),
            x2 = pt(layout.px(to.0)),
            y2 = pt(layout.py(to.1)),
        );
    };
    for (pair, days) in net.windows(2).zip(layout.days.windows(2)) {
        let (Some(a), Some(b)) = (pair[0], pair[1]) else {
            continue;
        };
        let (day_a, day_b) = (days[0], days[1]);
        if (a >= 0.0) == (b >= 0.0) {
            segment((day_a, a), (day_b, b), net_color(b));
        } else {
            let cross = day_a + (day_b - day_a) * a.abs() / (a.abs() + b.abs());
            segment((day_a, a), (cross, 0.0), net_color(a));
            segment((cross, 0.0), (day_b, b), net_color(b));
        }
    }
    svg
}

fn hit_targets(layout: &Layout, series: &Series) -> Vec<Target> {
    let centers: Vec<f64> = layout.days.iter().map(|&day| layout.px(day)).collect();
    let baseline = layout.baseline();
    centers
        .iter()
        .enumerate()
        .map(|(index, &center)| {
            let left = index
                .checked_sub(1)
                .map_or(AXES_LEFT, |previous| (centers[previous] + center) / 2.0);
            let right = centers
                .get(index + 1)
                .map_or(AXES_RIGHT, |next| (center + next) / 2.0);
            Target {
                bounds: [left, AXES_TOP, right - left, AXES_HEIGHT],
                net_point: series.net[index].map(|value| [center, layout.py(value)]),
                lower_edge: series.negative[index].map(|value| layout.py(value)),
                baseline,
            }
        })
        .collect()
}

fn hit_target_markup(
    index: usize,
    target: &Target,
    period: &ChartPeriod,
    interval: Interval,
) -> String {
    let totals = &period.with_file_changes;
    let label = match (totals.positive, totals.negative, totals.net) {
        (Some(positive), Some(negative), Some(net)) => format!(
            "{}. Improved {positive:+.3}, reduced {negative:+.3}, net {net:+.3}.",
            period.label
        ),
        _ => format!(
            "{}. No earlier {} to compare.",
            period.label,
            interval.as_str()
        ),
    };
    let [x, y, width, height] = target.bounds;
    format!(
        "<rect class=\"week-hit\" data-week=\"{index}\" x=\"{x:.4}\" y=\"{y:.4}\" \
         width=\"{width:.4}\" height=\"{height:.4}\" tabindex=\"0\" role=\"button\" \
         aria-haspopup=\"dialog\" aria-controls=\"week-details\" aria-label=\"{}\"/>",
        escape_html(&label)
    )
}

/// The figure title and legend of the standalone file, at the positions
/// matplotlib laid them out.
fn title_and_legend(title: &str) -> String {
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        "<text x=\"80.64\" y=\"93.96\" style=\"font-weight: 600; font-size: 30px; {FONT}; \
         fill: {INK}\">{}</text>",
        escape_html(title)
    );
    let label_style = format!("font-size: 12px; {FONT}; fill: {INK}");
    let _ = writeln!(
        svg,
        "<rect x=\"87.984\" y=\"548.5444\" width=\"24\" height=\"8.4\" fill=\"{GREEN}\"/>\n\
         <text x=\"121.584\" y=\"556.9444\" style=\"{label_style}\">Improved quality</text>\n\
         <rect x=\"247.4059\" y=\"548.5444\" width=\"24\" height=\"8.4\" fill=\"{RED}\"/>\n\
         <text x=\"281.0059\" y=\"556.9444\" style=\"{label_style}\">Reduced quality</text>\n\
         <path d=\"M 401.8759 552.7444 L 425.8759 552.7444\" fill=\"none\" stroke=\"{NET_GREEN}\" \
         stroke-width=\"2.5\" stroke-linecap=\"square\"/>\n\
         <text x=\"435.4759\" y=\"556.9444\" style=\"{label_style}\">Net change (green + / red −)</text>"
    );
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> NaiveDate {
        NaiveDate::parse_from_str(text, "%Y-%m-%d").unwrap()
    }

    fn assert_close(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
        }
    }

    #[test]
    fn nice_ticks_match_matplotlib_for_the_interface_weekly_range() {
        let ticks = nice_ticks(-0.2510737640884069 * 1.20, 0.1450 * 1.35);
        assert_close(&ticks, &[-0.4, -0.3, -0.2, -0.1, 0.0, 0.1, 0.2]);
    }

    #[test]
    fn nice_ticks_use_a_one_and_a_half_step() {
        let ticks = nice_ticks(-0.5963297730129606, 0.28851004396989316);
        assert_close(&ticks, &[-0.6, -0.45, -0.3, -0.15, 0.0, 0.15, 0.3]);
    }

    #[test]
    fn nice_ticks_use_a_four_step() {
        let ticks = nice_ticks(-1.39029153747919, 0.49739326460734684);
        assert_close(&ticks, &[-1.6, -1.2, -0.8, -0.4, 0.0, 0.4, 0.8]);
    }

    #[test]
    fn nice_ticks_use_a_two_and_a_half_step() {
        let ticks = nice_ticks(-0.06, 0.09);
        assert_close(
            &ticks,
            &[-0.075, -0.05, -0.025, 0.0, 0.025, 0.05, 0.075, 0.1],
        );
    }

    #[test]
    fn nice_ticks_cover_the_minimum_magnitudes() {
        let ticks = nice_ticks(-0.012, 0.0135);
        assert_close(&ticks, &[-0.015, -0.01, -0.005, 0.0, 0.005, 0.01, 0.015]);
    }

    #[test]
    fn divmod_follows_python_for_negative_values() {
        let (quotient, remainder) = divmod(-0.35, 0.1);
        assert_eq!(quotient, -4.0);
        assert!((remainder - 0.05).abs() < 1e-12);
        assert_eq!(divmod(7.0, 2.0), (3.0, 1.0));
    }

    #[test]
    fn month_ends_fall_on_the_last_day() {
        assert_eq!(
            period_end("2024-02-01", Interval::Month).unwrap(),
            date("2024-02-29")
        );
        assert_eq!(
            period_end("2024-12-01", Interval::Month).unwrap(),
            date("2024-12-31")
        );
    }

    #[test]
    fn week_ends_six_days_after_the_start() {
        assert_eq!(
            period_end("2025-06-30", Interval::Week).unwrap(),
            date("2025-07-06")
        );
    }

    #[test]
    fn rejects_an_invalid_period_start() {
        assert!(period_end("2025-13-01", Interval::Week).is_err());
    }

    #[test]
    fn weekly_labels_within_one_year_show_month_and_day_every_fourth_week() {
        let ends: Vec<NaiveDate> = (0..10)
            .map(|week| date("2025-03-02") + Days::new(7 * week))
            .collect();
        let ticks = date_ticks(&ends, Interval::Week);
        let labels: Vec<(usize, &str, &str)> = ticks
            .iter()
            .map(|tick| (tick.index, tick.label.as_str(), tick.anchor))
            .collect();
        assert_eq!(
            labels,
            [
                (0, "Mar 02", "middle"),
                (4, "Mar 30", "middle"),
                (8, "Apr 27", "middle"),
            ]
        );
    }

    #[test]
    fn labels_spanning_years_align_the_ends_and_add_the_last_period() {
        let ends: Vec<NaiveDate> = (0..64)
            .map(|week| date("2025-07-06") + Days::new(7 * week))
            .collect();
        let ticks = date_ticks(&ends, Interval::Week);
        let labels: Vec<(usize, &str, &str)> = ticks
            .iter()
            .map(|tick| (tick.index, tick.label.as_str(), tick.anchor))
            .collect();
        assert_eq!(
            labels,
            [
                (0, "Jul 2025", "start"),
                (12, "Sep 2025", "middle"),
                (24, "Dec 2025", "middle"),
                (36, "Mar 2026", "middle"),
                (48, "Jun 2026", "middle"),
                (63, "Sep 2026", "end"),
            ]
        );
    }

    #[test]
    fn monthly_labels_within_one_year_show_every_month() {
        let ends = vec![date("2024-01-31"), date("2024-02-29"), date("2024-03-31")];
        let ticks = date_ticks(&ends, Interval::Month);
        let labels: Vec<&str> = ticks.iter().map(|tick| tick.label.as_str()).collect();
        assert_eq!(labels, ["Jan", "Feb", "Mar"]);
    }

    #[test]
    fn a_single_period_gets_one_label() {
        let ticks = date_ticks(&[date("2024-01-31")], Interval::Month);
        assert_eq!(ticks.len(), 1);
        assert_eq!(ticks[0].anchor, "middle");
    }

    #[test]
    fn formats_coordinates_without_trailing_zeros() {
        assert_eq!(pt(97.92), "97.92");
        assert_eq!(pt(267.5017264), "267.501726");
        assert_eq!(pt(-0.0000001), "0");
        assert_eq!(pt(24.0), "24");
    }
}
