//! Terminal presentation for `qlty slop-one`: the stepped progress of a
//! report run, the spinner while files score, the closing sparkline, and
//! the explanations that turn library errors into advice.
//!
//! Progress goes to stderr and hides itself when stderr is not a terminal,
//! where only the finished step lines are printed, so logs stay readable
//! and stdout stays clean for `--json`.

use std::collections::VecDeque;
use std::io::IsTerminal as _;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use qlty_slop_one::{JevProvider, Usage};
use qlty_slop_one_trends::periods::Interval;
use qlty_slop_one_trends::pipeline::{Observer, Stage};
use qlty_slop_one_trends::run::Manifest;

use crate::{CommandError, CommandSuccess};

const SIGNUP_URL: &str = "https://typesafe.ai/";
const OPENROUTER_KEYS_URL: &str = "https://openrouter.ai/settings/keys";
const TICK_CHARS: &str = "⠁⠂⠄⡀⢀⠠⠐⠈ ";
const TICK: Duration = Duration::from_millis(100);
const LABEL_WIDTH: usize = 11;
/// How far back the pace of analyzed contents is measured.
const PACE_WINDOW: Duration = Duration::from_secs(30);
/// The estimate is shown once this share of contents is done, or this much
/// time has passed, whichever comes first.
const ESTIMATE_AFTER_SHARE: f64 = 0.05;
const ESTIMATE_AFTER: Duration = Duration::from_secs(15);
const SPARK_CELLS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// The three steps a report run shows: reading the history into snapshots,
/// scoring, and rendering, which covers export and render.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Snapshots,
    Scoring,
    Report,
}

impl Step {
    fn label(self) -> &'static str {
        match self {
            Self::Snapshots => "Reading",
            Self::Scoring => "Scoring",
            Self::Report => "Rendering",
        }
    }
}

struct Running {
    step: Step,
    bar: ProgressBar,
    started: Instant,
}

#[derive(Default)]
struct Scoring {
    total: usize,
    done: usize,
    cached: usize,
    analyzed: usize,
    failed: usize,
    without_cached_analysis: usize,
    /// When each analyzed content finished, for the pace.
    recent: VecDeque<Instant>,
    usage: Option<Usage>,
}

struct State {
    running: Option<Running>,
    frozen: Vec<(Interval, usize, String)>,
    scoring: Scoring,
    report: Option<String>,
}

/// Shows a report run as it happens.
pub struct ReportProgress {
    multi: MultiProgress,
    interactive: bool,
    state: Mutex<State>,
}

impl ReportProgress {
    pub fn new() -> Self {
        let interactive = std::io::stderr().is_terminal();
        let multi = MultiProgress::new();
        if !interactive {
            multi.set_draw_target(ProgressDrawTarget::hidden());
        }
        Self {
            multi,
            interactive,
            state: Mutex::new(State {
                running: None,
                frozen: Vec::new(),
                scoring: Scoring::default(),
                report: None,
            }),
        }
    }

    fn start(&self, state: &mut State, step: Step) {
        if state
            .running
            .as_ref()
            .is_some_and(|running| running.step == step)
        {
            return;
        }
        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(spinner_style());
        bar.set_prefix(format!("{:<LABEL_WIDTH$}", step.label()));
        bar.enable_steady_tick(TICK);
        state.running = Some(Running {
            step,
            bar,
            started: Instant::now(),
        });
    }

    fn finish(&self, state: &mut State, step: Step, body: String) {
        let Some(running) = state.running.take() else {
            return;
        };
        if running.step != step {
            state.running = Some(running);
            return;
        }
        let line = finished_line(step.label(), &body, running.started.elapsed());
        self.print_line(&running.bar, line);
    }

    /// Replaces a step's bar with its finished line. The line is printed
    /// above whatever is still animating, so it is never padded or
    /// redrawn.
    fn print_line(&self, bar: &ProgressBar, line: String) {
        bar.finish_and_clear();
        if self.interactive {
            let _ = self.multi.println(&line);
        } else {
            eprintln!("{line}");
        }
    }

