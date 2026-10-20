use indoc::indoc;
use jsonschema::error::ValidationErrorKind;
use serde_json::Value;
use std::{fs, path::Path, process::Command};

const SARIF_SCHEMA: &str = include_str!("schemas/sarif-schema-2.1.0.json");

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".qlty")).unwrap();
    fs::write(
        root.join(".gitignore"),
        ".qlty/logs\n.qlty/out\n.qlty/results\n.qlty/sources\n.qlty/tmp\n",
    )
    .unwrap();
    fs::write(
        root.join("sample.py"),
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
    )
    .unwrap();
    fs::write(
        root.join("ruff-output.json"),
        indoc! {r#"
            [
              {
                "code": "F401",
                "filename": "sample.py",
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
            script = "echo sample.py:4 LINE: issue reported with a line number only"
            success_codes = [0]
            output = "stdout"
            output_format = "regex"
            output_regex = '((?P<path>.*):(?P<line>-?\d+) (?P<code>\S+): (?P<message>.+))'
            output_level = "medium"

            [plugins.definitions.fixable-linter]
            file_types = ["python"]
            known_good_version = "1.0.0"

            [plugins.definitions.fixable-linter.drivers.lint]
            script = "cat ruff-output.json"
            success_codes = [0]
            output = "stdout"
            output_format = "ruff"

            [[plugin]]
            name = "line-linter"
            version = "1.0.0"

            [[plugin]]
            name = "fixable-linter"
            version = "1.0.0"
        "#},
    )
    .unwrap();
}

fn sarif_output(args: &[&str]) -> Value {
    let project = tempfile::tempdir().unwrap();
    write_project(project.path());
    let _repository = qlty_test_utilities::git::init(project.path());

    let output = Command::new(env!("CARGO_BIN_EXE_qlty"))
        .current_dir(project.path())
        .env("QLTY_TELEMETRY", "off")
        .args(args)
        .output()
        .unwrap();

    serde_json::from_slice(&output.stdout).unwrap()
}

fn sarif_schema_errors(sarif: &Value) -> Vec<String> {
    let schema: Value = serde_json::from_str(SARIF_SCHEMA).unwrap();
    let validator = jsonschema::options()
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
    let sarif = sarif_output(&[
        "check",
        "--all",
        "--sarif",
        "--no-fail",
        "--no-cache",
        "--no-progress",
        "--no-upgrade-check",
    ]);

    assert_eq!(sarif_schema_errors(&sarif), Vec::<String>::new());
}

#[test]
fn smells_sarif_output_is_valid_sarif() {
    let sarif = sarif_output(&["smells", "--all", "--sarif", "--no-upgrade-check"]);

    assert_eq!(sarif_schema_errors(&sarif), Vec::<String>::new());
}
