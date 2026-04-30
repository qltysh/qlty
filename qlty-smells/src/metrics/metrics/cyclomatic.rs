use qlty_analysis::code::{File, NodeFilter, Visitor};
use qlty_analysis::Language;
use tree_sitter::Node;
use tree_sitter::TreeCursor;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    let mut processor = CyclomaticComplexity::new(source_file, filter);
    processor.process_node(&mut node.walk());
    processor.count
}

pub struct CyclomaticComplexity<'a> {
    pub count: usize,
    source_file: &'a File,
    filter: &'a NodeFilter,
}

impl<'a> CyclomaticComplexity<'a> {
    fn new(source_file: &'a File, filter: &'a NodeFilter) -> Self {
        Self {
            count: 1,
            source_file,
            filter,
        }
    }
}

impl Visitor for CyclomaticComplexity<'_> {
    fn language(&self) -> &Box<dyn Language + Sync> {
        self.source_file.language()
    }

    fn skip_node(&self, node: &Node) -> bool {
        !node.is_named() || self.filter.exclude(node)
    }

    fn visit_if(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_elsif(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_conditional_assignment(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_ternary(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_case(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_loop(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_except(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_try_expression(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_binary(&mut self, cursor: &mut TreeCursor) {
        self.count += 1;
        self.process_children(cursor);
    }

    fn visit_call(&mut self, cursor: &mut TreeCursor) {
        let node = cursor.node();
        let (_, method_name) = self.language().call_identifiers(self.source_file, &node);

        if self
            .language()
            .iterator_method_identifiers()
            .contains(&method_name.as_str())
        {
            self.count += 1;
        }

        self.process_children(cursor);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod java {
        use super::*;

        #[test]
        fn cyclo_basic() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    x = 1;
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
        fn cyclo_if() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
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
        fn cyclo_if_else() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    } else {
                        y = 2;
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
        fn cyclo_if_elsif() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    } else if(z) {
                        y = 2;
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
        fn cyclo_if_elsif_else() {
            let source_file = File::from_string(
                "java",
                r#"
                public int foo() {
                    if(x) {
                        y = 1;
                    } else if(z) {
                        y = 2;
                    } else {
                        y = 3;
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
    }
}