    /// Leaves the current step marked as failed, so the error that follows
    /// reads in context.
    pub fn fail(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(running) = state.running.take() {
            let line = format!(
                "  {} {:<LABEL_WIDTH$} {}",
                style("✖").red().bold(),
                running.step.label(),
                style("stopped").dim()
            );
            self.print_line(&running.bar, line);
        }
    }

    fn scoring_message(scoring: &Scoring, elapsed: Duration) -> String {
        if scoring.total == 0 {
            return String::new();
        }
        let mut parts = Vec::new();
        let ready = scoring.done as f64 >= scoring.total as f64 * ESTIMATE_AFTER_SHARE
            || elapsed >= ESTIMATE_AFTER;
        if scoring.analyzed == 0 {
            parts.push("from cache".to_owned());
        } else if let Some(left) = ready.then(|| estimate(scoring)).flatten() {
            parts.push(format!("~{} left", duration(left)));
        } else {
            parts.push("estimating…".to_owned());
        }
        if let Some(usage) = &scoring.usage {
            if usage.requests > 0 {
                parts.push(spend(usage));
            }
        }
        parts.join(" · ")
    }

    fn scoring_summary(scoring: &Scoring) -> String {
        if scoring.total == 0 {
            return "nothing new to score".to_owned();
        }
        let mut parts = vec![
            format!("{} cached", count(scoring.cached)),
            format!("{} analyzed", count(scoring.analyzed)),
        ];
        if let Some(usage) = &scoring.usage {
            if usage.requests > 0 {
                parts.push(spend(usage));
            }
        }
        if scoring.failed > 0 {
            let mut note = format!("{} could not be analyzed", count(scoring.failed));
            if scoring.without_cached_analysis > 0 {
                note.push_str(&format!(
                    " ({} without cached analysis; run without --offline)",
                    count(scoring.without_cached_analysis)
                ));
            }
            parts.push(note);
        }
        format!(
            "{} contents   {}",
            count(scoring.total),
            style(parts.join(" · ")).dim()
        )
    }
}

