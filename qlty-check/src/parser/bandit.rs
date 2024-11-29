use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BanditOutput {
    pub results: Vec<BanditResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanditResult {
    pub col_offset: u32,
    pub end_col_offset: u32,
    pub filename: String,
    pub issue_severity: String,
    pub issue_text: String,
    pub line_number: u32,
    pub line_range: Vec<u32>,
    pub more_info: String,
    pub test_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Bandit {}

impl Parser for Bandit {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let bandit_output: BanditOutput = serde_json::from_str(output)?;

        for result in bandit_output.results {
            let issue = Issue {
                tool: "bandit".into(),
                message: result.issue_text,
                category: Category::DependencyAlert.into(),
                level: severity_to_level(result.issue_severity).into(),
                rule_key: result.test_id,
                documentation_url: result.more_info,
                location: Some(Location {
                    path: result.filename,
                    range: Some(Range {
                        start_line: result.line_number,
                        start_column: result.col_offset,
                        end_line: *result.line_range.last().unwrap_or(&result.line_number),
                        end_column: result.end_col_offset,
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

fn severity_to_level(severity: String) -> Level {
    match severity.as_str() {
        "low" => Level::Low,
        "medium" => Level::Medium,
        "high" => Level::High,
        _ => Level::High,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {
          "errors": [],
          "generated_at": "2024-04-22T22:20:34Z",
          "metrics": {
            "./basic.in.py": {
              "CONFIDENCE.HIGH": 2,
              "CONFIDENCE.LOW": 0,
              "CONFIDENCE.MEDIUM": 0,
              "CONFIDENCE.UNDEFINED": 0,
              "SEVERITY.HIGH": 0,
              "SEVERITY.LOW": 1,
              "SEVERITY.MEDIUM": 1,
              "SEVERITY.UNDEFINED": 0,
              "loc": 6,
              "nosec": 0,
              "skipped_tests": 0
            },
            "_totals": {
              "CONFIDENCE.HIGH": 2,
              "CONFIDENCE.LOW": 0,
              "CONFIDENCE.MEDIUM": 0,
              "CONFIDENCE.UNDEFINED": 0,
              "SEVERITY.HIGH": 0,
              "SEVERITY.LOW": 1,
              "SEVERITY.MEDIUM": 1,
              "SEVERITY.UNDEFINED": 0,
              "loc": 6,
              "nosec": 0,
              "skipped_tests": 0
            }
          },
          "results": [
            {
              "code": "1 import dill\n2 import StringIO\n3 \n",
              "col_offset": 0,
              "end_col_offset": 11,
              "filename": "./basic.in.py",
              "issue_confidence": "HIGH",
              "issue_cwe": {
                "id": 502,
                "link": "https://cwe.mitre.org/data/definitions/502.html"
              },
              "issue_severity": "LOW",
              "issue_text": "Consider possible security implications associated with dill module.",
              "line_number": 1,
              "line_range": [
                1
              ],
              "more_info": "https://bandit.readthedocs.io/en/1.7.8/blacklists/blacklist_imports.html#b403-import-pickle",
              "test_id": "B403",
              "test_name": "blacklist"
            },
            {
              "code": "5 pick = dill.dumps({\"a\": \"b\", \"c\": \"d\"})\n6 print(dill.loads(pick))\n7 \n",
              "col_offset": 6,
              "end_col_offset": 22,
              "filename": "./basic.in.py",
              "issue_confidence": "HIGH",
              "issue_cwe": {
                "id": 502,
                "link": "https://cwe.mitre.org/data/definitions/502.html"
              },
              "issue_severity": "MEDIUM",
              "issue_text": "Pickle and modules that wrap it can be unsafe when used to deserialize untrusted data, possible security issue.",
              "line_number": 6,
              "line_range": [
                6
              ],
              "more_info": "https://bandit.readthedocs.io/en/1.7.8/blacklists/blacklist_calls.html#b301-pickle",
              "test_id": "B301",
              "test_name": "blacklist"
            }
          ]
        }
        "###;

        let issues = Bandit::default().parse("bandit", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r###"
        - tool: bandit
          ruleKey: B403
          message: Consider possible security implications associated with dill module.
          level: LEVEL_HIGH
          category: CATEGORY_DEPENDENCY_ALERT
          documentationUrl: "https://bandit.readthedocs.io/en/1.7.8/blacklists/blacklist_imports.html#b403-import-pickle"
          location:
            path: "./basic.in.py"
            range:
              startLine: 1
              endLine: 1
              endColumn: 11
        - tool: bandit
          ruleKey: B301
          message: "Pickle and modules that wrap it can be unsafe when used to deserialize untrusted data, possible security issue."
          level: LEVEL_HIGH
          category: CATEGORY_DEPENDENCY_ALERT
          documentationUrl: "https://bandit.readthedocs.io/en/1.7.8/blacklists/blacklist_calls.html#b301-pickle"
          location:
            path: "./basic.in.py"
            range:
              startLine: 6
              startColumn: 6
              endLine: 6
              endColumn: 22
        "###);
    }
}
