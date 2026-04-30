use crate::metrics::metrics::complexity;
use qlty_analysis::code::{capture_by_name, capture_source, File, NodeFilter};
use qlty_types::analysis::v1::{Issue, Level};
use qlty_types::calculate_effort_minutes;
use std::collections::HashMap;
use std::sync::Arc;
use tree_sitter::Tree;

use super::issue_for;

pub const CHECK_NAME: &str = "function-complexity";

const BASE_EFFORT_MINUTES: u32 = 10;
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

        let count = complexity(&source_file, &function_capture.node, &NodeFilter::empty());
        let function_name = capture_source(function_query, "name", &function_match, &source_file);
        let mut partial_fingerprints = HashMap::new();

        partial_fingerprints.insert("function.name".to_string(), function_name.clone());

        if count >= threshold {
            let value_delta = count as u32 - threshold as u32;

            let message = format!(
                "Function with high complexity (count = {}): {}",
                count, function_name,
            );

            issues.push(Issue {
                rule_key: CHECK_NAME.to_string(),
                message,
                level: Level::Medium.into(),
                value: count as u32,
                value_delta,
                effort_minutes: calculate_effort_minutes(
                    value_delta,
                    BASE_EFFORT_MINUTES,
                    EFFORT_MINUTES_PER_VALUE_DELTA,
                ),
                partial_fingerprints,
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
        fn function_complexity_not_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                    def foo():
                        pass
                "#,
            ));
            assert_eq!(0, check(1, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn function_complexity_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                    def foo():
                        if bar:
                            if baz:
                                if qux:
                                    if quux:
                                        if quuz:
                                            pass
                "#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: function-complexity
              message: "Function with high complexity (count = 15): foo"
              level: LEVEL_MEDIUM
              language: LANGUAGE_PYTHON
              category: CATEGORY_STRUCTURE
              snippet: "def foo():\n                        if bar:\n                            if baz:\n                                if qux:\n                                    if quux:\n                                        if quuz:\n                                            pass"
              snippetWithContext: "def foo():\n                        if bar:\n                            if baz:\n                                if qux:\n                                    if quux:\n                                        if quuz:\n                                            pass"
              effortMinutes: 80
              value: 15
              valueDelta: 14
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 7
                  endColumn: 49
                  startByte: 0
                  endByte: 261
              partialFingerprints:
                function.name: foo
            "#);
        }
    }
}