impl Default for ReportProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl Observer for ReportProgress {
    fn stage_started(&self, stage: Stage) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match stage {
            Stage::Snapshots => self.start(&mut state, Step::Snapshots),
            Stage::Scoring => self.start(&mut state, Step::Scoring),
            Stage::Export | Stage::Render => self.start(&mut state, Step::Report),
        }
    }

    fn snapshots_frozen(&self, manifest: &Manifest) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.frozen.push((
            manifest.interval,
            manifest.snapshots.len(),
            manifest.since.clone(),
        ));
        if let Some(running) = &state.running {
            running.bar.set_message(format!(
                "{} {}",
                count(manifest.snapshots.len()),
                match manifest.interval {
                    Interval::Week => "weekly",
                    Interval::Month => "monthly",
                }
            ));
        }
    }

    fn scoring_started(&self, pending: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.scoring.total = pending;
        if let Some(running) = &state.running {
            if pending > 0 {
                running.bar.set_style(scoring_style());
                running.bar.set_length(pending as u64);
                running.bar.set_position(0);
            }
        }
    }

    fn content_scored(&self, cached: bool, usage: &Usage) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Instant::now();
        let scoring = &mut state.scoring;
        scoring.done += 1;
        if cached {
            scoring.cached += 1;
        } else {
            scoring.analyzed += 1;
            scoring.recent.push_back(now);
            while scoring
                .recent
                .front()
                .is_some_and(|instant| now.duration_since(*instant) > PACE_WINDOW)
            {
                scoring.recent.pop_front();
            }
        }
        scoring.usage = Some(usage.clone());
        let elapsed = state
            .running
            .as_ref()
            .map_or(Duration::ZERO, |running| running.started.elapsed());
        let message = Self::scoring_message(&state.scoring, elapsed);
        if let Some(running) = &state.running {
            running.bar.set_position(state.scoring.done as u64);
            running.bar.set_message(message);
        }
    }

    fn content_failed(&self, _key: &str, error: &qlty_slop_one::Error) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.scoring.done += 1;
        state.scoring.failed += 1;
        if matches!(error, qlty_slop_one::Error::Offline) {
            state.scoring.without_cached_analysis += 1;
        }
        if let Some(running) = &state.running {
            running.bar.set_position(state.scoring.done as u64);
        }
    }

    fn report_rendered(&self, path: &Path) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.report = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
    }

    fn stage_finished(&self, stage: Stage) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match stage {
            Stage::Snapshots => {
                let counts: Vec<String> = state
                    .frozen
                    .iter()
                    .map(|(interval, count, _)| {
                        format!(
                            "{} {}",
                            self::count(*count),
                            match interval {
                                Interval::Week => "weekly",
                                Interval::Month => "monthly",
                            }
                        )
                    })
                    .collect();
                let since = state
                    .frozen
                    .first()
                    .map(|(_, _, since)| since.as_str())
                    .and_then(|since| NaiveDate::parse_from_str(since, "%Y-%m-%d").ok())
                    .map(|since| since.format("%b %-d, %Y").to_string())
                    .unwrap_or_default();
                let body = format!(
                    "{} snapshots   {}",
                    counts.join(" · "),
                    style(format!("{since} → today")).dim()
                );
                self.finish(&mut state, Step::Snapshots, body);
            }
            Stage::Scoring => {
                let body = Self::scoring_summary(&state.scoring);
                self.finish(&mut state, Step::Scoring, body);
            }
            Stage::Export => {}
            Stage::Render => {
                let body = state.report.clone().unwrap_or_default();
                self.finish(&mut state, Step::Report, body);
            }
        }
    }
}

/// The seconds left, from the pace of analyzed contents over the recent
/// window and the share of contents that have needed analysis so far.
fn estimate(scoring: &Scoring) -> Option<Duration> {
    let (first, last) = (scoring.recent.front()?, scoring.recent.back()?);
    if scoring.recent.len() < 2 || scoring.done == 0 {
        return None;
    }
    let span = last.duration_since(*first).as_secs_f64().max(1.0);
    let per_second = (scoring.recent.len() - 1) as f64 / span;
    let remaining = scoring.total.saturating_sub(scoring.done) as f64;
    let share_analyzed = scoring.analyzed as f64 / scoring.done as f64;
    let seconds = remaining * share_analyzed / per_second;
    seconds
        .is_finite()
        .then(|| Duration::from_secs_f64(seconds))
}

/// A spinner while files score. Silent when stderr is not a terminal.
pub struct FileProgress {
    bar: ProgressBar,
    started: Instant,
    done: Mutex<usize>,
    total: usize,
}

impl FileProgress {
    pub fn new(total: usize) -> Self {
        let bar = if std::io::stderr().is_terminal() {
            ProgressBar::new(total as u64)
        } else {
            ProgressBar::hidden()
        };
        bar.set_style(files_style());
        bar.set_prefix(format!("Scoring {} {}", total, plural(total, "file")));
        bar.enable_steady_tick(TICK);
        Self {
            bar,
            started: Instant::now(),
            done: Mutex::new(0),
            total,
        }
    }

    pub fn file_done(&self) {
        let mut done = self
            .done
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *done += 1;
        self.bar.set_position(*done as u64);
        let elapsed = self.started.elapsed().as_secs_f64();
        if *done < self.total && elapsed > 0.0 {
            let left = (self.total - *done) as f64 * elapsed / *done as f64;
            self.bar
                .set_message(format!("~{} left", duration(Duration::from_secs_f64(left))));
        }
    }

