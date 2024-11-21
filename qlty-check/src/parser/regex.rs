use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};

#[derive(Debug)]
pub struct Regex {
    pub matcher: String,
    regex: regex::Regex,
    level: Option<Level>,
    category: Option<Category>,
}

impl Regex {
    pub fn new(matcher: &str, level: Option<Level>, category: Option<Category>) -> Self {
        Self {
            matcher: matcher.to_string(),
            regex: regex::Regex::new(matcher).unwrap(),
            category,
            level,
        }
    }
}

impl Parser for Regex {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        for captures in self.regex.captures_iter(&output.replace('\r', "")) {
            let mut issue = Issue {
                tool: plugin_name.into(),
                message: captures.name("message").unwrap().as_str().to_string(),
                category: self.category.unwrap_or(Category::Lint).into(),
                level: self.level.unwrap_or(Level::Medium).into(),
                rule_key: captures.name("code").unwrap().as_str().to_string(),
                location: Some(Location {
                    path: captures.name("path").unwrap().as_str().to_string(),
                    range: Some(Range {
                        start_line: captures.name("line").unwrap().as_str().parse().unwrap(),
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            };

            if let Some(severity) = captures.name("severity") {
                issue.level = severity_to_level(severity.as_str()).into();
            }

            if let Some(col) = captures.name("col") {
                let col = col.as_str().parse().unwrap();
                issue
                    .location
                    .as_mut()
                    .unwrap()
                    .range
                    .as_mut()
                    .unwrap()
                    .start_column = col;
            }

            if let Some(end_line) = captures.name("end_line") {
                let end_line = end_line.as_str().parse().unwrap();
                issue
                    .location
                    .as_mut()
                    .unwrap()
                    .range
                    .as_mut()
                    .unwrap()
                    .end_line = end_line;
            }

            if let Some(end_col) = captures.name("end_col") {
                let end_col = end_col.as_str().parse().unwrap();
                issue
                    .location
                    .as_mut()
                    .unwrap()
                    .range
                    .as_mut()
                    .unwrap()
                    .end_column = end_col;
            }

            self.add_documentation_url(&mut issue);
            issues.push(issue)
        }

        Ok(issues)
    }
}

fn severity_to_level(severity: &str) -> Level {
    match severity {
        "error" => Level::High,
        "warning" => Level::Medium,
        "info" => Level::Low,
        "note" => Level::Low,
        _ => Level::Medium,
    }
}

impl Regex {
    fn add_documentation_url(&self, issue: &mut Issue) {
        if issue.tool == "yamllint" {
            issue.documentation_url = format!(
                "https://yamllint.readthedocs.io/en/stable/rules.html#module-yamllint.rules.{}",
                issue.rule_key.replace('-', "_")
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
crates/cli/tests/fixtures/samples/sample.py:1:7: F821 undefined name 'foo'
        "###;

        let matcher =
            "((?P<path>.*):(?P<line>-?\\d+):(?P<col>-?\\d+): (?P<code>\\S+) (?P<message>.+))";

        let parser = Regex::new(matcher, None, None);
        let issues = parser.parse("regex", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: regex
          ruleKey: F821
          message: "undefined name 'foo'"
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: crates/cli/tests/fixtures/samples/sample.py
            range:
              startLine: 1
              startColumn: 7
        "#);
    }

    #[test]
    fn parse_dotenv_linter() {
        let input = r###"
basic.in.env:1 LeadingCharacter: Invalid leading character detected
        "###;

        let matcher = "((?P<path>.*):(?P<line>-?\\d+) (?P<code>\\S+): (?P<message>.+))\n";

        let parser = Regex::new(matcher, None, None);
        let issues = parser.parse("regex", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r"
        - tool: regex
          ruleKey: LeadingCharacter
          message: Invalid leading character detected
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: basic.in.env
            range:
              startLine: 1
        ");
    }

    #[test]
    fn parse_biome() {
        let input = r###"
::error title=lint/style/useEnumInitializers,file=basic.in.ts,line=4,endLine=4,col=6,endColumn=9::This enum declaration contains members that are implicitly initialized.
::error title=lint/complexity/noUselessLoneBlockStatements,file=basic.in.ts,line=13,endLine=13,col=3,endColumn=22::This block statement doesn't serve any purpose and can be safely removed.
        "###;

        let matcher = "::(?P<severity>[^ ]+) title=(?P<code>[^,]+),file=(?P<path>[^,]+),line=(?P<line>\\d+),endLine=(?P<end_line>\\d+),col=(?P<col>\\d+),endColumn=(?P<end_col>\\d+)::(?P<message>.+)";

        let parser = Regex::new(matcher, None, None);
        let issues = parser.parse("regex", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: regex
          ruleKey: lint/style/useEnumInitializers
          message: This enum declaration contains members that are implicitly initialized.
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.ts
            range:
              startLine: 4
              startColumn: 6
              endLine: 4
              endColumn: 9
        - tool: regex
          ruleKey: lint/complexity/noUselessLoneBlockStatements
          message: "This block statement doesn't serve any purpose and can be safely removed."
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.ts
            range:
              startLine: 13
              startColumn: 3
              endLine: 13
              endColumn: 22
        "#);
    }

    #[test]
    fn parse_redocly() {
        let input = r###"
::error title=spec,file=openapi.in.yaml,line=1,col=1,endLine=13,endColumn=62::The field `info` must be present on this level.
::error title=no-empty-servers,file=openapi.in.yaml,line=1,col=1,endLine=1,endColumn=8::Servers must be present.
::warning title=spec,file=openapi.in.yaml,line=5,col=7,endLine=5,endColumn=14::Property `content` is not expected here.
        "###;

        let matcher = "::(?P<severity>[^ ]+) title=(?P<code>[^,]+),file=(?P<path>[^,]+),line=(?P<line>\\d+),col=(?P<col>\\d+),endLine=(?P<end_line>\\d+),endColumn=(?P<end_col>\\d+)::(?P<message>.+)";

        let parser = Regex::new(matcher, None, None);
        let issues = parser.parse("redocly", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: redocly
          ruleKey: spec
          message: "The field `info` must be present on this level."
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: openapi.in.yaml
            range:
              startLine: 1
              startColumn: 1
              endLine: 13
              endColumn: 62
        - tool: redocly
          ruleKey: no-empty-servers
          message: Servers must be present.
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: openapi.in.yaml
            range:
              startLine: 1
              startColumn: 1
              endLine: 1
              endColumn: 8
        - tool: redocly
          ruleKey: spec
          message: "Property `content` is not expected here."
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: openapi.in.yaml
            range:
              startLine: 5
              startColumn: 7
              endLine: 5
              endColumn: 14
        "#);
    }
}
