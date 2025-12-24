use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Default)]
pub struct Mypy {}

#[derive(Debug, Deserialize)]
struct MypyMessage {
    file: String,
    line: u32,
    column: i32,
    message: String,
    code: Option<String>,
    severity: String,
}

impl Parser for Mypy {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        for raw_line in output.lines() {
            let raw_line = raw_line.trim();
            if raw_line.is_empty() {
                continue;
            }

            let message: MypyMessage = match serde_json::from_str(raw_line) {
                Ok(message) => message,
                Err(err) => {
                    warn!("Failed to parse mypy output ({}): {}", err, raw_line);
                    continue;
                }
            };

            let MypyMessage {
                file,
                line,
                column,
                message,
                code,
                severity,
            } = message;

            let start_column = normalize_column(column);
            let rule_key = match code.as_deref() {
                Some(code) if !code.trim().is_empty() => code.to_string(),
                _ => "mypy_issue".to_string(),
            };

            issues.push(Issue {
                tool: plugin_name.into(),
                message,
                category: Category::Lint.into(),
                level: severity_to_level(&severity).into(),
                rule_key,
                location: Some(Location {
                    path: file.clone(),
                    range: Some(Range {
                        start_line: line,
                        start_column,
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            });
        }

        Ok(issues)
    }
}

// Mypy columns in json output are 0-based, with -1 indicating no column.
// convert to 1-based, with a minimum of 1.
fn normalize_column(column: i32) -> u32 {
    if column > 0 {
        (column + 1) as u32
    } else {
        1
    }
}

fn severity_to_level(severity: &str) -> Level {
    match severity {
        "error" => Level::High,
        "warning" => Level::Medium,
        "note" => Level::Low,
        _ => Level::Low,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
{"file": "basic.in.py", "line": 1, "column": 0, "message": "Library stubs not installed for \"google.protobuf.descriptor_pb2\"", "hint": "Hint: \"python3 -m pip install types-protobuf\"\n(or run \"mypy --install-types\" to install all missing stub packages)\nSee https://mypy.readthedocs.io/en/stable/running_mypy.html#missing-imports", "code": "import-untyped", "severity": "error"}
{"file": "basic.in.py", "line": 1, "column": 0, "message": "Library stubs not installed for \"google.protobuf\"", "hint": null, "code": "import-untyped", "severity": "error"}
{"file": "basic.in.py", "line": 13, "column": 9, "message": "Argument 1 to \"greeting\" has incompatible type \"int\"; expected \"str\"", "hint": null, "code": "arg-type", "severity": "error"}
{"file": "basic.in.py", "line": 14, "column": 9, "message": "Argument 1 to \"greeting\" has incompatible type \"bytes\"; expected \"str\"", "hint": null, "code": "arg-type", "severity": "error"}
{"file": "basic.in.py", "line": 15, "column": 4, "message": "\"printer\" does not return a value (it only ever returns None)", "hint": null, "code": "func-returns-value", "severity": "error"}
{"file": "basic.in.py", "line": 16, "column": 9, "message": "Incompatible types in assignment (expression has type \"int\", variable has type \"str\")", "hint": null, "code": "assignment", "severity": "error"}
{"file": "basic.in.py", "line": 23, "column": -1, "message": "The return type of a generator function should be \"Generator\" or one of its supertypes", "hint": null, "code": "misc", "severity": "error"}
        "###;

        let issues = Mypy::default().parse("mypy", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: mypy
          ruleKey: import-untyped
          message: "Library stubs not installed for \"google.protobuf.descriptor_pb2\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 1
              startColumn: 1
        - tool: mypy
          ruleKey: import-untyped
          message: "Library stubs not installed for \"google.protobuf\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 1
              startColumn: 1
        - tool: mypy
          ruleKey: arg-type
          message: "Argument 1 to \"greeting\" has incompatible type \"int\"; expected \"str\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 13
              startColumn: 10
        - tool: mypy
          ruleKey: arg-type
          message: "Argument 1 to \"greeting\" has incompatible type \"bytes\"; expected \"str\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 14
              startColumn: 10
        - tool: mypy
          ruleKey: func-returns-value
          message: "\"printer\" does not return a value (it only ever returns None)"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 15
              startColumn: 5
        - tool: mypy
          ruleKey: assignment
          message: "Incompatible types in assignment (expression has type \"int\", variable has type \"str\")"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 16
              startColumn: 10
        - tool: mypy
          ruleKey: misc
          message: "The return type of a generator function should be \"Generator\" or one of its supertypes"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 23
              startColumn: 1
        "###);
    }
}
