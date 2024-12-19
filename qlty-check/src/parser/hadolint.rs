// [
//    {
//       "code":"DL3045",
//       "column":1,
//       "file":"foo.Dockerfile",
//       "level":"warning",
//       "line":2,
//       "message":"`COPY` to a relative destination without `WORKDIR` set."
//    },
//    {
//       "code":"DL3045",
//       "column":1,
//       "file":"foo.Dockerfile",
//       "level":"warning",
//       "line":6,
//       "message":"`COPY` to a relative destination without `WORKDIR` set."
//    }
// ]

use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HadolintMessage {
    pub code: String,
    pub level: String,
    pub message: String,
    pub file: String,
    pub line: Option<i32>,
    pub column: Option<i32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Hadolint {}

impl Parser for Hadolint {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        let messages: Vec<HadolintMessage> = serde_json::from_str(output)?;

        for message in messages {
            let line = message.line.unwrap_or(1);
            let column = message.line.unwrap_or(1);

            let issue = Issue {
                tool: "hadolint".into(),
                message: message.message,
                category: Category::Lint.into(),
                level: level_to_level(&message.level).into(),
                rule_key: message.code,
                location: Some(Location {
                    path: message.file,
                    range: Some(Range {
                        start_line: line as u32,
                        start_column: column as u32,
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

fn level_to_level(level: &str) -> Level {
    match level {
        "error" => Level::High,
        "warning" => Level::Medium,
        "info" => Level::Low,
        "style" => Level::Low,
        _ => Level::Low,
    }
}
