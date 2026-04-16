use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct StylelintFile {
    pub source: String,
    pub warnings: Vec<StylelintMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StylelintMessage {
    pub line: Option<i32>,
    pub column: Option<i32>,
    #[serde(alias = "endLine")]
    pub end_line: Option<i32>,
    #[serde(alias = "endColumn")]
    pub end_column: Option<i32>,
    pub rule: Option<String>,
    pub severity: String,
    pub text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Stylelint {}

impl Parser for Stylelint {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let json = extract_json_array(output);
        let files: Vec<StylelintFile> = serde_json::from_str(json)?;

        for file in files {
            for message in file.warnings {
                let line = message.line.unwrap_or(1);
                let column = message.column.unwrap_or(1);

                let issue = Issue {
                    tool: "stylelint".into(),
                    message: message.text,
                    category: Category::Lint.into(),
                    level: severity_to_level(message.severity).into(),
                    rule_key: message.rule.unwrap_or_default(),
                    location: Some(Location {
                        path: file.source.clone(),
                        range: Some(Range {
                            start_line: line as u32,
                            start_column: column as u32,
                            end_line: message.end_line.unwrap_or(line) as u32,
                            end_column: message.end_column.unwrap_or(column) as u32,
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                };

                issues.push(issue);
            }
        }

        Ok(issues)
    }
}

// Node emits diagnostic warnings like `(node:XXX) ExperimentalWarning: ...`
// to stderr before stylelint's own output, and stylelint >=16 routes its
// JSON report to stderr. Skip leading non-JSON lines so the parser works
// regardless of Node's diagnostic chatter. We scan for the first `[`, which
// always begins stylelint's top-level array, and fall back to the full
// output if none is found so malformed input still surfaces as a parse error.
fn extract_json_array(output: &str) -> &str {
    match output.find('[') {
        Some(idx) => &output[idx..],
        None => output,
    }
}

fn severity_to_level(severity: String) -> Level {
    match severity.as_str() {
        // In eslint, issues come with a `fatal` attribute that we use to determine if the issue is Level::High or Level::Medium.
        // We don't have a `fatal` attribute here so we're considering all "error"s as Level::Medium and "warning"s as Level::Low.
        "warning" => Level::Low,
        "error" => Level::Medium,
        _ => Level::Low,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        [
          {
            "source": "/src/main.css",
            "deprecations": [],
            "invalidOptionWarnings": [],
            "parseErrors": [],
            "errored": true,
            "warnings": [
              {
                "line": 6,
                "column": 5,
                "endLine": 8,
                "endColumn": 2,
                "rule": "block-no-empty",
                "severity": "error",
                "text": "Unexpected empty block (block-no-empty)"
              },
              {
                "line": 15,
                "column": 1,
                "endLine": 15,
                "endColumn": 4,
                "rule": "no-duplicate-selectors",
                "severity": "error",
                "text": "Unexpected duplicate selector \"div\", first used at line 6 (no-duplicate-selectors)"
              }
            ]
          },
          {
            "source": "/src/other.css",
            "deprecations": [],
            "invalidOptionWarnings": [],
            "parseErrors": [],
            "errored": true,
            "warnings": [
              {
                "line": 1,
                "column": 1,
                "endLine": 1,
                "endColumn": 4,
                "rule": "selector-type-no-unknown",
                "severity": "error",
                "text": "Unexpected unknown type selector \"aaa\" (selector-type-no-unknown)"
              }
            ]
          }
        ]
        "###;

        let issues = Stylelint::default().parse("stylelint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: stylelint
          ruleKey: block-no-empty
          message: Unexpected empty block (block-no-empty)
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /src/main.css
            range:
              startLine: 6
              startColumn: 5
              endLine: 8
              endColumn: 2
        - tool: stylelint
          ruleKey: no-duplicate-selectors
          message: "Unexpected duplicate selector \"div\", first used at line 6 (no-duplicate-selectors)"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /src/main.css
            range:
              startLine: 15
              startColumn: 1
              endLine: 15
              endColumn: 4
        - tool: stylelint
          ruleKey: selector-type-no-unknown
          message: "Unexpected unknown type selector \"aaa\" (selector-type-no-unknown)"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /src/other.css
            range:
              startLine: 1
              startColumn: 1
              endLine: 1
              endColumn: 4
        "#);
    }

    #[test]
    fn parse_strips_node_warning_prefix() {
        let input = "(node:543) ExperimentalWarning: Importing JSON modules is an experimental feature\n(Use `node --trace-warnings ...` to show where the warning was created)\n[{\"source\":\"/src/main.css\",\"warnings\":[{\"line\":1,\"column\":3,\"endLine\":1,\"endColumn\":5,\"rule\":\"block-no-empty\",\"severity\":\"error\",\"text\":\"Unexpected empty block (block-no-empty)\"}]}]";

        let issues = Stylelint::default().parse("stylelint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: stylelint
          ruleKey: block-no-empty
          message: Unexpected empty block (block-no-empty)
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: /src/main.css
            range:
              startLine: 1
              startColumn: 3
              endLine: 1
              endColumn: 5
        "#);
    }
}
