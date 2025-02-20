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

        let formatters: Vec<Box<dyn SimplecovFormatter>> = vec![
            Box::new(Simplecov018OrNewer {}),
            Box::new(SimplecovLegacy {}),
            Box::new(SimplecovJson {}),
        ];

        for formatter in formatters {
            if formatter.matches(&json_value) {
                return formatter.parse(&json_value);
            }
        }

        Ok(vec![])
    }
}

trait SimplecovFormatter {
    fn matches(&self, json_value: &Value) -> bool;
    fn parse(&self, json_value: &Value) -> Result<Vec<FileCoverage>>;
}

struct Simplecov018OrNewer;

impl SimplecovFormatter for Simplecov018OrNewer {
    fn matches(&self, json_value: &Value) -> bool {
        if let Some(meta) = json_value.get("meta") {
            if let Some(version_str) = meta.get("simplecov_version").and_then(|v| v.as_str()) {
                if let Ok(version) = Version::parse(version_str) {
                    return version >= Version::parse("0.18.0").expect("Parsing version failed");
                }
            }
        }
        false
    }

    fn parse(&self, json_value: &Value) -> Result<Vec<FileCoverage>> {
        let mut file_coverages = vec![];
        if let Some(coverage) = json_value.get("coverage").and_then(|c| c.as_object()) {
            file_coverages.extend(Simplecov::extract_file_coverage(coverage));
        }
        Ok(file_coverages)
    }
}

struct SimplecovLegacy;

impl SimplecovFormatter for SimplecovLegacy {
    fn matches(&self, json_value: &Value) -> bool {
        if let Some(obj) = json_value.as_object() {
            obj.values().any(|obj| obj.get("coverage").is_some())
        } else {
            false
        }
    }

    fn parse(&self, json_value: &Value) -> Result<Vec<FileCoverage>> {
        let mut file_coverages = vec![];
        if let Some(groups) = json_value.as_object() {
            for group in groups.values() {
                if let Some(coverage) = group.get("coverage").and_then(|c| c.as_object()) {
                    file_coverages.extend(Simplecov::extract_file_coverage(coverage));
                }
            }
        }
        Ok(file_coverages)
    }
}

struct SimplecovJson;

impl SimplecovFormatter for SimplecovJson {
    fn matches(&self, json_value: &Value) -> bool {
        json_value.get("files").is_some()
    }

    fn parse(&self, json_value: &Value) -> Result<Vec<FileCoverage>> {
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
                            Simplecov::parse_line_coverage(&Value::Array(coverage_arr.clone()));
                        file_coverages.push(FileCoverage {
                            path: filename_str.to_string(),
                            hits: line_hits,
                            ..Default::default()
                        });
                    }
                }
            }
        }
        Ok(file_coverages)
    }
}

impl Simplecov {
    fn extract_file_coverage(map: &Map<String, Value>) -> Vec<FileCoverage> {
        map.iter()
            .map(|(key, value)| {
                let line_hits = Self::parse_line_coverage(value);

                FileCoverage {
                    path: key.to_string(),
                    hits: line_hits,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn parse_line_coverage(data: &Value) -> Vec<i64> {
        match data {
            Value::Object(obj) => obj
                .get("lines")
                .and_then(|v| v.as_array())
                .map_or(vec![], |arr| arr.iter().map(Self::parse_lines).collect()),
            Value::Array(arr) => arr.iter().map(Self::parse_lines).collect(),
            _ => vec![],
        }
    }

    fn parse_lines(value: &Value) -> i64 {
        match value {
            Value::Number(n) => n.as_i64().unwrap_or(-1),
            Value::String(s) if s == "ignored" => -2,
            Value::Null => -1,
            _ => -1,
        }
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
