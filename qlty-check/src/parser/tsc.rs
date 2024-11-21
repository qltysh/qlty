use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};
use tracing::trace;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Tsc {}

impl Parser for Tsc {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let lines = output.lines();

        for line in lines {
            // app/routes/api.dev.login/route.tsx(8,32): error TS2339: Property 'request' does not exist on type 'String'.
            let parts: Vec<&str> = line.splitn(3, ": ").collect();
            if parts.len() != 3 {
                trace!(
                    "Issue line does not have 3 parts splitting on ': ': {}",
                    line
                );
                continue;
            }

            let location_string = parts[0];
            let check_string = parts[1];
            let message = parts[2];

            let location_parts: Vec<&str> = location_string.splitn(2, '(').collect();
            if location_parts.len() != 2 {
                trace!("Location does not have 2 parts splitting on (: {}", line);
                continue;
            }

            let path = location_parts[0];
            let location_parts: Vec<&str> = location_parts[1].splitn(2, ',').collect();

            if location_parts.len() != 2 {
                trace!("Location does not have 2 parts splitting on ,: {}", line);
                continue;
            }

            let line = location_parts[0].parse::<i32>()?;
            let column = location_parts[1].replace(')', "").parse::<i32>()?;

            let check_parts: Vec<&str> = check_string.splitn(2, ' ').collect();
            if check_parts.len() != 2 {
                trace!("Check does not have 2 parts splitting on ' ': {}", line);
                continue;
            }

            let rule_key = check_parts[1];

            issues.push(Issue {
                tool: plugin_name.to_string(),
                rule_key: rule_key.to_string(),
                level: Level::High as i32, // TODO: Add error level
                message: message.to_string(),
                location: Some(Location {
                    path: path.to_string(),
                    range: Some(Range {
                        start_line: line as u32,
                        start_column: column as u32,
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            });
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_issues() {
        let parser = Tsc::default();
        let issues = parser.parse("tsc", "").unwrap();
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_one_issue() {
        let parser = Tsc::default();
        let issues = parser.parse("tsc", "app/routes/api.dev.login/route.tsx(8,32): error TS2339: Property 'request' does not exist on type 'String'.").unwrap();
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.tool, "tsc");
        assert_eq!(issue.rule_key, "TS2339");
        assert_eq!(issue.level, qlty_types::analysis::v1::Level::High as i32);
        assert_eq!(
            issue.message,
            "Property 'request' does not exist on type 'String'."
        );
        assert_eq!(issue.path().unwrap(), "app/routes/api.dev.login/route.tsx");
        let range = issue.range().unwrap();

        assert_eq!(range.start_line, 8);
        assert_eq!(range.start_column, 32);
    }
}
