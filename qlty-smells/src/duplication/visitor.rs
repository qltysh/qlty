use super::code::build_node;
use super::Node;
use qlty_analysis::code::{File, NodeFilter, Visitor};
use std::sync::Arc;
use tree_sitter::TreeCursor;

const MAX_DEPTH: usize = 1024;

pub struct NodeVisitor<'a> {
    pub depth: usize,
    pub nodes: Vec<Arc<Node>>,
    pub stack: Vec<Vec<Arc<Node>>>,
    pub filter: &'a NodeFilter,
    pub source_file: &'a File,
}

impl<'a> Visitor for NodeVisitor<'a> {
    fn source_file(&self) -> &File {
        self.source_file
    }

    fn process_node(&mut self, cursor: &mut TreeCursor) {
        let current = cursor.node();

        if self.skip_node(&current) {
            return;
        }

        self.depth += 1;
        self.stack.push(vec![]);

        self.process_children(cursor);

        let children = self.stack.pop().unwrap();
        self.depth -= 1;

        let node = build_node(self.source_file, &current, children);

        let index = self.stack.len() - 1;
        let current_frame = self.stack.get_mut(index).unwrap();

        current_frame.push(node.clone());
        self.nodes.push(node);
    }

    fn skip_node(&self, node: &tree_sitter::Node) -> bool {
        self.depth >= MAX_DEPTH || !node.is_named() || self.filter.exclude(node)
    }
}
