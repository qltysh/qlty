use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Issue, Level, Location, Range};

#[derive(Debug, Default, Clone)]
pub struct Taplo {}

#[derive(Debug, Clone, Copy)]
enum ParserState {
    Start,
    Error,
}

impl Parser for Taplo {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let mut state = ParserState::Start;
        let mut message = "";
        let lines = output.lines();

        for line in lines {
            match state {
                ParserState::Start => {
                    if line.starts_with("error: ") {
                        state = ParserState::Error;
                        message = line.strip_prefix("error: ").unwrap_or("unknown");
                    }
                }
                ParserState::Error => {
                    // Example: "  ┌─ /Users/bhelmkamp/p/brynary/qlty-test-taplo/foo.toml:2:1"

                    let parts = line.split_whitespace();
                    let words = parts.clone().collect::<Vec<&str>>();

                    let location = words.get(1).ok_or(anyhow::anyhow!("location not found"))?;
                    let location_parts = location.split(":").collect::<Vec<&str>>();

                    let path = location_parts.first()
                        .ok_or(anyhow::anyhow!("path not found"))?;

                    let line_string = location_parts
                        .get(1)
                        .ok_or(anyhow::anyhow!("line not found"))?;
                    let line: u32 = line_string.parse::<u32>()?;

                    let column_string = location_parts
                        .get(2)
                        .ok_or(anyhow::anyhow!("column not found"))?;
                    let column = column_string.parse::<u32>()?;

                    issues.push(Issue {
                        message: message.to_string(),
                        level: Level::High as i32,
                        tool: "taplo".to_string(),
                        rule_key: "error".to_string(),
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

                    state = ParserState::Start;
                }
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let output = r#"
INFO taplo:lint_files:collect_files: found files total=1 excluded=0 cwd="/Users/bhelmkamp/p/brynary/qlty-test-taplo"
error: conflicting keys
  ┌─ /Users/bhelmkamp/p/brynary/qlty-test-taplo/foo.toml:2:1
  │
1 │ foo = 1
  │ --- duplicate found here
2 │ foo = 2
  │ ^^^ duplicate key

error: expected array of tables
  ┌─ /Users/bhelmkamp/p/brynary/qlty-test-taplo/foo.toml:3:1
  │
3 │ bar = [1]
  │ ^^^ expected array of tables
4 │
5 │ [[bar]]
  │   --- required by this key

ERROR taplo:lint_files: invalid file error=semantic errors found path="/Users/bhelmkamp/p/brynary/qlty-test-taplo/foo.toml"
ERROR operation failed error=some files were not valid
"#;
        let parser = Taplo::default();
        let issues = parser.parse("taplo", output).unwrap();
        assert_eq!(issues.len(), 2);
    }
}
