use crate::Parser;
use anyhow::{Context, Result};
use qlty_types::tests::v1::FileCoverage;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Qlty {}

impl Qlty {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser for Qlty {
    fn parse_text(&self, text: &str) -> Result<Vec<FileCoverage>> {
        text.lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let json_line: Value =
                    serde_json::from_str(line).with_context(|| "Failed to parse JSONL line")?;

                let path = json_line
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("Missing or invalid `path` field"))?;

                let hits: Vec<i64> = if let Some(hits) = json_line.get("hits") {
                    hits.as_array()
                        .ok_or_else(|| anyhow::anyhow!("Invalid `hits` field"))?
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Invalid hit value, expected a string")
                                })?
                                .parse::<i64>()
                                .map_err(|e| anyhow::anyhow!("Failed to parse hit value: {}", e))
                        })
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    vec![]
                };

                Ok(FileCoverage {
                    path: path.to_string(),
                    hits,
                    ..Default::default()
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn qlty_single_file() {
        let input = r#"{"path":"lib/fish.rb","hits":["-1","1","1","0","-1","-1","1","0","-1","-1","1","0","-1","-1"]}"#;
        let results = Qlty::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: lib/fish.rb
          hits:
            - "-1"
            - "1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
        "#);
    }

    #[test]
    fn qlty_no_hits() {
        let input = r#"{"path":"lib/fish.rb"}"#;
        let results = Qlty::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: lib/fish.rb
        "#);
    }

    #[test]
    fn qlty_multiple_files() {
        let input = r#"
{"path":"lib/dog.rb","hits":["-1","1","1","2","-1","-1","-1","1","0","-1","-1","-1","-1","1","1","-1","-1","1","0","-1","-1","1","0","-1","-1","1","1","-1","-1","1","-1","0","-1","-1","-1","-1","-1","-1","-1","1","0","0","0","0","0","0","0","0","0","0","0","-1","0","-1","-1","-1"]}
{"path":"lib/fish.rb","hits":["-1","1","1","0","-1","-1","1","0","-1","-1","1","0","-1","-1"]}
"#;
        let results = Qlty::new().parse_text(input).unwrap();
        insta::assert_yaml_snapshot!(results, @r#"
        - path: lib/dog.rb
          hits:
            - "-1"
            - "1"
            - "1"
            - "2"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "1"
            - "-1"
            - "-1"
            - "1"
            - "-1"
            - "0"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "0"
            - "-1"
            - "0"
            - "-1"
            - "-1"
            - "-1"
        - path: lib/fish.rb
          hits:
            - "-1"
            - "1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
            - "1"
            - "0"
            - "-1"
            - "-1"
        "#);
    }
}
