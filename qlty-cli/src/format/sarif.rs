use anyhow::Result;
use qlty_config::version::{BUILD_DATE, LONG_VERSION, QLTY_VERSION};
use qlty_formats::Formatter;
use qlty_types::analysis::v1::{Category, Issue, Language, Level, Location, Message, MessageLevel};
use serde_json::{json, Map, Value};
use std::convert::TryFrom;
use std::io::Write;

#[derive(Debug)]
pub struct SarifFormatter {
    pub messages: Vec<Message>,
    pub issues: Vec<Issue>,
}

impl SarifFormatter {
    pub fn new(messages: Vec<Message>, issues: Vec<Issue>) -> Self {
        Self { messages, issues }
    }

    pub fn boxed(messages: Vec<Message>, issues: Vec<Issue>) -> Box<dyn Formatter> {
        Box::new(Self::new(messages, issues))
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

    fn get_sarif_locations(&self, location: &Option<Location>, language: i32) -> Vec<Value> {
        if let Some(location) = location {
            let mut region = json!({});

            if let Some(range) = &location.range {
                let mut region_obj = json!({
                    "startLine": range.start_line,
                    "startColumn": range.start_column,
                    "endLine": range.end_line,
                    "endColumn": range.end_column
                });

                if language != 0 {
                    if let Ok(lang) = Language::try_from(language) {
                        if lang != Language::Unspecified {
                            let lang_str = format!("{:?}", lang).to_lowercase();
                            region_obj["sourceLanguage"] = json!(lang_str);
                        }
                    }
                }

                region = region_obj;
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

    fn get_related_locations(&self, other_locations: &[Location]) -> Vec<Value> {
        other_locations
            .iter()
            .map(|location| {
                let mut region = json!({});

                if let Some(range) = &location.range {
                    region = json!({
                        "startLine": range.start_line,
                        "startColumn": range.start_column,
                        "endLine": range.end_line,
                        "endColumn": range.end_column
                    });
                }

                json!({
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": location.path
                        },
                        "region": region
                    }
                })
            })
            .collect()
    }

    fn get_fixes(&self, suggestions: &[qlty_types::analysis::v1::Suggestion]) -> Vec<Value> {
        if suggestions.is_empty() {
            return vec![];
        }

        suggestions
            .iter()
            .map(|suggestion| {
                let replacements = suggestion
                    .replacements
                    .iter()
                    .map(|replacement| {
                        let mut location_obj = json!({});

                        if let Some(location) = &replacement.location {
                            let mut region = json!({});

                            if let Some(range) = &location.range {
                                region = json!({
                                    "startLine": range.start_line,
                                    "startColumn": range.start_column,
                                    "endLine": range.end_line,
                                    "endColumn": range.end_column
                                });
                            }

                            location_obj = json!({
                                "physicalLocation": {
                                    "artifactLocation": {
                                        "uri": location.path
                                    },
                                    "region": region
                                }
                            });
                        }

                        json!({
                            "deletedRegion": location_obj["physicalLocation"]["region"],
                            "insertedContent": {
                                "text": replacement.data
                            }
                        })
                    })
                    .collect::<Vec<_>>();

                json!({
                    "description": {
                        "text": suggestion.description
                    },
                    "replacements": replacements
                })
            })
            .collect()
    }

    fn serialize_issue(&self, issue: &Issue) -> Value {
        let mut result = json!({
            "ruleId": format!("{}:{}", issue.tool, issue.rule_key),
            "level": self.convert_level(Level::try_from(issue.level).unwrap_or(Level::Medium)),
            "message": {
                "text": issue.message
            },
            "locations": self.get_sarif_locations(&issue.location, issue.language)
        });

        let mut partial_fingerprints = Map::new();

        if !issue.source_checksum.is_empty() {
            partial_fingerprints.insert(
                format!("sourceHash/{}", issue.source_checksum_version),
                json!(issue.source_checksum),
            );
        }

        for (key, value) in &issue.partial_fingerprints {
            partial_fingerprints.insert(key.clone(), json!(value));
        }

        if !partial_fingerprints.is_empty() {
            result["partialFingerprints"] = Value::Object(partial_fingerprints);
        }

        if !issue.other_locations.is_empty() {
            result["relatedLocations"] = json!(self.get_related_locations(&issue.other_locations));
        }

        if !issue.suggestions.is_empty() {
            result["fixes"] = json!(self.get_fixes(&issue.suggestions));
        }

        if issue.category != 0 {
            if let Ok(category) = Category::try_from(issue.category) {
                if category != Category::Unspecified {
                    let category_str = format!("{:?}", category).to_lowercase();
                    result["taxa"] = json!([{
                        "id": category_str,
                        "name": category_str
                    }]);
                }
            }
        }

        let mut properties = Map::new();

        if !issue.tags.is_empty() {
            properties.insert("tags".to_string(), json!(issue.tags));
        }

        if let Some(props) = &issue.properties {
            for (key, value) in &props.fields {
                properties.insert(
                    key.clone(),
                    serde_json::to_value(value).unwrap_or(Value::Null),
                );
            }
        }

        if !properties.is_empty() {
            result["properties"] = Value::Object(properties);
        }

        result
    }

    fn convert_message_level(&self, level: MessageLevel) -> &'static str {
        match level {
            MessageLevel::Unspecified => "none",
            MessageLevel::Debug => "note",
            MessageLevel::Info => "note",
            MessageLevel::Warning => "warning",
            MessageLevel::Error => "error",
            MessageLevel::Fatal => "error",
        }
    }

    fn serialize_notification(&self, message: &Message) -> Value {
        let level = MessageLevel::try_from(message.level).unwrap_or(MessageLevel::Info);
        let level_str = self.convert_message_level(level);

        let mut descriptor = json!({
            "id": format!("qlty:message:{}", message.ty),
            "name": message.module,
            "shortDescription": {
                "text": message.message
            },
        });

        if !message.details.is_empty() {
            descriptor["fullDescription"]["text"] = json!(message.details);
        }

        let mut notification = json!({
            "level": level_str,
            "message": {
                "text": message.message
            },
            "descriptor": descriptor
        });

        if !message.details.is_empty() {
            notification["message"]["text"] =
                json!(format!("{}\n\n{}", message.message, message.details));
        }

        notification
    }

    fn create_sarif_document(&self) -> Value {
        let mut rules = vec![];
        let mut rule_ids = std::collections::HashSet::new();

        for issue in &self.issues {
            if !rule_ids.contains(&issue.rule_key) {
                rule_ids.insert(issue.rule_key.clone());

                let mut rule = json!({
                    "id": format!("{}:{}", issue.tool, issue.rule_key),
                });

                if !issue.documentation_url.is_empty() {
                    rule["helpUri"] = json!(issue.documentation_url.clone());
                }

                rules.push(rule);
            }
        }

        let results = self
            .issues
            .iter()
            .map(|issue| self.serialize_issue(issue))
            .collect::<Vec<_>>();

        let notifications = self
            .messages
            .iter()
            .map(|message| self.serialize_notification(message))
            .collect::<Vec<_>>();

        let mut sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "qlty",
                            "informationUri": "https://github.com/qlty/qlty",
                            "semanticVersion": QLTY_VERSION,
                            "version": LONG_VERSION.as_str(),
                            "releaseDateUtc": BUILD_DATE,
                            "rules": rules
                        }
                    },
                    "results": results
                }
            ]
        });

        if !notifications.is_empty() {
            sarif["runs"][0]["tool"]["driver"]["notifications"] = json!(notifications);
        }

        sarif
    }
}

