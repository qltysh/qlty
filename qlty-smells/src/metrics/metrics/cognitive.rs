use qlty_analysis::code::{File, NodeExt, NodeFilter, Visitor};
use qlty_analysis::Language;
use tree_sitter::Node;
use tree_sitter::TreeCursor;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    let mut complexity = CognitiveComplexity {
        count: 0,
        logic_level: 0,
        functions: vec![],
        counted_functions: vec![],
        filter,
        source_file,
        last_operator: None,
    };
    complexity.process_node(&mut node.walk());
    complexity.count
}

pub struct CognitiveComplexity<'a> {
    pub count: usize,
    logic_level: usize,
    functions: Vec<String>,
    counted_functions: Vec<String>,
    filter: &'a NodeFilter,
    source_file: &'a File,
    last_operator: Option<String>,
}

impl Visitor for CognitiveComplexity<'_> {
    fn language(&self) -> &Box<dyn Language + Sync> {
        self.source_file.language()
    }

    fn skip_node(&self, node: &Node) -> bool {
        !node.is_named() || self.filter.exclude(node)
    }

    fn visit_if(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();

        if self.is_elsif(&node) {
            self.process_children(cursor);
        } else if node.is_if_statement_alternative(self.language()) {
            self.visit_elsif(cursor);
        } else {
            self.on_control_node(cursor);
        }
    }

    fn visit_ternary(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_switch(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_loop(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_except(&mut self, cursor: &mut TreeCursor) {
        self.on_control_node(cursor);
    }

    fn visit_else(&mut self, cursor: &mut TreeCursor) {
        self.on_incrementor(cursor);
    }

    fn visit_elsif(&mut self, cursor: &mut TreeCursor) {
        self.on_incrementor(cursor);
    }

    fn visit_jump(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();

        if self.should_count_jump_node(&node) {
            self.on_incrementor(cursor);
        } else {
            self.process_children(cursor);
        }
    }

    fn visit_binary(&mut self, cursor: &mut TreeCursor) {
        if cursor.goto_first_child() {
            self.process_binary_child_node(cursor); // Process left child

            if cursor.goto_next_sibling() {
                self.process_boolean_operator_node(cursor); // Process operator

                if cursor.goto_next_sibling() {
                    self.process_binary_child_node(cursor); // Process right child
                }
            }

            cursor.goto_parent();
        }
    }

    fn visit_call(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();

        if self.is_recursive_call(&node) {
            self.on_incrementor(cursor);
        } else {
            self.process_children(cursor);
        }
    }

    fn visit_function(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let name = self.source_file.language().function_name_node(&node);

        let name_string = name
            .utf8_text(self.source_file.contents.as_bytes())
            .unwrap()
            .to_string();

        let decorator = self.source_file.language().is_decorator_function(&node);

        self.functions.push(name_string.clone());

        if !decorator {
            self.counted_functions.push(name_string);
        }

        if self.counted_functions.len() > 1 {
            self.on_nested_function(cursor);
        } else {
            self.process_children(cursor);
        }

        self.functions.pop();

        if !decorator {
            self.counted_functions.pop();
        }
    }

    fn visit_closure(&mut self, cursor: &mut TreeCursor) {
        self.on_nested_function(cursor);
    }

    fn visit_conditional_assignment(&mut self, cursor: &mut TreeCursor) {
        self.on_incrementor(cursor);
    }

    fn visit_block(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();

        if node.is_if_statement_alternative(self.language()) {
            self.visit_else(cursor);
        } else {
            self.process_children(cursor);
        }
    }
}

impl<'a> CognitiveComplexity<'a> {
    fn is_elsif(&self, node: &Node) -> bool {
        let parent = node.parent().unwrap();
        let parent_kind = parent.kind();

        self.language().else_nodes().contains(&parent_kind)
    }

    fn should_count_jump_node(&self, node: &Node) -> bool {
        let child = node.named_child(0);

        !self.language().has_labeled_jumps()
            || (child.is_some() && self.language().is_jump_label(&child.unwrap()))
    }

    fn is_recursive_call(&self, node: &Node) -> bool {
        let (receiver, function_name) = self
            .source_file
            .language()
            .call_identifiers(self.source_file, node);

        receiver.as_deref() == self.language().self_keyword()
            && !self.functions.is_empty()
            && function_name == *self.functions.last().unwrap()
    }

    fn process_binary_child_node(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();

        if self.language().binary_nodes().contains(&node.kind()) {
            self.visit_binary(cursor);
        } else {
            self.process_node(cursor);
        }
    }

    fn process_boolean_operator_node(&mut self, cursor: &mut TreeCursor) {
        let operator_node = cursor.node();
        let operator = operator_node
            .utf8_text(self.source_file.contents.as_bytes())
            .unwrap();

        if self.language().boolean_operator_nodes().contains(&operator) {
            if let Some(last_operator) = self.last_operator.as_ref() {
                if last_operator != &operator {
                    self.on_incrementor(cursor);
                }
            } else {
                self.on_incrementor(cursor);
            }
            self.last_operator = Some(operator.to_string());
        }
    }

    fn on_incrementor(&mut self, cursor: &mut TreeCursor) {
        self.increment_counter(1);
        self.process_children(cursor);
    }

    fn on_control_node(&mut self, cursor: &mut TreeCursor) {
        self.logic_level += 1;
        self.increment_counter(self.logic_level);
        self.process_children(cursor);
        self.logic_level -= 1;
    }

    fn on_nested_function(&mut self, cursor: &mut TreeCursor) {
        self.logic_level += 1;
        self.process_children(cursor);
        self.logic_level -= 1;
    }

    fn increment_counter(&mut self, number: usize) {
        self.count += number;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod java {
        use super::*;

        #[test]
        fn count_if() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        if (Boolean.TRUE) { // +1
                            return;
                        }

                        if (Boolean.TRUE) { // +1
                            return;
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_statement_modifiers() {
            let source_file = File::from_string(
                "ruby",
                r#"
                def foo(bar)
                    return 1 unless bar # +1
                    return 2 if bar # +1
                    if bar # +1
                        return 3
                    end
                    unless bar # +1
                        return 4
                    end
                end
                "#,
            );
            assert_eq!(
                4,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_else_if() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        if (Boolean.TRUE) { // +1
                            return;
                        } else if (Boolean.TRUE) { // +1
                            return;
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_else() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        if (Boolean.TRUE) { // +1
                            return;
                        } else { // +1
                            return;
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_ternary() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static int foo() {
                        return Boolean.TRUE ? 1 : 0; // +1
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_switch() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static String foo(int number) {
                        switch (number) { // +1
                            case 1:
                                return "one";
                            case 2:
                                return "a couple";
                            case 3:
                                return "a few";
                            default:
                                return "lots";
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_for() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        for (int i=1; i<=10; i++) { // +1
                            System.out.println("Count is: " + i);
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_catch() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        try {
                            return;
                        } catch (IOException e) { // +1
                            return;
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        // According to SonarSource specs: https://www.sonarsource.com/docs/CognitiveComplexity.pdf
        // jump nodes do not count nesting increments
        fn count_labeled_break() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        while (Boolean.TRUE) { // +1
                            break label; // +1
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn dont_count_unlabeled_break() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        while (Boolean.TRUE) { // +1
                            break;
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        // According to SonarSource specs: https://www.sonarsource.com/docs/CognitiveComplexity.pdf
        // jump nodes do not count nesting increments
        fn count_labeled_continue() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        while (Boolean.TRUE) { // +1
                            continue label; // +1
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                2,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn dont_count_unlabeled_continue() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        while (Boolean.TRUE) { // +1
                            continue;
                        }
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_first_logical_operator() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static void foo() {
                        return (Boolean.TRUE && Boolean.TRUE); // +1
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_change_of_logical_operator() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static Boolean foo() {
                        return (Boolean.TRUE && Boolean.TRUE || Boolean.FALSE || Boolean.TRUE && Boolean.FALSE); // +3
                    }
                }
                "#,
            );
            assert_eq!(
                3,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_enclosed_expression() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static Boolean foo() {
                        return (Boolean.TRUE || (Boolean.FALSE || Boolean.TRUE)); // +2
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn count_method_recursion() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                    public static Boolean foo() {
                        return foo(); // +1
                    }
                }
                "#,
            );
            assert_eq!(
                1,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn nesting_if() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                  public static void foo() {
                    if (Boolean.TRUE) { // +1 (nesting=0)
                      if (Boolean.TRUE) { // +2 (nesting=1)
                        if (Boolean.TRUE) { // +3 (nesting=2)
                          return;
                        }
                      }
                    }
                  }
                }
                "#,
            );

            assert_eq!(
                6,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn nesting_else_if() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                  public static void foo() {
                    if (Boolean.TRUE) { // +1 (nesting=0)
                      return;
                    } else if (Boolean.TRUE) { // +1
                      if (Boolean.TRUE) { // +2 (nesting=1)
                        return;
                      }
                    }
                  }
                }
                "#,
            );

            assert_eq!(
                4,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn nesting_else() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                  public static void foo() {
                    if (Boolean.TRUE) { // +1 (nesting=0)
                      return;
                    } else { // +1
                      if (Boolean.TRUE) { // +2 (nesting=1)
                        return;
                      }
                    }
                  }
                }
                "#,
            );

            assert_eq!(
                4,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }

        #[test]
        fn nesting_methods() {
            let source_file = File::from_string(
                "java",
                r#"
                public class Test {
                  public static void foo() {
                    class Inner {
                      static void bar() {
                        if (Boolean.TRUE) { // +2
                          if (Boolean.TRUE) { // +2 (nesting=1)
                            if (Boolean.TRUE) { // +3 (nesting=2)
                              return;
                            }
                          }
                        }

                        class SuperInner {
                          static void baz() {
                            if (Boolean.TRUE) { // +2 (nesting=1)
                              if (Boolean.TRUE) { // +3 (nesting=2)
                                if (Boolean.TRUE) { // +4 (nesting=3)
                                  return;
                                }
                              }
                            }
                          } // complexity=6
                        }
                      } // complexity=12
                    }
                  } // complexity=21
                }
                "#,
            );

            assert_eq!(
                21,
                count(
                    &source_file,
                    &source_file.parse().root_node(),
                    &NodeFilter::empty()
                )
            );
        }
    }
}
