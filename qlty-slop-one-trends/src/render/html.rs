//! The standalone HTML report: joins one interval's exports into the
//! embedded `reports` JSON, draws the chart, and fills the template with the
//! carried-over styles, scripts, font, and icons so the file works offline.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use regex::{Captures, Regex};
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::chart_data::{chart_periods, drilldown_periods};
use crate::error::{Error, Result};
use crate::periods::Interval;
use crate::render::escape_html;
use crate::render::svg::{chart, Chart, Target};
use crate::report::{
    Annotation, ChartPeriod, DrilldownPeriod, ExplanationsExport, FileRow, FilesExport, Member,
    ReportData,
};
use crate::run::read_json;

const TEMPLATE: &str = include_str!("../../assets/report.html");
const STYLES: &str = include_str!("../../assets/report.css");
const REFACTOR_PROMPT_SCRIPT: &str = include_str!("../../assets/refactor_prompt.js");
const ANNOTATIONS_MARKUP: &str = include_str!("../../assets/annotations.html");
const ANNOTATIONS_SCRIPT: &str = include_str!("../../assets/annotations.js");
const CHART_OPTIONS_MARKUP: &str = include_str!("../../assets/chart_options.html");
const CHART_OPTIONS_SCRIPT: &str = include_str!("../../assets/chart_options.js");
const INTER_FONT: &[u8] = include_bytes!("../../assets/inter-4.1/InterVariable.woff2");
const INTER_LICENSE: &str = include_str!("../../assets/inter-4.1/LICENSE.txt");
const HEROICONS_LICENSE: &str = include_str!("../../assets/icons/heroicons-2.2.0/LICENSE");
const LUCIDE_LICENSE: &str = include_str!("../../assets/icons/lucide-1.38.0/LICENSE");

const HEROICONS: [(&str, &str); 10] = [
    (
        "arrow-down",
        include_str!("../../assets/icons/heroicons-2.2.0/arrow-down.svg"),
    ),
    (
        "arrow-up",
        include_str!("../../assets/icons/heroicons-2.2.0/arrow-up.svg"),
    ),
    (
        "arrow-up-right",
        include_str!("../../assets/icons/heroicons-2.2.0/arrow-up-right.svg"),
    ),
    (
        "arrows-up-down",
        include_str!("../../assets/icons/heroicons-2.2.0/arrows-up-down.svg"),
    ),
    (
        "chevron-down",
        include_str!("../../assets/icons/heroicons-2.2.0/chevron-down.svg"),
    ),
    (
        "chevron-left",
        include_str!("../../assets/icons/heroicons-2.2.0/chevron-left.svg"),
    ),
    (
        "chevron-right",
        include_str!("../../assets/icons/heroicons-2.2.0/chevron-right.svg"),
    ),
    (
        "ellipsis-horizontal",
        include_str!("../../assets/icons/heroicons-2.2.0/ellipsis-horizontal.svg"),
    ),
    (
        "magnifying-glass",
        include_str!("../../assets/icons/heroicons-2.2.0/magnifying-glass.svg"),
    ),
    (
        "x-mark",
        include_str!("../../assets/icons/heroicons-2.2.0/x-mark.svg"),
    ),
];

const LUCIDE_ICONS: [(&str, &str); 6] = [
    (
        "book-open",
        include_str!("../../assets/icons/lucide-1.38.0/book-open.svg"),
    ),
    (
        "check",
        include_str!("../../assets/icons/lucide-1.38.0/check.svg"),
    ),
    (
        "copy",
        include_str!("../../assets/icons/lucide-1.38.0/copy.svg"),
    ),
    (
        "info",
        include_str!("../../assets/icons/lucide-1.38.0/info.svg"),
    ),
    (
        "terminal",
        include_str!("../../assets/icons/lucide-1.38.0/terminal.svg"),
    ),
    (
        "wrench",
        include_str!("../../assets/icons/lucide-1.38.0/wrench.svg"),
    ),
];

const SVG_NAMESPACE: &str = " xmlns=\"http://www.w3.org/2000/svg\"";

/// One interval's data as the page embeds it under `reports[interval]`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub values: Vec<ChartPeriod>,
    pub details: Details,
    pub svg: String,
    pub annotations: Vec<Value>,
    /// The chart with its title and legend, which the page offers as a
    /// download.
    pub standalone_svg: String,
}

