use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::Deserialize;

pub struct Reek {}

// JSON format (test): https://github.com/troessner/reek/blob/master/features/reports/json.feature
// JSON format code: https://github.com/troessner/reek/blob/master/lib/reek/report/json_report.rb

#[derive(Debug, Deserialize, Clone)]
struct ReekSmell {
    pub context: String,
    pub lines: Vec<i32>,
    pub message: String,
    pub smell_type: String,
    pub source: String,
    pub documentation_link: String,
}

impl Parser for Reek {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let reek_smells: Vec<ReekSmell> = serde_json::from_str(output)?;

        for smell in reek_smells {
            for line in smell.lines {
                let issue = Issue {
                    tool: plugin_name.into(),
                    documentation_url: smell.documentation_link.clone(),
                    message: format!("{} {}", smell.context.trim(), smell.message.trim()),
                    category: Category::Lint.into(),
                    level: Level::Medium.into(),
                    rule_key: smell.smell_type.clone(),
                    location: Some(Location {
                        path: smell.source.clone(),
                        range: Some(Range {
                            start_line: line as u32,
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                };

                issues.push(issue);
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"[{
          "context":"Foo#check_response",
          "lines":[4,6],
          "message":"manually dispatches method call",
          "smell_type":"ManualDispatch",
          "source":"linters/reek/fixtures/basic.in.rb",
          "documentation_link":"https://github.com/troessner/reek/blob/v6.3.0/docs/Manual-Dispatch.md"
        }]"###;

        let issues = Reek {}.parse("Reek", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: Reek
          ruleKey: ManualDispatch
          message: "Foo#check_response manually dispatches method call"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/troessner/reek/blob/v6.3.0/docs/Manual-Dispatch.md"
          location:
            path: linters/reek/fixtures/basic.in.rb
            range:
              startLine: 4
        - tool: Reek
          ruleKey: ManualDispatch
          message: "Foo#check_response manually dispatches method call"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/troessner/reek/blob/v6.3.0/docs/Manual-Dispatch.md"
          location:
            path: linters/reek/fixtures/basic.in.rb
            range:
              startLine: 6
        "#);
    }
}
