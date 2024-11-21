use qlty_analysis::code::{File, NodeExt, Visitor};
use qlty_analysis::Language;
use qlty_types::analysis::v1::{Issue, Level};
use qlty_types::calculate_effort_minutes;
use std::sync::Arc;
use tree_sitter::{Tree, TreeCursor};

use super::issue_for;

pub const CHECK_NAME: &str = "nested-control-flow";

const BASE_EFFORT_MINUTES: u32 = 15;
const EFFORT_MINUTES_PER_VALUE_DELTA: u32 = 10;

pub fn check(threshold: usize, source_file: Arc<File>, tree: &Tree) -> Vec<Issue> {
    let mut processor = Processor::new(source_file, threshold);
    processor.process_node(&mut tree.root_node().walk());
    processor.issues
}

pub struct Processor {
    threshold: usize,
    issues: Vec<Issue>,
    level: usize,
    source_file: Arc<File>,
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

    fn on_control_node(&mut self, cursor: &mut TreeCursor) {
        self.level += 1;

        if self.level == self.threshold {
            let message = format!("Deeply nested control flow (level = {})", self.level);

            self.issues.push(Issue {
                rule_key: CHECK_NAME.to_string(),
                message,
                level: Level::Medium.into(),
                value: self.level as u32,
                value_delta: 0,
                effort_minutes: calculate_effort_minutes(
                    0,
                    BASE_EFFORT_MINUTES,
                    EFFORT_MINUTES_PER_VALUE_DELTA,
                ),
                ..issue_for(&self.source_file, &cursor.node())
            });
        }
        self.process_children(cursor);
        self.level -= 1;
    }
}

impl Visitor for Processor {
    fn language(&self) -> &Box<dyn Language + Sync> {
        self.source_file.language()
    }

    fn visit_if(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let parent_kind = node.parent().unwrap().kind();

        if self
            .source_file
            .language()
            .else_nodes()
            .contains(&parent_kind)
            || node.is_if_statement_alternative(self.language())
        {
            self.process_children(cursor);
        } else {
            self.on_control_node(cursor);
        }
    }

