use qlty_analysis::code::{matches_count, File, NodeFilter};
use tree_sitter::Node;

pub fn count<'a>(source_file: &'a File, node: &Node<'a>, filter: &NodeFilter) -> usize {
    matches_count(
        source_file.language().function_declaration_query(),
        node,
        "definition.function",
        filter,
    )
}
