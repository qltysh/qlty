//! The HTML report and SVG chart, rendered from hand-built two-period exports
//! and, when `SLOPDETECT_DIR` points at a slopdetect checkout, compared with
//! the Python renderer's output for the frozen Interface run.

use std::fs;
use std::path::{Path, PathBuf};

use qlty_slop_one_trends::render::html::render;
use qlty_slop_one_trends::run::write_json_gz;
use regex::Regex;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/render")
        .join(name)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Lays out one run's exports the way the pipeline writes them, compressing
/// the companions and stamping the explanations with the export hashes.
fn stage(directory: &Path, name: &str) -> PathBuf {
    let data_path = directory.join(format!("{name}.json"));
    fs::copy(fixture(&format!("{name}.json")), &data_path).unwrap();
    let files: Value =
        serde_json::from_slice(&fs::read(fixture(&format!("{name}-files.json"))).unwrap()).unwrap();
    let files_path = directory.join(format!("{name}-files.json.gz"));
    write_json_gz(&files_path, &files).unwrap();
    let mut explanations: Value =
        serde_json::from_slice(&fs::read(fixture(&format!("{name}-explanations.json"))).unwrap())
            .unwrap();
    explanations["data_sha256"] = sha256_hex(&fs::read(&data_path).unwrap()).into();
    explanations["files_sha256"] = sha256_hex(&fs::read(&files_path).unwrap()).into();
    write_json_gz(
        &directory.join(format!("{name}-explanations.json.gz")),
        &explanations,
    )
    .unwrap();
    fs::copy(
        fixture(&format!("{name}-annotations.json")),
        directory.join(format!("{name}-annotations.json")),
    )
    .unwrap();
    data_path
}

struct Rendered {
    _directory: TempDir,
    html: String,
    svg: String,
}

fn render_example(with_monthly: bool) -> Rendered {
    let directory = tempfile::tempdir().unwrap();
    let weekly = stage(directory.path(), "example-weekly");
    let monthly = with_monthly.then(|| stage(directory.path(), "example-monthly"));
    let stem = directory.path().join("out").join("example");
    render(&weekly, &stem, monthly.as_deref()).unwrap();
    let html = fs::read_to_string(stem.with_extension("html")).unwrap();
    let svg = embedded_reports(&html)["week"]["standalone_svg"]
        .as_str()
        .unwrap()
        .to_owned();
    Rendered {
        html,
        svg,
        _directory: directory,
    }
}

/// The page without the base64 font payload, whose random letters would
/// otherwise match any short word.
fn without_font(html: &str) -> String {
    Regex::new(r"base64,[A-Za-z0-9+/=]+")
        .unwrap()
        .replace_all(html, "base64,FONT")
        .into_owned()
}

