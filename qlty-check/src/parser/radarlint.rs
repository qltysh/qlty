use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RadarlintIssue {
    #[serde(default = "default_severity")]
    severity: String,
    rule_key: String,
    primary_message: String,
    file_uri: String,
    text_range: Option<TextRange>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TextRange {
    start_line: i32,
    start_line_offset: i32,
    end_line: i32,
    end_line_offset: i32,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Radarlint {}

impl Parser for Radarlint {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];

        for (line_idx, radarlint_output) in output.trim().lines().enumerate() {
            let radarlint_issue: RadarlintIssue =
                serde_json::from_str(radarlint_output).map_err(|err| {
                    anyhow::anyhow!(
                        "Failed to parse Radarlint output at line {}: {}\nOutput: {}",
                        line_idx + 1,
                        err,
                        radarlint_output
                    )
                })?;

            let rule_key = radarlint_issue.rule_key.replace(":", ".");

            let issue = Issue {
                tool: plugin_name.to_string(),
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
                            // Convert values to u32, treating negative values as 0
                            Some(Range {
                                start_line: text_range.start_line.max(0) as u32,
                                start_column: text_range.start_line_offset.max(0) as u32,
                                end_line: text_range.end_line.max(0) as u32,
                                end_column: text_range.end_line_offset.max(0) as u32,
                                ..Default::default()
                            })
                        },
                    ),
                }),
                ..Default::default()
            };

            issues.push(issue);
        }

        Ok(issues)
    }
}

// Default severity function for serde
fn default_severity() -> String {
    "MAJOR".to_string()
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
          ruleKey: java.S1598
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
          ruleKey: java.S100
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
          ruleKey: java.S1172
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
          ruleKey: java.S100
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
          ruleKey: java.S106
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
          ruleKey: java.S1220
          message: Move this file to a named package.
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: "file:///Users/arslan/work/code_climate/plugins/Empty.java"
            range: {}
        "#);
    }

    #[test]
    fn parse_fails_on_invalid_json() {
        let input = r###"
        {"severity":"MINOR","ruleKey":"java:S100","primaryMessage":"Valid issue","fileUri":"file:///path/to/file.java","textRange":{"startLine":1,"startLineOffset":2,"endLine":3,"endLineOffset":4}}
        This is not valid JSON and should cause parser to return Err
        {"severity":"MAJOR","ruleKey":"java:S101","primaryMessage":"Another valid issue","fileUri":"file:///path/to/another.java","textRange":{"startLine":5,"startLineOffset":6,"endLine":7,"endLineOffset":8}}
        "###;

        let result = Radarlint::default().parse("radarlint", input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse Radarlint output"));
    }

    #[test]
    fn parse_negative_range_values() {
        let input = r###"
        {"severity":"CRITICAL","ruleKey":"java:S1598","primaryMessage":"Issue with negative range","fileUri":"file:///path/to/file.java","textRange":{"startLine":-1,"startLineOffset":-1,"endLine":-1,"endLineOffset":-1}}
        "###;

        let issues = Radarlint::default().parse("radarlint", input).unwrap();
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.rule_key, "java.S1598");
        let range = issue.location.as_ref().unwrap().range.as_ref().unwrap();
        assert_eq!(range.start_line, 0);
        assert_eq!(range.start_column, 0);
        assert_eq!(range.end_line, 0);
        assert_eq!(range.end_column, 0);
    }

    #[test]
    fn parse_fails_on_missing_required_fields() {
        let input1 = r###"
        {"ruleKey":"missing:fields","primaryMessage":"Missing file_uri field"}
        "###;
        let result1 = Radarlint::default().parse("radarlint", input1);
        assert!(result1.is_err());
        let err_msg = result1.unwrap_err().to_string();
        assert!(err_msg.contains("fileUri"));

        let input2 = r###"
        {"primaryMessage":"Missing rule_key field","fileUri":"file:///path/to/file.java"}
        "###;
        let result2 = Radarlint::default().parse("radarlint", input2);
        assert!(result2.is_err());
        let err_msg = result2.unwrap_err().to_string();
        assert!(err_msg.contains("ruleKey"));

        let input3 = r###"
        {"ruleKey":"missing:message","fileUri":"file:///path/to/file.java"}
        "###;
        let result3 = Radarlint::default().parse("radarlint", input3);
        assert!(result3.is_err());
        let err_msg = result3.unwrap_err().to_string();
        assert!(err_msg.contains("primaryMessage"));

        let input4 = r###"
        {"ruleKey":"missing:severity","primaryMessage":"Missing severity field","fileUri":"file:///path/to/file.java"}
        "###;
        let result4 = Radarlint::default().parse("radarlint", input4);
        assert!(result4.is_ok());
        let issues = result4.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].level, 40);
    }
}
