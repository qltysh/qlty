use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Position {
    line: u32,
    column: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ByteOffset {
    start: u32,
    end: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AstGrepRange {
    #[serde(rename = "byteOffset")]
    byte_offset: ByteOffset,
    start: Position,
    end: Position,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Label {
    text: String,
    range: AstGrepRange,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AstGrepIssue {
    text: String,
    range: AstGrepRange,
    file: String,
    lines: String,
    language: String,
    #[serde(rename = "ruleId")]
    rule_id: String,
    severity: String,
    note: Option<String>,
    message: String,
    labels: Vec<Label>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AstGrep {}

impl Parser for AstGrep {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let ast_grep_issues: Vec<AstGrepIssue> = serde_json::from_str(output)?;

        for ast_grep_issue in ast_grep_issues {
            let location = Some(Location {
                path: ast_grep_issue.file.clone(),
                range: Some(Range {
                    // ast-grep outputs 0-indexed line/column numbers,
                    // but qlty expects 1-indexed values.
                    start_line: ast_grep_issue.range.start.line + 1,
                    start_column: ast_grep_issue.range.start.column + 1,
                    end_line: ast_grep_issue.range.end.line + 1,
                    end_column: ast_grep_issue.range.end.column + 1,
                    ..Default::default()
                }),
            });

            // Generate a meaningful message if the original message is empty
            let message = if ast_grep_issue.message.is_empty() {
                format!("AST pattern match found: {}", ast_grep_issue.rule_id)
            } else {
                ast_grep_issue.message
            };

            // Map severity to Level
            let level = match ast_grep_issue.severity.as_str() {
                "error" => Level::High,
                "warning" => Level::Medium,
                "hint" | "info" => Level::Low,
                _ => Level::Medium, // Default to Medium
            };

            let issue = Issue {
                tool: plugin_name.into(),
                rule_key: ast_grep_issue.rule_id,
                message,
                category: Category::Lint.into(),
                level: level.into(),
                location,
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_line_numbers_are_one_indexed() {
        // ast-grep outputs 0-indexed line/column numbers.
        // qlty expects 1-indexed. Verify the parser converts correctly.
        let input = r#"[{
            "text": "argument :foo, String",
            "range": {
                "byteOffset": { "start": 0, "end": 20 },
                "start": { "line": 0, "column": 0 },
                "end": { "line": 0, "column": 20 }
            },
            "file": "test.rb",
            "lines": "argument :foo, String",
            "charCount": { "leading": 0, "trailing": 0 },
            "language": "Ruby",
            "metaVariables": { "single": {}, "multi": {}, "transformed": {} },
            "ruleId": "test-rule",
            "severity": "error",
            "note": null,
            "message": "test message",
            "labels": []
        }]"#;

        let issues = AstGrep::default().parse("ast-grep", input).unwrap();
        assert_eq!(issues.len(), 1);

        let range = issues[0].location.as_ref().unwrap().range.as_ref().unwrap();
        // Line 0 in ast-grep should become line 1 in qlty
        assert_eq!(range.start_line, 1, "start_line should be 1-indexed");
        assert_eq!(range.end_line, 1, "end_line should be 1-indexed");
        assert_eq!(range.start_column, 1, "start_column should be 1-indexed");
        assert_eq!(range.end_column, 21, "end_column should be 1-indexed");
    }

    #[test]
    fn parse() {
        let input = r###"
        [
  {
    "text": "Promise.all([await foo()])",
    "range": {
      "byteOffset": {
        "start": 25,
        "end": 51
      },
      "start": {
        "line": 2,
        "column": 0
      },
      "end": {
        "line": 2,
        "column": 26
      }
    },
    "file": "sample.ts",
    "lines": "Promise.all([await foo()]);",
    "charCount": {
      "leading": 0,
      "trailing": 1
    },
    "language": "TypeScript",
    "metaVariables": {
      "single": {
        "A": {
          "text": "[await foo()]",
          "range": {
            "byteOffset": {
              "start": 37,
              "end": 50
            },
            "start": {
              "line": 2,
              "column": 12
            },
            "end": {
              "line": 2,
              "column": 25
            }
          }
        }
      },
      "multi": {
        "secondary": [
          {
            "text": "await foo()",
            "range": {
              "byteOffset": {
                "start": 38,
                "end": 49
              },
              "start": {
                "line": 2,
                "column": 13
              },
              "end": {
                "line": 2,
                "column": 24
              }
            }
          }
        ]
      },
      "transformed": {}
    },
    "ruleId": "no-await-in-promise-all",
    "severity": "hint",
    "note": null,
    "message": "",
    "labels": [
      {
        "text": "await foo()",
        "range": {
          "byteOffset": {
            "start": 38,
            "end": 49
          },
          "start": {
            "line": 2,
            "column": 13
          },
          "end": {
            "line": 2,
            "column": 24
          }
        }
      }
    ]
  }
  ]
"###;
        let issues = AstGrep::default().parse("ast-grep", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: ast-grep
          ruleKey: no-await-in-promise-all
          message: "AST pattern match found: no-await-in-promise-all"
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: sample.ts
            range:
              startLine: 3
              startColumn: 1
              endLine: 3
              endColumn: 27
        "#);
    }
}