    pub fn finish(&self) {
        self.bar.finish_and_clear();
    }
}

/// Twelve cells of weekly net change as levels around zero, so the shape
/// follows the chart's net line: the middle is no change, green cells
/// rise above it and red cells fall below, with the latest week called out.
pub fn sparkline_line(recent: &[Option<f64>]) -> Option<String> {
    let latest = recent.last().copied().flatten()?;
    // The direction follows the printed three decimals, so a change that
    // rounds to zero reads as flat rather than as a fall to −0.000.
    let latest = (latest * 1000.0).round() / 1000.0;
    let scale = recent
        .iter()
        .flatten()
        .map(|net| net.abs())
        .fold(0.0_f64, f64::max);
    let cells: String = recent
        .iter()
        .map(|net| match net {
            None => style("·").dim().to_string(),
            Some(net) => {
                let top = (SPARK_CELLS.len() - 1) as f64;
                let level = if scale > 0.0 {
                    ((net / scale + 1.0) / 2.0 * top).round() as usize
                } else {
                    (top / 2.0).round() as usize
                };
                let cell = SPARK_CELLS[level.min(SPARK_CELLS.len() - 1)];
                if *net > 0.0 {
                    style(cell).green().to_string()
                } else if *net < 0.0 {
                    style(cell).red().to_string()
                } else {
                    style(cell).dim().to_string()
                }
            }
        })
        .collect();
    let arrow = if latest > 0.0 {
        style("▲").green().to_string()
    } else if latest < 0.0 {
        style("▼").red().to_string()
    } else {
        style("—").dim().to_string()
    };
    Some(format!(
        "  Last {} {}   {}   this week net {} {}",
        recent.len(),
        plural(recent.len(), "week"),
        cells,
        signed(latest),
        arrow
    ))
}

/// Advice for a report failure, or the plain error for anything unforeseen.
pub fn explain_trends(error: qlty_slop_one_trends::Error) -> CommandError {
    use qlty_slop_one_trends::Error;
    match error {
        Error::Score(inner) => explain_scoring(inner),
        Error::NotARepository(_) => explained(
            "This directory is not inside a Git repository",
            &["Run qlty slop-one from a checkout, or pass files to score them directly."],
        ),
        Error::Configuration(message) if message.contains("no origin remote") => explained(
            "The repository has no origin remote",
            &[
                "The report links back to the repository, so it needs a web URL. Add an origin",
                "remote, or run the advanced form:",
                "qlty slop-one trends build --repository-url <URL> --since <DATE>",
            ],
        ),
        Error::TooFewSnapshots => explained(
            "Not enough history for a trend",
            &[
                "A trend needs at least two snapshots, and this repository has commits in only",
                "one period since the start date. Try an earlier --since date.",
            ],
        ),
        Error::Frozen(message) => explained(
            "The run is already frozen with different settings",
            &[
                &message,
                "A frozen run keeps its ref, start, timezone, and cutoff. Use another --name",
                "for different settings.",
            ],
        ),
        other => CommandError::Unknown {
            source: other.into(),
        },
    }
}

/// Advice for a scoring failure, or the plain error for anything unforeseen.
pub fn explain_scoring(error: qlty_slop_one::Error) -> CommandError {
    use qlty_slop_one::Error;
    match error {
        Error::MissingCredential(variable) if variable == JevProvider::OpenRouter.credential() => {
            explained(
                "No OpenRouter API key found",
                &[
                    "SlopOne uses Jev, from TypeSafe AI, through OpenRouter to read your code.",
                    &format!("Set {variable} in your environment, or create a key at"),
                    OPENROUTER_KEYS_URL,
                    "",
                    "Results already in the cache still work with --offline.",
                ],
            )
        }
        Error::MissingCredential(variable) => explained(
            "No Jev API key found",
            &[
                &format!(
                    "SlopOne uses Jev, from TypeSafe AI, to read your code. Set {variable} in your"
                ),
                &format!("environment, or sign up for access at {SIGNUP_URL}"),
                "",
                "Results already in the cache still work with --offline.",
            ],
        ),
        Error::BudgetExhausted => explained(
            "Spending limit reached",
            &[
                "Everything analyzed before the limit is cached, so running again with a higher",
                "--budget continues where this stopped.",
            ],
        ),
        Error::Offline => explained(
            "No cached analysis for this file",
            &["Run again without --offline to analyze it."],
        ),
        Error::Jev(message) => explained(
            "Jev request failed",
            &[
                &message,
                "",
                "Check your network connection and API key, or use --offline to work from",
                "cached results.",
            ],
        ),
        other => CommandError::Unknown {
            source: other.into(),
        },
    }
}

