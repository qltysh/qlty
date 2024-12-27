use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use semver::Version;
use serde_json::Value;

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

        let mut file_coverages = vec![];

        let coverage_data = if self.is_version_018_or_newer(&json_value) {
            json_value.get("coverage").and_then(|c| c.as_object())
        } else {
            json_value
                .as_object()
                .and_then(|obj| obj.values().next())
                .and_then(|group| group.get("coverage").and_then(|c| c.as_object()))
        };

        if let Some(coverage) = coverage_data {
            for (file_path, data) in coverage {
                let line_hits = self.parse_line_coverage(data);

                let file_coverage = FileCoverage {
                    path: file_path.to_string(),
                    hits: line_hits,
                    ..Default::default()
                };

                file_coverages.push(file_coverage);
            }
        }

        Ok(file_coverages)
    }
}

impl Simplecov {
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
}
