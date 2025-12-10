use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct XccovLine {
    line: i64,
    #[serde(rename = "isExecutable")]
    is_executable: bool,
    #[serde(rename = "executionCount")]
    execution_count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XccovJson {}

impl XccovJson {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for XccovJson {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let report: HashMap<String, Vec<XccovLine>> =
            serde_json::from_str(text).with_context(|| "Failed to parse XCCov JSON")?;

        let mut file_coverages: Vec<FileCoverage> = Vec::new();
        let mut sorted_paths: Vec<_> = report.keys().collect();
        sorted_paths.sort();

        for path in sorted_paths {
            let lines = &report[path];
            let max_line = lines.iter().map(|l| l.line).max().unwrap_or(0);
            let mut hits: Vec<i64> = vec![-1; max_line as usize];

            for line_data in lines {
                if line_data.line > 0 {
                    let index = (line_data.line - 1) as usize;
                    if index < hits.len() {
                        hits[index] = if line_data.is_executable {
                            line_data.execution_count.unwrap_or(0)
                        } else {
                            -1
                        };
                    }
                }
            }

            file_coverages.push(FileCoverage {
                path: path.clone(),
                hits,
                ..Default::default()
            });
        }

        Ok(file_coverages)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_file() {
        let input = r#"
        {
            "/path/to/File.swift": [
                { "isExecutable": false, "line": 1 },
                { "isExecutable": false, "line": 2 },
                { "isExecutable": true, "line": 3, "executionCount": 5 },
                { "isExecutable": true, "line": 4, "executionCount": 0 },
                { "isExecutable": false, "line": 5 }
            ]
        }
        "#;

        let results = XccovJson::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: /path/to/File.swift
          hits:
            - "-1"
            - "-1"
            - "5"
            - "0"
            - "-1"
        "#);
    }

    #[test]
    fn multiple_files() {
        let input = r#"
        {
            "/path/to/First.swift": [
                { "isExecutable": true, "line": 1, "executionCount": 1 },
                { "isExecutable": true, "line": 2, "executionCount": 2 }
            ],
            "/path/to/Second.swift": [
                { "isExecutable": true, "line": 1, "executionCount": 3 },
                { "isExecutable": false, "line": 2 }
            ]
        }
        "#;

        let results = XccovJson::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: /path/to/First.swift
          hits:
            - "1"
            - "2"
        - path: /path/to/Second.swift
          hits:
            - "3"
            - "-1"
        "#);
    }

    #[test]
    fn empty_report() {
        let input = r#"{}"#;
        let results = XccovJson::new().parse_text(input).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn all_non_executable() {
        let input = r#"
        {
            "/path/to/File.swift": [
                { "isExecutable": false, "line": 1 },
                { "isExecutable": false, "line": 2 },
                { "isExecutable": false, "line": 3 }
            ]
        }
        "#;

        let results = XccovJson::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: /path/to/File.swift
          hits:
            - "-1"
            - "-1"
            - "-1"
        "#);
    }

    #[test]
    fn sparse_line_numbers() {
        let input = r#"
        {
            "/path/to/File.swift": [
                { "isExecutable": true, "line": 5, "executionCount": 1 },
                { "isExecutable": true, "line": 10, "executionCount": 2 }
            ]
        }
        "#;

        let results = XccovJson::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: /path/to/File.swift
          hits:
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "2"
        "#);
    }

    #[test]
    fn zero_execution_count() {
        let input = r#"
        {
            "/path/to/File.swift": [
                { "isExecutable": true, "line": 1, "executionCount": 0 },
                { "isExecutable": true, "line": 2, "executionCount": 0 }
            ]
        }
        "#;

        let results = XccovJson::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: /path/to/File.swift
          hits:
            - "0"
            - "0"
        "#);
    }

    #[test]
    fn with_subranges() {
        let input = r#"
        {
            "/path/to/File.swift": [
                { "isExecutable": true, "line": 1, "executionCount": 4, "subranges": [{"column": 36, "executionCount": 2, "length": 0}] }
            ]
        }
        "#;

        let results = XccovJson::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: /path/to/File.swift
          hits:
            - "4"
        "#);
    }
}