    fn visit_switch(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_loop(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_ternary(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_except(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod go {
        use super::*;

        #[test]
        fn nested_control_not_found() {
            let source_file = Arc::new(File::from_string(
                "go",
                r#"
                    if foo {
                        // empty
                    }
                "#,
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_not_found_if_threshold_not_breached() {
            let source_file = Arc::new(File::from_string(
                "go",
                r#"
                    if bar {
                        if baz {
                            if qux {
                                if quux {
                                    fmt.Println("Not deeply nested enough!")
                                }
                            }
                        }
                    }
                "#
                .trim(),
            ));
            assert_eq!(0, check(5, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_found_if_above_threshold() {
            let source_file = Arc::new(File::from_string(
                "go",
                r#"
                    if bar {
                        if baz {
                            if qux {
                                if quux {
                                    // empty
                                }
                            }
                        }
                    }
                "#,
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: nested-control-flow
              message: Deeply nested control flow (level = 1)
              level: LEVEL_MEDIUM
              language: LANGUAGE_GO
              category: CATEGORY_STRUCTURE
              snippet: "if bar {\n                        if baz {\n                            if qux {\n                                if quux {\n                                    // empty\n                                }\n                            }\n                        }\n                    }"
              snippetWithContext: "\n                    if bar {\n                        if baz {\n                            if qux {\n                                if quux {\n                                    // empty\n                                }\n                            }\n                        }\n                    }\n                "
              effortMinutes: 15
              value: 1
              location:
                path: STRING
                range:
                  startLine: 2
                  startColumn: 21
                  endLine: 10
                  endColumn: 22
                  startByte: 21
                  endByte: 298
            "#);
        }

        #[test]
        fn nesting_not_found_for_elif() {
            let source_file = Arc::new(File::from_string(
                "go",
                r#"
                    if foo == 1 {
                        result = "bar1"
                    } else if foo == 2 {
                        result = "bar2"
                    } else if foo == 3 {
                        result = "bar3"
                    } else if foo == 4 {
                        result = "bar4"
                    } else if foo == 5 {
                        result = "bar5"
                    } else {
                        result = "bar6"
                    }
                "#
                .trim(),
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }
    }

    mod python {
        use super::*;

        #[test]
        fn nested_control_not_found() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                    if foo:
                        pass
                "#,
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_not_found_if_threshold_not_breached() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                if bar:
                    if baz:
                        if qux:
                            if quux:
                                print("Not deeply nested enough!")
                "#
                .trim(),
            ));
            assert_eq!(0, check(5, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_found_if_above_threshold() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                    if bar:
                        if baz:
                            if qux:
                                if quux:
                                    pass
                "#,
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: nested-control-flow
              message: Deeply nested control flow (level = 1)
              level: LEVEL_MEDIUM
              language: LANGUAGE_PYTHON
              category: CATEGORY_STRUCTURE
              snippet: "if bar:\n                        if baz:\n                            if qux:\n                                if quux:\n                                    pass"
              snippetWithContext: "\n                    if bar:\n                        if baz:\n                            if qux:\n                                if quux:\n                                    pass\n                "
              effortMinutes: 15
              value: 1
              location:
                path: STRING
                range:
                  startLine: 2
                  startColumn: 21
                  endLine: 6
                  endColumn: 41
                  startByte: 21
                  endByte: 178
            "#);
        }

        #[test]
        fn nesting_not_found_for_elif() {
            let source_file = Arc::new(File::from_string(
                "python",
                r#"
                if foo == 1:
                    result = "bar1"
                elif foo == 2:
                    result = "bar2"
                elif foo == 3:
                    result = "bar3"
                elif foo == 4:
                    result = "bar4"
                elif foo == 5:
                    result = "bar5"
                else:
                    result = "bar6"
                "#
                .trim(),
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }
    }

    mod typescript {
        use super::*;

        #[test]
        fn nested_control_not_found() {
            let source_file = Arc::new(File::from_string(
                "typescript",
                r#"
                componentDidUpdate(prevProps) {
                    if (
                        prevProps.teamId !== this.props.teamId ||
                        prevProps.applicationId !== this.props.applicationId ||
                        prevProps.chartSource !== this.props.chartSource ||
                        prevProps.startDate !== this.props.startDate ||
                        prevProps.endDate !== this.props.endDate ||
                        prevProps.dateRange !== this.props.dateRange
                    ) {
                    this.setState(
                        {
                        isLoading: true,
                        },
                        this.fetch
                    )
                    }
                }
                "#,
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_not_found_if_threshold_not_breached() {
            let source_file = Arc::new(File::from_string(
                "typescript",
                r#"
                if (bar) {
                    if (baz) {
                        if (qux) {
                            if (quux) {
                                console.log('Not deeply nested enough!');
                            }
                        }
                    }
                }
                "#
                .trim(),
            ));
            assert_eq!(0, check(5, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_found_if_threshold_breached() {
            let source_file = Arc::new(File::from_string(
                "typescript",
                r#"
                if (bar) {
                    if (baz) {
                        if (qux) {
                            if (quux) {
                                console.log('Deeply nested!');
                            }
                        }
                    }
                }
                "#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: nested-control-flow
              message: Deeply nested control flow (level = 1)
              level: LEVEL_MEDIUM
              language: LANGUAGE_TYPESCRIPT
              category: CATEGORY_STRUCTURE
              snippet: "if (bar) {\n                    if (baz) {\n                        if (qux) {\n                            if (quux) {\n                                console.log('Deeply nested!');\n                            }\n                        }\n                    }\n                }"
              snippetWithContext: "if (bar) {\n                    if (baz) {\n                        if (qux) {\n                            if (quux) {\n                                console.log('Deeply nested!');\n                            }\n                        }\n                    }\n                }"
              effortMinutes: 15
              value: 1
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 9
                  endColumn: 18
                  startByte: 0
                  endByte: 275
            "#);
        }

        #[test]
        fn nesting_not_found_for_else_if() {
            let source_file = Arc::new(File::from_string(
                "typescript",
                r#"
                let result: string;
                if (foo === 1) {
                    result = "bar1";
                } else if (foo === 2) {
                    result = "bar2";
                } else if (foo === 3) {
                    result = "bar3";
                } else if (foo === 4) {
                    result = "bar4";
                } else if (foo === 5) {
                    result = "bar5";
                } else {
                    result = "bar6";
                }
                "#
                .trim(),
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }
    }

    mod ruby {
        use super::*;

        #[test]
        fn nested_control_not_found() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                def not_nested(foo, bar)
                    if foo == "cat" && bar == "dog" || foo == "dog" && bar == "cat"
                        puts "Got a cat and a dog!"
                    else
                        puts "Got nothing"
                    end
                end
                "#,
            ));
            assert_eq!(0, check(4, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_not_found_if_threshold_not_breached() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                if bar
                    if baz
                        if qux
                            if quux
                                puts 'Not deeply nested enough!'
                            end
                        end
                    end
                end
                "#
                .trim(),
            ));
            assert_eq!(0, check(5, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_found_if_threshold_breached() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                if bar
                    if baz
                        if qux
                            if quux
                                puts 'Deeply nested!'
                            end
                        end
                    end
                end
                "#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: nested-control-flow
              message: Deeply nested control flow (level = 1)
              level: LEVEL_MEDIUM
              language: LANGUAGE_RUBY
              category: CATEGORY_STRUCTURE
              snippet: "if bar\n                    if baz\n                        if qux\n                            if quux\n                                puts 'Deeply nested!'\n                            end\n                        end\n                    end\n                end"
              snippetWithContext: "if bar\n                    if baz\n                        if qux\n                            if quux\n                                puts 'Deeply nested!'\n                            end\n                        end\n                    end\n                end"
              effortMinutes: 15
              value: 1
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 9
                  endColumn: 20
                  startByte: 0
                  endByte: 258
            "#);
        }

        #[test]
        fn nesting_not_found_for_elsif() {
            let source_file = Arc::new(File::from_string(
                "ruby",
                r#"
                if foo1
                    "bar1"
                elsif foo2
                    "bar2"
                elsif foo3
                    "bar3"
                elsif foo4
                    "bar4"
                elsif foo5
                    "bar5"
                else
                    "bar6"
                end
                "#
                .trim(),
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }
    }

    mod java {
        use super::*;

        #[test]
        fn nested_control_not_found() {
            let source_file = Arc::new(File::from_string(
                "java",
                r#"
                public not_nested(String foo, String bar) {
                    if(foo == "cat" && bar == "dog" || foo == "dog" && bar == "cat") {
                        System.out.println("Got a cat and a dog!");
                    } else {
                        System.out.println("Got nothing");
                    }
                }
                "#,
            ));
            assert_eq!(0, check(4, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_not_found_if_threshold_not_breached() {
            let source_file = Arc::new(File::from_string(
                "java",
                r#"
                if(bar) {
                    if(baz) {
                        if(qux) {
                            if(quux) {
                                System.out.println("Not deeply nested enough!");
                            }
                        }
                    }
                }
                "#
                .trim(),
            ));
            assert_eq!(0, check(5, source_file.clone(), &source_file.parse()).len());
        }

        #[test]
        fn nested_control_found_if_threshold_breached() {
            let source_file = Arc::new(File::from_string(
                "java",
                r#"
                if(bar) {
                    if(baz) {
                        if(qux) {
                            if(quux) {
                                System.out.println("Deeply nested!");
                            }
                        }
                    }
                }
                "#
                .trim(),
            ));
            insta::assert_yaml_snapshot!(check(1, source_file.clone(), &source_file.parse()), @r#"
            - tool: qlty
              driver: structure
              ruleKey: nested-control-flow
              message: Deeply nested control flow (level = 1)
              level: LEVEL_MEDIUM
              language: LANGUAGE_JAVA
              category: CATEGORY_STRUCTURE
              snippet: "if(bar) {\n                    if(baz) {\n                        if(qux) {\n                            if(quux) {\n                                System.out.println(\"Deeply nested!\");\n                            }\n                        }\n                    }\n                }"
              snippetWithContext: "if(bar) {\n                    if(baz) {\n                        if(qux) {\n                            if(quux) {\n                                System.out.println(\"Deeply nested!\");\n                            }\n                        }\n                    }\n                }"
              effortMinutes: 15
              value: 1
              location:
                path: STRING
                range:
                  startLine: 1
                  startColumn: 1
                  endLine: 9
                  endColumn: 18
                  startByte: 0
                  endByte: 278
            "#);
        }

        #[test]
        fn nesting_not_found_for_elsif() {
            let source_file = Arc::new(File::from_string(
                "java",
                r#"
                if(foo1) {
                    // bar1
                } else if(foo2) {
                    // bar2
                } else if(foo3) {
                    // bar3
                } else if(foo4) {
                    // bar4
                } else if(foo5) {
                    // bar5
                } else {
                    // bar6
                }
                "#
                .trim(),
            ));
            assert_eq!(0, check(2, source_file.clone(), &source_file.parse()).len());
        }
    }
}
