// qlty-ignore: +ripgrep
use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RipgrepIssue {
    #[serde(rename = "type")]
    rg_type: String,
    #[serde(default)]
    data: RipgrepData,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct RipgrepData {
    path: Option<Path>,
    line_number: Option<u32>,
    #[serde(default)]
    submatches: Vec<Submatch>,
    lines: Option<Lines>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Path {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Lines {
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Submatch {
    #[serde(rename = "match")]
    rg_match: Match,
    #[serde(default)]
    start: u32,
    #[serde(default)]
    end: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Match {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Ripgrep {}

impl Parser for Ripgrep {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        for ripgrep_output in output.trim().lines() {
            let result: Result<RipgrepIssue, _> = serde_json::from_str(ripgrep_output);
            if let Err(e) = result {
                warn!("Failed to parse ripgrep output ({}): {}", e, ripgrep_output);
                continue;
            }

            let ripgrep_issue = result?;
            if ripgrep_issue.rg_type != "match" {
                continue;
            }
            let lines = match ripgrep_issue.data.lines {
                Some(lines) => lines,
                None => {
                    warn!("Failed to parse lines: {}", ripgrep_output);
                    continue;
                }
            };
            let line_number = match ripgrep_issue.data.line_number {
                Some(line_number) => line_number,
                None => {
                    warn!("Failed to parse line number: {}", ripgrep_output);
                    continue;
                }
            };
            let path = match &ripgrep_issue.data.path {
                Some(path) => path,
                None => {
                    warn!("Failed to parse path: {}", ripgrep_output);
                    continue;
                }
            };
            issues.extend(ripgrep_issue.data.submatches.iter().flat_map(|submatch| {
                lines.text.iter().flat_map(|text| {
                    Some(Issue {
                        tool: "ripgrep".into(),
                        message: text.trim().to_string(),
                        category: Category::Lint.into(),
                        level: Level::Note.into(),
                        rule_key: submatch.rg_match.text.clone(),
                        location: Some(Location {
                            path: path.text.clone(),
                            range: Some(Range {
                                start_line: line_number,
                                start_column: submatch.start + 1, // submatch uses 1-based indexing
                                end_line: line_number,
                                end_column: submatch.end + 1, // submatch uses 1-based indexing
                                ..Default::default()
                            }),
                        }),
                        ..Default::default()
                    })
                })
            }));
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {"type":"begin","data":{"path":{"text":"basic_e.in.rs"}}}
        {"type":"match","data":{"path":{"text":"basic_e.in.rs"},"lines":{"text":"    // FIXME TODO\n"},"line_number":2,"absolute_offset":12,"submatches":[{"match":{"text":"FIXME"},"start":7,"end":12},{"match":{"text":"TODO"},"start":13,"end":17}]}}
        {"type":"end","data":{"path":{"text":"basic_e.in.rs"},"binary_offset":null,"stats":{"elapsed":{"secs":0,"nanos":117084,"human":"0.000117s"},"searches":1,"searches_with_match":1,"bytes_searched":147,"bytes_printed":299,"matched_lines":1,"matches":2}}}
        {"type":"begin","data":{"path":{"text":"basic.in.rs"}}}
        {"type":"match","data":{"path":{"text":"basic.in.rs"},"lines":{"text":"    // NOTE\n"},"line_number":2,"absolute_offset":12,"submatches":[{"match":{"text":"NOTE"},"start":7,"end":11}]}}
        {"type":"match","data":{"path":{"text":"basic.in.rs"},"lines":{"text":"    // FIXME TODO\n"},"line_number":3,"absolute_offset":24,"submatches":[{"match":{"text":"FIXME"},"start":7,"end":12},{"match":{"text":"TODO"},"start":13,"end":17}]}}
        {"type":"match","data":{"path":{"text":"basic.in.rs"},"lines":{"text":"    // HACK\n"},"line_number":4,"absolute_offset":42,"submatches":[{"match":{"text":"HACK"},"start":7,"end":11}]}}
        {"type":"end","data":{"path":{"text":"basic.in.rs"},"binary_offset":null,"stats":{"elapsed":{"secs":0,"nanos":193250,"human":"0.000193s"},"searches":1,"searches_with_match":1,"bytes_searched":171,"bytes_printed":667,"matched_lines":3,"matches":4}}}
        {"data":{"elapsed_total":{"human":"0.003794s","nanos":3794125,"secs":0},"stats":{"bytes_printed":966,"bytes_searched":318,"elapsed":{"human":"0.000310s","nanos":310334,"secs":0},"matched_lines":4,"matches":6,"searches":2,"searches_with_match":2}},"type":"summary"}
        "###;

        let issues = Ripgrep::default().parse("ripgrep", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r"
        - tool: ripgrep
          ruleKey: FIXME
          message: // FIXME TODO
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic_e.in.rs
            range:
              startLine: 2
              startColumn: 8
              endLine: 2
              endColumn: 13
        - tool: ripgrep
          ruleKey: TODO
          message: // FIXME TODO
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic_e.in.rs
            range:
              startLine: 2
              startColumn: 14
              endLine: 2
              endColumn: 18
        - tool: ripgrep
          ruleKey: NOTE
          message: // NOTE
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic.in.rs
            range:
              startLine: 2
              startColumn: 8
              endLine: 2
              endColumn: 12
        - tool: ripgrep
          ruleKey: FIXME
          message: // FIXME TODO
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic.in.rs
            range:
              startLine: 3
              startColumn: 8
              endLine: 3
              endColumn: 13
        - tool: ripgrep
          ruleKey: TODO
          message: // FIXME TODO
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic.in.rs
            range:
              startLine: 3
              startColumn: 14
              endLine: 3
              endColumn: 18
        - tool: ripgrep
          ruleKey: HACK
          message: // HACK
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic.in.rs
            range:
              startLine: 4
              startColumn: 8
              endLine: 4
              endColumn: 12
        ");
    }

    #[test]
    #[traced_test]
    fn parse_missing_fields() {
        let input = r###"
        {INVALID_JSON}
        {"type":"match"}
        {"type":"match","data":{"lines":{"text":"// FIXME\n"}}}
        {"type":"match","data":{"lines":{"text":"// FIXME\n"},"line_number":1}}
        {"type":"match","data":{"lines":{"bytes":"123"},"line_number":1,"path":{"text":"basic.in.rs"}}}
        {"type":"match","data":{"path":{"text":"basic.in.rs"},"lines":{"text":"// NOTE\n"},"line_number":2,"submatches":[{"match":{"text":"NOTE"}}]}}
        "###;

        let issues = Ripgrep::default().parse("ripgrep", input).ok();
        assert!(logs_contain("Failed to parse ripgrep output"));
        assert!(logs_contain("Failed to parse lines"));
        assert!(logs_contain("Failed to parse line number"));
        assert!(logs_contain("Failed to parse path"));

        insta::assert_yaml_snapshot!(issues.unwrap(), @r"
        - tool: ripgrep
          ruleKey: NOTE
          message: // NOTE
          level: LEVEL_NOTE
          category: CATEGORY_LINT
          location:
            path: basic.in.rs
            range:
              startLine: 2
              startColumn: 1
              endLine: 2
              endColumn: 1
        ");
    }
}
