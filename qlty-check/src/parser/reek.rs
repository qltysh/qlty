use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

pub struct Reek {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReekJson {
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
        let reek_issues: Vec<ReekJson> = serde_json::from_str(output)?;

        for smell in reek_issues {
            let issue = Issue {
                tool: plugin_name.into(),
                documentation_url: smell.documentation_link,
                message: smell.message.trim().to_string(),
                category: Category::Lint.into(),
                level: Level::Medium.into(),
                rule_key: smell.smell_type,
                location: Some(Location {
                    path: smell.source,
                    range: Some(Range {
                        start_line: smell.lines[0] as u32,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"[
          {
            "context":"Shipment#referer",
            "lines":[13],
            "message":"is a writable attribute",
            "smell_type":"Attribute",
            "source":"app/models/shipment.rb",
            "documentation_link":"https://github.com/troessner/reek/blob/v6.3.0/docs/Attribute.md"
          },
          {
            "context":"Shipment",
            "lines":[1],
            "message":"has no descriptive comment",
            "smell_type":"IrresponsibleModule",
            "source":"app/models/shipment.rb",
            "documentation_link":"https://github.com/troessner/reek/blob/v6.3.0/docs/Irresponsible-Module.md"
          }
        ]"###;

        let issues = Reek {}.parse("Reek", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: Reek
          ruleKey: Attribute
          message: is a writable attribute
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/troessner/reek/blob/v6.3.0/docs/Attribute.md"
          location:
            path: app/models/shipment.rb
            range:
              startLine: 13
        - tool: Reek
          ruleKey: IrresponsibleModule
          message: has no descriptive comment
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/troessner/reek/blob/v6.3.0/docs/Irresponsible-Module.md"
          location:
            path: app/models/shipment.rb
            range:
              startLine: 1
        "#);
    }
}
