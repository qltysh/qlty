use qlty_analysis::code::{capture_by_name, capture_source};
use qlty_analysis::code::{File, NodeCounter};
use qlty_types::analysis::v1::{Issue, Level};
use qlty_types::calculate_effort_minutes;
use std::sync::Arc;
use tree_sitter::Tree;

use super::issue_for;

pub const CHECK_NAME: &str = "return-statements";

const BASE_EFFORT_MINUTES: u32 = 15;
const EFFORT_MINUTES_PER_VALUE_DELTA: u32 = 5;

pub fn check(threshold: usize, source_file: Arc<File>, tree: &Tree) -> Vec<Issue> {
    let mut issues: Vec<Issue> = Vec::new();

    let language = source_file.language();
    let function_query = language.function_declaration_query();

    let mut query_cursor = tree_sitter::QueryCursor::new();
    query_cursor.set_match_limit(qlty_analysis::code::QUERY_MATCH_LIMIT as u32);

    let all_matches = query_cursor.matches(
        function_query,
        tree.root_node(),
        source_file.contents.as_bytes(),
    );

    for function_match in all_matches {
        let function_capture =
            capture_by_name(function_query, "definition.function", &function_match);

        let return_count = NodeCounter::count(
            &source_file,
            &source_file.language().return_nodes(),
            &function_capture.node,
        );

        if return_count >= threshold {
            let value_delta = return_count as u32 - threshold as u32;
            let message = format!(
                "Function with many returns (count = {}): {}",
                return_count,
                capture_source(function_query, "name", &function_match, &source_file)
            );

            issues.push(Issue {
                rule_key: CHECK_NAME.to_string(),
                message,
                level: Level::Medium.into(),
                value: return_count as u32,
                value_delta,
                effort_minutes: calculate_effort_minutes(
                    value_delta,
                    BASE_EFFORT_MINUTES,
                    EFFORT_MINUTES_PER_VALUE_DELTA,
                ),
                ..issue_for(&source_file, &function_capture.node)
            });
        }
    }

    issues
}

#[cfg(test)]
mod test {
    use super::*;

    mod python {
        use super::*;

        #[test]
        fn returns_not_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                    def foo():
                        return
                "#,
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn returns_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"def foo():
                        return
                        return
                        return"#
                    .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: return-statements
              message: "Function with many returns (count = 3): foo"
              level: LEVEL_MEDIUM
              language: LANGUAGE_PYTHON
              category: CATEGORY_STRUCTURE
              snippet: "def foo():\n                        return\n                        return\n                        return"
              snippetWithContext: "def foo():\n                        return\n                        return\n                        return"
              effortMinutes: 25
              value: 3
              valueDelta: 2
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 4
                  endColumn: 31
                  startByte: 0
                  endByte: 103
            "#);
        }
    }

    mod typescript {
        use super::*;

        #[test]
        fn returns_not_found() {
            let source_file = Arc::new(File::from_string(
                "typescript",
                r#"
                function foo() {
                    return;
                }
                "#,
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn returns_found() {
            let source_file = Arc::new(File::from_string(
                "typescript",
                r#"function foo() {
                    return;
                    return;
                    return;
                }"#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: return-statements
              message: "Function with many returns (count = 3): foo"
              level: LEVEL_MEDIUM
              language: LANGUAGE_TYPESCRIPT
              category: CATEGORY_STRUCTURE
              snippet: "function foo() {\n                    return;\n                    return;\n                    return;\n                }"
              snippetWithContext: "function foo() {\n                    return;\n                    return;\n                    return;\n                }"
              effortMinutes: 25
              value: 3
              valueDelta: 2
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 5
                  endColumn: 18
                  startByte: 0
                  endByte: 118
            "#);
        }
    }

    mod ruby {
        use super::*;

        #[test]
        fn returns_not_found() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                    def foo
                    end
                "#,
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn returns_found() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                    def foo(thing)
                        return
                        return
                        return
                        return
                    end
                "#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: return-statements
              message: "Function with many returns (count = 4): foo"
              level: LEVEL_MEDIUM
              language: LANGUAGE_RUBY
              category: CATEGORY_STRUCTURE
              snippet: "def foo(thing)\n                        return\n                        return\n                        return\n                        return\n                    end"
              snippetWithContext: "def foo(thing)\n                        return\n                        return\n                        return\n                        return\n                    end"
              effortMinutes: 30
              value: 4
              valueDelta: 3
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 6
                  endColumn: 24
                  startByte: 0
                  endByte: 162
            "#);
        }
    }
}