fn embedded_reports(html: &str) -> Value {
    let pattern =
        Regex::new(r#"<script type="application/json" id="reports-data">(.*?)</script>"#).unwrap();
    let json = &pattern.captures(html).unwrap()[1];
    serde_json::from_str(json).unwrap()
}

#[test]
fn writes_the_html_and_embeds_the_standalone_svg() {
    let rendered = render_example(false);
    assert!(rendered.html.starts_with("<!doctype html>"));
    assert!(rendered.svg.starts_with("<?xml version=\"1.0\""));
    assert!(rendered.svg.contains("viewBox=\"0 0 1152 648\""));
    let written: Vec<String> = fs::read_dir(rendered._directory.path().join("out"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(written, ["example.html"]);
    assert!(rendered.html.contains("id=\"export-svg\""));
}

#[test]
fn html_has_no_placeholder_residue() {
    let rendered = render_example(true);
    let placeholder = Regex::new(r"__[A-Z][A-Za-z_-]*__").unwrap();
    assert_eq!(placeholder.find(&rendered.html), None);
}

#[test]
fn html_has_no_mcp_or_deep_link_residue() {
    let rendered = render_example(true);
    let lowered = without_font(&rendered.html).to_lowercase();
    assert!(!lowered.contains("mcp"));
    assert!(!lowered.contains("claude.ai"));
    assert!(!lowered.contains("cursor://"));
    assert!(!lowered.contains("chatgpt.com"));
    assert!(!lowered.contains("slopdetect"));
    assert!(rendered.html.contains("analyzed by qlty slop-one"));
}

#[test]
fn html_embeds_the_font_and_icons() {
    let rendered = render_example(false);
    assert!(rendered.html.contains("base64,d09GMg"));
    assert!(rendered.html.contains("<template id=\"icon-wrench\">"));
    assert!(rendered.html.contains("<template id=\"icon-x-mark\">"));
    assert!(rendered
        .html
        .contains("<svg class=\"icon\" viewBox=\"0 0 16 16\""));
    assert!(!rendered.html.contains("<template id=\"icon-claude\">"));
}

#[test]
fn week_details_select_rows_and_refactor_from_a_floating_pill() {
    let rendered = render_example(false);
    assert!(rendered
        .html
        .contains("<th scope=\"col\" class=\"select\">"));
    assert_eq!(
        rendered
            .html
            .matches("data-select-all aria-label=\"Select all shown files\"")
            .count(),
        2
    );
    assert!(rendered.html.contains("id=\"week-refactor-bar\""));
    assert!(rendered.html.contains("id=\"week-refactor-count\""));
    assert!(!rendered
        .html
        .contains("class=\"details-nav\">\n        <div class=\"refactor-menu\">"));
    assert!(rendered.html.contains("files selected in the report"));
}

#[test]
fn licenses_cover_only_the_embedded_assets() {
    let rendered = render_example(false);
    assert!(rendered
        .html
        .contains("Inter 4.1 — https://github.com/rsms/inter/tree/v4.1"));
    assert!(rendered.html.contains("Heroicons 2.2.0"));
    assert!(rendered.html.contains("Lucide React 1.38.0"));
    assert!(!rendered.html.contains("Simple Icons"));
    assert!(!rendered.html.contains("LobeHub"));
}

#[test]
fn hit_targets_count_the_periods() {
    let rendered = render_example(true);
    let reports = embedded_reports(&rendered.html);
    let hits = Regex::new(r#"<rect class="week-hit""#).unwrap();
    assert_eq!(hits.find_iter(&rendered.html).count(), 2);
    for interval in ["week", "month"] {
        let report = &reports[interval];
        let svg = report["svg"].as_str().unwrap();
        assert_eq!(
            hits.find_iter(svg).count(),
            report["values"].as_array().unwrap().len()
        );
    }
}

#[test]
fn hit_target_labels_describe_each_period() {
    let rendered = render_example(false);
    assert!(rendered
        .html
        .contains("aria-label=\"Week ending September 13, 2026. No earlier week to compare.\""));
    assert!(rendered.html.contains(
        "aria-label=\"Week ending September 20, 2026. Improved +0.733, reduced -1.333, net -0.600.\""
    ));
}

#[test]
fn embedded_report_joins_values_details_and_annotations() {
    let rendered = render_example(false);
    let reports = embedded_reports(&rendered.html);
    let week = &reports["week"];
    let totals = &week["values"][1]["with_file_changes"];
    assert!((totals["net"].as_f64().unwrap() + 0.6).abs() < 1e-12);
    let details = &week["details"];
    assert_eq!(details["repository"], "https://example.com/owner/example");
    assert_eq!(details["interval"], "week");
    assert_eq!(details["threshold_score"], 5.0);
    assert_eq!(
        details["model"]["score_scale"],
        "threshold-centered-1-10-v1"
    );
    assert_eq!(details["model"]["measurement_policy"]["id"], "test-policy");
    assert_eq!(
        details["questions"]["boolean-logic"],
        "Is the Boolean logic hard to follow?"
    );
    assert_eq!(details.get("mcp"), None);
    let period = &details["periods"][1];
    assert_eq!(period["period_start"], "2026-09-14");
    assert_eq!(period["before_commit"], "c1".repeat(20));
    let row = &period["rows"][0];
    assert_eq!(row["kind"], "Changed");
    assert_eq!(row["diff_anchor"], sha256_hex(b"src/a.ts"));
    assert_eq!(row["explanation"]["score_change"], -2.0);
    assert_eq!(row["after"][0]["code_lines"], 144);
    assert_eq!(row["after"][0]["passed"], false);
    assert_eq!(row["before"][0]["language"], "TypeScript");
    assert_eq!(row["after"][0]["source_scope"]["policy"], "whole-file");
    assert_eq!(period["rows"][1]["kind"], "Added");
    assert_eq!(period["rows"][1]["explanation"]["mode"], "score");
    assert_eq!(period["rows"][2]["kind"], "Removed");
    let annotation = &week["annotations"][0];
    assert_eq!(annotation["period_index"], 1);
    assert_eq!(annotation["label"], "Week ending September 20, 2026");
    assert_eq!(annotation["title"], "Checkout refactor");
    let point = annotation["point"].as_array().unwrap();
    assert!(point[0].as_f64().unwrap() > 80.0 && point[0].as_f64().unwrap() < 1130.0);
    assert!(annotation["lower_edge"].as_f64().unwrap() > point[1].as_f64().unwrap());
    assert!(annotation["baseline"].as_f64().unwrap() > annotation["lower_edge"].as_f64().unwrap());
}

#[test]
fn single_interval_hides_the_picker() {
    let rendered = render_example(false);
    assert!(rendered
        .html
        .contains("<span class=\"select-control interval-control\" hidden>"));
    assert!(rendered
        .html
        .contains("<option value=\"week\" selected>Weekly</option></select>"));
    assert!(rendered.html.contains("let interval = 'week';"));
}

#[test]
fn combined_intervals_show_the_picker() {
    let rendered = render_example(true);
    assert!(rendered
        .html
        .contains("<span class=\"select-control interval-control\" >"));
    assert!(rendered.html.contains(
        "<option value=\"week\" selected>Weekly</option><option value=\"month\">Monthly</option>"
    ));
    let reports = embedded_reports(&rendered.html);
    assert_eq!(reports["month"]["values"][1]["label"], "September 2026");
    assert_eq!(reports["month"]["details"]["interval"], "month");
    assert!(reports["month"]["svg"]
        .as_str()
        .unwrap()
        .contains("Monthly code quality"));
}

#[test]
fn standalone_svg_carries_the_title_and_legend() {
    let rendered = render_example(false);
    assert!(rendered
        .svg
        .contains("<title>Example code quality changes</title>"));
    assert!(rendered
        .svg
        .contains(">Example code quality changes</text>"));
    assert!(rendered.svg.contains("Improved quality"));
    assert!(rendered.svg.contains("Net change (green + / red −)"));
    assert!(!rendered.svg.contains("week-hit"));
}

#[test]
fn escapes_json_for_inline_script() {
    let rendered = render_example(false);
    let start = rendered.html.find("id=\"reports-data\">").unwrap();
    let end = rendered.html[start..].find("</script>").unwrap();
    let json = &rendered.html[start..start + end];
    assert!(!json.contains('<'));
    assert!(json.contains("\\u003c"));
}

#[test]
fn rejects_explanations_from_other_inputs() {
    let directory = tempfile::tempdir().unwrap();
    let weekly = stage(directory.path(), "example-weekly");
    let mut data: Value = serde_json::from_slice(&fs::read(&weekly).unwrap()).unwrap();
    data["created_at"] = "2026-09-21T00:00:00+00:00".into();
    fs::write(&weekly, serde_json::to_vec(&data).unwrap()).unwrap();
    let error = render(&weekly, &directory.path().join("out"), None).unwrap_err();
    assert!(error
        .to_string()
        .contains("Score explanations do not match the report inputs"));
}

#[test]
fn rejects_an_annotation_on_the_baseline_period() {
    let directory = tempfile::tempdir().unwrap();
    let weekly = stage(directory.path(), "example-weekly");
    let annotations = directory.path().join("example-weekly-annotations.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&annotations).unwrap()).unwrap();
    value[0]["period_start"] = "2026-09-07".into();
    value[0]["commit"] = "c1".repeat(20).into();
    fs::write(&annotations, serde_json::to_vec(&value).unwrap()).unwrap();
    let error = render(&weekly, &directory.path().join("out"), None).unwrap_err();
    assert!(error.to_string().contains("measured change"));
}

#[test]
fn rejects_an_annotation_with_the_wrong_commit() {
    let directory = tempfile::tempdir().unwrap();
    let weekly = stage(directory.path(), "example-weekly");
    let annotations = directory.path().join("example-weekly-annotations.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&annotations).unwrap()).unwrap();
    value[0]["commit"] = "other".into();
    fs::write(&annotations, serde_json::to_vec(&value).unwrap()).unwrap();
    let error = render(&weekly, &directory.path().join("out"), None).unwrap_err();
    assert!(error
        .to_string()
        .contains("does not match the report snapshot"));
}

/// Every number as a float, so `4` and `4.0` compare equal.
fn normalized(value: &Value) -> Value {
    match value {
        Value::Number(number) => number.as_f64().map_or(Value::Null, Value::from),
        Value::Array(items) => Value::Array(items.iter().map(normalized).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, value)| (key.clone(), normalized(value)))
                .collect::<Map<_, _>>(),
        ),
        other => other.clone(),
    }
}

fn without_geometry(annotations: &Value) -> Value {
    let stripped = annotations
        .as_array()
        .unwrap()
        .iter()
        .map(|annotation| {
            let mut fields = annotation.as_object().unwrap().clone();
            for key in ["point", "lower_edge", "baseline"] {
                fields.remove(key);
            }
            Value::Object(fields)
        })
        .collect();
    Value::Array(stripped)
}

struct HitTarget {
    bounds: [f64; 4],
    label: String,
}

fn hit_targets(svg: &str) -> Vec<HitTarget> {
    let pattern = Regex::new(
        r#"<rect class="week-hit" data-week="\d+" x="([-\d.]+)" y="([-\d.]+)" width="([-\d.]+)" height="([-\d.]+)"[^>]*aria-label="([^"]*)""#,
    )
    .unwrap();
    pattern
        .captures_iter(svg)
        .map(|capture| HitTarget {
            bounds: [
                capture[1].parse().unwrap(),
                capture[2].parse().unwrap(),
                capture[3].parse().unwrap(),
                capture[4].parse().unwrap(),
            ],
            label: capture[5].to_owned(),
        })
        .collect()
}

fn numbers(svg: &str, pattern: &str) -> Vec<Vec<f64>> {
    Regex::new(pattern)
        .unwrap()
        .captures_iter(svg)
        .map(|capture| {
            (1..capture.len())
                .map(|index| capture[index].parse().unwrap())
                .collect()
        })
        .collect()
}

fn assert_within(actual: &[f64], expected: &[f64], tolerance: f64, what: &str) {
    assert_eq!(actual.len(), expected.len(), "{what}: count");
    for (actual, expected) in actual.iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{what}: {actual} vs {expected}"
        );
    }
}

/// Renders the frozen Interface run and compares the embedded report with
/// the Python renderer's. `SLOPDETECT_RENDER_OUTPUT` keeps the rendered
/// files for screenshot comparison instead of a temporary directory.
#[test]
#[ignore = "needs SLOPDETECT_DIR with the frozen Interface exports"]
fn matches_the_python_report_for_interface() {
    let Some(slopdetect) = std::env::var_os("SLOPDETECT_DIR") else {
        return;
    };
    let results = Path::new(&slopdetect).join(".ai/results");
    let output = tempfile::tempdir().unwrap();
    let output_dir = std::env::var_os("SLOPDETECT_RENDER_OUTPUT")
        .map_or_else(|| output.path().to_path_buf(), PathBuf::from);
    let stem = output_dir.join("interface-weekly-003");
    render(
        &results.join("interface-weekly-003.json"),
        &stem,
        Some(&results.join("interface-monthly-001.json")),
    )
    .unwrap();
    let ours = embedded_reports(&fs::read_to_string(stem.with_extension("html")).unwrap());
    let theirs =
        embedded_reports(&fs::read_to_string(results.join("interface-weekly-003.html")).unwrap());
    for interval in ["week", "month"] {
        let ours = &ours[interval];
        let mut theirs = theirs[interval].clone();
        theirs["details"].as_object_mut().unwrap().remove("mcp");
        assert_eq!(
            normalized(&ours["values"]),
            normalized(&theirs["values"]),
            "{interval} values"
        );
        assert_eq!(
            normalized(&ours["details"]),
            normalized(&theirs["details"]),
            "{interval} details"
        );
        assert_eq!(
            normalized(&without_geometry(&ours["annotations"])),
            normalized(&without_geometry(&theirs["annotations"])),
            "{interval} annotations"
        );
        for (ours, theirs) in ours["annotations"]
            .as_array()
            .unwrap()
            .iter()
            .zip(theirs["annotations"].as_array().unwrap())
        {
            let point = |value: &Value| -> Vec<f64> {
                value["point"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|number| number.as_f64().unwrap())
                    .collect()
            };
            assert_within(&point(ours), &point(theirs), 0.5, "annotation point");
            for key in ["lower_edge", "baseline"] {
                assert_within(
                    &[ours[key].as_f64().unwrap()],
                    &[theirs[key].as_f64().unwrap()],
                    0.5,
                    key,
                );
            }
        }
        let our_svg = ours["svg"].as_str().unwrap();
        let their_svg = theirs["svg"].as_str().unwrap();
        let our_targets = hit_targets(our_svg);
        let their_targets = hit_targets(their_svg);
        assert_eq!(our_targets.len(), their_targets.len());
        for (ours, theirs) in our_targets.iter().zip(&their_targets) {
            assert_within(&ours.bounds, &theirs.bounds, 0.5, "hit target bounds");
            assert_eq!(ours.label, theirs.label);
        }
        let our_markers: Vec<f64> = numbers(our_svg, r#"<circle cx="([-\d.]+)" cy="([-\d.]+)""#)
            .into_iter()
            .flatten()
            .collect();
        let their_markers: Vec<f64> = numbers(
            their_svg,
            r##"<use xlink:href="#[^"]+" x="([-\d.]+)" y="([-\d.]+)""##,
        )
        .into_iter()
        .flatten()
        .collect();
        assert_within(&our_markers, &their_markers, 0.5, "net markers");
        let our_gridlines: Vec<f64> = numbers(
            our_svg,
            r##"<path d="M [-\d.]+ ([-\d.]+) L [-\d.]+ [-\d.]+" fill="none" stroke="#e2ded8""##,
        )
        .into_iter()
        .flatten()
        .collect();
        let their_gridlines: Vec<f64> = numbers(
            their_svg,
            r##"<path d="M [-\d.]+ ([-\d.]+) L [-\d.]+ [-\d.]+ " clip-path="url\(#[^)]+\)" style="fill: none; stroke: #e2ded8"##,
        )
        .into_iter()
        .flatten()
        .collect();
        assert_within(&our_gridlines, &their_gridlines, 0.5, "gridlines");
        let our_labels: Vec<f64> = numbers(our_svg, r#"<text x="([-\d.]+)" y="519.318281""#)
            .into_iter()
            .flatten()
            .collect();
        let their_labels: Vec<f64> = numbers(
            their_svg,
            r#"<text style="[^"]*" x="([-\d.]+)" y="519.318281""#,
        )
        .into_iter()
        .flatten()
        .collect();
        assert_within(&our_labels, &their_labels, 0.5, "date label positions");
    }
}
