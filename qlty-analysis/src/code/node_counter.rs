use crate::code::{File, Visitor};
use tree_sitter::{Node, TreeCursor};

pub struct NodeCounter<'a> {
    pub count: usize,
    kinds: &'a [&'a str],
    source_file: &'a File,
}

impl Visitor for NodeCounter<'_> {
    fn source_file(&self) -> &File {
        self.source_file
    }

    fn process_node(&mut self, cursor: &mut TreeCursor) {
        if self.kinds.contains(&cursor.node().kind()) {
            self.count += 1;
        } else {
            self.process_children(cursor);
        }
    }
}

impl<'a> NodeCounter<'a> {
    pub fn count(source_file: &'a File, kinds: &'a [&'a str], node: &Node<'a>) -> usize {
        let mut counter = Self {
            count: 0,
            kinds,
            source_file,
        };
        counter.process_node(&mut node.walk());
        counter.count
    }
}
