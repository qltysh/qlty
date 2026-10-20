use indoc::indoc;
use jsonschema::{error::ValidationErrorKind, Draft};
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command, sync::OnceLock};

const SARIF_SCHEMA: &str = include_str!("schemas/sarif-schema-2.1.0.json");

const DUPLICATED_FUNCTION: &str = indoc! {r#"
    def total_price(items, tax_rate):
        subtotal = 0
        for item in items:
            if item.quantity > 0:
                subtotal += item.price * item.quantity
            else:
                subtotal -= item.refund
        if subtotal > 100:
            discount = subtotal * 0.1
        else:
            discount = 0
        tax = (subtotal - discount) * tax_rate
        shipping = 5 if subtotal < 50 else 0
        total = subtotal - discount + tax + shipping
        if total < 0:
            total = 0
        return round(total, 2)
"#};

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".qlty")).unwrap();
    fs::write(
        root.join(".gitignore"),
        ".qlty/logs\n.qlty/out\n.qlty/results\n.qlty/sources\n.qlty/tmp\n",
    )
    .unwrap();
    fs::write(
        root.join("sample file.py"),
        format!(
            "{}\n\n{}",
            indoc! {r#"
                import os


                def classify(value):
                    if value == 1:
                        return "one"
                    if value == 2:
                        return "two"
                    if value == 3:
                        return "three"
                    if value == 4:
                        return "four"
                    if value == 5:
                        return "five"
                    return "many"
            "#},
            DUPLICATED_FUNCTION
        ),
    )
    .unwrap();
    fs::write(root.join("copy.py"), DUPLICATED_FUNCTION).unwrap();
    fs::write(
        root.join("ruff-output.json"),
        indoc! {r#"
            [
              {
                "code": "F401",
                "filename": "sample file.py",
                "location": { "row": 1, "column": 1 },
                "end_location": { "row": 1, "column": 10 },
                "message": "`os` imported but unused",
                "url": "https://docs.astral.sh/ruff/rules/unused-import",
                "fix": {
                  "applicability": "safe",
                  "message": "Remove unused import: `os`",
                  "edits": [
                    {
                      "content": "",
                      "location": { "row": 1, "column": 1 },
                      "end_location": { "row": 2, "column": 1 }
                    }
                  ]
                }
              }
            ]
        "#},
    )
    .unwrap();
    fs::write(
        root.join(".qlty/qlty.toml"),
        indoc! {r#"
            config_version = "0"

            [plugins.definitions.line-linter]
            file_types = ["python"]
            known_good_version = "1.0.0"

            [plugins.definitions.line-linter.drivers.lint]
            script = "echo sample file.py:4 LINE: issue reported with a line number only"
            success_codes = [0]
            output = "stdout"
            output_format = "regex"
            output_regex = '((?P<path>.*):(?P<line>-?\d+) (?P<code>\S+): (?P<message>.+))'
            output_level = "medium"
            batch = true

            [plugins.definitions.fixable-linter]
            file_types = ["python"]
            known_good_version = "1.0.0"

            [plugins.definitions.fixable-linter.drivers.lint]
            prepare_script = "mkdir ${linter} && echo @type %1 > ${linter}/cat.cmd || echo @type %1 > ${linter}/cat.cmd"
            script = "cat ruff-output.json"
            success_codes = [0]
            output = "stdout"
            output_format = "ruff"
            batch = true

            [plugins.definitions.pass-fail]
            file_types = ["python"]
            known_good_version = "1.0.0"

            [plugins.definitions.pass-fail.drivers.lint]
            script = "false"
            success_codes = [0]
            output = "pass_fail"
            batch = true

            [[plugin]]
            name = "line-linter"
            version = "1.0.0"

            [[plugin]]
            name = "fixable-linter"
            version = "1.0.0"

            [[plugin]]
            name = "pass-fail"
            version = "1.0.0"
        "#},
    )
    .unwrap();
}

fn run_sarif(args: &[&str]) -> Value {
    let project = tempfile::tempdir().unwrap();
    write_project(project.path());
    let _repository = qlty_test_utilities::git::init(project.path());

    let output = Command::new(env!("CARGO_BIN_EXE_qlty"))
        .current_dir(project.path())
        .env("QLTY_TELEMETRY", "off")
        .args(args)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn check_sarif() -> &'static Value {
    static SARIF: OnceLock<Value> = OnceLock::new();
    SARIF.get_or_init(|| {
        run_sarif(&[
            "check",
            "--all",
            "--sarif",
            "--no-fail",
            "--no-cache",
            "--no-progress",
            "--no-upgrade-check",
        ])
    })
}

fn smells_sarif() -> &'static Value {
    static SARIF: OnceLock<Value> = OnceLock::new();
    SARIF.get_or_init(|| run_sarif(&["smells", "--all", "--sarif", "--no-upgrade-check"]))
}

fn results(sarif: &Value) -> &Vec<Value> {
    sarif["runs"][0]["results"].as_array().unwrap()
}

fn result_for_rule<'a>(sarif: &'a Value, rule_id: &str) -> &'a Value {
    results(sarif)
        .iter()
        .find(|result| result["ruleId"] == rule_id)
        .unwrap()
}