/// The drilldown data and the scoring context the page and prompts read.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Details {
    pub repository: String,
    pub interval: Interval,
    pub threshold_score: f64,
    pub model: Map<String, Value>,
    pub questions: BTreeMap<String, String>,
    pub periods: Vec<DrilldownPeriod>,
}

/// The page heading: the project name, capitalized unless it carries its
/// own punctuation, followed by "code quality changes".
pub fn report_title(data: &ReportData) -> String {
    let project = data.project();
    let name = if project.chars().any(is_punctuation) {
        project.to_owned()
    } else {
        capitalize(project)
    };
    format!("{name} code quality changes")
}

/// Unicode punctuation (general category P*) for the scripts project names
/// use in practice: ASCII, Latin-1, and the General Punctuation block.
fn is_punctuation(character: char) -> bool {
    matches!(
        character,
        '!' | '"' | '#' | '%' | '&' | '\'' | '(' | ')' | '*' | ',' | '-' | '.' | '/' | ':' | ';'
            | '?' | '@' | '[' | '\\' | ']' | '_' | '{' | '}' | '¡' | '§' | '«' | '¶' | '·' | '»'
            | '¿' | '\u{2010}'..='\u{2027}' | '\u{2030}'..='\u{205e}'
    )
}

fn capitalize(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        Some(first) => first
            .to_uppercase()
            .chain(characters.flat_map(char::to_lowercase))
            .collect(),
        None => String::new(),
    }
}

/// Joins one interval's saved scores, changes, and verified explanations
/// into the report the page embeds. `data_path` names `<name>.json`; the
/// companion exports sit beside it.
pub fn load_report(data_path: &Path, data: &ReportData) -> Result<Report> {
    let inputs = Inputs::new(data_path)?;
    let files_bytes = fs::read(inputs.path("-files.json.gz"))?;
    let files = FilesExport::from_value(decompress_json(&files_bytes)?)?;
    let values = chart_periods(data, &files)?;
    if values.is_empty() {
        return Err(Error::Render(
            "The report has no snapshots to render".to_owned(),
        ));
    }
    let mut periods = drilldown_periods(data, &files)?;
    let explanations_bytes = fs::read(inputs.path("-explanations.json.gz"))?;
    let explanations = ExplanationsExport::from_value(decompress_json(&explanations_bytes)?)?;
    let data_bytes = fs::read(data_path)?;
    verify_explanations(&explanations, data, &data_bytes, &files_bytes)?;
    join_explanations(&mut periods, &explanations, &files)?;

    let chart = chart(data, &values)?;
    let annotations = load_annotations(&inputs, data, &values, &chart.targets)?;
    let details = Details {
        repository: data.repository().to_owned(),
        interval: data.interval,
        threshold_score: data.threshold_score(),
        model: details_model(data)?,
        questions: explanations.questions,
        periods,
    };
    let Chart {
        svg,
        standalone_svg,
        ..
    } = chart;
    Ok(Report {
        values,
        details,
        svg,
        annotations,
        standalone_svg,
    })
}

/// The export directory and run name a data path identifies.
struct Inputs {
    directory: PathBuf,
    name: String,
}

impl Inputs {
    fn new(data_path: &Path) -> Result<Self> {
        let name = data_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| {
                Error::Render(format!("report path {} has no name", data_path.display()))
            })?;
        Ok(Self {
            directory: data_path.parent().unwrap_or(Path::new("")).to_path_buf(),
            name: name.to_owned(),
        })
    }

    fn path(&self, suffix: &str) -> PathBuf {
        self.directory.join(format!("{}{suffix}", self.name))
    }
}

