use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use semver::Version;
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Simplecov {}

impl Simplecov {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Simplecov {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        let json_value: Value =
            serde_json::from_str(text).with_context(|| "Failed to parse JSON text")?;

        if self.using_simplecov_json_formatter(&json_value) {
            Ok(self.parse_simplecov_json_formatter(&json_value))
        } else if self.is_version_018_or_newer(&json_value) {
            Ok(self.parse_version_018_or_newer_coverage(&json_value))
        } else {
            Ok(self.parse_legacy_coverage(&json_value))
        }
    }
}

impl Simplecov {
    fn extract_coverage<'a>(&self, json_value: &'a Value) -> Option<&'a Map<String, Value>> {
        json_value.get("coverage").and_then(|c| c.as_object())
    }

    fn parse_version_018_or_newer_coverage(&self, json_value: &Value) -> Vec<FileCoverage> {
        let mut file_coverages = vec![];

        if let Some(coverage) = self.extract_coverage(&json_value) {
            file_coverages.extend(self.extract_file_coverage(coverage));
        }
        
        file_coverages
    }

    fn parse_legacy_coverage(&self, json_value: &Value) -> Vec<FileCoverage> {
        let mut file_coverages = vec![];

        if let Some(groups) = json_value.as_object() {
            for group in groups.values() {
                if let Some(coverage) = self.extract_coverage(group) {
                    file_coverages.extend(self.extract_file_coverage(coverage));
                }
            }
        }

        file_coverages
    }

    fn parse_simplecov_json_formatter(&self, json_value: &Value) -> Vec<FileCoverage> {
        let mut file_coverages = vec![];

        if let Some(files) = json_value.get("files").and_then(|v| v.as_array()) {
            for file in files {
                if let (Some(filename), Some(coverage)) =
                    (file.get("filename"), file.get("coverage"))
                {
                    if let (Some(filename_str), Some(coverage_arr)) =
                        (filename.as_str(), coverage.as_array())
                    {
                        let line_hits =
                            self.parse_line_coverage(&Value::Array(coverage_arr.clone()));
                        file_coverages.push(FileCoverage {
                            path: filename_str.to_string(),
                            hits: line_hits,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        file_coverages
    }

    fn parse_line_coverage(&self, data: &Value) -> Vec<i64> {
        match data {
            Value::Object(obj) => {
                // Post-0.18.0 format with "lines" key
                obj.get("lines")
                    .and_then(|v| v.as_array())
                    .map_or(vec![], |arr| {
                        arr.iter().map(|x| self.parse_lines(x)).collect()
                    })
            }
            Value::Array(arr) => {
                // Pre-0.18.0 format, directly an array
                arr.iter().map(|x| self.parse_lines(x)).collect()
            }
            _ => vec![],
        }
    }

    fn extract_file_coverage(&self, map: &Map<String, Value>) -> Vec<FileCoverage> {
        map.iter()
            .map(|(key, value)| {
                let line_hits = self.parse_line_coverage(value);

                FileCoverage {
                    path: key.to_string(),
                    hits: line_hits,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn parse_lines(&self, value: &Value) -> i64 {
        match value {
            Value::Number(n) => n.as_i64().unwrap_or(-1),
            Value::String(s) if s == "ignored" => -2,
            Value::Null => -1,
            _ => -1,
        }
    }

    fn is_version_018_or_newer(&self, json_value: &serde_json::Value) -> bool {
        if let Some(meta) = json_value.get("meta") {
            if let Some(version_str) = meta.get("simplecov_version").and_then(|v| v.as_str()) {
                if let Ok(version) = Version::parse(version_str) {
                    return version >= Version::parse("0.18.0").expect("Parsing version failed");
                }
            }
        }
        false
    }

    // https://github.com/vicentllongo/simplecov-json
    fn using_simplecov_json_formatter(&self, json_value: &serde_json::Value) -> bool {
        json_value.get("files").is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simplecov_report() {
        let input = r#"
        {
            "meta": {
                "simplecov_version": "0.21.2"
            },
            "coverage": {
                "sample.rb": {
                    "lines": [null, 1, 1, 1, 1, null, null, 1, 1, null, null, 1, 1, 0, null, 1, null, null, null, "ignored", "ignored", "ignored", "ignored", "ignored", null]
                }
            },
            "groups": {}
        }
        "#;
        let results = Simplecov::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: sample.rb
          hits:
            - "-1"
            - "1"
            - "1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "0"
            - "-1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "-2"
            - "-2"
            - "-2"
            - "-2"
            - "-2"
            - "-1"
        "#);
    }

    #[test]
    fn simplecov_legacy_report() {
        let input = r#"
        {
            "Unit Tests": {
                "coverage": {
                    "development/mygem/lib/mygem/errors.rb": [1, null, 1, 1, 0, null, null, null, 1, null, null, null, 1, null, null, null, 1, null, null, null, null]
                },
                "timestamp": 1488827968
            }
        }
        "#;
        let results = Simplecov::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: development/mygem/lib/mygem/errors.rb
          hits:
            - "1"
            - "-1"
            - "1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
        "#);
    }

    #[test]
    fn simplecov_legacy_report_two_sections() {
        let input = r#"
        {
            "Unit Tests": {
                "coverage": {
                    "development/mygem/lib/mygem/errors.rb": [1, 0, 1, null]
                },
                "timestamp": 1488827968
            },
            "Integration Tests": {
                "coverage": {
                    "development/mygem/lib/mygem/errors.rb": [1, 2, null, null]
                },
                "timestamp": 1488827968
            }
        }
        "#;
        let results: Vec<FileCoverage> = Simplecov::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: development/mygem/lib/mygem/errors.rb
          hits:
            - "1"
            - "0"
            - "1"
            - "-1"
        - path: development/mygem/lib/mygem/errors.rb
          hits:
            - "1"
            - "2"
            - "-1"
            - "-1"
        "#);
    }

    #[test]
    fn simplecov_fixture() {
        let input = include_str!("../../tests/fixtures/simplecov/sample.json");
        let parsed_results = Simplecov::new().parse_text(input).unwrap();

        insta::assert_yaml_snapshot!(parsed_results, @r#"
    - path: sample.rb
      hits:
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "1"
        - "0"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
        - "-2"
        - "-2"
        - "-2"
        - "-2"
        - "-2"
        - "-1"
    - path: sample_2.rb
      hits:
        - "1"
        - "1"
        - "1"
        - "0"
        - "-1"
        - "-1"
        - "1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
    "#);
    }

    #[test]
    fn simplecov_json_fixture() {
        // When using https://github.com/vicentllongo/simplecov-json
        let input = include_str!("../../tests/fixtures/simplecov/sample-json.json");
        let parsed_results = Simplecov::new().parse_text(input).unwrap();

        insta::assert_yaml_snapshot!(parsed_results, @r###"
        - path: app/controllers/base_controller.rb
          hits:
            - "1"
            - "1"
            - "1"
            - "1"
            - "-1"
            - "1"
            - "-1"
            - "1"
            - "-1"
            - "1"
            - "-1"
            - "1"
            - "-1"
            - "0"
            - "-1"
            - "0"
            - "26"
            - "-1"
            - "-1"
            - "1"
            - "20"
            - "-1"
            - "-1"
            - "-1"
        - path: app/controllers/sample_controller.rb
          hits:
            - "1"
            - "1"
            - "1"
            - "-1"
            - "1"
            - "0"
            - "0"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "-1"
        "###);
    }
}