fn sarif_schema_errors(sarif: &Value) -> Vec<String> {
    let schema: Value = serde_json::from_str(SARIF_SCHEMA).unwrap();
    // The schema declares draft-04, which has no uri-reference format, so it
    // is validated as draft-07 to check artifactLocation.uri as well.
    let validator = jsonschema::options()
        .with_draft(Draft::Draft7)
        .should_validate_formats(true)
        .build(&schema)
        .unwrap();

    validator
        .iter_errors(sarif)
        .flat_map(|error| match error.kind() {
            ValidationErrorKind::AdditionalProperties { unexpected } => unexpected
                .iter()
                .map(|property| {
                    format!(
                        "{}: additional property {property:?} is not allowed",
                        error.instance_path()
                    )
                })
                .collect(),
            _ => vec![format!("{}: {error}", error.instance_path())],
        })
        .collect()
}

#[test]
fn check_sarif_output_is_valid_sarif() {
    assert_eq!(sarif_schema_errors(check_sarif()), Vec::<String>::new());
}

#[test]
fn check_sarif_output_has_a_result_for_each_located_issue() {
    let mut rule_ids = results(check_sarif())
        .iter()
        .map(|result| result["ruleId"].as_str().unwrap())
        .collect::<Vec<_>>();
    rule_ids.sort();

    assert_eq!(rule_ids, ["line-linter:LINE", "ruff:F401"]);
}

#[test]
fn check_sarif_output_locates_line_only_issue() {
    let result = result_for_rule(check_sarif(), "line-linter:LINE");

    assert_eq!(
        result["locations"][0]["physicalLocation"],
        json!({
            "artifactLocation": {"uri": "sample%20file.py"},
            "region": {"startLine": 4}
        })
    );
}

#[test]
fn check_sarif_output_leaves_out_fixes() {
    let result = result_for_rule(check_sarif(), "ruff:F401");

    assert_eq!(result.get("fixes"), None);
}

#[test]
fn check_sarif_output_reports_issue_without_location_as_notification() {
    let texts = check_sarif()["runs"][0]["invocations"][0]["toolExecutionNotifications"]
        .as_array()
        .unwrap()
        .iter()
        .map(|notification| notification["message"]["text"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(texts.contains(&"pass-fail failed (pass-fail:fail)"));
}

#[test]
fn smells_sarif_output_is_valid_sarif() {
    assert_eq!(sarif_schema_errors(smells_sarif()), Vec::<String>::new());
}

#[test]
fn smells_sarif_output_links_duplicated_code() {
    let result = results(smells_sarif())
        .iter()
        .find(|result| {
            result["ruleId"] == "qlty:identical-code"
                && result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    == "copy.py"
        })
        .unwrap();

    assert_eq!(
        result["relatedLocations"][0]["physicalLocation"]["artifactLocation"],
        json!({"uri": "sample%20file.py"})
    );
}
