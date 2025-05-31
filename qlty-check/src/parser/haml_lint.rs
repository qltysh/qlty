use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::Deserialize;

pub struct HamlLint {}

#[derive(Debug, Deserialize, Clone)]
struct HamlLintOutput {
    pub files: Vec<HamlLintFiles>,
}

#[derive(Debug, Deserialize, Clone)]
struct HamlLintFiles {
    pub path: String,
    pub offenses: Vec<HamlLintOffense>,
}

#[derive(Debug, Deserialize, Clone)]
struct HamlLintOffense {
    pub severity: String,
    pub message: String,
    pub location: HamlLintLocation,
    pub linter_name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct HamlLintLocation {
    pub line: u32,
}

impl Parser for HamlLint {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let haml_lint_output: HamlLintOutput = serde_json::from_str(output)?;

        for file in haml_lint_output.files {
            for offense in file.offenses {
                let level = match offense.severity.as_str() {
                    "error" => Level::High,
                    "warning" => Level::Medium,
                    _ => Level::Medium,
                };

                let issue = Issue {
                    tool: plugin_name.into(),
                    message: offense.message,
                    category: Category::Lint.into(),
                    level: level.into(),
                    rule_key: offense.linter_name,
                    location: Some(Location {
                        path: file.path.clone(),
                        range: Some(Range {
                            start_line: offense.location.line,
                            end_line: offense.location.line,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
{
  "metadata": {
    "haml_lint_version": "0.61.1",
    "ruby_engine": "ruby",
    "ruby_patchlevel": "123",
    "ruby_platform": "arm64-darwin23"
  },
  "files": [
    {
      "path": "basic.in.haml",
      "offenses": [
        {
          "severity": "warning",
          "message": "Empty script should be removed",
          "location": { "line": 5 },
          "linter_name": "EmptyScript"
        },
        {
          "severity": "warning",
          "message": "`id` attribute must be in lisp-case",
          "location": { "line": 2 },
          "linter_name": "IdNames"
        },
        {
          "severity": "error",
          "message": "Couldn't process the file for linting. ArgumentError: comparison of Integer with nil failed",
          "location": { "line": 0 },
          "linter_name": "RuboCop"
        },
        {
          "severity": "warning",
          "message": "The - symbol should have one space separating it from code",
          "location": { "line": 5 },
          "linter_name": "SpaceBeforeScript"
        }
      ]
    }
  ],
  "summary": {
    "offense_count": 4,
    "target_file_count": 1,
    "inspected_file_count": 1
  }
}

        "###;

        let issues = HamlLint {}.parse("HamlLint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: HamlLint
          ruleKey: EmptyScript
          message: Empty script should be removed
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.haml
            range:
              startLine: 5
              endLine: 5
        - tool: HamlLint
          ruleKey: IdNames
          message: "`id` attribute must be in lisp-case"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.haml
            range:
              startLine: 2
              endLine: 2
        - tool: HamlLint
          ruleKey: RuboCop
          message: "Couldn't process the file for linting. ArgumentError: comparison of Integer with nil failed"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.haml
            range: {}
        - tool: HamlLint
          ruleKey: SpaceBeforeScript
          message: The - symbol should have one space separating it from code
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.haml
            range:
              startLine: 5
              endLine: 5
        "###);
    }
}
