use super::Parser;
use anyhow::Result;
use qlty_types::analysis::v1::{Category, Issue, Level, Location, Range};
use regex::Regex;

#[derive(Debug, Default)]
pub struct Mypy {}

impl Parser for Mypy {
    fn parse(&self, plugin_name: &str, output: &str) -> Result<Vec<Issue>> {
        let mut issues = vec![];
        let error_regex = Regex::new(r"^(.*?):(\d+):(\d+): error: (.*?)  \[(.*?)\]").unwrap();
        let note_regex = Regex::new(r"^(.*?):(\d+):(\d+): note: (.*)").unwrap();

        for line in output.lines() {
            if let Some(caps) = error_regex.captures(line) {
                let file = caps[1].to_string();
                let line: u32 = caps[2].parse()?;
                let column: u32 = caps[3].parse()?;
                let message = caps[4].to_string();
                let error_type = caps[5].to_string();

                issues.push(Issue {
                    tool: plugin_name.into(),
                    message,
                    category: Category::Lint.into(),
                    level: Level::High.into(),
                    rule_key: error_type,
                    location: Some(Location {
                        path: file,
                        range: Some(Range {
                            start_line: line,
                            start_column: column,
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                });
            } else if let Some(caps) = note_regex.captures(line) {
                let file = caps[1].to_string();
                let line: u32 = caps[2].parse()?;
                let column: u32 = caps[3].parse()?;
                let message = caps[4].to_string();

                let find_multiline_note = issues.iter_mut().find(|issue| {
                    let issue_range: Range;
                    let issue_path = if let Some(location) = issue.location.as_ref() {
                        issue_range = if let Some(range) = location.range.as_ref() {
                            *range
                        } else {
                            return false;
                        };
                        location.path.clone()
                    } else {
                        return false;
                    };

                    issue.rule_key == "note"
                        && issue_path == file
                        && issue_range.start_line == line
                        && issue_range.start_column == column
                });

                if let Some(issue) = find_multiline_note {
                    issue.message.push_str(&format!(" {}", message));
                    continue;
                }

                issues.push(Issue {
                    tool: plugin_name.into(),
                    message,
                    category: Category::Lint.into(),
                    level: Level::Low.into(),
                    rule_key: "note".into(),
                    location: Some(Location {
                        path: file,
                        range: Some(Range {
                            start_line: line,
                            start_column: column,
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                });
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
        let input = r###"
basic.in.py:1:1: error: Library stubs not installed for "google.protobuf.descriptor_pb2"  [import-untyped]
basic.in.py:1:1: note: Hint: "python3 -m pip install types-protobuf"
basic.in.py:1:1: note: (or run "mypy --install-types" to install all missing stub packages)
basic.in.py:1:1: note: See https://mypy.readthedocs.io/en/stable/running_mypy.html#missing-imports
basic.in.py:1:1: error: Library stubs not installed for "google.protobuf"  [import-untyped]
basic.in.py:13:10: error: Argument 1 to "greeting" has incompatible type "int"; expected "str"  [arg-type]
basic.in.py:14:10: error: Argument 1 to "greeting" has incompatible type "bytes"; expected "str"  [arg-type]
basic.in.py:15:5: error: "printer" does not return a value (it only ever returns None)  [func-returns-value]
basic.in.py:16:10: error: Incompatible types in assignment (expression has type "int", variable has type "str")  [assignment]
Found 6 errors in 1 file (checked 1 source file)
        "###;

        let issues = Mypy::default().parse("mypy", input);
        insta::assert_yaml_snapshot!(issues.unwrap(), @r#"
        - tool: mypy
          ruleKey: import-untyped
          message: "Library stubs not installed for \"google.protobuf.descriptor_pb2\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 1
              startColumn: 1
        - tool: mypy
          ruleKey: note
          message: "Hint: \"python3 -m pip install types-protobuf\" (or run \"mypy --install-types\" to install all missing stub packages) See https://mypy.readthedocs.io/en/stable/running_mypy.html#missing-imports"
          level: LEVEL_LOW
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 1
              startColumn: 1
        - tool: mypy
          ruleKey: import-untyped
          message: "Library stubs not installed for \"google.protobuf\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 1
              startColumn: 1
        - tool: mypy
          ruleKey: arg-type
          message: "Argument 1 to \"greeting\" has incompatible type \"int\"; expected \"str\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 13
              startColumn: 10
        - tool: mypy
          ruleKey: arg-type
          message: "Argument 1 to \"greeting\" has incompatible type \"bytes\"; expected \"str\""
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 14
              startColumn: 10
        - tool: mypy
          ruleKey: func-returns-value
          message: "\"printer\" does not return a value (it only ever returns None)"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 15
              startColumn: 5
        - tool: mypy
          ruleKey: assignment
          message: "Incompatible types in assignment (expression has type \"int\", variable has type \"str\")"
          level: LEVEL_HIGH
          category: CATEGORY_LINT
          location:
            path: basic.in.py
            range:
              startLine: 16
              startColumn: 10
        "#);
    }
}
