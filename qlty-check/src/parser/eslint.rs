use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range, SuggestionSource};
use serde::{Deserialize, Serialize};
use tracing::debug;

const DEFAULT_ESLINT_URL_FORMAT: &str = "https://eslint.org/docs/rules/${rule}";
const REACT_HOOKS_URL: &str = "https://react.dev/reference/rules/rules-of-hooks"; // out of two rules in react-hooks there is only a url for one rule
const REACT_URL_FORMAT: &str =
    "https://github.com/jsx-eslint/eslint-plugin-react/blob/master/docs/rules/${rule}.md";
const IMPORT_URL_FORMAT: &str =
    "https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/${rule}.md";
const JSX_ESLINT_URL_FORMAT: &str =
    "https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/main/docs/rules/${rule}.md";
const TESTING_LIBRARY_URL_FORMAT: &str = "https://github.com/testing-library/eslint-plugin-testing-library/tree/main/docs/rules/${rule}.md";
const TYPESCRIPT_ESLINT_URL_FORMAT: &str = "https://typescript-eslint.io/rules/${rule}";

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct EslintFile {
    #[serde(alias = "filePath")]
    pub file_path: String,
    pub messages: Vec<EslintMessage>,
    #[serde(skip_serializing)]
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EslintMessage {
    #[serde(alias = "ruleId")]
    pub rule_id: Option<String>,
    pub fatal: Option<bool>,
    pub severity: i32,
    pub message: String,
    pub line: Option<i32>,
    pub column: Option<i32>,
    #[serde(alias = "endLine")]
    pub end_line: Option<i32>,
    #[serde(alias = "endColumn")]
    pub end_column: Option<i32>,
    #[serde(default)]
    pub suggestions: Vec<EslintSuggestion>,

    pub fix: Option<EslintFix>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EslintSuggestion {
    #[serde(alias = "messageId", default)]
    pub message_id: String,
    pub desc: String,
    pub fix: EslintFix,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EslintFix {
    pub range: Vec<i32>,
    pub text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Eslint {}

impl Parser for Eslint {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let files: Vec<EslintFile> = serde_json::from_str(output)?;

        for file in files {
            for message in file.messages {
                let line = message.line.unwrap_or(1);
                let column = message.column.unwrap_or(1);
                let rule_key = message.rule_id.unwrap_or_default();
                let start_line = line as u32;
                let start_column = column as u32;
                let end_line = message.end_line.unwrap_or(line) as u32;
                let end_column = message.end_column.unwrap_or(column) as u32;

                let suggestions = if let Some(fix) = message.fix {
                    let mut start_byte = fix.range[0] as u32;
                    let mut end_byte = fix.range[1] as u32;
                    if let Some(ref source) = file.source {
                        start_byte =
                            source.char_indices().nth(start_byte as usize).unwrap().0 as u32;
                        end_byte = source.char_indices().nth(end_byte as usize).unwrap().0 as u32;
                    } else {
                        debug!(
                            "Failed to translate characters to bytes for {}",
                            file.file_path
                        );
                    }

                    vec![qlty_types::analysis::v1::Suggestion {
                        id: String::from("fix"),
                        description: String::from("fix"),
                        source: SuggestionSource::Tool.into(),
                        replacements: vec![qlty_types::analysis::v1::Replacement {
                            data: fix.text.clone(),
                            location: Some(Location {
                                path: file.file_path.clone(),
                                range: Some(Range {
                                    start_byte: Some(start_byte),
                                    end_byte: Some(end_byte),
                                    start_line,
                                    start_column,
                                    end_line,
                                    end_column,
                                }),
                            }),
                        }],
                        ..Default::default()
                    }]
                } else {
                    message
                        .suggestions
                        .iter()
                        .map(|suggestion| {
                            let replacement = suggestion.fix.text.clone();

                            qlty_types::analysis::v1::Suggestion {
                                id: suggestion.message_id.clone(),
                                description: suggestion.desc.clone(),
                                source: SuggestionSource::Tool.into(),
                                replacements: vec![qlty_types::analysis::v1::Replacement {
                                    data: replacement.clone(),
                                    location: Some(Location {
                                        path: file.file_path.clone(),
                                        range: Some(Range {
                                            start_byte: Some(suggestion.fix.range[0] as u32),
                                            end_byte: Some(suggestion.fix.range[1] as u32),
                                            start_line,
                                            start_column,
                                            end_line,
                                            end_column,
                                        }),
                                    }),
                                }],
                                ..Default::default()
                            }
                        })
                        .collect()
                };

                let issue = Issue {
                    tool: "eslint".into(),
                    message: message.message,
                    category: category(&rule_key).into(),
                    level: severity_to_level(message.fatal, message.severity).into(),
                    documentation_url: generate_document_url(rule_key.clone()),
                    rule_key,
                    location: Some(Location {
                        path: file.file_path.clone(),
                        range: Some(Range {
                            start_line,
                            start_column,
                            end_line,
                            end_column,
                            ..Default::default()
                        }),
                    }),
                    suggestions,
                    ..Default::default()
                };

                issues.push(issue);
            }
        }

        Ok(issues)
    }
}

fn category(rule_key: &str) -> Category {
    if rule_key.contains("a11y") {
        Category::Accessibility
    } else {
        Category::Lint
    }
}

fn generate_document_url(rule_key: String) -> String {
    let rule_config: Vec<&str> = rule_key.split('/').collect();

    match rule_config.as_slice() {
        [_] => {
            // no extra package detected, use default plugin url format
            DEFAULT_ESLINT_URL_FORMAT
                .to_string()
                .replace("${rule}", &rule_key)
        }
        [package, package_rule] => {
            // extra package detected in rule
            match EslintExtraPackages::package_to_url_format(package) {
                Some(package_url_format) => {
                    // in case of known package
                    package_url_format.replace("${rule}", package_rule)
                }
                _ => {
                    // in case of unknown package
                    EslintExtraPackages::missing_rule_url_format(rule_key)
                }
            }
        }
        _ => {
            // in case of unknown rule format
            EslintExtraPackages::missing_rule_url_format(rule_key)
        }
    }
}

#[derive(Debug)]
struct EslintExtraPackages {}

impl EslintExtraPackages {
    fn package_to_url_format(package: &str) -> Option<&str> {
        match package {
            "@typescript-eslint" => Some(TYPESCRIPT_ESLINT_URL_FORMAT),
            "import" => Some(IMPORT_URL_FORMAT),
            "jsx-a11y" => Some(JSX_ESLINT_URL_FORMAT),
            "react-hooks" => Some(REACT_HOOKS_URL),
            "react" => Some(REACT_URL_FORMAT),
            "testing-library" => Some(TESTING_LIBRARY_URL_FORMAT),
            _ => None,
        }
    }

    // gracefully return empty string when unknown package/rule is detected
    fn missing_rule_url_format(rule: String) -> String {
        debug!("No URL format found for rule: {}", rule);
        "".to_string()
    }
}

fn severity_to_level(fatal: Option<bool>, severity: i32) -> Level {
    if matches!(fatal, Some(true)) {
        return Level::High;
    }

    match severity {
        1 => Level::Low,
        2 => Level::Medium,
        _ => Level::Medium,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        [
          {
            "filePath": "/workspace-ro/.eslintrc.js",
            "messages": [
              {
                "ruleId": "no-undef",
                "severity": 2,
                "message": "'module' is not defined.",
                "line": 1,
                "column": 1,
                "nodeType": "Identifier",
                "messageId": "undef",
                "endLine": 1,
                "endColumn": 7
              }
            ],
            "suppressedMessages": [],
            "errorCount": 1,
            "fatalErrorCount": 0,
            "warningCount": 0,
            "fixableErrorCount": 0,
            "fixableWarningCount": 0,
            "source": "...",
            "usedDeprecatedRules": []
          }
        ]
        "###;

        let issues = Eslint::default().parse("eslint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: eslint
          ruleKey: no-undef
          message: "'module' is not defined."
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://eslint.org/docs/rules/no-undef"
          location:
            path: /workspace-ro/.eslintrc.js
            range:
              startLine: 1
              startColumn: 1
              endLine: 1
              endColumn: 7
        "#);
    }

    #[test]
    fn parse_with_fix() {
        let input = r###"
        [
          {
            "filePath": "/workspace-ro/.eslintrc.js",
            "messages": [
              {
                "ruleId": "no-undef",
                "severity": 2,
                "message": "'module' is not defined.",
                "line": 1,
                "column": 5,
                "nodeType": "Identifier",
                "messageId": "undef",
                "endLine": 1,
                "endColumn": 6,
                "fix": {
                    "range": [4, 5],
                    "text": "y"
                },
                "suggestions": []
              }
            ],
            "suppressedMessages": [],
            "errorCount": 1,
            "fatalErrorCount": 0,
            "warningCount": 0,
            "fixableErrorCount": 0,
            "fixableWarningCount": 0,
            "source": "let x = 1;\n",
            "usedDeprecatedRules": []
          }
        ]
        "###;

        let issues = Eslint::default().parse("eslint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: eslint
          ruleKey: no-undef
          message: "'module' is not defined."
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://eslint.org/docs/rules/no-undef"
          location:
            path: /workspace-ro/.eslintrc.js
            range:
              startLine: 1
              startColumn: 5
              endLine: 1
              endColumn: 6
          suggestions:
            - id: fix
              description: fix
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: y
                  location:
                    path: /workspace-ro/.eslintrc.js
                    range:
                      startLine: 1
                      startColumn: 5
                      endLine: 1
                      endColumn: 6
                      startByte: 4
                      endByte: 5
        "#);
    }

    #[test]
    fn parse_with_suggestions() {
        let input = r###"
        [
          {
            "filePath": "/workspace-ro/.eslintrc.js",
            "messages": [
              {
                "ruleId": "no-undef",
                "severity": 2,
                "message": "'module' is not defined.",
                "line": 1,
                "column": 5,
                "nodeType": "Identifier",
                "messageId": "undef",
                "endLine": 1,
                "endColumn": 6,
                "suggestions": [
                  {
                    "messageId": "rename",
                    "desc": "Rename 'x' to 'y'",
                    "fix": {
                    "range": [4, 5],
                    "text": "y"
                    }
                  },
                  {
                    "messageId": "replace",
                    "desc": "Replace variable multiple variables",
                    "fix": {
                    "range": [4, 5],
                    "text": "x = \n  y"
                    }
                  }
                ]
              }
            ],
            "suppressedMessages": [],
            "errorCount": 1,
            "fatalErrorCount": 0,
            "warningCount": 0,
            "fixableErrorCount": 0,
            "fixableWarningCount": 0,
            "source": "let x = 1;\n",
            "usedDeprecatedRules": []
          }
        ]
        "###;

        let issues = Eslint::default().parse("eslint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: eslint
          ruleKey: no-undef
          message: "'module' is not defined."
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://eslint.org/docs/rules/no-undef"
          location:
            path: /workspace-ro/.eslintrc.js
            range:
              startLine: 1
              startColumn: 5
              endLine: 1
              endColumn: 6
          suggestions:
            - id: rename
              description: "Rename 'x' to 'y'"
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: y
                  location:
                    path: /workspace-ro/.eslintrc.js
                    range:
                      startLine: 1
                      startColumn: 5
                      endLine: 1
                      endColumn: 6
                      startByte: 4
                      endByte: 5
            - id: replace
              description: Replace variable multiple variables
              source: SUGGESTION_SOURCE_TOOL
              replacements:
                - data: "x = \n  y"
                  location:
                    path: /workspace-ro/.eslintrc.js
                    range:
                      startLine: 1
                      startColumn: 5
                      endLine: 1
                      endColumn: 6
                      startByte: 4
                      endByte: 5
        "#);
    }

    #[test]
    fn test_default_url_format() {
        let rule_key = "no-package-rule".to_string();
        let expected_url = "https://eslint.org/docs/rules/no-package-rule".to_string();

        assert_eq!(generate_document_url(rule_key), expected_url);
    }

    #[test]
    fn test_known_package_url() {
        let rule_key = "@typescript-eslint/rule-name".to_string();
        let expected_url = "https://typescript-eslint.io/rules/rule-name".to_string();

        assert_eq!(generate_document_url(rule_key), expected_url);
    }

    #[test]
    fn test_unknown_package_url() {
        let rule_key = "unknown-package/rule-name".to_string();

        assert_eq!(generate_document_url(rule_key), "".to_string());
    }

    #[test]
    fn test_missing_rule_url_format() {
        let rule_key = "complex/rule/structure".to_string();

        assert_eq!(generate_document_url(rule_key), "".to_string());
    }
}
