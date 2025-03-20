use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RadarlintIssue {
    severity: String,
    rule_key: String,
    primary_message: String,
    file_uri: String,
    text_range: Option<TextRange>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TextRange {
    start_line: u32,
    start_line_offset: u32,
    end_line: u32,
    end_line_offset: u32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Radarlint {}

impl Parser for Radarlint {
    fn parse(&self, _plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        output.trim().lines().for_each(|radarlint_output| {
            let radarlint_issue: RadarlintIssue = serde_json::from_str(radarlint_output).unwrap();

            let rule_key = radarlint_issue.rule_key.replace(":", "/");

            let issue = Issue {
                tool: "radarlint".to_string(),
                rule_key,
                message: radarlint_issue.primary_message,
                level: Radarlint::severity_to_level(&radarlint_issue.severity).into(),
                category: Category::Lint.into(),
                location: Some(Location {
                    path: radarlint_issue.file_uri,
                    range: radarlint_issue.text_range.map_or_else(
                        || {
                            Some(Range {
                                start_line: 0,
                                start_column: 0,
                                end_line: 0,
                                end_column: 0,
                                ..Default::default()
                            })
                        },
                        |text_range| {
                            Some(Range {
                                start_line: text_range.start_line,
                                start_column: text_range.start_line_offset,
                                end_line: text_range.end_line,
                                end_column: text_range.end_line_offset,
                                ..Default::default()
                            })
                        },
                    ),
                }),
                ..Default::default()
            };

            issues.push(issue);
        });

        Ok(issues)
    }
}

impl Radarlint {
    fn severity_to_level(severity: &str) -> Level {
        match severity {
            "CRITICAL" => Level::High,
            "MAJOR" => Level::Medium,
            "MINOR" => Level::Low,
            _ => Level::Medium,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = r###"
        {"severity":"CRITICAL","type":"CODE_SMELL","cleanCodeAttribute":"CONVENTIONAL","impacts":{"MAINTAINABILITY":"HIGH"},"ruleKey":"java:S1598","primaryMessage":"This file \"Foo.in.java\" should be located in \"foo\" directory, not in \"/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8\".","fileUri":"file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java","flows":[],"quickFixes":[],"textRange":{"startLine":1,"startLineOffset":8,"endLine":1,"endLineOffset":11}}
        {"severity":"MINOR","type":"CODE_SMELL","cleanCodeAttribute":"IDENTIFIABLE","impacts":{"MAINTAINABILITY":"LOW"},"ruleKey":"java:S100","primaryMessage":"Rename this method name to match the regular expression \u0027^[a-z][a-zA-Z0-9]*$\u0027.","fileUri":"file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java","flows":[],"quickFixes":[],"textRange":{"startLine":4,"startLineOffset":14,"endLine":4,"endLineOffset":23}}
        {"severity":"MAJOR","type":"CODE_SMELL","cleanCodeAttribute":"CLEAR","impacts":{"MAINTAINABILITY":"MEDIUM"},"ruleKey":"java:S1172","primaryMessage":"Remove this unused method parameter \"i\".","fileUri":"file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java","flows":[],"quickFixes":[{"inputFileEdits":[{"target":"file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java","textEdits":[{"range":{"startLine":8,"startLineOffset":19,"endLine":8,"endLineOffset":24},"newText":""}]}],"message":"Remove \"i\""}],"textRange":{"startLine":8,"startLineOffset":23,"endLine":8,"endLineOffset":24}}
        {"severity":"MINOR","type":"CODE_SMELL","cleanCodeAttribute":"IDENTIFIABLE","impacts":{"MAINTAINABILITY":"LOW"},"ruleKey":"java:S100","primaryMessage":"Rename this method name to match the regular expression \u0027^[a-z][a-zA-Z0-9]*$\u0027.","fileUri":"file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java","flows":[],"quickFixes":[],"textRange":{"startLine":13,"startLineOffset":15,"endLine":13,"endLineOffset":31}}
        {"severity":"MAJOR","type":"CODE_SMELL","cleanCodeAttribute":"MODULAR","impacts":{"MAINTAINABILITY":"MEDIUM"},"ruleKey":"java:S106","primaryMessage":"Replace this use of System.out by a logger.","fileUri":"file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java","flows":[],"quickFixes":[],"textRange":{"startLine":14,"startLineOffset":4,"endLine":14,"endLineOffset":14}}
        {"severity":"MINOR","type":"CODE_SMELL","cleanCodeAttribute":"MODULAR","impacts":{"MAINTAINABILITY":"LOW"},"ruleKey":"java:S1220","primaryMessage":"Move this file to a named package.","fileUri":"file:///Users/arslan/work/code_climate/plugins/Empty.java","flows":[],"quickFixes":[]}
        "###;

        let issues = Radarlint::default().parse("radarlint", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: radarlint
          ruleKey: java/S1598
          message: "This file \"Foo.in.java\" should be located in \"foo\" directory, not in \"/private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8\"."
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: "file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java"
            range:
              startLine: 1
              startColumn: 8
              endLine: 1
              endColumn: 11
        - tool: radarlint
          ruleKey: java/S100
          message: "Rename this method name to match the regular expression '^[a-z][a-zA-Z0-9]*$'."
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: "file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java"
            range:
              startLine: 4
              startColumn: 14
              endLine: 4
              endColumn: 23
        - tool: radarlint
          ruleKey: java/S1172
          message: "Remove this unused method parameter \"i\"."
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: "file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java"
            range:
              startLine: 8
              startColumn: 23
              endLine: 8
              endColumn: 24
        - tool: radarlint
          ruleKey: java/S100
          message: "Rename this method name to match the regular expression '^[a-z][a-zA-Z0-9]*$'."
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: "file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java"
            range:
              startLine: 13
              startColumn: 15
              endLine: 13
              endColumn: 31
        - tool: radarlint
          ruleKey: java/S106
          message: Replace this use of System.out by a logger.
          level: LEVEL_MEDIUM
          category: CATEGORY_LINT
          location:
            path: "file:///private/var/folders/b9/flqsg2gj0zs94d9802z004qw0000gn/T/plugins_Y7AmW8/Foo.in.java"
            range:
              startLine: 14
              startColumn: 4
              endLine: 14
              endColumn: 14
        - tool: radarlint
          ruleKey: java/S1220
          message: Move this file to a named package.
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: "file:///Users/arslan/work/code_climate/plugins/Empty.java"
            range: {}
        "#);
    }
}
