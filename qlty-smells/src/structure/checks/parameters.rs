use qlty_analysis::code::{capture_by_name_option, capture_source, File};
use qlty_types::analysis::v1::{Issue, Level};
use qlty_types::calculate_effort_minutes;
use std::sync::Arc;
use tree_sitter::Tree;

use super::issue_for;

pub const CHECK_NAME: &str = "function-parameters";

const BASE_EFFORT_MINUTES: u32 = 15;
const EFFORT_MINUTES_PER_VALUE_DELTA: u32 = 2;

pub fn check(threshold: usize, source_file: Arc<File>, tree: &Tree) -> Vec<Issue> {
    let mut issues: Vec<Issue> = Vec::new();

    let language = source_file.language();
    let query = language.function_declaration_query();

    let mut query_cursor = tree_sitter::QueryCursor::new();
    query_cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

    let all_matches =
        query_cursor.matches(query, tree.root_node(), source_file.contents.as_bytes());

    for function_match in all_matches {
        let parameters_capture = capture_by_name_option(query, "parameters", &function_match);
        if parameters_capture.is_none() {
            continue;
        }
        let parameters_node = parameters_capture.unwrap().node;
        let parameter_names = language.get_parameter_names(parameters_node, &source_file);

        if parameter_names.len() >= threshold {
            let value_delta = parameter_names.len() as u32 - threshold as u32;
            let message = format!(
                "Function with many parameters (count = {}): {}",
                parameter_names.len(),
                capture_source(query, "name", &function_match, &source_file)
            );

            issues.push(Issue {
                rule_key: CHECK_NAME.to_string(),
                message,
                level: Level::Medium.into(),
                value: parameter_names.len() as u32,
                value_delta,
                effort_minutes: calculate_effort_minutes(
                    value_delta,
                    BASE_EFFORT_MINUTES,
                    EFFORT_MINUTES_PER_VALUE_DELTA,
                ),
                ..issue_for(&source_file, &parameters_node)
            });
        }
    }

    issues
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parameters_not_found() {
        let source_file = Arc::new(File::from_string(
            "python",
            r#"
                def foo(a, b):
                    pass
            "#,
        ));
        assert_eq!(0, check(3, source_file.clone(), &source_file.parse()).len());
    }

    #[test]
    fn parameters_found() {
        let source_file = Arc::new(File::from_string(
            "python",
            r#"def foo(a, b, c, d, e, f):
                    pass"#
                .trim(),
        ));
        insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
        - tool: qlty
          driver: structure
          ruleKey: function-parameters
          message: "Function with many parameters (count = 6): foo"
          level: LEVEL_MEDIUM
          language: LANGUAGE_PYTHON
          category: CATEGORY_STRUCTURE
          snippet: "(a, b, c, d, e, f)"
          snippetWithContext: "def foo(a, b, c, d, e, f):\n                    pass"
          effortMinutes: 25
          value: 6
          valueDelta: 5
          location:
            path: STRING
            range:
              startLine: 1
              startColumn: 8
              endLine: 1
              endColumn: 26
              startByte: 7
              endByte: 25
        "#);
    }

    #[test]
    fn parameters_self() {
        let source_file = Arc::new(File::from_string(
            "python",
            r#"def foo(self, b, c):
                    pass"#
                .trim(),
        ));
        let result = check(3, source_file.clone(), &source_file.parse());
        assert!(result.is_empty());
    }

    #[test]
    fn parameters_not_found_typescript() {
        let source_file = Arc::new(File::from_string(
            "typescript",
            r#"
            function foo(a: any, b: any) {}
            "#,
        ));
        assert_eq!(0, check(3, source_file.clone(), &source_file.parse()).len());
    }

    #[test]
    fn parameters_found_typescript() {
        let source_file = Arc::new(File::from_string(
            "typescript",
            r#"function foo(a: any, b: any, c: any, d: any, e: any, f: any) {}"#.trim(),
        ));
        insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
        - tool: qlty
          driver: structure
          ruleKey: function-parameters
          message: "Function with many parameters (count = 6): foo"
          level: LEVEL_MEDIUM
          language: LANGUAGE_TYPESCRIPT
          category: CATEGORY_STRUCTURE
          snippet: "(a: any, b: any, c: any, d: any, e: any, f: any)"
          snippetWithContext: "function foo(a: any, b: any, c: any, d: any, e: any, f: any) {}"
          effortMinutes: 25
          value: 6
          valueDelta: 5
          location:
            path: STRING
            range:
              startLine: 1
              startColumn: 13
              endLine: 1
              endColumn: 61
              startByte: 12
              endByte: 60
        "#);
    }

    mod ruby {
        use super::*;

        #[test]
        fn singleton_method_with_parameters() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                def self.bar(dog, cat)
                end
                "#,
            ));
            assert_eq!(1, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn singleton_method_no_parameters() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                def self.bar
                end
                "#,
            ));
            assert_eq!(0, check(1, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn no_parameters() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                def foo
                end
                "#,
            ));
            assert_eq!(0, check(3, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn parameters_not_found() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                def foo(a, b)
                end
                "#,
            ));
            assert_eq!(0, check(3, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn parameters_not_found_variable_assignment() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                path = "/repos/#{repo.id}/comparisons/#{comparison_commit_sha}...#{commit_sha}/coverage"
                "#,
            ));
            assert_eq!(0, check(3, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn parameters_found() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"def foo(a, b, c, d, e, f)
                end"#
                    .trim(),
            ));
            insta::assert_yaml_snapshot!(check(5, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: function-parameters
              message: "Function with many parameters (count = 6): foo"
              level: LEVEL_MEDIUM
              language: LANGUAGE_RUBY
              category: CATEGORY_STRUCTURE
              snippet: "(a, b, c, d, e, f)"
              snippetWithContext: "def foo(a, b, c, d, e, f)\n                end"
              effortMinutes: 17
              value: 6
              valueDelta: 1
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 8
                  endLine: 1
                  endColumn: 26
                  startByte: 7
                  endByte: 25
            "#);
        }
    }
}
