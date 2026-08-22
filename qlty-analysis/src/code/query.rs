use crate::code::{File, NodeFilter};
use tree_sitter::{Node, Query};

pub const QUERY_MATCH_LIMIT: usize = 256;

/// `source_file` supplies the text that tree-sitter needs to evaluate a query's text
/// predicates (`#match?`, `#eq?`, `#not-match?`). Queries without text predicates are
/// unaffected by it.
pub fn matches_count(
    query: &Query,
    node: &Node,
    capture_name: &str,
    filter: &NodeFilter,
    source_file: &File,
) -> usize {
    let mut cursor = tree_sitter::QueryCursor::new();
    cursor.set_match_limit(QUERY_MATCH_LIMIT as u32);
    let all_matches = cursor.matches(query, *node, source_file.contents.as_bytes());

    let mut count = 0;

    for each_match in all_matches {
        let capture = capture_by_name(query, capture_name, &each_match);
        let node = capture.node;

        if !filter.exclude(&node) {
            count += 1;
        }
    }

    count
}

pub fn all_captured_nodes<'a>(
    query_source: &'a str,
    source_file: &'a File,
    node: &'a Node,
) -> Vec<Node<'a>> {
    let query_with_capture = format!("{} @the-capture", query_source);
    let query = source_file.query(&query_with_capture);

    let mut cursor = tree_sitter::QueryCursor::new();
    cursor.set_match_limit(QUERY_MATCH_LIMIT as u32);

    cursor
        .matches(&query, *node, source_file.contents.as_bytes())
        .flat_map(|each_match| each_match.captures)
        .map(|capture| capture.node)
        .collect()
}

pub fn capture_source<'a>(
    query: &'a Query,
    name: &'a str,
    query_match: &'a tree_sitter::QueryMatch,
    source_file: &File,
) -> String {
    let capture = capture_by_name(query, name, query_match);
    node_source(&capture.node, source_file)
}

pub fn child_source(parent: &tree_sitter::Node, name: &str, source_file: &File) -> String {
    let child = parent.child_by_field_name(name).unwrap();
    node_source(&child, source_file)
}

pub fn node_source(node: &tree_sitter::Node, source_file: &File) -> String {
    node.utf8_text(source_file.contents.as_bytes())
        .unwrap()
        .to_string()
}

pub fn capture_by_name_option<'a>(
    query: &'a Query,
    name: &'a str,
    query_match: &'a tree_sitter::QueryMatch,
) -> Option<&'a tree_sitter::QueryCapture<'a>> {
    let index = query.capture_index_for_name(name)?;

    query_match
        .captures
        .iter()
        .find(|capture| capture.index == index)
}

pub fn capture_by_name<'a>(
    query: &'a Query,
    name: &'a str,
    query_match: &'a tree_sitter::QueryMatch,
) -> &'a tree_sitter::QueryCapture<'a> {
    let index = query.capture_index_for_name(name).unwrap();

    query_match
        .captures
        .iter()
        .find(|capture| capture.index == index)
        .unwrap()
}
