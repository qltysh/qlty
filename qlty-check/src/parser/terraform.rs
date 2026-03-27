use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize)]
struct DiagnosticRange {
    filename: String,
    start: DiagnosticPosition,
    end: DiagnosticPosition,
}
#[derive(Debug, Deserialize)]
struct DiagnosticPosition {
    line: u32,
    column: u32,
}
#[derive(Debug, Deserialize)]
struct Diagnostic {
    severity: String,
    summary: String,
    range: Option<DiagnosticRange>,
}
#[derive(Debug, Deserialize)]
struct TerraformOutput {
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Terraform {}

impl Parser for Terraform {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let parsed: TerraformOutput = serde_json::from_str(output)?;
        let mut issues = vec![];
        for diag in parsed.diagnostics {
            let location = if let Some(range) = diag.range {
                Some(Location {
                    path: range.filename,
                    range: Some(Range {
                        start_line: range.start.line,
                        start_column: range.start.column,
                        end_line: range.end.line,
                        end_column: range.end.column,
                        ..Default::default()
                    }),
                })
            } else {
                None
            };

            issues.push(Issue {
                tool: "terraform".to_string(),
                message: diag.summary,
                category: Category::Lint.into(),
                level: severity_to_level(&diag.severity).into(),
                location,
                ..Default::default()
            });
        }
        Ok(issues)
    }
}

fn severity_to_level(severity: &str) -> Level {
    match severity {
        "error" => Level::High,
        "warning" => Level::Medium,
        "info" => Level::Low,
        _ => Level::Medium,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
          {
    "format_version": "1.0",
    "valid": false,
    "error_count": 1,
    "warning_count": 0,
    "diagnostics": [
      {
        "severity": "error",
        "summary": "Invalid quoted type constraints",
        "detail": "Terraform 0.11 and earlier required type constraints to be given in quotes, but that form is now deprecated and will be removed in a future version of Terraform. Remove the quotes around \"map\" and write map(string) instead to explicitly indicate that the map elements are strings.",
        "range": {
          "filename": "aws.in.tf",
          "start": {
            "line": 2,
            "column": 10,
            "byte": 40
          },
          "end": {
            "line": 2,
            "column": 15,
            "byte": 45
          }
        },
        "snippet": {
          "context": "variable \"ssl_certificates\"",
          "code": "  type = \"map\"",
          "start_line": 2,
          "highlight_start_offset": 9,
          "highlight_end_offset": 14,
          "values": []
        }
      }
    ]
  }
        "###;

        let issues = Terraform::default().parse("terraform", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: terraform
          message: Invalid quoted type constraints
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: aws.in.tf
            range:
              startLine: 2
              startColumn: 10
              endLine: 2
              endColumn: 15
        "###);
    }

    #[test]
    fn parse_missing_location() {
        let input = r###"
          {
            "format_version": "1.0",
            "valid": false,
            "error_count": 2,
            "warning_count": 0,
            "diagnostics": [
              {
                "severity": "error",
                "summary": "Missing required provider",
                "detail": "This configuration requires provider registry.terraform.io/hashicorp/aws, but that provider isn't available. You may be able to install it automatically by running:\n  terraform init"
              },
              {
                "severity": "error",
                "summary": "Missing required provider",
                "detail": "This configuration requires provider registry.terraform.io/hashicorp/random, but that provider isn't available. You may be able to install it automatically by running:\n  terraform init"
              }
            ]
          }
        "###;

        let issues = Terraform::default().parse("terraform", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: terraform
          message: Missing required provider
          level: LEVEL_HIGH
          category: CATEGORY_LINT
        - tool: terraform
          message: Missing required provider
          level: LEVEL_HIGH
          category: CATEGORY_LINT
        "###);
    }
}
