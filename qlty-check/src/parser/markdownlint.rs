use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

const DEFAULT_MARKDOWNLINT_URL_FORMAT: &str = "https://github.com/DavidAnson/markdownlint/blob/main/doc/Rules.md#${rule}";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarkdownlintMessage {
    #[serde(alias = "fileName")]
    pub file_name: String,
    #[serde(alias = "lineNumber")]
    pub line_number: Option<i32>,
    #[serde(alias = "ruleDescription")]
    pub rule_description: String,
    #[serde(alias = "ruleInformation")]
    pub rule_information: String,
    #[serde(alias = "ruleNames")]
    pub rule_names: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Markdownlint {}

impl Parser for Markdownlint {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        if output.trim().is_empty() {
            return Ok(issues);
        }

        let messages: Vec<MarkdownlintMessage> = match serde_json::from_str(output) {
            Ok(messages) => messages,
            Err(_) => return Ok(issues),
        };

        for message in messages {
            let line = message.line_number.unwrap_or(1);
            let rule_key = message.rule_names.join(" - ");

            let issue = Issue {
                tool: "markdownlint".into(),
                message: format!(
                    "{} ({})",
                    message.rule_description, message.rule_information
                ),
                category: Category::Style.into(),
                level: Level::Fmt.into(),
                documentation_url: generate_document_url(rule_key.clone()),
                rule_key,
                location: Some(Location {
                    path: message.file_name,
                    range: Some(Range {
                        start_line: line as u32,
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

fn generate_document_url(rule_key: String) -> String {
    DEFAULT_MARKDOWNLINT_URL_FORMAT
        .to_string()
        .replace(
        "${rule}",
        rule_key
            .split(' ')
            .next()
            .unwrap_or_else(|| &rule_key)
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_replacement() {
        let rule_key = "rule-name - some other text".to_string();
        let expected = "https://github.com/DavidAnson/markdownlint/blob/main/doc/Rules.md#rule-name".to_string();

        assert_eq!(generate_document_url(rule_key), expected);
    }
}