fn decompress_json(bytes: &[u8]) -> Result<Value> {
    let mut text = String::new();
    GzDecoder::new(bytes).read_to_string(&mut text)?;
    Ok(serde_json::from_str(&text)?)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Explanations are only shown when they were computed from exactly these
/// exports with this model.
fn verify_explanations(
    explanations: &ExplanationsExport,
    data: &ReportData,
    data_bytes: &[u8],
    files_bytes: &[u8],
) -> Result<()> {
    let model = data.model();
    let matches = explanations.interval == data.interval
        && explanations.schema_version == 3
        && model.get("score_scale").and_then(Value::as_str)
            == Some(explanations.score_scale.as_str())
        && model.get("sha256").and_then(Value::as_str) == Some(explanations.model_sha256.as_str())
        && explanations.data_sha256 == sha256_hex(data_bytes)
        && explanations.files_sha256 == sha256_hex(files_bytes);
    if matches {
        Ok(())
    } else {
        Err(Error::Mismatch(
            "Score explanations do not match the report inputs; regenerate them first".to_owned(),
        ))
    }
}

/// Attaches each row's explanation, diff anchor, and the members' saved
/// measurements, and records each period's start date.
fn join_explanations(
    periods: &mut [DrilldownPeriod],
    explanations: &ExplanationsExport,
    files: &FilesExport,
) -> Result<()> {
    if periods.len() != explanations.periods.len() || periods.len() != files.periods.len() {
        return Err(Error::Mismatch(
            "Score explanations cover a different number of periods".to_owned(),
        ));
    }
    let mut previous_files: HashMap<&str, &FileRow> = HashMap::new();
    for ((period, explained), snapshot) in periods
        .iter_mut()
        .zip(&explanations.periods)
        .zip(&files.periods)
    {
        if period.commit != explained.commit {
            return Err(Error::Mismatch(
                "Score explanations use different period commits".to_owned(),
            ));
        }
        if period.rows.len() != explained.rows.len() {
            return Err(Error::Mismatch(
                "Score explanations cover a different number of rows".to_owned(),
            ));
        }
        period.period_start = Some(snapshot.start().to_owned());
        let current_files: HashMap<&str, &FileRow> = snapshot
            .files
            .iter()
            .flatten()
            .map(|file| (file.path.as_str(), file))
            .collect();
        for (row, explanation) in period.rows.iter_mut().zip(&explained.rows) {
            row.diff_anchor = Some(sha256_hex(row.primary_path().as_bytes()));
            row.explanation = Some(match explanation {
                Some(explanation) => serde_json::to_value(explanation)?,
                None => Value::Null,
            });
            for member in &mut row.before {
                complete_member(member, &previous_files)?;
            }
            for member in &mut row.after {
                complete_member(member, &current_files)?;
            }
        }
        previous_files = current_files;
    }
    Ok(())
}

fn complete_member(member: &mut Member, files: &HashMap<&str, &FileRow>) -> Result<()> {
    let saved = files.get(member.path.as_str()).ok_or_else(|| {
        Error::Mismatch(format!("the file export has no row for {}", member.path))
    })?;
    member.passed = saved.passed;
    member.language = saved.language.clone();
    member.code_lines = saved.code_lines;
    member.cyclomatic = saved.cyclomatic;
    member.source_scope = saved.source_scope.clone();
    Ok(())
}

fn details_model(data: &ReportData) -> Result<Map<String, Value>> {
    let model = data.model();
    let mut details = Map::new();
    for key in ["id", "sha256", "score_meaning", "measurement_policy"] {
        let value = model
            .get(key)
            .ok_or_else(|| Error::Render(format!("the report model has no {key}")))?;
        details.insert(key.to_owned(), value.clone());
    }
    details.insert(
        "score_scale".to_owned(),
        model.get("score_scale").cloned().unwrap_or(Value::Null),
    );
    Ok(details)
}

/// Reads the optional `<name>-annotations.json`, checks each annotation
/// against its snapshot, and anchors it to the chart geometry.
fn load_annotations(
    inputs: &Inputs,
    data: &ReportData,
    values: &[ChartPeriod],
    targets: &[Target],
) -> Result<Vec<Value>> {
    let path = inputs.path("-annotations.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let annotations: Vec<Annotation> = read_json(&path)?;
    annotations
        .into_iter()
        .map(|annotation| {
            let index = data
                .periods
                .iter()
                .position(|period| period.start() == annotation.period_start)
                .filter(|&index| data.periods[index].commit == annotation.commit)
                .ok_or_else(|| {
                    Error::Mismatch(
                        "Chart annotation does not match the report snapshot".to_owned(),
                    )
                })?;
            let (Some(point), Some(lower_edge)) =
                (targets[index].net_point, targets[index].lower_edge)
            else {
                return Err(Error::Mismatch(
                    "Chart annotations need a period with a measured change".to_owned(),
                ));
            };
            let mut value = serde_json::to_value(&annotation)?;
            let Some(object) = value.as_object_mut() else {
                return Err(Error::Render("annotation is not an object".to_owned()));
            };
            object.insert("period_index".to_owned(), index.into());
            object.insert("label".to_owned(), values[index].label.clone().into());
            object.insert("point".to_owned(), serde_json::to_value(point)?);
            object.insert("lower_edge".to_owned(), lower_edge.into());
            object.insert("baseline".to_owned(), targets[index].baseline.into());
            Ok(value)
        })
        .collect()
}

/// A weekly report and its monthly companion must describe the same project,
/// scoring policy, exclusions, and cutoff.
pub fn validate_intervals(weekly: &ReportData, monthly: &ReportData) -> Result<()> {
    if weekly.interval != Interval::Week || monthly.interval != Interval::Month {
        return Err(Error::Mismatch(
            "a combined report needs weekly data and a monthly companion".to_owned(),
        ));
    }
    let (Some(last_week), Some(last_month)) = (weekly.periods.last(), monthly.periods.last())
    else {
        return Err(Error::Mismatch(
            "Both intervals must contain snapshots".to_owned(),
        ));
    };
    for key in [
        "project",
        "repository",
        "timezone",
        "model",
        "measurement_identity",
        "qlty",
        "source_exclusion_policy",
        "main_commit",
    ] {
        if weekly.metadata.get(key) != monthly.metadata.get(key) {
            return Err(Error::Mismatch(format!(
                "Weekly and monthly reports have different {key}"
            )));
        }
    }
    if last_week.as_of != last_month.as_of {
        return Err(Error::Mismatch(
            "Weekly and monthly reports have different snapshot cutoffs".to_owned(),
        ));
    }
    Ok(())
}

/// Renders `<stem>.html` from `<name>.json` and its companions, adding the
/// interval picker when a monthly export is given. Returns the first
/// report's chart periods, the values the chart draws.
pub fn render(
    data_path: &Path,
    output_stem: &Path,
    monthly_data_path: Option<&Path>,
) -> Result<Vec<ChartPeriod>> {
    let data = ReportData::from_value(read_json(data_path)?)?;
    let mut reports = vec![(data.interval, load_report(data_path, &data)?)];
    if let Some(monthly_path) = monthly_data_path {
        let monthly = ReportData::from_value(read_json(monthly_path)?)?;
        validate_intervals(&data, &monthly)?;
        reports.push((monthly.interval, load_report(monthly_path, &monthly)?));
    }
    let markup = report_html(&data, &reports)?;
    if let Some(parent) = output_stem
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(with_suffix(output_stem, ".html"), markup)?;
    let (_, first) = reports.swap_remove(0);
    Ok(first.values)
}

fn with_suffix(stem: &Path, suffix: &str) -> PathBuf {
    let mut name = stem.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Fills the template with the reports, the first of which opens initially.
pub fn report_html(data: &ReportData, reports: &[(Interval, Report)]) -> Result<String> {
    let Some((initial, first)) = reports.first() else {
        return Err(Error::Render("no reports to render".to_owned()));
    };
    let options: String = [(Interval::Week, "Weekly"), (Interval::Month, "Monthly")]
        .into_iter()
        .filter(|(interval, _)| reports.iter().any(|(present, _)| present == interval))
        .map(|(interval, label)| {
            let selected = if interval == *initial {
                " selected"
            } else {
                ""
            };
            format!(
                "<option value=\"{}\"{selected}>{label}</option>",
                interval.as_str()
            )
        })
        .collect();
    let mut embedded = Map::new();
    for (interval, report) in reports {
        embedded.insert(interval.as_str().to_owned(), serde_json::to_value(report)?);
    }
    let reports_json = serde_json::to_string(&embedded)?.replace('<', "\\u003c");
    let markup = TEMPLATE
        .replace("__REPORT_TITLE__", &escape_html(&report_title(data)))
        .replace("__CHART_SVG__", &first.svg)
        .replace("__INITIAL_INTERVAL__", initial.as_str())
        .replace(
            "__INTERVAL_PICKER_HIDDEN__",
            if reports.len() == 1 { "hidden" } else { "" },
        )
        .replace("__INTERVAL_OPTIONS__", &options)
        .replace("__REPORTS__", &reports_json);
    embed_assets(markup)
}

/// Embeds the pinned, licensed assets so the report remains one offline file.
fn embed_assets(markup: String) -> Result<String> {
    let markup = markup
        .replace("__CHART_OPTIONS__", CHART_OPTIONS_MARKUP)
        .replace("__CHART_OPTIONS_SCRIPT__", CHART_OPTIONS_SCRIPT)
        .replace("__ANNOTATION_POPOVERS__", ANNOTATIONS_MARKUP)
        .replace("__ANNOTATION_SCRIPT__", ANNOTATIONS_SCRIPT)
        .replace(
            "__STYLES__",
            &STYLES.replace("__INTER_FONT__", &base64(INTER_FONT)),
        )
        .replace("__REFACTOR_PROMPT_SCRIPT__", REFACTOR_PROMPT_SCRIPT);
    let icons = icons();
    let placeholder =
        Regex::new(r"__ICON_([a-z-]+)__").map_err(|err| Error::Render(err.to_string()))?;
    for capture in placeholder.captures_iter(&markup) {
        let name = &capture[1];
        if !icons.iter().any(|(known, _)| *known == name) {
            return Err(Error::Render(format!("unknown icon {name}")));
        }
    }
    let markup = placeholder.replace_all(&markup, |capture: &Captures| {
        icons
            .iter()
            .find(|(name, _)| *name == &capture[1])
            .map(|(_, svg)| svg.clone())
            .unwrap_or_default()
    });
    let templates: String = icons
        .iter()
        .map(|(name, svg)| format!("<template id=\"icon-{name}\">{svg}</template>"))
        .collect();
    let licenses = format!(
        "Inter 4.1 — https://github.com/rsms/inter/tree/v4.1\n\n{INTER_LICENSE}\n\n\
         Heroicons 2.2.0 — https://github.com/tailwindlabs/heroicons/tree/v2.2.0\n\n{HEROICONS_LICENSE}\n\n\
         Lucide React 1.38.0 — https://lucide.dev\n\n{LUCIDE_LICENSE}"
    );
    Ok(markup
        .replace("__ICON_TEMPLATES__", &templates)
        .replace("__LICENSES__", &licenses))
}

/// The inline icons, trimmed to inherit color from the page.
fn icons() -> Vec<(&'static str, String)> {
    let heroicons = HEROICONS.iter().map(|(name, svg)| {
        let svg = svg
            .replace(SVG_NAMESPACE, "")
            .replace(" fill=\"currentColor\"", "")
            .replacen("<svg ", "<svg class=\"icon\" ", 1);
        (*name, svg)
    });
    let lucide = LUCIDE_ICONS.iter().map(|(name, svg)| {
        let svg = svg
            .replace(SVG_NAMESPACE, "")
            .replace(" stroke=\"currentColor\"", "")
            .replacen("class=\"lucide ", "class=\"icon lucide ", 1);
        (*name, svg)
    });
    heroicons.chain(lucide).collect()
}

/// Standard base64 with padding, for the embedded font.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut triple = [0_u8; 3];
        triple[..chunk.len()].copy_from_slice(chunk);
        let bits = u32::from_be_bytes([0, triple[0], triple[1], triple[2]]);
        for position in 0..4 {
            if position <= chunk.len() {
                let sextet = (bits >> (18 - 6 * position)) & 63;
                encoded.push(ALPHABET[sextet as usize] as char);
            } else {
                encoded.push('=');
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report_data(interval: Interval, project: &str) -> ReportData {
        let key = interval.records_key();
        ReportData::from_value(json!({
            "interval": interval.as_str(),
            "project": project,
            "repository": "https://example.com/project",
            "timezone": "America/New_York",
            "model": {"sha256": "model"},
            "measurement_identity": "measurements",
            "qlty": {"sha256": "binary"},
            "source_exclusion_policy": {"id": "source-only"},
            "main_commit": "tip",
            key: [period("2026-09-14", interval)],
        }))
        .unwrap()
    }

    fn period(start: &str, interval: Interval) -> Value {
        let (start_key, end_key) = match interval {
            Interval::Week => ("week_start", "week_end_exclusive"),
            Interval::Month => ("period_start", "period_end_exclusive"),
        };
        json!({
            start_key: start,
            end_key: "2026-09-21",
            "as_of": "2026-09-20T00:00:00+00:00",
            "partial": false,
            "commit": "tip",
            "committed_at": "2026-09-19T00:00:00+00:00",
            "first_parent_index": 0,
            "summary": {
                "candidate_files": 0, "files": 0, "fails": 0, "errors": 0,
                "inline_only_skips": 0, "loc": 0, "mass": 0.0,
                "maintainable_fraction": null, "median": null, "loc_weighted_mean": null
            },
            "test_paths_excluded": 0,
            "analysis_errors": [],
            "change": null
        })
    }

    #[test]
    fn capitalizes_a_plain_project_name() {
        assert_eq!(
            report_title(&report_data(Interval::Week, "interface")),
            "Interface code quality changes"
        );
    }

    #[test]
    fn lowercases_the_rest_of_a_plain_project_name() {
        assert_eq!(
            report_title(&report_data(Interval::Week, "qltySH")),
            "Qltysh code quality changes"
        );
    }

    #[test]
    fn keeps_a_punctuated_project_name() {
        assert_eq!(
            report_title(&report_data(Interval::Week, "my-app")),
            "my-app code quality changes"
        );
    }

    #[test]
    fn keeps_a_project_name_with_unicode_punctuation() {
        assert_eq!(
            report_title(&report_data(Interval::Week, "app—two")),
            "app—two code quality changes"
        );
    }

    #[test]
    fn matching_reports_pass_interval_validation() {
        let weekly = report_data(Interval::Week, "example");
        let monthly = report_data(Interval::Month, "example");
        validate_intervals(&weekly, &monthly).unwrap();
    }

    #[test]
    fn rejects_a_companion_with_a_different_model() {
        let weekly = report_data(Interval::Week, "example");
        let mut monthly = report_data(Interval::Month, "example");
        monthly
            .metadata
            .insert("model".to_owned(), json!({"sha256": "other"}));
        let error = validate_intervals(&weekly, &monthly).unwrap_err();
        assert!(error.to_string().contains("model"));
    }

    #[test]
    fn rejects_a_companion_with_a_different_cutoff() {
        let weekly = report_data(Interval::Week, "example");
        let mut monthly = report_data(Interval::Month, "example");
        monthly.periods[0].as_of = "2026-09-21T00:00:00+00:00".to_owned();
        let error = validate_intervals(&weekly, &monthly).unwrap_err();
        assert!(error.to_string().contains("cutoff"));
    }

    #[test]
    fn rejects_swapped_intervals() {
        let weekly = report_data(Interval::Week, "example");
        let monthly = report_data(Interval::Month, "example");
        let error = validate_intervals(&monthly, &weekly).unwrap_err();
        assert!(error.to_string().contains("monthly companion"));
    }

    #[test]
    fn rejects_a_companion_without_snapshots() {
        let weekly = report_data(Interval::Week, "example");
        let mut monthly = report_data(Interval::Month, "example");
        monthly.periods.clear();
        let error = validate_intervals(&weekly, &monthly).unwrap_err();
        assert!(error.to_string().contains("contain snapshots"));
    }

    #[test]
    fn encodes_base64_with_padding() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn icons_inherit_the_page_color() {
        let icons = icons();
        let (_, arrow) = icons
            .iter()
            .find(|(name, _)| *name == "arrow-down")
            .unwrap();
        assert!(arrow.starts_with("<svg class=\"icon\" viewBox"));
        assert!(!arrow.contains("currentColor"));
        let (_, copy) = icons.iter().find(|(name, _)| *name == "copy").unwrap();
        assert!(copy.contains("class=\"icon lucide lucide-copy\""));
        assert!(!copy.contains("stroke=\"currentColor\""));
    }

    #[test]
    fn rejects_an_unknown_icon_placeholder() {
        let error = embed_assets("__ICON_missing__".to_owned()).unwrap_err();
        assert!(error.to_string().contains("missing"));
    }
}
