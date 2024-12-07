use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::Deserialize;

pub struct Reek {}

// JSON format spec (test): https://github.com/troessner/reek/blob/master/spec/reek/report/json_report_spec.rb
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
            let issue = Issue {
                tool: plugin_name.into(),
                documentation_url: smell.documentation_link,
                message: format!("{} {}", smell.context.trim(), smell.message.trim()),
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
          message: "Shipment#referer is a writable attribute"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          documentationUrl: "https://github.com/troessner/reek/blob/v6.3.0/docs/Attribute.md"
          location:
            path: app/models/shipment.rb
            range:
              startLine: 13
        - tool: Reek
          ruleKey: IrresponsibleModule
          message: Shipment has no descriptive comment
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