/// Reports an error in a pre-push hook as a warning and lets the push
/// continue: only a declined file blocks a push.
pub fn allow_push(error: CommandError) -> CommandSuccess {
    eprintln!(
        "{} {}",
        style("⚠").yellow().bold(),
        style("SlopOne could not check this push, so it does not block it.").bold()
    );
    match error {
        CommandError::Explained { title, detail } => {
            eprintln!("  {title}");
            for line in detail {
                eprintln!("  {line}");
            }
        }
        CommandError::Unknown { source } => eprintln!("  {source:#}"),
        other => eprintln!("  {other}"),
    }
    CommandSuccess::default()
}

fn explained(title: &str, detail: &[&str]) -> CommandError {
    CommandError::Explained {
        title: title.to_owned(),
        detail: detail.iter().map(|line| (*line).to_owned()).collect(),
    }
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.green} {prefix} {msg:.dim}")
        .expect("valid template")
        .tick_chars(TICK_CHARS)
}

fn scoring_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "  {spinner:.green} {prefix} {pos}/{len}  {bar:24.green/dim}  {percent:>3}%  {msg:.dim}",
    )
    .expect("valid template")
    .tick_chars(TICK_CHARS)
    .progress_chars("━╸░")
}

fn files_style() -> ProgressStyle {
    ProgressStyle::with_template("  {spinner:.green} {prefix}   {pos}/{len}   {msg:.dim}")
        .expect("valid template")
        .tick_chars(TICK_CHARS)
}

fn finished_line(label: &str, body: &str, elapsed: Duration) -> String {
    let mut line = format!("  {} {:<LABEL_WIDTH$} {body}", style("✔").green(), label);
    if !body.is_empty() {
        line.push_str("   ");
    }
    line.push_str(&style(duration(elapsed)).dim().to_string());
    line
}

fn spend(usage: &Usage) -> String {
    let mut text = format!("${:.2}", usage.cost_upper_bound_usd);
    if usage.budget_usd.is_finite() && usage.budget_usd < f64::MAX {
        text.push_str(&format!(" of ${:.2}", usage.budget_usd));
    }
    text
}

/// `1m 05s`, `12.3s`, `0.8s`.
pub fn duration(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs_f64();
    if seconds >= 3600.0 {
        format!(
            "{}h {:02}m",
            seconds as u64 / 3600,
            (seconds as u64 % 3600) / 60
        )
    } else if seconds >= 60.0 {
        format!("{}m {:02}s", seconds as u64 / 60, seconds as u64 % 60)
    } else {
        format!("{seconds:.1}s")
    }
}

/// `9,682`.
pub fn count(value: usize) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

fn plural(value: usize, word: &str) -> String {
    if value == 1 {
        word.to_owned()
    } else {
        format!("{word}s")
    }
}

