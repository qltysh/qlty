use qlty_analysis::code::{File, Visitor};
use qlty_types::analysis::v1::{Issue, Level};
use qlty_types::calculate_effort_minutes;
use std::sync::Arc;
use tree_sitter::{Tree, TreeCursor};

use super::issue_for;

pub const CHECK_NAME: &str = "boolean-logic";

const BASE_EFFORT_MINUTES: u32 = 10;
const EFFORT_MINUTES_PER_VALUE_DELTA: u32 = 12;

pub fn check(threshold: usize, source_file: Arc<File>, tree: &Tree) -> Vec<Issue> {
    let mut processor = Processor::new(source_file, threshold);
    processor.process_node(&mut tree.root_node().walk());
    processor.issues
}

pub struct Processor {
    source_file: Arc<File>,
    threshold: usize,
    level: usize,
    max_level: usize,
    issues: Vec<Issue>,
}

impl Processor {
    fn new(source_file: Arc<File>, threshold: usize) -> Self {
        Self {
            threshold,
            issues: Vec::new(),
            level: 0,
            max_level: 0,
            source_file,
        }
    }
}

impl Visitor for Processor {
    fn source_file(&self) -> &File {
        &self.source_file
    }

    fn visit_binary(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();

        let operator = if self.language().has_field_names() {
            node.child(1)
                .unwrap()
                .utf8_text(self.source_file.contents.as_bytes())
                .unwrap()
        } else {
            node.child_by_field_name("operator")
                .unwrap()
                .utf8_text(self.source_file.contents.as_bytes())
                .unwrap()
        };

        let normalized_operator = self.language().normalize_identifier(operator);
        if self
            .language()
            .boolean_operator_nodes()
            .contains(&normalized_operator.as_str())
        {
            let first_issue = self.issues.len();
            if self.level == 0 {
                self.max_level = 0;
            }
            self.level += 1;
            self.max_level = self.max_level.max(self.level);

            if self.level == self.threshold {
                let message = "Complex binary expression";
                self.issues.push(Issue {
                    rule_key: CHECK_NAME.to_string(),
                    message: message.to_string(),
                    level: Level::Medium.into(),
                    value: self.level as u32,
                    value_delta: 0,
                    effort_minutes: calculate_effort_minutes(
                        0,
                        BASE_EFFORT_MINUTES,
                        EFFORT_MINUTES_PER_VALUE_DELTA,
                    ),
                    ..issue_for(&self.source_file, &node, self.threshold, self.level)
                });
            }

            self.process_children(cursor);

            self.level -= 1;
            if self.level == 0 {
                for issue in &mut self.issues[first_issue..] {
                    issue.set_property_number("actual", self.max_level as f64);
                }
            }
        } else {
            self.process_children(cursor);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn actual_reports_full_operator_depth() {
        let source_file = Arc::new(File::from_string(
            "rust",
            "fn f() { a || b || c || d || e || f || g || h || i; }",
        ));
        let issues = check(4, source_file.clone(), &source_file.parse());

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].get_property_number("threshold"), 4.0);
        assert_eq!(issues[0].get_property_number("actual"), 8.0);
        assert_eq!(issues[0].value, 4);
        assert_eq!(issues[0].value_delta, 0);
    }

    #[test]
    fn actual_measures_depth_instead_of_total_operators() {
        let source_file = Arc::new(File::from_string(
            "rust",
            "fn f() { ((a || b) || (c || d)) || ((e || f) || (g || h)); }",
        ));
        let issues = check(2, source_file.clone(), &source_file.parse());

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].get_property_number("actual"), 3.0);
        assert_eq!(issues[1].get_property_number("actual"), 3.0);
    }

    #[test]
    fn actual_resets_between_expressions() {
        let source_file = Arc::new(File::from_string(
            "python",
            "a or b or c or d or e or f or g or h or i\na or b or c or d or e",
        ));
        let issues = check(4, source_file.clone(), &source_file.parse());

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].get_property_number("actual"), 8.0);
        assert_eq!(issues[1].get_property_number("actual"), 4.0);
    }

    #[test]
    fn actual_is_independent_of_line_wrapping() {
        let compact = Arc::new(File::from_string(
            "java",
            "class Example { boolean f() { return a || b || c || d || e; } }",
        ));
        let wrapped = Arc::new(File::from_string(
            "java",
            "class Example { boolean f() { return a\n || b\n || c\n || d\n || e; } }",
        ));
        let compact_issues = check(4, compact.clone(), &compact.parse());
        let wrapped_issues = check(4, wrapped.clone(), &wrapped.parse());

        assert_eq!(compact_issues[0].get_property_number("actual"), 4.0);
        assert_eq!(wrapped_issues[0].get_property_number("actual"), 4.0);
    }

    mod python {
        use super::*;

        #[test]
        fn boolean_logic_not_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                def foo(a, b, c, d):
                    x = a + b - c + d
                    return x
            "#,
            ));
            assert_eq!(0, check(1, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn boolean_logic_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                if foo and bar and baz and qux:
                    pass
            "#
                .trim(),
            ));

            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), { "[].properties" => insta::sorted_redaction() }, @r#"
            - tool: qlty
              driver: structure
              ruleKey: boolean-logic
              message: Complex binary expression
              level: LEVEL_MEDIUM
              language: LANGUAGE_PYTHON
              category: CATEGORY_STRUCTURE
              snippet: foo and bar and baz and qux
              snippetWithContext: "if foo and bar and baz and qux:\n                    pass"
              effortMinutes: 10
              value: 1
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 4
                  endLine: 1
                  endColumn: 31
                  startByte: 3
                  endByte: 30
              properties:
                actual: 3
                threshold: 1
            "#);
        }
    }

    mod elixir {
        use super::*;

        #[test]
        fn boolean_logic_found() {
            let source_file = Arc::new(File::from_string(
                "elixir",
                "defmodule M do\n def f(a, b, c, d, e), do: a and b and c and d and e\nend",
            ));
            assert_eq!(1, check(4, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn boolean_logic_not_found_below_threshold() {
            let source_file = Arc::new(File::from_string(
                "elixir",
                "defmodule M do\n def f(a, b), do: a and b\nend",
            ));
            assert_eq!(0, check(4, source_file.clone(), &source_file.parse()).len());
        }
    }
}
