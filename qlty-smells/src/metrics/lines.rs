use qlty_analysis::code::File;
use qlty_analysis::code::NodeFilter;
use qlty_analysis::code::Visitor;
use qlty_analysis::Language;
use serde::Serialize;
use std::collections::HashSet;
use std::ops::{Add, AddAssign};
use tree_sitter::{Node, TreeCursor};

impl Lines {
    pub fn for_node(source_file: &File, node: &Node, filter: &NodeFilter) -> Self {
        let mut processor = LinesProcessor::new(source_file, node, filter);

        processor.process_node(&mut node.walk());
        processor.calculate();
        processor.lines
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Lines {
    pub total: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
}

struct LinesProcessor<'a> {
    lines: Lines,
    source_file: &'a File,
    code_lines: HashSet<usize>,
    comment_lines: HashSet<usize>,
    filter: &'a NodeFilter,
    first_line: usize,
    last_line: usize,
}

impl<'a> LinesProcessor<'a> {
    pub fn new(source_file: &'a File, node: &Node, filter: &'a NodeFilter) -> Self {
        // If the node is the root node, we want to count all lines in the file
        let (first_line, last_line, total_line_count) = if node.parent().is_none() {
            let total = source_file.contents.lines().count();
            (1, total.max(1), total)
        } else {
            let range = node.range();
            let first = range.start_point.row + 1;
            let last = range.end_point.row + 1;
            (first, last, last - first + 1)
        };

        Self {
            lines: Lines {
                total: total_line_count,
                code_lines: 0,
                comment_lines: 0,
                blank_lines: 0,
            },
            code_lines: HashSet::new(),
            comment_lines: HashSet::new(),
            source_file,
            filter,
            first_line,
            last_line,
        }
    }

    fn clamp(&self, line: usize) -> usize {
        line.clamp(self.first_line, self.last_line)
    }

    fn calculate(&mut self) {
        // For lines that have code and comments, we count them as code
        self.comment_lines = &self.comment_lines - &self.code_lines;

        self.lines.code_lines = self.code_lines.len();
        self.lines.comment_lines = self.comment_lines.len();
        self.lines.blank_lines = self
            .lines
            .total
            .saturating_sub(self.lines.code_lines)
            .saturating_sub(self.lines.comment_lines);

        debug_assert_eq!(
            self.lines.total,
            self.lines.code_lines + self.lines.comment_lines + self.lines.blank_lines
        );
    }

    fn record_node(&mut self, node: &Node) {
        let start_line = self.clamp(node.range().start_point.row + 1);
        let end_line = self.clamp(node.range().end_point.row + 1);

        self.code_lines.insert(start_line);
        self.code_lines.insert(end_line);
    }

    fn record_string(&mut self, node: &Node) {
        let start_line = self.clamp(node.range().start_point.row + 1);
        let end_line = self.clamp(node.range().end_point.row + 1);

        for line in start_line..=end_line {
            self.code_lines.insert(line);
        }
    }

    fn record_comment(&mut self, node: &Node) {
        let start_line = self.clamp(node.range().start_point.row + 1);
        let end_line = self.clamp(node.range().end_point.row + 1);

        for line in start_line..=end_line {
            self.comment_lines.insert(line);
        }
    }
}

impl Visitor for LinesProcessor<'_> {
    fn language(&self) -> &Box<dyn Language + Sync> {
        self.source_file.language()
    }

    fn skip_node(&self, node: &Node) -> bool {
        !node.is_named() || self.filter.exclude(node)
    }

    fn visit_if(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_else(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_elsif(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_ternary(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_switch(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_loop(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_except(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_jump(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_return(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_binary(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_field(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_call(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_function(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_closure(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_comment(&mut self, cursor: &mut TreeCursor) {
        self.record_comment(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_string(&mut self, cursor: &mut TreeCursor) {
        self.record_string(&cursor.node());
        self.process_children(cursor);
    }

    fn visit_invisible_container(&mut self, cursor: &mut TreeCursor) {
        self.process_children(cursor);
    }

    fn visit_unknown(&mut self, cursor: &mut TreeCursor) {
        self.record_node(&cursor.node());
        self.process_children(cursor);
    }
}

impl Add for Lines {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            total: self.total + other.total,
            code_lines: self.code_lines + other.code_lines,
            comment_lines: self.comment_lines + other.comment_lines,
            blank_lines: self.blank_lines + other.blank_lines,
        }
    }
}

impl AddAssign for Lines {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod csharp {
        use super::*;

        #[test]
        fn end_row_past_eof_is_clamped() {
            let source_file = File::from_string(
                "csharp",
                "namespace Demo\n{\n    public class Foo\n    {\n    }\n}\n",
            );
            let lines = Lines::for_node(
                &source_file,
                &source_file.parse().root_node(),
                &NodeFilter::empty(),
            );

            assert_eq!(lines.total, 6);
            assert_eq!(
                lines.code_lines + lines.comment_lines + lines.blank_lines,
                lines.total
            );
        }
    }

    mod python {
        use super::*;

        #[test]
        fn blanks() {
            let source_file = File::from_string("python", "\n\n\n");
            let lines = Lines::for_node(
                &source_file,
                &source_file.parse().root_node(),
                &NodeFilter::empty(),
            );
            insta::assert_yaml_snapshot!(lines, @r"
            total: 3
            code_lines: 0
            comment_lines: 0
            blank_lines: 3
            ");
        }

        #[test]
        fn blanks_no_trailing_newline() {
            let source_file = File::from_string("python", "\nfoo");
            let lines = Lines::for_node(
                &source_file,
                &source_file.parse().root_node(),
                &NodeFilter::empty(),
            );
            insta::assert_yaml_snapshot!(lines, @r"
            total: 2
            code_lines: 1
            comment_lines: 0
            blank_lines: 1
            ");
        }

        #[test]
        fn code_lines() {
            let source_file = File::from_string(
                "python",
                r#"
                class Foo:

                    def bar():
                        pass

                def baz():
                    pass"#,
            );
            let lines = Lines::for_node(
                &source_file,
                &source_file.parse().root_node(),
                &NodeFilter::empty(),
            );
            insta::assert_yaml_snapshot!(lines, @r"
            total: 8
            code_lines: 5
            comment_lines: 0
            blank_lines: 3
            ");
        }

        #[test]
        fn comment_lines() {
            let source_file = File::from_string(
                "python",
                r#"
                # line 1
                # line 2
                def baz():
                    pass # does not count as comment line"#,
            );
            let lines = Lines::for_node(
                &source_file,
                &source_file.parse().root_node(),
                &NodeFilter::empty(),
            );
            insta::assert_yaml_snapshot!(lines, @r"
            total: 5
            code_lines: 2
            comment_lines: 2
            blank_lines: 1
            ");
        }
    }
}
