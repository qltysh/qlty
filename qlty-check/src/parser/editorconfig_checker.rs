// [
//   {
//     "check_name": "editorconfig-checker",
//     "description": "Trailing whitespace",
//     "fingerprint": "451537eb8214cc2a82277e294151a4e1",
//     "severity": "minor",
//     "location": {
//       "path": "test.py",
//       "lines": { "begin": 2, "end": 0 }
//     }
//   }
// ]

use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CodeClimateIssue {
    description: String,
    severity: String,
    location: CodeClimateLocation,
}

#[derive(Debug, Deserialize)]
struct CodeClimateLocation {
    path: String,
    lines: CodeClimateLines,
}

#[derive(Debug, Deserialize)]
struct CodeClimateLines {
    begin: i64,
    #[serde(default)]
    end: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct EditorconfigChecker {}

impl Parser for EditorconfigChecker {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let messages: Vec<CodeClimateIssue> = serde_json::from_str(output)?;

        for message in messages {
            let rule_key = derive_rule_key(&message.description);
            // editorconfig-checker's codeclimate formatter sets `end` to
            // AdditionalIdenticalErrorCount (a relative count) rather than
            // an absolute line number, so we compute: begin + end.
            // The tool may emit -1 for unknown locations; clamp to 0.
            let start_line = message.location.lines.begin.max(0) as u32;
            let end_line = if message.location.lines.end > 0 {
                (message.location.lines.begin + message.location.lines.end).max(0) as u32
            } else {
                start_line
            };

            let issue = Issue {
                tool: "editorconfig-checker".into(),
                message: message.description,
                category: Category::Lint.into(),
                level: severity_to_level(&message.severity).into(),
                rule_key,
                location: Some(Location {
                    path: message.location.path,
                    range: Some(Range {
                        start_line,
                        end_line,
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

fn derive_rule_key(description: &str) -> String {
    if description.starts_with("Trailing whitespace") {
        "trim_trailing_whitespace".into()
    } else if description.starts_with("Wrong indent style") {
        "indent_style".into()
    } else if description.starts_with("Wrong amount of left-padding") {
        "indent_size".into()
    } else if description.starts_with("Final newline expected")
        || description.starts_with("No final newline expected")
    {
        "insert_final_newline".into()
    } else if description.starts_with("Line too long") {
        "max_line_length".into()
    } else if description.starts_with("Wrong line endings")
        || description.starts_with("Not all lines have the correct end of line")
    {
        "end_of_line".into()
    } else if description.starts_with("Wrong character encoding") {
        "charset".into()
    } else {
        "unknown".into()
    }
}

fn severity_to_level(severity: &str) -> Level {
    match severity {
        "blocker" | "critical" => Level::High,
        "major" => Level::Medium,
        "info" | "minor" => Level::Low,
        _ => Level::Medium,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_negative_line_number() {
        let input = r###"
[{"check_name":"editorconfig-checker","description":"Wrong character encoding","fingerprint":"abc123","severity":"minor","location":{"path":"foo.h","lines":{"begin":-1,"end":0}}}]
        "###;

        let issues = EditorconfigChecker::default().parse("editorconfig-checker", input);
        assert!(issues.is_ok(), "should not fail on -1 line number");
        let issues = issues.unwrap();
        assert_eq!(issues.len(), 1);
        let range = issues[0].location.as_ref().unwrap().range.as_ref().unwrap();
        assert_eq!(range.start_line, 0);
        assert_eq!(range.end_line, 0);
    }

    #[test]
    fn parse() {
        let input = r###"
[{"check_name":"editorconfig-checker","description":"Trailing whitespace","fingerprint":"451537eb8214cc2a82277e294151a4e1","severity":"minor","location":{"path":"test.py","lines":{"begin":2,"end":0}}},{"check_name":"editorconfig-checker","description":"Wrong amount of left-padding spaces(want multiple of 4)","fingerprint":"e901227647c3654fdacb6620554b5828","severity":"minor","location":{"path":"test.py","lines":{"begin":2,"end":0}}},{"check_name":"editorconfig-checker","description":"Wrong indent style found (tabs instead of spaces)","fingerprint":"36189515c2ba2d76f277d94c5a36c6d4","severity":"minor","location":{"path":"test.py","lines":{"begin":3,"end":1}}}]
        "###;

        let issues = EditorconfigChecker::default().parse("editorconfig-checker", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: editorconfig-checker
          ruleKey: trim_trailing_whitespace
          message: Trailing whitespace
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: test.py
            range:
              startLine: 2
              endLine: 2
        - tool: editorconfig-checker
          ruleKey: indent_size
          message: Wrong amount of left-padding spaces(want multiple of 4)
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: test.py
            range:
              startLine: 2
              endLine: 2
        - tool: editorconfig-checker
          ruleKey: indent_style
          message: Wrong indent style found (tabs instead of spaces)
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: test.py
            range:
              startLine: 3
              endLine: 4
        "###);
    }
}