fn signed(value: f64) -> String {
    let sign = if value > 0.0 {
        "+"
    } else if value < 0.0 {
        "−"
    } else {
        ""
    };
    format!("{sign}{:.3}", value.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_group_thousands() {
        assert_eq!(count(0), "0");
        assert_eq!(count(999), "999");
        assert_eq!(count(1000), "1,000");
        assert_eq!(count(9682), "9,682");
        assert_eq!(count(1_234_567), "1,234,567");
    }

    #[test]
    fn durations_read_naturally() {
        assert_eq!(duration(Duration::from_millis(800)), "0.8s");
        assert_eq!(duration(Duration::from_secs(65)), "1m 05s");
        assert_eq!(duration(Duration::from_secs(3725)), "1h 02m");
    }

    #[test]
    fn signed_values_use_a_true_minus() {
        assert_eq!(signed(0.0123), "+0.012");
        assert_eq!(signed(-0.5), "−0.500");
        assert_eq!(signed(0.0), "0.000");
    }

    #[test]
    fn estimate_scales_by_the_share_needing_analysis() {
        let start = Instant::now();
        let mut scoring = Scoring {
            total: 100,
            done: 20,
            cached: 10,
            analyzed: 10,
            ..Scoring::default()
        };
        for step in 0..10 {
            scoring.recent.push_back(start + Duration::from_secs(step));
        }
        let left = estimate(&scoring).unwrap().as_secs_f64();
        assert!((left - 40.0).abs() < 1e-9);
    }

    #[test]
    fn estimate_needs_two_analyzed_contents() {
        let mut scoring = Scoring {
            total: 10,
            done: 1,
            analyzed: 1,
            ..Scoring::default()
        };
        scoring.recent.push_back(Instant::now());
        assert!(estimate(&scoring).is_none());
    }

    #[test]
    fn sparkline_calls_a_rounded_zero_flat() {
        console::set_colors_enabled(false);
        let line = sparkline_line(&[Some(0.2), Some(-0.0002)]).unwrap();
        assert!(line.ends_with("this week net 0.000 —"), "{line}");
    }

    #[test]
    fn sparkline_needs_a_latest_week() {
        assert!(sparkline_line(&[]).is_none());
        assert!(sparkline_line(&[Some(0.1), None]).is_none());
    }

    #[test]
    fn sparkline_levels_sit_around_the_middle() {
        console::set_colors_enabled(false);
        let line = sparkline_line(&[Some(0.4), None, Some(-0.4), Some(0.0)]).unwrap();
        assert!(line.contains("█·▁▅"), "{line}");
        assert!(line.ends_with("this week net 0.000 —"), "{line}");
        assert!(line.starts_with("  Last 4 weeks"));
    }

    #[test]
    fn missing_credential_points_at_signup() {
        let error = explain_scoring(qlty_slop_one::Error::MissingCredential("TYPESAFE_API_KEY"));
        let CommandError::Explained { title, detail } = error else {
            panic!("expected an explained error");
        };
        assert_eq!(title, "No Jev API key found");
        assert!(detail.iter().any(|line| line.contains("TYPESAFE_API_KEY")));
        assert!(detail.iter().any(|line| line.contains(SIGNUP_URL)));
        assert!(!detail.iter().any(|line| line.contains("Vercel")));
    }

    #[test]
    fn missing_openrouter_credential_points_at_openrouter_keys() {
        let error = explain_scoring(qlty_slop_one::Error::MissingCredential(
            "OPENROUTER_API_KEY",
        ));
        let CommandError::Explained { title, detail } = error else {
            panic!("expected an explained error");
        };
        assert_eq!(title, "No OpenRouter API key found");
        assert!(detail
            .iter()
            .any(|line| line.contains("OPENROUTER_API_KEY")));
        assert!(detail.iter().any(|line| line.contains(OPENROUTER_KEYS_URL)));
        assert!(!detail.iter().any(|line| line.contains(SIGNUP_URL)));
    }

    #[test]
    fn unforeseen_errors_keep_their_message() {
        let error = explain_trends(qlty_slop_one_trends::Error::Render("boom".to_owned()));
        assert!(matches!(error, CommandError::Unknown { .. }));
        assert!(error.to_string().contains("Unknown"));
    }
}
