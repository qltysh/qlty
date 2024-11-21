use qlty_analysis::code::{File, Visitor};
use qlty_analysis::Language;
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
    issues: Vec<Issue>,
}

impl Processor {
    fn new(source_file: Arc<File>, threshold: usize) -> Self {
        Self {
            threshold,
            issues: Vec::new(),
            level: 0,
            source_file,
        }
    }
}

impl Visitor for Processor {
    fn language(&self) -> &Box<dyn Language + Sync> {
        self.source_file.language()
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

        if self.language().boolean_operator_nodes().contains(&operator) {
            self.level += 1;

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
                    ..issue_for(&self.source_file, &node)
                });
            }

            self.process_children(cursor);

            self.level -= 1;
        } else {
            self.process_children(cursor);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
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
            "#);
        }
    }
}
