use super::Formatter;
use qlty_types::analysis::v1::{Issue, Level, Location};
use serde_json::{json, Value};
use std::convert::TryFrom;
use std::io::Write;

#[derive(Debug)]
pub struct SarifFormatter {
    issues: Vec<Issue>,
}

impl SarifFormatter {
    pub fn new(issues: Vec<Issue>) -> Self {
        Self { issues }
    }

    pub fn boxed(issues: Vec<Issue>) -> Box<dyn Formatter> {
        Box::new(Self::new(issues))
    }

    fn convert_level(&self, level: Level) -> &'static str {
        match level {
            Level::Unspecified => "none",
            Level::Note => "note",
            Level::Fmt => "note",
            Level::Low => "note",
            Level::Medium => "warning",
            Level::High => "error",
        }
    }

    fn get_sarif_locations(&self, location: &Option<Location>) -> Vec<Value> {
        if let Some(location) = location {
            let mut region = json!({});

            if let Some(range) = &location.range {
                region = json!({
                    "startLine": range.start_line,
                    "startColumn": range.start_column,
                    "endLine": range.end_line,
                    "endColumn": range.end_column
                });
            }

            return vec![json!({
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": location.path
                    },
                    "region": region
                }
            })];
        }

        vec![]
    }

    fn create_sarif_document(&self) -> Value {
        let mut rules = vec![];
        let mut rule_ids = std::collections::HashSet::new();

        for issue in &self.issues {
            if !rule_ids.contains(&issue.rule_key) {
                rule_ids.insert(issue.rule_key.clone());

                let mut rule = json!({
                    "id": issue.rule_key
                });

                if !issue.documentation_url.is_empty() {
                    rule["helpUri"] = json!(issue.documentation_url.clone());
                }

                rules.push(rule);
            }
        }

        let results = self.issues.iter().map(|issue| {
            json!({
                "ruleId": issue.rule_key,
                "level": self.convert_level(Level::try_from(issue.level).unwrap_or(Level::Medium)),
                "message": {
                    "text": issue.message
                },
                "locations": self.get_sarif_locations(&issue.location)
            })
        }).collect::<Vec<_>>();

        json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "qlty",
                            "informationUri": "https://github.com/qlty/qlty",
                            "rules": rules
                        }
                    },
                    "results": results
                }
            ]
        })
    }
}

impl Formatter for SarifFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        let sarif = self.create_sarif_document();
        let json = serde_json::to_string_pretty(&sarif)?;
        writer.write_all(json.as_bytes())?;
        writer.write_all(b"\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use qlty_types::analysis::v1::{Category, Range};

    #[test]
    fn test_sarif_formatter() {
        let issues = vec![
            Issue {
                rule_key: "test-rule-1".to_string(),
                message: "Test message 1".to_string(),
                level: Level::High.into(),
                location: Some(Location {
                    path: "src/test.rs".to_string(),
                    range: Some(Range {
                        start_line: 10,
                        start_column: 5,
                        end_line: 10,
                        end_column: 20,
                        ..Default::default()
                    }),
                }),
                documentation_url: "https://example.com/docs/test-rule-1".to_string(),
                tool: "test-tool".to_string(),
                category: Category::Lint.into(),
                ..Default::default()
            },
            Issue {
                rule_key: "test-rule-2".to_string(),
                message: "Test message 2".to_string(),
                level: Level::Medium.into(),
                location: Some(Location {
                    path: "src/test2.rs".to_string(),
                    range: Some(Range {
                        start_line: 15,
                        start_column: 1,
                        end_line: 20,
                        end_column: 2,
                        ..Default::default()
                    }),
                }),
                tool: "test-tool".to_string(),
                category: Category::Lint.into(),
                ..Default::default()
            },
        ];

        let formatter = SarifFormatter::boxed(issues);
        let output = formatter.read().unwrap();
        let output_str = String::from_utf8_lossy(&output);

        let json_value: Value = serde_json::from_str(&output_str).unwrap();

        insta::assert_json_snapshot!(json_value);
    }
}