impl Formatter for SarifFormatter {
    fn write_to(&self, writer: &mut dyn Write) -> Result<()> {
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
    use qlty_analysis::{workspace_entries::TargetMode, IssueCount};
    use qlty_check::Report;
    use qlty_types::analysis::v1::{
        Category, ExecutionVerb, Mode, Range, Replacement, Suggestion, SuggestionSource,
    };
    use std::collections::HashSet;

    #[test]
    fn test_sarif_formatter() {
        let mut tags = Vec::new();
        tags.push("test-tag".to_string());
        tags.push("security".to_string());
        tags.push("critical".to_string());

        let comprehensive_issue = Issue {
            rule_key: "test-rule-1".to_string(),
            message: "Test message 1".to_string(),
            level: Level::High.into(),
            language: Language::Rust.into(),
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
            category: Category::Vulnerability.into(),
            source_checksum: "abc123def456".to_string(),
            source_checksum_version: 1,
            other_locations: vec![
                Location {
                    path: "src/related.rs".to_string(),
                    range: Some(Range {
                        start_line: 20,
                        start_column: 3,
                        end_line: 22,
                        end_column: 10,
                        ..Default::default()
                    }),
                },
                Location {
                    path: "src/related2.rs".to_string(),
                    range: Some(Range {
                        start_line: 30,
                        start_column: 1,
                        end_line: 35,
                        end_column: 2,
                        ..Default::default()
                    }),
                },
            ],
            suggestions: vec![
                Suggestion {
                    id: "suggestion-1".to_string(),
                    description: "Fix by replacing with safer code".to_string(),
                    patch: "some patch content".to_string(),
                    r#unsafe: false,
                    source: SuggestionSource::Tool.into(),
                    replacements: vec![Replacement {
                        data: "safeCode()".to_string(),
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
                    }],
                },
                Suggestion {
                    id: "suggestion-2".to_string(),
                    description: "Alternative fix".to_string(),
                    patch: "alternative patch content".to_string(),
                    r#unsafe: true,
                    source: SuggestionSource::Llm.into(),
                    replacements: vec![Replacement {
                        data: "differentSolution()".to_string(),
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
                    }],
                },
            ],
            tags,
            mode: Mode::Block.into(),
            on_added_line: true,
            effort_minutes: 60,
            value: 100,
            ..Default::default()
        };

        let simple_issue = Issue {
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
            language: Language::Typescript.into(),
            category: Category::Lint.into(),
            ..Default::default()
        };

        let info_message = Message {
            message: "Info message".to_string(),
            details: "Detailed info".to_string(),
            level: MessageLevel::Info.into(),
            module: "test-module".to_string(),
            ty: "INFO".to_string(),
            timestamp: None,
            ..Default::default()
        };

        let warning_message = Message {
            message: "Warning message".to_string(),
            details: "".to_string(),
            level: MessageLevel::Warning.into(),
            module: "warning-module".to_string(),
            ty: "WARNING".to_string(),
            ..Default::default()
        };

        let report = Report {
            verb: ExecutionVerb::Check,
            target_mode: TargetMode::default(),
            messages: vec![info_message, warning_message],
            invocations: vec![],
            issues: vec![comprehensive_issue, simple_issue],
            formatted: vec![],
            fixed: HashSet::new(),
            fixable: HashSet::new(),
            counts: IssueCount::default(),
        };

        let formatter = SarifFormatter::boxed(report.messages.clone(), report.issues.clone());
        let output = formatter.read().unwrap();
        let output_str = String::from_utf8_lossy(&output);

        let json_value: Value = serde_json::from_str(&output_str).unwrap();

        insta::assert_json_snapshot!(json_value, {
            ".runs[0].tool.driver.semanticVersion" => "[version]",
            ".runs[0].tool.driver.version" => "[version_string]",
            ".runs[0].tool.driver.releaseDateUtc" => "[date]",
            ".runs[0].results[0].partialFingerprints" => insta::sorted_redaction()
        });
    }
}
