use anyhow::Result;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use qlty_config::version::{BUILD_DATE, LONG_VERSION, QLTY_VERSION};
use qlty_formats::Formatter;
use qlty_types::analysis::v1::{
    Category, Issue, Language, Level, Location, Message, MessageLevel, Range, Suggestion,
};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::io::Write;

const SARIF_SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
const CATEGORY_TAXONOMY: &str = "qlty-categories";

// Characters a URI reference cannot contain unencoded (RFC 3986). ':' is
// included so that a relative path is never read as a URI scheme.
const URI_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

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

    fn artifact_location(&self, path: &str) -> Value {
        json!({
            "uri": utf8_percent_encode(path, URI_ENCODE_SET).to_string()
        })
    }

    // Parsers leave a range field at 0 when the tool does not report it, but
    // SARIF requires lines and columns of at least 1, so unknown fields are
    // left out. Without a start line there is no valid region at all.
    fn region(&self, range: &Range) -> Option<Value> {
        if range.start_line == 0 {
            return None;
        }

        let mut region = Map::new();
        region.insert("startLine".to_string(), json!(range.start_line));

        for (key, value) in [
            ("startColumn", range.start_column),
            ("endLine", range.end_line),
            ("endColumn", range.end_column),
        ] {
            if value > 0 {
                region.insert(key.to_string(), json!(value));
            }
        }

        Some(Value::Object(region))
    }

    fn physical_location(&self, location: &Location) -> Value {
        let mut physical_location = json!({
            "artifactLocation": self.artifact_location(&location.path)
        });

        if let Some(region) = location.range.as_ref().and_then(|range| self.region(range)) {
            physical_location["region"] = region;
        }

        physical_location
    }

    fn get_sarif_locations(&self, location: &Option<Location>, language: i32) -> Vec<Value> {
        if let Some(location) = location {
            let mut physical_location = self.physical_location(location);

            if language != 0 && physical_location.get("region").is_some() {
                if let Ok(lang) = Language::try_from(language) {
                    if lang != Language::Unspecified {
                        let lang_str = format!("{:?}", lang).to_lowercase();
                        physical_location["region"]["sourceLanguage"] = json!(lang_str);
                    }
                }
            }

            return vec![json!({ "physicalLocation": physical_location })];
        }

        vec![]
    }

    fn get_related_locations(&self, other_locations: &[Location]) -> Vec<Value> {
        unique(
            other_locations
                .iter()
                .map(|location| json!({ "physicalLocation": self.physical_location(location) }))
                .collect(),
        )
    }

    fn get_fixes(&self, suggestions: &[Suggestion]) -> Vec<Value> {
        unique(
            suggestions
                .iter()
                .filter_map(|suggestion| self.serialize_fix(suggestion))
                .collect(),
        )
    }

    fn serialize_fix(&self, suggestion: &Suggestion) -> Option<Value> {
        let mut changes_by_path: Vec<(&str, Vec<Value>)> = vec![];

        for replacement in &suggestion.replacements {
            let Some(location) = &replacement.location else {
                continue;
            };
            let Some(deleted_region) = location.range.as_ref().and_then(|range| self.region(range))
            else {
                continue;
            };

            let replacement = json!({
                "deletedRegion": deleted_region,
                "insertedContent": {
                    "text": replacement.data
                }
            });

            match changes_by_path
                .iter_mut()
                .find(|(path, _)| *path == location.path)
            {
                Some((_, replacements)) => replacements.push(replacement),
                None => changes_by_path.push((&location.path, vec![replacement])),
            }
        }

        if changes_by_path.is_empty() {
            return None;
        }

        let artifact_changes = changes_by_path
            .into_iter()
            .map(|(path, replacements)| {
                json!({
                    "artifactLocation": self.artifact_location(path),
                    "replacements": replacements
                })
            })
            .collect::<Vec<_>>();

        let mut fix = json!({ "artifactChanges": artifact_changes });

        if !suggestion.description.is_empty() {
            fix["description"] = json!({ "text": suggestion.description });
        }

        Some(fix)
    }

    fn category_id(&self, category: i32) -> Option<String> {
        match Category::try_from(category) {
            Ok(Category::Unspecified) | Err(_) => None,
            Ok(category) => Some(format!("{:?}", category).to_lowercase()),
        }
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

        let fixes = self.get_fixes(&issue.suggestions);

        if !fixes.is_empty() {
            result["fixes"] = json!(fixes);
        }

        if let Some(category) = self.category_id(issue.category) {
            result["taxa"] = json!([{
                "id": category,
                "toolComponent": {
                    "name": CATEGORY_TAXONOMY
                }
            }]);
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

    fn notification_descriptor_id(&self, message: &Message) -> String {
        format!("qlty:message:{}", message.ty)
    }

    fn serialize_notification(&self, message: &Message) -> Value {
        let level = MessageLevel::try_from(message.level).unwrap_or(MessageLevel::Info);
        let level_str = self.convert_message_level(level);

        let text = if message.details.is_empty() {
            message.message.clone()
        } else {
            format!("{}\n\n{}", message.message, message.details)
        };

        json!({
            "level": level_str,
            "message": {
                "text": text
            },
            "descriptor": {
                "id": self.notification_descriptor_id(message)
            }
        })
    }

    fn create_sarif_document(&self) -> Value {
        let mut rules = vec![];
        let mut rule_ids = HashSet::new();

        for issue in &self.issues {
            let rule_id = format!("{}:{}", issue.tool, issue.rule_key);

            if rule_ids.insert(rule_id.clone()) {
                let mut rule = json!({
                    "id": rule_id,
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

        let category_taxa = unique(
            self.issues
                .iter()
                .filter_map(|issue| self.category_id(issue.category))
                .map(|category| json!({ "id": category, "name": category }))
                .collect(),
        );

        let notifications = self
            .messages
            .iter()
            .map(|message| self.serialize_notification(message))
            .collect::<Vec<_>>();

        let notification_descriptors = unique(
            self.messages
                .iter()
                .map(|message| json!({ "id": self.notification_descriptor_id(message) }))
                .collect(),
        );

        let mut sarif = json!({
            "$schema": SARIF_SCHEMA,
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "qlty",
                            "informationUri": "https://github.com/qltysh/qlty",
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

        if !category_taxa.is_empty() {
            sarif["runs"][0]["taxonomies"] = json!([{
                "name": CATEGORY_TAXONOMY,
                "taxa": category_taxa
            }]);
        }

        if !notifications.is_empty() {
            sarif["runs"][0]["tool"]["driver"]["notifications"] = json!(notification_descriptors);
            sarif["runs"][0]["invocations"] = json!([{
                "executionSuccessful": true,
                "toolExecutionNotifications": notifications
            }]);
        }

        sarif
    }
}

fn unique(values: Vec<Value>) -> Vec<Value> {
    let mut unique_values = Vec::with_capacity(values.len());

    for value in values {
        if !unique_values.contains(&value) {
            unique_values.push(value);
        }
    }

    unique_values
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
    fn smell_measurements_survive_json_and_sarif_serialization() {
        let mut issue = Issue {
            tool: "qlty".to_string(),
            rule_key: "boolean-logic".to_string(),
            ..Default::default()
        };
        issue.set_property_number("threshold", 4.0);
        issue.set_property_number("actual", 8.0);

        let json = serde_json::to_value(&issue).unwrap();
        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(json["properties"], json!({"threshold": 4.0, "actual": 8.0}));
        assert_eq!(sarif["properties"], json["properties"]);
    }

    #[test]
    fn zero_smell_measurements_survive_json_and_sarif_serialization() {
        let mut issue = Issue::default();
        issue.set_property_number("threshold", 0.0);
        issue.set_property_number("actual", 0.0);

        let json = serde_json::to_value(&issue).unwrap();
        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(json["properties"], json!({"threshold": 0.0, "actual": 0.0}));
        assert_eq!(sarif["properties"], json["properties"]);
    }

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

    #[test]
    fn region_leaves_out_unknown_range_fields() {
        let issue = Issue {
            location: Some(Location {
                path: "sample.py".to_string(),
                range: Some(Range {
                    start_line: 4,
                    ..Default::default()
                }),
            }),
            ..Default::default()
        };

        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(
            sarif["locations"][0]["physicalLocation"]["region"],
            json!({"startLine": 4})
        );
    }

    #[test]
    fn location_without_start_line_has_no_region() {
        let issue = Issue {
            location: Some(Location {
                path: "sample.py".to_string(),
                range: Some(Range::default()),
            }),
            ..Default::default()
        };

        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(
            sarif["locations"][0]["physicalLocation"],
            json!({"artifactLocation": {"uri": "sample.py"}})
        );
    }

    #[test]
    fn artifact_uri_is_percent_encoded() {
        let issue = Issue {
            location: Some(Location {
                path: "src/my file#1.rs".to_string(),
                range: None,
            }),
            ..Default::default()
        };

        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(
            sarif["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
            json!("src/my%20file%231.rs")
        );
    }

    #[test]
    fn duplicate_related_locations_are_removed() {
        let location = Location {
            path: "src/related.rs".to_string(),
            range: Some(Range {
                start_line: 3,
                ..Default::default()
            }),
        };
        let issue = Issue {
            other_locations: vec![location.clone(), location],
            ..Default::default()
        };

        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(
            sarif["relatedLocations"],
            json!([{
                "physicalLocation": {
                    "artifactLocation": {"uri": "src/related.rs"},
                    "region": {"startLine": 3}
                }
            }])
        );
    }

    #[test]
    fn fix_replacements_are_grouped_by_file() {
        let replacement = |path: &str, line: u32| Replacement {
            data: "fixed".to_string(),
            location: Some(Location {
                path: path.to_string(),
                range: Some(Range {
                    start_line: line,
                    ..Default::default()
                }),
            }),
        };
        let issue = Issue {
            suggestions: vec![Suggestion {
                replacements: vec![
                    replacement("src/a.rs", 1),
                    replacement("src/b.rs", 2),
                    replacement("src/a.rs", 3),
                ],
                ..Default::default()
            }],
            ..Default::default()
        };

        let sarif = SarifFormatter::new(vec![], vec![]).serialize_issue(&issue);

        assert_eq!(
            sarif["fixes"],
            json!([{
                "artifactChanges": [
                    {
                        "artifactLocation": {"uri": "src/a.rs"},
                        "replacements": [
                            {"deletedRegion": {"startLine": 1}, "insertedContent": {"text": "fixed"}},
                            {"deletedRegion": {"startLine": 3}, "insertedContent": {"text": "fixed"}}
                        ]
                    },
                    {
                        "artifactLocation": {"uri": "src/b.rs"},
                        "replacements": [
                            {"deletedRegion": {"startLine": 2}, "insertedContent": {"text": "fixed"}}
                        ]
                    }
                ]
            }])
        );
    }
}
